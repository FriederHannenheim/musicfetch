use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use anyhow::Result;

use super::Module;

pub struct Infocopy;

impl Module for Infocopy {
    fn deps() -> Vec<String> {
        vec!["jsonfetch".to_string()]
    }

    fn run(_global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        let mut songs = songs.lock().unwrap();
        let mut songs = songs.as_array_mut().unwrap();
        Ok(())
    }
}
