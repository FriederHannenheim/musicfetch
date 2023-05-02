use anyhow::{Ok, bail};
use cursive::{
    theme::Theme,
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, TextView},
    Cursive, CursiveExt,
};
use serde_json::Value;

use crate::modules::jsonfetch::Jsonfetch;

use super::Module;

#[derive(Default)]
struct AlbumMetadata {
    title: String,
    artist: String,
    year: u32,
    genre: String,
}

macro_rules! get_value_if_exists {
    ($metadata_key:expr, $key:expr, $song:ident) => {
        if let Some(value) = $song["songinfo"].get($key) {
            let Some(value) = value.as_str() else {
                bail!("Error in songinfo for {}: {} value is not a string", $song["yt_dlp"]["title"], $key);
            };
            $metadata_key = value.to_owned()
        }
    };

    ($metadata_key:expr, $key:expr, $song:ident, $conversion:ident) => {
        if let Some(value) = $song["songinfo"].get($key) {
            let Some(value) = value.$conversion() else {
                bail!("Error in songinfo for {}: {} value is not a string", $song["yt_dlp"]["title"], $key);
            };
            $metadata_key = value as u32;
        }
    };
}

pub struct Album;

impl Module for Album {
    fn name() -> String {
        String::from("albumui")
    }

    fn deps() -> Vec<String> {
        vec![Jsonfetch::name()]
    }

    fn run(
        _global: std::sync::Arc<std::sync::Mutex<serde_json::Value>>,
        songs: std::sync::Arc<std::sync::Mutex<serde_json::Value>>,
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
                        // TODO: Fix crash when no year is entered
                        v.get_content().parse::<u32>().unwrap()
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
                    if album.year != 0 {
                        album.year.to_string()
                    } else {
                        String::new()
                    }
                })
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
        .child(EditView::new().content(album.genre).with_name("genre"))
}

