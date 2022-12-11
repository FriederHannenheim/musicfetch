#![feature(let_else)]
#![feature(exit_status_error)]

use std::env;
use std::str;
use std::process::{Command,Stdio};
use std::error::Error;
use std::io::{Write, Read};

use spinners::{Spinner, Spinners};

use musicfetch_common::Song;


const YT_DLP_ARGS : [&str; 10] = [
    "-x",
    "-f", "ba",
    "-P", "/tmp",
    "--audio-format", "mp3",
    "--restrict-filenames",
    "-o", "%(title)s.%(ext)s",

];

pub fn get_yt_dlp_json(url: &str) -> Result<String, Box<dyn Error>> {
    let mut sp = Spinner::new(Spinners::Line, "Fetching Song/Playlist info".into());
    let json_output = Command::new("yt-dlp")
        .arg("-J")
        .arg(&url)
        .stderr(Stdio::inherit())
        .output()?;

    sp.stop_with_newline();
    
    // Check if command ran correctly
    json_output.status.exit_ok()?;
    
    Ok(String::from_utf8(json_output.stdout)?)
}

pub fn download_song(song: &mut Song, yt_dlp_json: &str) -> Result<(), Box<dyn Error>> {
    // Download song
    let mut download_process = Command::new("yt-dlp")
        .args(&YT_DLP_ARGS)
        .arg("--load-info-json")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()?;
    let stdin = download_process.stdin.as_mut().expect("Failed to write to yt-dlp stdin");
    stdin.write(&yt_dlp_json.as_bytes())?;
    download_process.wait()?.exit_ok().expect("Download failed");

    // Get path of downloaded file
    song.filename = get_downloaded_filename(&yt_dlp_json)?;

    Ok(())
}

fn get_downloaded_filename(yt_dlp_json: &str) -> Result<String,Box<dyn Error>> {
    let mut filename = String::new();
    
    let mut filename_process = Command::new("yt-dlp")
        .args(&YT_DLP_ARGS)
        .args([
              "--load-info-json", "-",
              "-O", "after_move:filepath",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = filename_process.stdin.as_mut().expect("Failed to write to yt-dlp stdin");
    stdin.write(&yt_dlp_json.as_bytes())?;

    let mut stdout = filename_process.stdout.take().expect("Failed to capture yt-dlp stdout");
    filename_process.wait()?.exit_ok().expect("Failed to get filename of downloaded file");
    stdout.read_to_string(&mut filename)?;

    return Ok(filename.trim().to_owned());
}