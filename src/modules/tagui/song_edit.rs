use cursive::{
    view::{Nameable, Resizable},
    views::{DummyView, EditView, LinearLayout, NamedView, ResizedView, TextView},
    Cursive,
};
use serde_json::Value;

use anyhow::Result;

use crate::module_util::song_to_string;

use super::{
    refresh_songlist,
    util::{get_song_field, remove_non_numeric_chars, set_song_field},
};

fn create_edit_view_for_song_field(
    first_song: &Value,
    field: &str,
    on_edit_alt: Option<Box<dyn Fn(&mut Cursive, &str) -> ()>>,
) -> Result<NamedView<EditView>> {
    let cloned_field = field.to_owned();

    let field_content = get_song_field(first_song, field).unwrap_or_default();

    let view = EditView::new()
        .content(field_content)
        .on_edit(move |siv, text, _| match &on_edit_alt {
            None => set_song_field(siv, &cloned_field, Value::from(text.to_string())),
            Some(func) => func(siv, text),
        })
        .with_name(field);
    Ok(view)
}

// Extensive refactoring has gone into this function to make it as legible as possible. I'm not entirely happy with it yet so feel free to improve it
pub fn create_song_edit_layout(first_song: &Value) -> Result<LinearLayout> {
    let header = ResizedView::with_fixed_height(
        3,
        TextView::new(&song_to_string(first_song))
            .center()
            .with_name("title_text"),
    );

    let title_edit_view = create_edit_view_for_song_field(
        first_song,
        "title",
        Some(Box::new(|siv: &mut Cursive, text: &str| {
            set_song_field(siv, "title", Value::from(text.to_string()));
            // Refresh here to show title changes in the list
            refresh_songlist(siv);
        })),
    )?;

    let year_edit_view =
        create_edit_view_for_song_field(first_song, "year", Some(Box::new(year_edit_callback)))?;

    let layout = LinearLayout::vertical()
        .child(header)
        .child(DummyView.fixed_height(1))
        .child(TextView::new("Title"))
        .child(title_edit_view)
        .child(TextView::new("Album"))
        .child(create_edit_view_for_song_field(first_song, "album", None)?)
        .child(TextView::new("Artist"))
        .child(create_edit_view_for_song_field(first_song, "artist", None)?)
        .child(TextView::new("Year"))
        .child(year_edit_view)
        .child(TextView::new("Genre"))
        .child(create_edit_view_for_song_field(first_song, "genre", None)?)
        .child(DummyView.fixed_height(1))
        .child(create_track_no_input(
            &get_song_field(first_song, "track_no").unwrap_or_default(),
        ));
    Ok(layout)
}

pub fn year_edit_callback(siv: &mut Cursive, text: &str) {
    let year = &remove_non_numeric_chars(text)
        .chars()
        .take(4)
        .collect::<String>();

    siv.call_on_name("year", |view: &mut EditView| {
        view.set_content(year);
    });
    set_song_field(siv, "year", Value::from(year.parse::<u64>().ok()));
}

pub fn create_track_no_input(
    content: &str,
) -> ResizedView<LinearLayout> {
    LinearLayout::horizontal()
        .child(TextView::new("Track No."))
        .child(DummyView.full_width())
        .child(
            EditView::new()
                .content(content)
                .on_edit(move |siv, text, _cursor| {
                    let track_no = remove_non_numeric_chars(text);

                    siv.call_on_name("track_no", |view: &mut EditView| {
                        view.set_content(&track_no);
                    });
                    set_song_field(siv, "track_no", Value::from(track_no.parse::<u64>().ok()));

                    refresh_songlist(siv);
                })
                .with_name("track_no")
                .fixed_width(8),
        )
        .fixed_height(1)
}
