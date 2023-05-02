use std::sync::{Arc, Mutex};

use cursive::{
    direction::Direction,
    theme::Theme,
    view::{Nameable, Resizable},
    views::{
        Button, Dialog, DummyView, EditView, LinearLayout, ResizedView, ScrollView, SelectView,
        TextView,
    },
    Cursive, CursiveExt, View,
};
use serde_json::Value;

use crate::{modules::jsonfetch::Jsonfetch, set_song_field};

use self::{
    layout::{get_selectview, get_song_metadata_layout, get_total_tracks_input},
    util::{get_song_field, song_to_string, compare_songs_by_track_no},
};

use super::Module;

use anyhow::{bail, Result};

mod layout;
mod util;

pub struct TagUI;

impl Module for TagUI {
    fn name() -> String {
        String::from("tagui")
    }

    fn deps() -> Vec<String> {
        vec![Jsonfetch::name()]
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

    siv.add_layer(Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new("Edit Tags").center())
            .child(DummyView.fixed_height(1))
            .child(
                LinearLayout::horizontal()
                    .child(get_selectview(songs))
                    .child(DummyView.fixed_width(1))
                    .child(ResizedView::with_fixed_width(
                        32,
                        get_song_metadata_layout(&songs[0])?,
                    )),
            )
            .child(get_total_tracks_input(get_song_field(
                &songs[0],
                "total_tracks",
            )?))
            .child(Button::new("Save", |siv| {
                let _songs = siv
                    .call_on_name("songlist", |v: &mut SelectView<Value>| {
                        v.iter()
                            .map(|(_, song)| song.to_owned())
                            .collect::<Vec<Value>>()
                    })
                    .expect("Failed getting songlist from selectview");
                siv.quit();
            })),
    ));
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
        let pos = songlist.iter().position(|(_itm, song)| song == sel.as_ref());
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
            get_song_field(song, "track_no").unwrap(),
            song_to_string(song)
        ));
    }
}