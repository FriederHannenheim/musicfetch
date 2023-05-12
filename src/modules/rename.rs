use crate::modules::download::DownloadModule;

use super::Module;

use serde_json::Value;
use std::sync::{Arc, Mutex};

use anyhow::Result;

pub struct RenameModule;

impl Module for RenameModule {
    fn name() -> String {
        String::from("rename")
    }

    fn deps() -> Vec<String> {
        vec![DownloadModule::name()]
    }

    fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        Ok(())
    }
}