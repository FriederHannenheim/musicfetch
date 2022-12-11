use std::error::Error;

use clap::Parser;

use musicfetch_common::Song;
use musicfetch_downloader::{get_yt_dlp_json, download_song};
use musicfetch_tagger::add_metadata;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // url of a song or a album playlist
    url: String,

    // url for the cover image
    cover_url: Option<String>,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let yt_dlp_json = get_yt_dlp_json(&args.url)?;

    // Create YoutubeVideo struct from yt-dlp json
    let mut song: Song = serde_json::from_str(&yt_dlp_json).expect("error parsing youtube video json");

    download_song(&mut song, &yt_dlp_json);

    add_metadata(song, args.cover_url)?;

    Ok(())
}
