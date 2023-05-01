#![feature(more_qualified_paths)]

use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
};

use serde_json::{Map, Value};

use anyhow::Result;

mod cmdline;
mod modules;

fn main() -> Result<()> {
    let config = toml::from_str::<serde_json::Value>(&fs::read_to_string(
        "/home/fried/.config/musicfetch.toml",
    )?)?;

    let mut m = Map::new();
    m.insert(
        String::from("args"),
        serde_json::to_value(cmdline::parse_args()?)?,
    );
    m.insert(String::from("config"), config);
    let global_data = Arc::new(Mutex::new(Value::from(m)));
    let song_data = Arc::new(Mutex::new(Value::from(Vec::<Value>::new())));

    run_stages(Arc::clone(&global_data), Arc::clone(&song_data));

    // println!("{:?}", *global_data.lock().unwrap());
    println!("{}", song_data.lock().unwrap().to_string());

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

        let stage_modules = stage
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_owned());

        for module in stage_modules.clone() {
            let module_fn = modules::get_module(&module).unwrap();

            for dependency in module_fn.0() {
                if !modules_ran.contains(&dependency) {
                    panic!("Module {module} depends on {dependency} but it hasn't run. Please move {dependency} to an earlier stage.");
                }
            }

            let _global = Arc::clone(&global_data);
            let _songs = Arc::clone(&song_data);

            let handle = thread::spawn(move || {
                module_fn.1(_global, _songs).expect(&format!("Error in module {}:", module))
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        modules_ran.append(
            &mut stage_modules
                .map(|s| String::from(s))
                .collect::<Vec<String>>(),
        );

        i += 1;
    }
}
