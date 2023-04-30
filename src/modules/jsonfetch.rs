use std::{
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

use anyhow::{Context, Ok, Result};
use serde_json::{Map, Value};

use super::Module;

pub struct Jsonfetch;

impl Module for Jsonfetch {
    fn name() -> String {
        String::from("fetch_song_info")
    }

    fn deps() -> Vec<String> {
        vec![]
    }

    fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        let mut global = global.lock().unwrap();
        let global = global.as_object_mut().unwrap();

        for url in global["args"]["urls"].as_array().unwrap() {
            let json_output = Command::new("yt-dlp")
                .arg("-j")
                .arg(url.as_str().unwrap())
                .stderr(Stdio::inherit())
                .output()
                .context("Running yt-dlp command failed. Check if it is installed")?;

            let json_output =
                String::from_utf8(json_output.stdout).context("Parsing yt-dlp output failed.")?;

            let mut songs = songs.lock().unwrap();
            let songs = songs.as_array_mut().unwrap();
            for line in json_output.lines() {
                let mut m = Map::new();
                m.insert(
                    String::from("yt_dlp"),
                    serde_json::from_str(line)
                        .with_context(|| format!("yt-dlp outputted invalid JSON: \n {}", line))?,
                );
                m.insert(String::from("songinfo"), Map::new().into());
                songs.push(m.into());
            }
        }
        Ok(())
    }
}
