use std::sync::{Arc, Mutex};

use cursive::{
    direction::Direction,
    theme::Theme,
    view::Resizable,
    views::{Button, Dialog, DummyView, LinearLayout, ResizedView, SelectView, TextView},
    Cursive, CursiveExt, View,
};
use serde_json::Value;

use crate::modules::jsonfetch::JsonfetchModule;

use self::{
    dialog::{create_dialog},
    util::{compare_songs_by_track_no, get_song_field, song_to_string}, song_select::create_song_select_view,
};

use super::Module;

use anyhow::Result;

mod dialog;
mod util;
mod song_select;
mod song_edit;

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

        *songs = Value::from(
            siv.take_user_data::<Vec<Value>>()
                .expect("Could not get Cursive user data."),
        );

        Ok(())
    }
}

// TODO: Move layout stuff to layout.rs and refactor
pub fn init_cursive(songs: Arc<Mutex<Value>>) -> Result<Cursive> {
    let mut siv = Cursive::default();

    siv.set_theme(Theme::terminal_default());

    let songs = songs.lock().unwrap();
    let songs = songs.as_array().unwrap();

    siv.add_layer(create_dialog(songs)?);

    Ok(siv)
}

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
