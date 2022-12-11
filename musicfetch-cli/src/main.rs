use std::error::Error;

use clap::Parser;

use musicfetch_common::{Song, Playlist};
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

    if let Ok(playlist) = serde_json::from_str::<Playlist>(&yt_dlp_json) {
        println!("Downloading playlist: {}", &playlist.title);
        for song_value in playlist.entries {
            let song: Song = serde_json::from_value(song_value.clone())?;

            println!("Downloading song: {}", &song.fulltitle);
            
            song.artist = playlist.channel.clone();
            song.album = playlist.title.clone();
            download_and_tag_song(song, &serde_json::to_string(&song_value)?, &args.cover_url)?;
        }
    }
    // Create YoutubeVideo struct from yt-dlp json
    if let Ok(song) = serde_json::from_str::<Song>(&yt_dlp_json) {
        println!("Downloading song: {}", &song.title);
        download_and_tag_song(song, &yt_dlp_json, &args.cover_url)?;
    };



    Ok(())
}

fn download_and_tag_song(mut song: Song, yt_dlp_json: &str, cover_url: &Option<String>) -> Result<(), Box<dyn Error>> {
    println!("{:?}", &cover_url);
    download_song(&mut song, &yt_dlp_json)?;
    add_metadata(song, cover_url)?;
    Ok(())
}