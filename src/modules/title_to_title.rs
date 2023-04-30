use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use anyhow::Result;

pub fn title_to_title(_global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
    let mut songs = songs.lock().unwrap();
    let mut songs = songs.as_array_mut().unwrap();
    // songs.push(json!("penis song"));
    Ok(())
}
