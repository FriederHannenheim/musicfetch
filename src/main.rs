use std::{
    fs::File,
    sync::{Arc, Mutex},
    thread,
};

use config::get_config;
use serde_json::{Map, Value};

use anyhow::{Context, Result};
use simplelog::{Config, WriteLogger};

mod cmdline;
mod config;
mod module_util;
mod modules;

// TODO: In the config multible versions of a module could be specified with different configs
// TODO: Allow stages up to 99 with spaces in between, that way stages can be fit in between the inherited order

fn main() -> Result<()> {
    WriteLogger::init(
        log::LevelFilter::Info,
        Config::default(),
        File::create("/tmp/musiclog")?,
    )?;

    let args = cmdline::parse_args()?;

    let config_name = args.config.clone().unwrap_or(String::from("default"));

    let config = get_config(&config_name).context("Failed to load config")?;

    let mut m = Map::new();
    m.insert(String::from("args"), serde_json::to_value(args)?);
    m.insert(String::from("config"), config);
    let global_data = Arc::new(Mutex::new(Value::from(m)));
    let song_data = Arc::new(Mutex::new(Value::from(Vec::<Value>::new())));

    run_stages(Arc::clone(&global_data), Arc::clone(&song_data));

    Ok(())
}

pub fn run_stages(global_data: Arc<Mutex<Value>>, song_data: Arc<Mutex<Value>>) {
    let _global = global_data.lock().unwrap();
    let stages = _global["config"]["stages"].clone();
    drop(_global);

    let mut modules_ran: Vec<String> = vec![];

    let mut i = 1;
    while let Some(stage) = stages.get(format!("stage{}", i)) {
        eprintln!("Running stage {}...", i);

        let mut handles = vec![];

        let stage_module_names = stage
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned());

        for module_name in stage_module_names.clone() {
            let module = modules::get_module(&module_name).unwrap();

            for dependency in module.deps {
                if !modules_ran.contains(&dependency) {
                    panic!("Module {module_name} depends on {dependency} but it hasn't run. Please move {dependency} to an earlier stage.");
                }
            }

            let _global = Arc::clone(&global_data);
            let _songs = Arc::clone(&song_data);

            let handle = thread::spawn(move || {
                (module.run_function)(_global, _songs)
                    .unwrap_or_else(|_| panic!("Error in module {}:", module_name))
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        modules_ran.append(
            &mut stage_module_names
                .map(String::from)
                .collect::<Vec<String>>(),
        );

        i += 1;
    }
}
