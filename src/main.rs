#![feature(let_else)]
#![feature(exit_status_error)]

use std::env;
use std::str;
use std::process::{Command,Stdio};
use std::error::Error;
use std::io::{Write, Read};

use serde::{Deserialize, Serialize};

use regex::Regex;

use id3::{Tag, TagLike, Version, Frame, Content};
use id3::frame::{Picture, PictureType};

use dialoguer::Input;

use crate::serde_helper::deserialize_null_default;

mod serde_helper;

#[derive(Debug, Serialize, Deserialize)]
struct YoutubeVideo {
    channel: String,
    description: String,
    thumbnail: String,
    fulltitle: String,
    #[serde(skip)]
    filename: String,
    #[serde(default)]
    track: String,
    #[serde(default)]
    album: String,
    #[serde(default)]
    artist: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_null_default")]
    release_year: i32,
    #[serde(default)]
    genre: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    
    let Some(video) = args.get(1) else {
        return Err(Box::<dyn Error>::from("Video not provided"));
    };

    let cover_url = args.get(2);

    let yt_dlp_args = [
              "-x",
              "-f", "ba",
              "-P", "/tmp",
              "--audio-format", "mp3",
    ];
    // Create YoutubeVideo struct from yt-dlp json
    let command_output = Command::new("yt-dlp")
        .args(&yt_dlp_args)
        .arg("-j")
        .arg(&video)
        .output()?;
    if let Err(error) = command_output.status.exit_ok() {
        println!("{}", str::from_utf8(&command_output.stdout)?);
        println!("{}", str::from_utf8(&command_output.stderr)?);
        return Err(Box::new(error));
    }
    let stdout_string = str::from_utf8(&command_output.stdout)?;
    let mut yt_video: YoutubeVideo = serde_json::from_str(stdout_string).expect("error parsing youtube video json");

    let mut download_process = Command::new("yt-dlp")
        .args(&yt_dlp_args)
        .arg("--load-info-json")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()?;
    let stdin = download_process.stdin.as_mut().expect("Failed to write to yt-dlp stdin");
    stdin.write(&stdout_string.as_bytes())?;
    download_process.wait()?.exit_ok().expect("Download failed");

    let mut filename_process = Command::new("yt-dlp")
        .args(&yt_dlp_args)
        .args([
              "--load-info-json", "-",
              "-O", "after_move:filepath",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    let stdin = filename_process.stdin.as_mut().expect("Failed to write to yt-dlp stdin");
    stdin.write(&stdout_string.as_bytes())?;
    let mut stdout = filename_process.stdout.take().expect("Failed to capture yt-dlp stdout");
    filename_process.wait()?.exit_ok().expect("Failed to get filename of downloaded file");
    stdout.read_to_string(&mut yt_video.filename)?;
    yt_video.filename = yt_video.filename.trim().to_owned();
    metadata_prompt(&mut yt_video)?;

    let artists = yt_video.artist.split(", ").collect::<Vec<&str>>();

    let mut tag = Tag::new();
    tag.set_title(&yt_video.track);
    tag.set_album(&yt_video.album);
    tag.set_year(yt_video.release_year);
    //tag.set_artist(artists[0]);
    tag.set_genre(&yt_video.genre);
    tag.add_frame(Frame::with_content("TPE1", Content::new_text_values(artists)));
    //tag.add_frame(Frame::with_content("IPLS", Content::Unknown(Content::new_text_values(artists).to_unknown()?.into_owned())));

    if let Some(url) = cover_url {
        println!("Downloading cover image...");
        add_image_to_tag(&mut tag, url)?;
    }
    
    println!("Adding metadata to {}", &yt_video.filename);
    tag.write_to_path(&yt_video.filename, Version::Id3v23).expect("Writing Id3 tag failed");

    Ok(())
}

fn add_image_to_tag(tag: &mut Tag, url: &str) -> Result<(), Box<dyn Error>> {
    let file_extension_re = Regex::new(r"\.(\w{3,4})(?:$|\?)")?;
    let file_extension = file_extension_re.find(url).unwrap().as_str();
    
    let resp = minreq::get(url).send().expect("Sending http request for cover image failed");

    tag.add_frame(Picture{
        mime_type: file_extension.to_owned(),
        picture_type: PictureType::CoverFront,
        description: "Cover".to_owned(),
        data: resp.as_bytes().into(),
    });

    Ok(())
}

fn metadata_prompt(video: &mut YoutubeVideo) -> Result<(), Box<dyn Error>> {
    video.track = Input::new()
        .with_prompt("Title")
        .with_initial_text(&video.track)
        .interact_text()?;
    video.album = Input::new()
        .with_prompt("Album")
        .with_initial_text(&video.album)
        .allow_empty(true)
        .interact_text()?;
    video.artist = Input::new()
        .with_prompt("Artists seperated by comma")
        .with_initial_text(&video.artist)
        .interact_text()?;
    video.release_year = Input::new()
        .with_prompt("Year")
        .with_initial_text(&video.release_year.to_string())
        .interact_text()?;
    video.genre = Input::new()
        .with_prompt("Genre seperated by comma")
        .with_initial_text(&video.genre)
        .allow_empty(true)
        .interact_text()?;

    Ok(())
}
