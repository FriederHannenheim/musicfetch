use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

use crate::modules::jsonfetch::JsonfetchModule;

use anyhow::Result;
use serde_json::Value;

use super::Module;

const YT_DLP_ARGS: [&str; 9] = [
    "--ignore-config",
    "-x",
    "-f",
    "ba",
    "--audio-format",
    "mp3",
    "--restrict-filenames",
    "-o",
    "%(title)s.%(ext)s",
];

struct DownloadModule;

impl Module for DownloadModule {
    fn name() -> String {
        String::from("download")
    }

    fn deps() -> Vec<String> {
        vec![JsonfetchModule::name()]
    }

    fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        let song_json_list;

        let module_config;
        {
            let mut _songs = songs.lock().unwrap();
            song_json_list = _songs.as_array().unwrap().clone();

            let mut _global = global.lock().unwrap();
            module_config = _global
                .pointer(&format!("/config/module/{}", Self::name()))
                .map(|v| v.to_owned());
        }

        let args = get_yt_dlp_args(module_config);

        for song_json in song_json_list {
            let yt_dlp_json = song_json.to_string();

            let mut download_process = Command::new("yt-dlp")
                .args(&args)
                .arg("--load-info-json")
                .arg("-")
                .stdin(Stdio::piped())
                .spawn()?;

            let stdin = download_process
                .stdin
                .as_mut()
                .expect("Failed to write to yt-dlp stdin");
            stdin.write(&yt_dlp_json.as_bytes())?;
            download_process.wait()?.exit_ok()?;
        }

        Ok(())
    }
}

fn get_yt_dlp_args(module_config: Option<Value>) -> Vec<String> {
    match module_config.and_then(|v| v["yt_dlp_args"].as_array().map(|v| v.to_owned())) {
        Some(v) => v
            .into_iter()
            .map(|v| {
                v.as_str()
                    .expect("Download module yt_dlp_json config is not an array of strings")
                    .to_owned()
            })
            .collect(),
        None => YT_DLP_ARGS.map(|s| s.to_owned()).to_vec(),
    }
}

fn get_downloaded_filename(yt_dlp_json: &str, args: &Vec<&str>) -> Result<String> {
    let mut filename = String::new();

    let mut filename_process = Command::new("yt-dlp")
        .args(args)
        .args(["--load-info-json", "-", "-q", "-O", "after_move:filepath"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = filename_process
        .stdin
        .as_mut()
        .expect("Failed to write to yt-dlp stdin");
    stdin.write(&yt_dlp_json.as_bytes())?;

    let mut stdout = filename_process
        .stdout
        .take()
        .expect("Failed to capture yt-dlp stdout");
    filename_process
        .wait()?
        .exit_ok()
        .expect("Failed to get filename of downloaded file");
    stdout.read_to_string(&mut filename)?;

    return Ok(filename.trim().to_owned());
}
