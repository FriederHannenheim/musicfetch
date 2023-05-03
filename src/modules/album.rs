use std::sync::{Arc, Mutex};

use anyhow::{bail, Ok};
use cursive::{
    theme::Theme,
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, TextView},
    Cursive, CursiveExt,
};
use serde_json::Value;

use crate::modules::jsonfetch::JsonfetchModule;

use super::Module;

#[derive(Default)]
struct AlbumMetadata {
    title: String,
    artist: String,
    year: Option<u64>,
    genre: String,
}

macro_rules! get_value_if_exists {
    ($metadata_key:expr, $key:expr, $song:ident) => {
        if let Value::String(value) = &$song["songinfo"][$key] {
            $metadata_key = value.to_owned();
        }
    };

    ($metadata_key:expr, $key:expr, $song:ident, $conversion:ident) => {
        if let Value::Number(value) = &$song["songinfo"][$key] {
            let Some(value) = value.$conversion() else {
                bail!("Error in songinfo for {}: {} value is not a unsigned integer", $song["yt_dlp"][$key], $key);
            };
            $metadata_key = Some(value as u64);
        }
    };
}

pub struct AlbumModule;

impl Module for AlbumModule {
    fn name() -> String {
        String::from("albumui")
    }

    fn deps() -> Vec<String> {
        vec![JsonfetchModule::name()]
    }

    fn run(
        _global: Arc<Mutex<Value>>,
        songs: Arc<Mutex<Value>>,
    ) -> anyhow::Result<()> {
        let mut album = AlbumMetadata::default();
        {
            let songs = songs.lock().unwrap();
            let songs = songs.as_array().unwrap();

            for song in songs {
                get_value_if_exists!(album.title, "album", song);
                get_value_if_exists!(album.artist, "artist", song);
                get_value_if_exists!(album.year, "year", song, as_u64);
                get_value_if_exists!(album.genre, "genre", song);
            }
        }

        let metadata = show_album_metadata_ui(album);

        let mut songs = songs.lock().unwrap();
        let songs = songs.as_array_mut().unwrap();

        for song in songs {
            song["songinfo"]["album"] = Value::from(metadata.title.clone());
            song["songinfo"]["artist"] = Value::from(metadata.artist.clone());
            song["songinfo"]["year"] = Value::from(metadata.year);
            song["songinfo"]["genre"] = Value::from(metadata.genre.clone());
        }

        Ok(())
    }
}

fn show_album_metadata_ui(album: AlbumMetadata) -> AlbumMetadata {
    let mut siv = Cursive::default();

    siv.set_theme(Theme::terminal_default());

    let inputs = get_album_metadata_layout(album);

    let dialog = Dialog::around(inputs)
        .button("Ok", |s| {
            let title = s
                .call_on_name("album", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            let artist = s
                .call_on_name("artist", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            let year = s
                .call_on_name("year", |v: &mut EditView| {
                    v.get_content().parse::<u64>().ok()
                })
                .unwrap();
            let genre = s
                .call_on_name("genre", |v: &mut EditView| v.get_content().to_string())
                .unwrap();
            s.set_user_data(AlbumMetadata {
                title,
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

    siv.take_user_data().unwrap()
}

fn get_album_metadata_layout(album: AlbumMetadata) -> LinearLayout {
    LinearLayout::vertical()
        .child(TextView::new("Album Title"))
        .child(EditView::new().content(album.title).with_name("album"))
        .child(TextView::new("Artist"))
        .child(EditView::new().content(album.artist).with_name("artist"))
        .child(TextView::new("Year"))
        .child(
            EditView::new()
                .content({
                    album.year.map(|v| v.to_string()).unwrap_or_default()
                })
                .on_edit(|s, t, _| {
                    s.call_on_name("year", |view: &mut EditView| {
                        view.set_content(
                            t.chars()
                                .filter(|c| c.is_ascii_digit())
                                .take(4)
                                .collect::<String>(),
                        );
                    });
                })
                .with_name("year"),
        )
        .child(TextView::new("Genre"))
        .child(EditView::new().content(album.genre).with_name("genre"))
}
