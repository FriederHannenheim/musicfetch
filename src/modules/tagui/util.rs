use std::cmp::Ordering;

use anyhow::{bail, Result};
use cursive::{
    views::{EditView, TextView},
    Cursive,
};
use serde_json::Value;

pub fn get_song_field(song: &Value, field: &str) -> Result<String> {
    let field_value = match song["songinfo"].get(field).unwrap() {
        Value::String(string) => string.to_owned(),
        Value::Number(number) => match number.as_i64().unwrap() {
            0 => String::new(),
            num => num.to_string(),
        },
        _ => bail!("Invalid Value type in songinfo field {}", field),
    };
    Ok(field_value)
}

pub fn set_cursive_fields_for_song(s: &mut Cursive, song: &Value) {
    s.call_on_name("title_text", |v: &mut TextView| {
        v.set_content(song_to_string(song));
    })
    .unwrap();

    for field in ["title", "album", "artist", "year", "genre", "track_no"] {
        let content = get_song_field(song, field).unwrap();
        let result = s.call_on_name(field, |v: &mut EditView| {
            v.set_content(content);
        });
        if result.is_none() {
            panic!("Cursive field {} does not exist", field);
        }
    }
}

pub fn song_to_string(song: &Value) -> String {
    match song["songinfo"].get("title") {
        Some(song_name) => song_name,
        None => &song["yt_dlp"]["title"],
    }
    .as_str()
    .unwrap()
    .to_owned()
}

#[macro_export]
macro_rules! set_song_field {
    ($siv:expr, $field:expr, $content:expr) => {
        $siv.call_on_name("songlist", |v: &mut SelectView<Value>| {
            let Some(selected) = v.selected_id() else { return; };
            let Some((_label, song)) = v.get_item_mut(selected) else { return; };
            song["songinfo"][$field] = Value::from($content);
        })
        .unwrap()
    };
}

pub fn compare_songs_by_track_no(song1: &Value, song2: &Value) -> Ordering {
    let song1_no = song1["songinfo"]
        .get("track_no")
        .map(|v| v.as_u64().unwrap())
        .unwrap_or(u64::MAX);
    let song2_no = song2["songinfo"]
        .get("track_no")
        .map(|v| v.as_u64().unwrap())
        .unwrap_or(u64::MAX);
    song1_no.cmp(&song2_no)
}