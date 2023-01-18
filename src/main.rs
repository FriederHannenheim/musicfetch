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
use cursive::view::{Nameable, Resizable};
use cursive::views::{Dialog, EditView, LinearLayout, TextView};

use crate::download::{download_song, fetch_yt_dlp_json};
use crate::structs::{AlbumMetadata, Playlist, Song, SongMetadata};
use crate::tagging::tag_song;

use id3::frame::{Picture, PictureType};
use id3::{Tag, TagLike};

use regex::Regex;

use sanitize_filename::sanitize;

use cursive::{Cursive, CursiveExt};

mod download;
mod structs;
mod tagging;

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
                let metadata: SongMetadata = serde_json::from_value(song_entry.clone())?;
                let json = serde_json::to_string(&song_entry)?;
                let path = download_song(&json, &args.output_dir)?;
                let tag = Tag::read_from_path(&path).ok();

                songs.push(Song {
                    path: path.into(),
                    tag,
                    song_metadata: metadata,
                });
            }
        }
        if let Ok(song_metadata) = serde_json::from_str::<SongMetadata>(&yt_dlp_json) {
            let path = download_song(&yt_dlp_json, &args.output_dir)?;
            let tag = Tag::read_from_path(&path).ok();

            songs.push(Song {
                path: path.into(),
                tag,
                song_metadata,
            });
        }
    }
    for path in &args.files {
        let tag = Tag::read_from_path(&path).ok();

        songs.push(Song {
            path: path.clone(),
            tag,
            song_metadata: default(),
        });
    }

    let cover_image = args.cover_url.as_ref().map(|url| fetch_cover_image(&url));

    complete_song_metadata(&mut songs, &args)?;

    for mut song in songs {
        song = tag_song(song, cover_image.clone(), &args)?;

        if args.rename {
            // TODO: Currently all files are mp3 but in future this should not be hardcoded
            let mut out_path = format!(
                "{}{}.mp3",
                &args.output_dir,
                &sanitize_and_remove_leading_dots(song.tag.as_ref().unwrap().title().unwrap())
            );

            let mut i = 1;
            while Path::new(&out_path).exists() && Path::new(&song.path) != Path::new(&out_path) {
                out_path = format!(
                    "{}{} ({}).mp3",
                    &args.output_dir,
                    &sanitize_and_remove_leading_dots(song.tag.as_ref().unwrap().title().unwrap()),
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

    let inputs = LinearLayout::vertical()
        .child(TextView::new("Album Title"))
        .child(EditView::new().with_name("album"))
        .child(TextView::new("Artist"))
        .child(EditView::new().with_name("artist"))
        .child(TextView::new("Year"))
        .child(
            EditView::new()
                .on_edit(|s, t, _| {
                    s.call_on_name("year", |view: &mut EditView| {
                        view.set_content(
                            t.chars()
                                .filter(|c| c.is_ascii_digit())
                                .take(4)
                                .fold(String::new(), |x, y| x + &y.to_string()),
                        );
                    });
                })
                .with_name("year"),
        )
        .child(TextView::new("Genre"))
        .child(EditView::new().with_name("genre"));
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
                    v.get_content().parse::<i32>().unwrap()
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

    siv.run();

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

    Picture {
        mime_type,
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
