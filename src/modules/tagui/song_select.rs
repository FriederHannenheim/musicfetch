use cursive::{
    view::Nameable,
    views::{EditView, NamedView, ScrollView, SelectView, TextView},
    Cursive,
};
use serde_json::Value;

use crate::module_util::song_to_string;

use super::util::get_song_field;

pub fn create_song_select_view(
    songs: &Vec<Value>,
) -> ScrollView<NamedView<SelectView<Value>>> {
    let cloned = songs.clone();
    let song_names = cloned.iter().map(|f| song_to_string(f));

    let selectview_items = song_names
        .zip(songs.clone().into_iter())
        .map(|(label, song)| {
            (
                format!(
                    "{} {}",
                    get_song_field(&song, "track_no").unwrap_or_default(),
                    label
                ),
                song,
            )
        });

    let song_selection = SelectView::new()
        .with_all(selectview_items)
        .on_select(|siv, _| update_edit_views(siv))
        .with_name("songlist");

    let scroll_view = ScrollView::new(song_selection);
    return scroll_view;
}

/// Updates the contents of the edit views to match the song selected
pub fn update_edit_views(siv: &mut Cursive) {
    let Some(song) = siv.call_on_name("songlist", |list: &mut SelectView<Value>| {
        return list.selection()
    }).unwrap() else { return; };

    siv.call_on_name("title_text", |v: &mut TextView| {
        v.set_content(song_to_string(&song));
    })
    .unwrap();

    for field in ["title", "album", "artist", "year", "genre", "track_no"] {
        let content = get_song_field(&song, field).unwrap_or_default();
        let result = siv.call_on_name(field, |v: &mut EditView| {
            v.set_content(content);
        });
        if result.is_none() {
            panic!("Cursive field {} does not exist", field);
        }
    }
}
