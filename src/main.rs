#![feature(exit_status_error)]
#![feature(default_free_fn)]
#![feature(let_chains)]

use std::default::default;
use std::error::Error;
use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::{ArgAction, ArgGroup, Parser};
use cursive::theme::Theme;
use cursive::view::Resizable;
use cursive::views::{Dialog, EditView};
use tagging::tag_songs_tui;

use crate::download::{download_song, fetch_yt_dlp_json};
use crate::structs::{AlbumMetadata, Playlist, Song, SongMetadata};
use crate::tagging::tag_song;

use lofty::{Accessor, MimeType, Picture, PictureType, Probe, Tag, TagExt, TagType, TaggedFileExt};

use regex::Regex;

use sanitize_filename::sanitize;

use cursive::{Cursive, CursiveExt};

mod download;
mod structs;
mod tagging;
mod tui;

/// musicfetch is a program for downloading and/or tagging music. It uses yt-dlp as a downloader so all sites supported by yt-dlp are also supported by musicfetch
#[derive(Parser, Debug, Default, Eq, PartialEq)]
#[command(author, version, about)]
#[command(group(
    ArgGroup::new("song")
        .required(true)
        .args(["url", "files", "yt_dlp_json"])
    ))]
pub struct Args {
    /// url of a song or a album playlist
    url: Option<String>,

    /// Instead of downloading, tag these local files
    #[arg(short, long, num_args = 1..)]
    files: Vec<PathBuf>,

    /// Path to read yt-dlp json from or "-" for stdin
    #[arg(short = 'j', long, value_name = "FILE")]
    yt_dlp_json: Option<PathBuf>,

    /// url for the cover image
    #[arg(short, long)]
    cover_url: Option<String>,

    /// Enable album mode. Artist, Album, Year, Genre will be queried at the start and set for all tracks.
    /// Track Number and Total Tracks will be set automatically.
    #[arg(short, long)]
    album: bool,

    #[arg(short, long, default_value = "./")]
    output_dir: String,

    /// Don't rename songs
    #[clap(long = "no-rename", action = ArgAction::SetFalse)]
    rename: bool,

    /// Rename songs to their titles [default]
    #[arg(long = "rename", overrides_with = "rename")]
    _no_rename: bool,

    #[arg(short, long)]
    yes: bool,
}

impl Args {
    fn hyphen_stdin(mut self) -> Self {
        self.yt_dlp_json = self.yt_dlp_json.map(|path| match path.to_str() {
            Some("-") => PathBuf::from("/dev/stdin"),
            _ => path,
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
                let Ok(metadata): Result<SongMetadata, _> = serde_json::from_value(song_entry.clone()) else {
                    // Some playlists have unavailable videos which are just 'null' in json
                    continue;
                };
                let json = serde_json::to_string(&song_entry)?;
                let path = download_song(&json, &args.output_dir);
                let path = match path {
                    Ok(p) => p,
                    Err(e) => {
                        eprint!("Downloading {} failed: {e}", metadata.fulltitle);
                        continue;
                    }
                };

                let tag = tag_for_file(&path.clone().into());

                songs.push(Song {
                    path: path.into(),
                    tag,
                    song_metadata: metadata,
                });
            }
        }
        if let Ok(song_metadata) = serde_json::from_str::<SongMetadata>(&yt_dlp_json) {
            let path = download_song(&yt_dlp_json, &args.output_dir)?;
            let tag = tag_for_file(&path.clone().into());

            songs.push(Song {
                path: path.into(),
                tag,
                song_metadata,
            });
        }
    }
    for path in &args.files {
        let tag = tag_for_file(path);

        songs.push(Song {
            path: path.clone(),
            tag,
            song_metadata: default(),
        });
    }

    let cover_image = args.cover_url.as_ref().map(|url| fetch_cover_image(&url));

    complete_song_metadata(&mut songs, &args)?;

    for song in &mut songs {
        let tag: &mut Tag = &mut song.tag;

        tagging::add_metadata_to_tag(&song.song_metadata, tag);
    }

    tag_songs_tui(&mut songs);

