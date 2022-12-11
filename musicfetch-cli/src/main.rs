use std::error::Error;

use clap::Parser;

use musicfetch_common::Song;
use musicfetch_downloader::download_song;
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

    let song = download_song(&args.url)?;

    add_metadata(song, args.cover_url)?;

    Ok(())
}
