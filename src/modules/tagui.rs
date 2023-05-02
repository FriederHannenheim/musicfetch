use std::sync::{Mutex, Arc};

use cursive::{Cursive, CursiveExt, views::{Button, SelectView, EditView, DummyView, LinearLayout, ResizedView, ScrollView, TextView, Dialog}, theme::Theme, view::{Nameable, Resizable}, direction::Direction, View};
use serde_json::Value;

use crate::modules::jsonfetch::Jsonfetch;

use super::Module;

use anyhow::{Result, bail};


pub struct TagUI;


impl Module for TagUI {
    fn name() -> String {
        String::from("tagui")
    }

    fn deps() -> Vec<String> {
        vec![Jsonfetch::name()]
    }

    fn run(_global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        let mut siv = Cursive::default();

        siv.set_theme(Theme::terminal_default());
        
        let songs = songs.lock().unwrap();
        let songs = songs.as_array().unwrap();

        let cloned = songs.clone();
        let song_names = cloned.iter().map(|f| song_to_string(f));
    
        let song_selection = SelectView::new()
            .with_all(
                song_names
                    .zip(songs.clone().into_iter())
                    .map(|(label, song)| {
                        (
                            format!("{} {}", get_song_field(&song, "track_no").unwrap(), label),
                            song,
                        )
                    }),
            )
            .on_select(|s, song: &Value| {
                set_cursive_fields_for_song(s, song).unwrap();
            })
            .with_name("songlist");
    
        let scroll_view = ScrollView::new(song_selection).fixed_width(32);
    
        siv.add_layer(Dialog::around(
            LinearLayout::vertical()
                .child(TextView::new("Edit Tags").center())
                .child(DummyView.fixed_height(1))
                .child(
                    LinearLayout::horizontal()
                        .child(scroll_view)
                        .child(DummyView.fixed_width(1))
                        .child(ResizedView::with_fixed_width(
                            32,
                            get_song_metadata_layout(&songs[0])?,
                        )),
                )
                .child(
                    LinearLayout::horizontal()
                        .child(DummyView.full_width())
                        .child(TextView::new("Total Tracks:"))
                        .child(DummyView.fixed_width(11))
                        .child(
                            EditView::new()
                                .content(
                                    get_song_field(&songs[0], "total_tracks")?
                                )
                                .on_edit(|siv, text, _cursor| {
                                    let total_tracks = text
                                        .chars()
                                        .filter(|c| c.is_ascii_digit())
                                        .fold(String::new(), |x, y| x + &y.to_string());
                                    
                                    siv.call_on_name("total_tracks", |view: &mut EditView| {
                                        view.set_content(&total_tracks);
                                    });
                                    
                                    siv.call_on_name("songlist", |v: &mut SelectView<Value>| {
                                        for (_lbl, song) in v.iter_mut() {
                                            song["songinfo"]["total_tracks"] = total_tracks.parse::<u32>().unwrap().into();
                                        }
                                    });
                                })
                                .with_name("total_tracks")
                                .fixed_width(8)
                        )
                        .fixed_width(65),
                )
                .child(Button::new("Save", |siv| siv.quit())),
        ));
    
        siv.run_crossterm()
            .expect("TUI initialization failed. Try using another Terminal");

        Ok(())
    }
}

pub fn get_song_field(song: &Value, field: &str) -> Result<String> {
    Ok(match song["songinfo"].get(field).unwrap() {
        Value::String(string) => string.to_owned(),
        Value::Number(number) => {
            match number.as_i64().unwrap() {
                0 => String::new(),
                num => num.to_string(),
            }
        }
        _ => bail!("Invalid Value type in songinfo field {}", field)
    })
}


fn set_cursive_fields_for_song(s: &mut Cursive, song: &Value) -> Result<()>{
    s.call_on_name("title_text", |v: &mut TextView| {
        v.set_content(song_to_string(song));
    })
    .unwrap();

    for field in ["title", "album", "artist", "year", "genre", "track_no"] {
        let content = get_song_field(song, field)?;
        let result = s.call_on_name(field, |v: &mut EditView| {
          v.set_content(content);
        });
        if result.is_none() {
            bail!("Cursive field {} does not exist", field);
        }
    }
    Ok(())
}


macro_rules! set_song_data {
    ($siv:expr, $field:expr, $content:expr) => {
        $siv.call_on_name("songlist", |v: &mut SelectView<Value>| {
            let Some(selected) = v.selected_id() else { return; };
            let Some((_label, song)) = v.get_item_mut(selected) else { return; };
            song["songinfo"][$field] = Value::from($content);
        })
        .unwrap();
    };
}

