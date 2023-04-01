use std::error::Error;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::str;

use spinners::{Spinner, Spinners};

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

pub fn fetch_yt_dlp_json(url: &str) -> Result<String, Box<dyn Error>> {
    let mut sp = Spinner::new(Spinners::Line, "Fetching Song/Playlist info. For playlists this can take a while depending on your internet speed.".into());
    let json_output = Command::new("yt-dlp")
        .arg("-J")
        .arg(&url)
        .stderr(Stdio::inherit())
        .output()
        .expect("Running yt-dlp command failed. Check if it is installed");

    sp.stop_with_newline();

    Ok(String::from_utf8(json_output.stdout).expect(
        "Could not parse the yt-dlp json. The yt-dlp command probably didn't run correctly",
    ))
}

pub fn download_song(yt_dlp_json: &str, dir: &str) -> Result<String, Box<dyn Error>> {
    // Download song
    let mut download_process = Command::new("yt-dlp")
        .args(&YT_DLP_ARGS)
        .arg("--load-info-json")
        .arg("-")
        .arg("-P")
        .arg(dir)
        .stdin(Stdio::piped())
        .spawn()?;
    let stdin = download_process
        .stdin
        .as_mut()
        .expect("Failed to write to yt-dlp stdin");
    stdin.write(&yt_dlp_json.as_bytes())?;
    download_process.wait()?.exit_ok()?;

    // Return path of downloaded file
    Ok(get_downloaded_filename(&yt_dlp_json)?)
}

fn get_downloaded_filename(yt_dlp_json: &str) -> Result<String, Box<dyn Error>> {
    let mut filename = String::new();

    let mut filename_process = Command::new("yt-dlp")
        .args(&YT_DLP_ARGS)
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
