use std::sync::{Arc, Mutex};

use cursive::{direction::Direction, theme::Theme, views::SelectView, Cursive, CursiveExt, View, event::{Event, Key}};
use serde_json::Value;

use crate::{modules::jsonfetch::JsonfetchModule, module_util::song_to_string};

use self::{
    dialog::create_dialog,
    util::{compare_songs_by_track_no, get_song_field}, song_select::update_edit_views,
};

use super::Module;

use anyhow::Result;

mod dialog;
mod song_edit;
mod song_select;
pub mod util;

pub struct TagUIModule;

impl Module for TagUIModule {
    fn name() -> String {
        String::from("tagui")
    }

    fn deps() -> Vec<String> {
        vec![JsonfetchModule::name()]
    }

    fn run(_global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        let mut siv = init_cursive(Arc::clone(&songs))?;

        siv.run_crossterm()
            .expect("TUI initialization failed. Try using another Terminal");

        let mut songs = songs.lock().unwrap();


        // TODO: Quit silently
        *songs = Value::from(
            siv.take_user_data::<Vec<Value>>()
                .expect("Could not get Cursive user data."),
        );

        Ok(())
    }
}


// TODO: Prevent saving if there are songs with missing fields
pub fn init_cursive(songs: Arc<Mutex<Value>>) -> Result<Cursive> {
    let mut siv = Cursive::default();

    siv.set_theme(Theme::terminal_default());

    let songs = songs.lock().unwrap();
    let songs = songs.as_array().unwrap();

    siv.add_layer(create_dialog(songs)?);

    add_global_callbacks(&mut siv);

    Ok(siv)
}

fn add_global_callbacks(siv: &mut Cursive) {
    // Callbacks for setting the track_no
    siv.add_global_callback(Event::Shift(Key::Up), |siv| {
        change_track_no_for_current_song(siv, ChangeType::Relative(-1))
    });
    siv.add_global_callback(Event::Shift(Key::Down), |siv| {
        change_track_no_for_current_song(siv, ChangeType::Relative(1))
    });
    for i in 1..=9 {
        let num_char = i.to_string().chars().next().unwrap();

        siv.add_global_callback(num_char, move |siv| {
            change_track_no_for_current_song(siv, ChangeType::Absolute(i))
        });
    }

    // Callbacks for changing selected song anywhere
    siv.add_global_callback(Event::Key(Key::PageUp), |siv| {
        let cb = siv.call_on_name("songlist", |list: &mut SelectView<Value>| {
            list.select_up(1)
        }).expect("UI Error");
        cb(siv);
    });
    siv.add_global_callback(Event::Key(Key::PageDown), |siv| {
        let cb = siv.call_on_name("songlist", |list: &mut SelectView<Value>| {
            list.select_down(1)
        }).expect("UI Error");
        cb(siv);
    });
}

enum ChangeType {
    Relative(i32),
    Absolute(i32),
}

fn change_track_no_for_current_song(siv: &mut Cursive, change: ChangeType) {
    siv.call_on_name("songlist", |list: &mut SelectView<Value>| {
        let Some(song) = list.selected_id() else {
            return;
        };
        if let Some((_, song)) = list.get_item_mut(song) {
            let new_value = match change {
                ChangeType::Relative(i) => {
                    let track_no = song["songinfo"]["track_no"].as_u64().unwrap_or(1);
                    Value::from((track_no as i32 + i).max(0))
                }
                ChangeType::Absolute(i) => Value::from(i.max(0))
            };

            song["songinfo"]["track_no"] = new_value;
        }
    });
    refresh_songlist(siv);

    update_edit_views(siv);
}


// TODO: Highlight Songs with missing fields 
fn refresh_songlist(siv: &mut Cursive) {
    siv.call_on_name("songlist", |songlist: &mut SelectView<Value>| {
        // Get the currently selected song
        let sel = songlist
            .selection()
            .expect("Either the list of songs is empty or something has gone horribly wrong");

        songlist.sort_by(compare_songs_by_track_no);

        refresh_songlist_labels(songlist);

        // Find the position of the edited song after the sort
        let pos = songlist
            .iter()
            .position(|(_itm, song)| song == sel.as_ref());
        // If the song is found, select it. If not give focus to the SelectView
        if let Some(pos) = pos {
            songlist.set_selection(pos);
        } else {
            songlist.take_focus(Direction::none()).unwrap();
        }
    });
}

fn refresh_songlist_labels(songlist: &mut SelectView<Value>) {
    for (label, song) in songlist.iter_mut() {
        label.remove_spans(0..1);
        label.compact();
        label.append(format!(
            "{} {}",
            get_song_field(song, "track_no").unwrap_or_default(),
            song_to_string(song)
        ));
    }
}