pub fn get_song_metadata_layout(first_song: &Value) -> Result<LinearLayout> {
    let layout = LinearLayout::vertical()
        .child(ResizedView::with_fixed_height(
            3,
            TextView::new(&song_to_string(first_song))
                .center()
                .with_name("title_text"),
        ))
        .child(DummyView.fixed_height(1))
        .child(TextView::new("Title"))
        .child(
            EditView::new()
                .content(
                    get_song_field(first_song, "title")?
                )
                .on_edit(|siv, text, _cursor| {
                    set_song_data!(siv, "title", text.to_string());
                    // Refresh here to show title changes in the list
                    refresh_songlist(siv);
                })
                .with_name("title"),
        )
        .child(TextView::new("Album"))
        .child(
            EditView::new()
                .content(
                    get_song_field(first_song, "album")?
                )
                .on_edit(|siv, text, _cursor| {
                    set_song_data!(siv, "album", text.to_string());
                })
                .with_name("album"),
        )
        .child(TextView::new("Artist"))
        .child(
            EditView::new()
                .content(get_song_field(first_song, "artist")?)
                .on_edit(|siv, text, _cursor| {
                    set_song_data!(siv, "artist", text.to_string());
                })
                .with_name("artist"),
        )
        .child(TextView::new("Year"))
        .child(
            EditView::new()
                .on_edit(|siv, t, _| {
                    let year = t
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .take(4)
                        .fold(String::new(), |x, y| x + &y.to_string());
                    siv.call_on_name("year", |view: &mut EditView| {
                        view.set_content(&year);
                    });
                    set_song_data!(siv, "year", year.parse::<u32>().unwrap());
                })
                .content(get_song_field(first_song, "year")?)
                .with_name("year"),
        )
        .child(TextView::new("Genre"))
        .child(
            EditView::new()
                .content(get_song_field(first_song, "genre")?)
                .on_edit(|siv, text, _cursor| {
                    set_song_data!(siv, "genre", text.to_string());
                })
                .with_name("genre"),
        )
        .child(DummyView.fixed_height(1))
        .child(
            LinearLayout::horizontal()
                .child(TextView::new("Track No:").max_width(9))
                .child(DummyView.full_width())
                .child(
                    EditView::new()
                        .content(get_song_field(first_song, "track_no")?)
                        .on_edit(|siv, text, _cursor| {
                            let track = text
                                .chars()
                                .filter(|c| c.is_ascii_digit())
                                .fold(String::new(), |x, y| x + &y.to_string());
                            siv.call_on_name("track_no", |view: &mut EditView| {
                                view.set_content(&track);
                            });
                            set_song_data!(siv, "track_no", track.parse::<u32>().ok());
                            refresh_songlist(siv);
                        })
                        .on_submit(|siv, _text| {
                            siv.focus_name("songlist").unwrap();
                        })
                        .with_name("track_no")
                        .fixed_width(8),
                )
                .full_width(),
        );
    Ok(layout)
}

fn refresh_songlist(siv: &mut Cursive) {
    siv.call_on_name("songlist", |v: &mut SelectView<Value>| {
        // Get the currently selected song
        let sel = v
            .selection()
            .expect("Either the list of songs is empty or something has gone horribly wrong");

        v.sort_by(|first, second| {
            let first_index = 
                first["songinfo"]
                .get("track_no")
                .map(|v| v.as_u64().unwrap())
                .unwrap_or(u64::MAX);
            let second_index = 
                second["songinfo"]
                .get("track_no")
                .map(|v| v.as_u64().unwrap())
                .unwrap_or(u64::MAX);
            first_index.cmp(&second_index)
        });
        for (label, song) in v.iter_mut() {
            label.remove_spans(0..1);
            label.compact();
            label.append(format!(
                "{} {}",
                get_song_field(song, "track_no").unwrap(),
                song_to_string(song)
            ));
        }

        // Find the position of the edited song after the sort
        let pos = v
            .iter()
            .position(|(_itm, song)| song == sel.as_ref());
        // If the song is found, select it. If not give focus to the SelectView
        if let Some(pos) = pos {
            v.set_selection(pos);
        } else {
            v.take_focus(Direction::none()).unwrap();
        }
    });
}

fn song_to_string(song: &Value) -> String {
    match song["songinfo"].get("title") {
        Some(song_name) => song_name,
        None => &song["yt_dlp"]["title"],
    }.as_str().unwrap().to_owned()
}