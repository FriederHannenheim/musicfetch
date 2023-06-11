use cursive::{
    view::{Resizable, Nameable},
    views::{Button, Dialog, DummyView, LinearLayout, SelectView, TextView, EditView}, align::HAlign,
};
use cursive_aligned_view::Alignable;
use serde_json::Value;

use anyhow::Result;

use super::{
    song_edit::create_song_edit_layout,
    song_select::create_song_select_view,
    util::{get_song_field, remove_non_numeric_chars},
};

pub fn create_dialog(songs: &Vec<Value>) -> Result<Dialog> {
    let dialog = Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new("Edit Tags").center())
            .child(DummyView.fixed_height(1))
            .child(
                LinearLayout::horizontal()
                    .child(create_song_select_view(songs).fixed_width(32))
                    .child(DummyView.fixed_width(1))
                    .child(create_song_edit_layout(&songs[0])?.fixed_width(32))
            )
            .child(create_total_tracks_input(&get_song_field(&songs[0], "total_tracks").unwrap_or_default()))
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
            }))
    );
    Ok(dialog)
}


fn create_total_tracks_input(initial_value: &str) -> LinearLayout {
    LinearLayout::vertical()
        .child(DummyView.fixed_height(1))
        .child(TextView::new("Total Tracks:").h_align(HAlign::Center))
        .child(
            EditView::new()
                .content(initial_value)
                .on_edit(move |siv, text, _cursor| {
                    let total_tracks = remove_non_numeric_chars(text);

                    siv.call_on_name("total_tracks", |view: &mut EditView| {
                        view.set_content(&total_tracks);
                    });
                    
                    siv.call_on_name("songlist", |list: &mut SelectView<Value>| {
                        for (_lbl, song) in list.iter_mut() {
                            song["total_tracks"] = Value::from(total_tracks.parse::<u64>().ok());
                        }
                    });
                })
                .with_name("total_tracks")
                .fixed_width(8)
                .align_center(),
        )
        .child(DummyView.fixed_height(1))
}