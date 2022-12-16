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

use clap::{Parser, ArgGroup, ArgAction, builder::ArgPredicate};

use crate::structs::{Song, Playlist, SongMetadata, AlbumMetadata};
use crate::download::{fetch_yt_dlp_json, download_song};
use crate::tagging::tag_song;

use id3::{Tag, TagLike};
use id3::frame::{Picture, PictureType};

use regex::Regex;

use dialoguer::{Input, Confirm};

mod structs;
mod download;
mod tagging;

#[derive(Parser, Debug, Default, Eq, PartialEq)]
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
    
    // Path to read yt-dlp json from or "-" for stdin
    #[arg(short = 'j', long, value_name = "FILE")]
    yt_dlp_json: Option<PathBuf>,

    // url for the cover image
    #[arg(short, long)]
    cover_url: Option<String>,

    // Enable album mode. Artist, Album, Year, Genre will be queried at the start and set for all tracks. 
    // Track Number and Total Tracks will be set automatically.
    #[arg(short, long)]
    album: bool,

    #[arg(short, long, default_value = "./")]
    output_dir: String,

    // Don't rename songs
    #[clap(long = "no-rename", action = ArgAction::SetFalse, default_value_if("files", ArgPredicate::IsPresent, Some("false")))]
    rename: bool,

    // Rename songs to their titles [default]
    #[arg(long = "rename", overrides_with = "rename")]
    _no_rename: bool,
}

impl Args {
    fn hyphen_stdin(mut self) -> Self {
        self.yt_dlp_json = self.yt_dlp_json.map(|path| {
            match path.to_str() {
                Some("-") => PathBuf::from("/dev/stdin"),
                _ => path,
            }
        });
        self
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse().hyphen_stdin();

    let mut songs: Vec<Song> = vec![];

    if args.yt_dlp_json.is_some() || args.url.is_some() {
        let yt_dlp_json = get_yt_dlp_json(&args)?;
        
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

    loop {
        println!("\nInput album metadata:");
        complete_song_metadata(&mut songs, &args)?;
        if Confirm::new().with_prompt("Metadata correct?").default(true).interact()? {
            break;
        }
    }

    for mut song in songs {
        song = tag_song(song, cover_image.clone())?;

        if args.rename {
            // TODO: Currently all files are mp3 but in future this should not be hardcoded
            let out_path = format!("{}{}.mp3", &args.output_dir, song.tag.unwrap().title().unwrap());
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
        let genre: String = 
            Input::new()
                .with_prompt("Genre")
                .allow_empty(true)
                .interact_text()?;
        
        let album_metadata = AlbumMetadata {
            album_title: album_title,
            artist: artist,
            year: year,
            genre: if genre.len() > 0 { Some(genre) } else { None },
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

#[test]
fn test_arg_matching() {
    assert_eq!(
        Args::parse_from(
            ["musicfetch", "-a", "-f", "file1", "file2", "file3"]
        ), 
        Args { 
            files: vec![
                PathBuf::from("file1"), PathBuf::from("file2"), PathBuf::from("file3")
            ], 
            album: true, 
            output_dir: "./".to_owned(),
            ..default()
        }
    );
    
    assert_eq!(
        Args::parse_from(
            ["musicfetch", "--rename", "--no-rename", "ytsearch:test me I dare you"]
        ),
        Args {
            url: Some("ytsearch:test me I dare you".to_owned()),
            output_dir: "./".to_owned(),
            ..default()
        }
    );

    assert_eq!(
        Args::parse_from(
            ["musicfetch", "--no-rename", "--rename", "-j", "playlist_json", "-c", "cover_url"]
        ),
        Args {
            output_dir: "./".to_owned(),
            rename: true,
            _no_rename: true,
            yt_dlp_json: Some(PathBuf::from("playlist_json")),
            cover_url: Some("cover_url".to_owned()),
            ..default()
        }
    );
}
