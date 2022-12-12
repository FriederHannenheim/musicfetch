#![feature(let_else)]
#![feature(exit_status_error)]

use std::error::Error;
use std::path::PathBuf;
use std::fs::File;
use std::io::Read;

use clap::Parser;

use crate::structs::{Song, Playlist};
use crate::download::{get_yt_dlp_json, download_song};
use crate::tagging::{add_metadata, fetch_cover_image};

use id3::frame::Picture;

mod structs;
mod download;
mod tagging;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // url of a song or a album playlist
    url: String,

    // url for the cover image
    cover_url: Option<String>,

    // yt-dlp json to download
    #[arg(short, long, value_name = "FILE")]
    yt_dlp_json: Option<PathBuf>,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let yt_dlp_json = if let Some(json_path) = args.yt_dlp_json {
        let mut contents = String::new();
        let mut file = File::open(json_path)?;
        file.read_to_string(&mut contents)?;
        contents
    } else {
        get_yt_dlp_json(&args.url)?
    };

    let cover_image = args.cover_url.map(|url| fetch_cover_image(&url));

    if let Ok(playlist) = serde_json::from_str::<Playlist>(&yt_dlp_json) {
        download_playlist(playlist, cover_image)?;
    } else 
    if let Ok(song) = serde_json::from_str::<Song>(&yt_dlp_json) {
        download_and_tag_song(song, &yt_dlp_json, cover_image)?;
    }

    Ok(())
}

fn download_playlist(playlist: Playlist, cover: Option<Picture>) -> Result<(), Box<dyn Error>> {
    println!("Downloading playlist: {}", &playlist.title);
    for song_value in playlist.entries {
        let mut song: Song = serde_json::from_value(song_value.clone())?;

        song.artist = playlist.channel.clone();
        song.album = playlist.title.clone();

        download_and_tag_song(song, &serde_json::to_string(&song_value)?, cover.clone())?;
    }
    Ok(())
}

fn download_and_tag_song(mut song: Song, yt_dlp_json: &str, cover: Option<Picture>) -> Result<(), Box<dyn Error>> {
    println!("Downloading song: {}", &song.fulltitle);
    
    download_song(&mut song, &yt_dlp_json)?;
    add_metadata(song, cover)?;
    Ok(())
}
