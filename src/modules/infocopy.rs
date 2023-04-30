use std::sync::{Arc, Mutex};

use serde_json::Value;

use anyhow::Result;

use crate::modules::jsonfetch::Jsonfetch;

use super::Module;

pub struct Infocopy;

impl Module for Infocopy {
    fn name() -> String {
        String::from("infocopy")
    }

    fn deps() -> Vec<String> {
        vec![Jsonfetch::name()]
    }

    fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        let mut songs = songs.lock().unwrap();
        let mut songs = songs.as_array_mut().unwrap();

        Ok(())
    }
}
