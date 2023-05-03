use cursive::{Cursive, views::{EditView, NamedView, LinearLayout, ResizedView, TextView, DummyView}, view::{Nameable, Resizable}};
use serde_json::Value;

use anyhow::Result;

use super::{util::{get_song_field, set_song_field, song_to_string, remove_non_numeric_chars}, refresh_songlist};

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
        .child(
            create_track_no_input(
                &get_song_field(first_song, "track_no")?, 
                "Track No.", 
                "track_no", 
                Box::new(|siv, _| refresh_songlist(siv))
            )?
        );
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

pub fn create_track_no_input(content: &str, label: &str, field: &str, on_edit_extra: Box<dyn Fn(&mut Cursive, String)>) -> Result<ResizedView<LinearLayout>> {
    let _field = field.to_owned();

    let layout = LinearLayout::horizontal()
        .child(TextView::new(label).max_width(9))
        .child(DummyView.full_width())
        .child(
            EditView::new()
                .content(content)
                .on_edit(move|siv, text, _cursor| {
                    let value = remove_non_numeric_chars(text);

                    siv.call_on_name(&_field, |view: &mut EditView| {
                        view.set_content(&value);
                    });
                    set_song_field(siv, &_field, Value::from(value.parse::<u64>().ok()));

                    on_edit_extra(siv, value);
                })
                .with_name(field)
                .fixed_width(8),
        )
        .fixed_width(11);
    Ok(layout)
}