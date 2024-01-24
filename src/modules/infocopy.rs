use std::sync::{Arc, Mutex};

use serde_json::Value;

use anyhow::{bail, Ok, Result};

use crate::{
    define_module,
    modules::{self, ModuleStruct},
};

define_module!("infocopy", run, [modules::jsonfetch::MODULE_NAME]);

// TODO: Allow using regex with captures to copy only parts of strings
fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
    let global = global.lock().unwrap();
    let infocopy_settings = global["config"]["module"]
        .get("infocopy")
        .expect("Module infocopy has no settings")
        .as_object()
        .expect("Infocopy settings is not a object")
        .clone();
    drop(global);

    let mut songs = songs.lock().unwrap();
    let songs = songs.as_array_mut().unwrap();

    for song in songs {
        for (key, value) in &infocopy_settings {
            let Some(yt_dlp_key) = value.as_str() else {
                bail!("Map key for {} is not a string", key)
            };

            let Some(yt_dlp_value) = song["yt_dlp"].get(yt_dlp_key) else {
                continue;
            };

            song["songinfo"][key] = yt_dlp_value.clone();
        }
    }

    Ok(())
}