    for mut song in songs {
        song = tag_song(song, cover_image.clone(), &args)?;

        if args.rename {
            // TODO: Currently all files are mp3 but in future this should not be hardcoded
            let mut out_path = format!(
                "{}{}.mp3",
                &args.output_dir,
                &sanitize_and_remove_leading_dots(&song.tag.title().unwrap())
            );

            let mut i = 1;
            while Path::new(&out_path).exists() && Path::new(&song.path) != Path::new(&out_path) {
                out_path = format!(
                    "{}{} ({}).mp3",
                    &args.output_dir,
                    &sanitize_and_remove_leading_dots(&song.tag.title().unwrap()),
                    i
                );
                i += 1;
            }

            if Path::new(&song.path) != Path::new(&out_path) {
                fs::rename(&song.path, &out_path)?;
            }
        }
    }

    Ok(())
}

// Remove leading dots from a filename so the file isn't hidden in Unix systems
fn sanitize_and_remove_leading_dots(filename: &str) -> String {
    let filename = sanitize(filename);
    let re = Regex::new(r"^\.*").unwrap();
    re.replace(&filename, "").into_owned()
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
    if args.album && !args.yes {
        let album_metadata = input_album_metadata()?;

        println!("{:?}", album_metadata);

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

fn input_album_metadata() -> Result<AlbumMetadata, Box<dyn Error>> {
    let mut siv = Cursive::default();

    siv.set_theme(Theme::terminal_default());

    let inputs = tui::get_album_metadata_layout();

    let dialog = Dialog::around(inputs)
        .button("Ok", |s| {
            let album_title = s
                .call_on_name("album", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            let artist = s
                .call_on_name("artist", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            let year = s
                .call_on_name("year", |v: &mut EditView| {
                    v.get_content().parse::<u32>().unwrap()
                })
                .unwrap();
            let genre = s
                .call_on_name("genre", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            s.set_user_data(AlbumMetadata {
                album_title,
                artist,
                year,
                genre,
            });
            s.quit();
        })
        .min_width(40);

    siv.add_layer(dialog);

    siv.run_crossterm()
        .expect("TUI initialization failed. Try using another Terminal");

    let album_metadata = siv.take_user_data().unwrap();

    Ok(album_metadata)
}

pub fn fetch_cover_image(url: &str) -> Picture {
    println!("Downloading cover image...");

    let resp = minreq::get(url)
        .send()
        .expect("Sending http request for cover image failed");
    let mime_type = get_mime_type(url)
        .expect("Failed to find file extension in cover url. Make sure it is a valid image url");

    Picture::new_unchecked(PictureType::CoverFront, mime_type, None, resp.into_bytes())
}

// TODO: check if file extension is an image
fn get_mime_type(url: &str) -> Option<MimeType> {
    let re = Regex::new(r"\.(\w{3,4})(?:$|\?)").unwrap();
    let captures = re.captures(url)?;
    let file_extension = captures.get(1)?.as_str();

    let mime_text = format!("image/{}", file_extension);
    Some(MimeType::from_str(&mime_text))
}
/// Get or create tag for the file in `path`. Will strip all other tags
fn tag_for_file(path: &PathBuf) -> Tag {
    let mut tagged_file = Probe::open(path)
        .expect("File not found")
        .read()
        .expect("Failed to read file");

    let mut tag = match tagged_file.primary_tag() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();

                tagged_file.insert_tag(Tag::new(tag_type));
                tagged_file.primary_tag_mut().unwrap()
            }
        }
    }
    .clone();

    // TODO: This should be moved to when the tag is written
    for old_tag in tagged_file.tags() {
        if old_tag.remove_from_path(path).is_err() {
            println!(
                "Unable to remove {:?} tag from {}",
                old_tag.tag_type(),
                path.display()
            );
        }
    }

    // Upgrade ID3v1 to ID3v2
    if tag.tag_type() == TagType::ID3v1 {
        tag.re_map(TagType::ID3v2);
    }

    tag
}

#[test]
fn test_arg_matching() {
    assert_eq!(
        Args::parse_from(["musicfetch", "-a", "-f", "file1", "file2", "file3"]),
        Args {
            files: vec![
                PathBuf::from("file1"),
                PathBuf::from("file2"),
                PathBuf::from("file3")
            ],
            album: true,
            output_dir: "./".to_owned(),
            ..default()
        }
    );

    assert_eq!(
        Args::parse_from([
            "musicfetch",
            "--rename",
            "--no-rename",
            "ytsearch:test me I dare you"
        ]),
        Args {
            url: Some("ytsearch:test me I dare you".to_owned()),
            output_dir: "./".to_owned(),
            ..default()
        }
    );

    assert_eq!(
        Args::parse_from([
            "musicfetch",
            "--no-rename",
            "--rename",
            "-j",
            "playlist_json",
            "-c",
            "cover_url"
        ]),
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
