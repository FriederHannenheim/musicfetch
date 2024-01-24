use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde_json::Value;

use crate::{
    define_module,
    modules::{self, ModuleStruct},
};

define_module!("trackcounter", run, [modules::jsonfetch::MODULE_NAME]);

fn run(_global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
    let mut songs = songs.lock().unwrap();
    let songs = songs.as_array_mut().unwrap();

    let count = songs.len();

    for (i, song) in songs.iter_mut().enumerate() {
        song["songinfo"]["track_no"] = Value::from(i + 1);
        song["songinfo"]["total_tracks"] = Value::from(count);
    }

    Ok(())
}
