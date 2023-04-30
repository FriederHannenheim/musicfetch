use std::{
    fs,
    sync::{Arc, Mutex},
    thread,
};

use serde_json::{Map, Value};

use anyhow::Result;
use toml::Table;

mod cmdline;
mod modules;

fn main() -> Result<()> {
    let config = fs::read_to_string("/home/fried/.config/musicfetch.toml")?.parse::<Table>()?;

    let mut m = Map::new();
    m.insert(
        String::from("args"),
        serde_json::to_value(cmdline::parse_args()?)?,
    );
    let global_data = Arc::new(Mutex::new(Value::from(m)));
    let song_data = Arc::new(Mutex::new(Value::from(Vec::<Value>::new())));

    let mut i = 1;
    while let Some(stage) = config["stages"].get(format!("stage{}", i)) {
        let mut handles = vec![];

        for module in stage.as_array().unwrap() {
            let module_fn = modules::MODULES
                .get(module.as_str().unwrap())
                .expect(&format!(
                    "Error in config: No module named {}",
                    module.as_str().unwrap()
                ));

            let _global = Arc::clone(&global_data);
            let _songs = Arc::clone(&song_data);

            let handle = thread::spawn(move || {
                module_fn(_global, _songs).unwrap();
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        i += 1;
    }

    // println!("{:?}", *global_data.lock().unwrap());
    // println!("{}", song_data.lock().unwrap().to_string());

    Ok(())
}
