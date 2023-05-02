use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde_json::Value;

use crate::modules::jsonfetch::Jsonfetch;

use super::Module;

pub struct Songcounter;

impl Module for Songcounter {
    fn name() -> String {
        String::from("trackcounter")
    }

    fn deps() -> Vec<String> {
        vec![Jsonfetch::name()]
    }

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
}
