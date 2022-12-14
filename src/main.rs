#![feature(let_else)]
#![feature(exit_status_error)]
#![feature(default_free_fn)]
#![feature(let_chains)]

use std::error::Error;
use std::path::PathBuf;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::default::default;

use clap::{Parser, ArgGroup, ArgAction};

use crate::structs::{Song, Playlist, SongMetadata, AlbumMetadata};
use crate::download::{fetch_yt_dlp_json, download_song};
use crate::tagging::tag_song;

use id3::{Tag, TagLike};
use id3::frame::{Picture, PictureType};

use regex::Regex;

use dialoguer::Input;

mod structs;
mod download;
mod tagging;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("song")
        .required(true)
        .args(["url", "files", "yt_dlp_json"])
    ))]
struct Args {
    // url of a song or a album playlist
    url: Option<String>,

    // Instead of downloading, tag these local files. Implies --no-rename
    #[arg(short, long, num_args = 1..)]
    files: Vec<PathBuf>,
    
    // yt-dlp json to download
    #[arg(short = 'j', long, value_name = "FILE")]
    yt_dlp_json: Option<PathBuf>,

    // url for the cover image
    #[arg(short, long)]
    cover_url: Option<String>,


    // Enable album mode. Artist, Album, Year, Genre will be queried at the start and set for all tracks. 
    // Track Number and Total Tracks will be set automatically
    #[arg(short, long)]
    album: bool,

    #[arg(short, long, default_value = "./")]
    output_dir: String,

    // Don't rename songs
    #[clap(long = "no-rename", action = ArgAction::SetFalse)]
    rename: bool,

    // Rename songs to their titles [default]
    #[arg(long = "rename", overrides_with = "rename")]
    _no_rename: bool,
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let mut songs: Vec<Song> = vec![];

    if args.yt_dlp_json.is_some() || args.url.is_some() {
        let yt_dlp_json = get_yt_dlp_json(&args)?;
        // TODO: Load Tag so previously tagged files will keep their data
        if let Ok(playlist) = serde_json::from_str::<Playlist>(&yt_dlp_json) {
            for song_entry in playlist.entries {
                let metadata: SongMetadata = serde_json::from_value(song_entry.clone())?;
                let json = serde_json::to_string(&song_entry)?;
                let path = download_song(&json, &args.output_dir)?;
                let tag = Tag::read_from_path(&path).ok();
                
                songs.push(Song {
                    path: path.into(),
                    tag: tag,
                    song_metadata: metadata,
                });
            }
        }
        if let Ok(song_metadata) = serde_json::from_str::<SongMetadata>(&yt_dlp_json) {
            let path = download_song(&yt_dlp_json, &args.output_dir)?;
            let tag = Tag::read_from_path(&path).ok();

            songs.push(Song {
                path: path.into(),
                tag: tag,
                song_metadata: song_metadata,
            });
        }
    }
    for path in &args.files {
        let tag = Tag::read_from_path(&path).ok();

        songs.push(Song {
            path: path.clone(),
            tag: tag,
            song_metadata: default(),
        });
    }

    let cover_image = args.cover_url.as_ref().map(|url| fetch_cover_image(&url));

    complete_song_metadata(&mut songs, &args)?;

    for mut song in songs {
        song = tag_song(song, cover_image.clone())?;

        if args.rename {
            let out_path = format!("{}{}", &args.output_dir, song.tag.unwrap().title().unwrap());
            fs::copy(&song.path, &out_path)?;
            fs::remove_file(&song.path)?;
        }
    }

    Ok(())
}

fn get_yt_dlp_json(args: &Args) -> Result<String, Box<dyn Error>> {
    let mut json = String::new();
    if let Some(json_path) = args.yt_dlp_json.as_ref() {
        let mut file = File::open(json_path)?;
        file.read_to_string(&mut json)?;
    } else {
        json = fetch_yt_dlp_json(args.url.as_ref().unwrap())?;
    }
    Ok(json)
}

fn complete_song_metadata(songs: &mut Vec<Song>, args: &Args) -> Result<(), Box<dyn Error>> {
    if args.album {
        let album_title: String = 
            Input::new()
                .with_prompt("Album Title")
                .interact_text()?;
        let artist: String = 
            Input::new()
                .with_prompt("Artist")
                .interact_text()?;
        let year: i32 = 
            Input::new()
                .with_prompt("Year")
                .interact_text()?;
        
        let album_metadata = AlbumMetadata {
            album_title: album_title,
            artist: artist,
            year: year,
        };

        let song_count = songs.len();
        for (i, song) in songs.iter_mut().enumerate() {
            let mut metadata = &mut song.song_metadata;

            metadata.apply(album_metadata.clone());
            metadata.track_no = Some(i as u32 + 1);
            metadata.total_tracks = Some(song_count as u32);
        }
    }
    Ok(())
}

pub fn fetch_cover_image(url: &str) -> Picture {
    println!("Downloading cover image...");

    let resp = minreq::get(url).send().expect("Sending http request for cover image failed");
    let mime_type = get_mime_type(url).expect("Failed to find file extension in cover url. Make sure it is a valid image url");

    Picture {
        mime_type: mime_type,
        picture_type: PictureType::CoverFront,
        description: "Cover".to_owned(),
        data: resp.as_bytes().into(),
    }
}

// TODO: check if file extension is an image
fn get_mime_type(url: &str) -> Option<String> {
    let re = Regex::new(r"\.(\w{3,4})(?:$|\?)").unwrap();
    let captures = re.captures(url)?;
    let file_extension = captures.get(1)?.as_str();

    Some(format!("image/{}", file_extension))
}