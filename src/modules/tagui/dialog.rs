use cursive::{
    view::Resizable,
    views::{
        DummyView, LinearLayout, ResizedView, SelectView, TextView, Dialog, Button,
    },
    Cursive,
};
use serde_json::Value;

use anyhow::Result;



use super::{util::{get_song_field}, song_select::create_song_select_view, song_edit::{create_track_no_input, create_song_edit_layout}};

pub fn create_dialog(songs: &Vec<Value>) -> Result<Dialog> {
let dialog = Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new("Edit Tags").center())
            .child(DummyView.fixed_height(1))
            .child(
                LinearLayout::horizontal()
                    .child(create_song_select_view(songs))
                    .child(DummyView.fixed_width(1))
                    .child(ResizedView::with_fixed_width(
                        32,
                        create_song_edit_layout(&songs[0])?,
                    )),
            )
            .child(
                create_track_no_input(
                    &get_song_field(&songs[0], "total_tracks")?, 
                    "Total Tracks:", 
                    "total_tracks", 
                    Box::new(|siv: &mut Cursive, total_tracks: String| {
                        siv.call_on_name("songlist", |v: &mut SelectView<Value>| {
                            for (_, song) in v.iter_mut() {
                                song["songinfo"]["total_tracks"] = Value::from(total_tracks.clone());
                            }
                        });
                    })
                )?
            )
            .child(Button::new("Save", |siv| {
                let _songs = siv
                    .call_on_name("songlist", |v: &mut SelectView<Value>| {
                        v.iter()
                            .map(|(_, song)| song.to_owned())
                            .collect::<Vec<Value>>()
                    })
                    .expect("Failed getting songlist from selectview");

                siv.set_user_data(_songs);

                siv.quit();
            })),
    );
    Ok(dialog)
}

