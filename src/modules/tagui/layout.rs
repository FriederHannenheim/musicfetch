use cursive::{
    view::{Nameable, Resizable},
    views::{
        DummyView, EditView, LinearLayout, NamedView, ResizedView, ScrollView, SelectView, TextView,
    },
    Cursive,
};
use serde_json::Value;

use anyhow::Result;

use crate::{modules::tagui::refresh_songlist, set_song_field};

use super::util::{get_song_field, set_cursive_fields_for_song, song_to_string};

pub fn get_selectview(songs: &Vec<Value>) -> ResizedView<ScrollView<NamedView<SelectView<Value>>>> {
    let cloned = songs.clone();
    let song_names = cloned.iter().map(|f| song_to_string(f));

    let selectview_items = song_names
        .zip(songs.clone().into_iter())
        .map(|(label, song)| {
            (
                format!("{} {}", get_song_field(&song, "track_no").unwrap(), label),
                song,
            )
        });

    let song_selection = SelectView::new()
        .with_all(selectview_items)
        .on_select(set_cursive_fields_for_song)
        .with_name("songlist");

    let scroll_view = ScrollView::new(song_selection).fixed_width(32);
    return scroll_view;
}

pub fn get_total_tracks_input(total_tracks: String) -> ResizedView<LinearLayout> {
    LinearLayout::horizontal()
        .child(DummyView.full_width())
        .child(TextView::new("Total Tracks:"))
        .child(DummyView.fixed_width(11))
        .child(
            EditView::new()
                .content(total_tracks)
                .on_edit(on_total_tracks_edit)
                .with_name("total_tracks")
                .fixed_width(8),
        )
        .fixed_width(65)
}

fn on_total_tracks_edit(siv: &mut Cursive, text: &str, _: usize) {
    let total_tracks = text
        .chars()
        .filter(|c| c.is_ascii_digit())
        .fold(String::new(), |x, y| x + &y.to_string());

    siv.call_on_name("total_tracks", |view: &mut EditView| {
        view.set_content(&total_tracks);
    });

    siv.call_on_name("songlist", |v: &mut SelectView<Value>| {
        for (_, song) in v.iter_mut() {
            song["songinfo"]["total_tracks"] = total_tracks.parse::<u32>().unwrap().into();
        }
    });
}

fn get_song_field_edit_view(
    first_song: &Value,
    field: &str,
    on_edit_alt: Option<Box<dyn Fn(&mut Cursive, &str) -> ()>>,
) -> Result<NamedView<EditView>> {
    let cloned_field = field.to_owned();

    let view = EditView::new()
        .content(get_song_field(first_song, field)?)
        .on_edit(move |siv, text, _| match &on_edit_alt {
            None => set_song_field!(siv, &cloned_field, text.to_string()),
            Some(func) => func(siv, text),
        })
        .with_name(field);
    Ok(view)
}

// Extensive refactoring has gone into this function to make it as legible as possible. I'm not entirely happy with it yet so feel free to improve it
pub fn get_song_metadata_layout(first_song: &Value) -> Result<LinearLayout> {
    let header = 
        ResizedView::with_fixed_height(
            3,
            TextView::new(&song_to_string(first_song))
                .center()
                .with_name("title_text"),
    );

    let title_edit_view = 
        get_song_field_edit_view(
            first_song,
            "title",
            Some(Box::new(
                |siv: &mut Cursive, text: &str| {
                    set_song_field!(siv, "title", text.to_string());
                    // Refresh here to show title changes in the list
                    refresh_songlist(siv);
                }
            )),
        )
    ?;

    let year_edit_view =
        get_song_field_edit_view(first_song, "year", Some(Box::new(year_edit_callback)))?;

    let layout = LinearLayout::vertical()
        .child(header)
        .child(DummyView.fixed_height(1))
        .child(TextView::new("Title"))
        .child(title_edit_view)
        .child(TextView::new("Album"))
        .child(get_song_field_edit_view(first_song, "album", None)?)
        .child(TextView::new("Artist"))
        .child(get_song_field_edit_view(first_song, "artist", None)?)
        .child(TextView::new("Year"))
        .child(year_edit_view)
        .child(TextView::new("Genre"))
        .child(get_song_field_edit_view(first_song, "genre", None)?)
        .child(DummyView.fixed_height(1))
        .child(get_track_no_layout(first_song)?);
    Ok(layout)
}

pub fn year_edit_callback(siv: &mut Cursive, text: &str) {
    let year = text
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(4)
        .fold(String::new(), |x, y| x + &y.to_string());

    siv.call_on_name("year", |view: &mut EditView| {
        view.set_content(&year);
    });

    set_song_field!(siv, "year", year.parse::<u32>().unwrap());
}

pub fn get_track_no_layout(first_song: &Value) -> Result<ResizedView<LinearLayout>> {
    let layout = LinearLayout::horizontal()
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
                    set_song_field!(siv, "track_no", track.parse::<u32>().ok());
                    refresh_songlist(siv);
                })
                .on_submit(|siv, _text| {
                    siv.focus_name("songlist").unwrap();
                })
                .with_name("track_no")
                .fixed_width(8),
        )
        .full_width();
    Ok(layout)
}
