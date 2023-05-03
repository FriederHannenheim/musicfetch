use std::cmp::Ordering;

use anyhow::{bail, Result};

use cursive::{Cursive, views::SelectView};
use serde_json::Value;

pub fn get_song_field(song: &Value, field: &str) -> Result<String> {
    let field_value_str = match &song["songinfo"][field] {
        Value::String(string) => string.to_owned(),
        Value::Number(number) => match number.as_i64().unwrap() {
            0 => String::new(),
            num => num.to_string(),
        },
        v => bail!("Invalid Value type in songinfo field {}. Content: {:#}", field, v),
    };
    Ok(field_value_str)
}

pub fn set_song_field(siv: &mut Cursive, field: &str, value: Value) {
    siv.call_on_name("songlist", |v: &mut SelectView<Value>| {
        let Some(selected) = v.selected_id() else { return; };
        let Some((_label, song)) = v.get_item_mut(selected) else { return; };
        song["songinfo"][field] = value;
    })
    .unwrap();
}

pub fn song_to_string(song: &Value) -> String {
    match song["songinfo"].get("title") {
        Some(song_name) => song_name,
        None => &song["yt_dlp"]["title"],
    }
    .as_str()
    .expect("Failed creating string from song")
    .to_owned()
}


pub fn compare_songs_by_track_no(song1: &Value, song2: &Value) -> Ordering {
    let song1_no = song1["songinfo"]
        .get("track_no")
        .map(|v| v.as_u64().unwrap_or(u64::MAX))
        .unwrap_or(u64::MAX);
    let song2_no = song2["songinfo"]
        .get("track_no")
        .map(|v| v.as_u64().unwrap_or(u64::MAX))
        .unwrap_or(u64::MAX);
    song1_no.cmp(&song2_no)
}

/// Removes all non-numeric characters from a String
pub fn remove_non_numeric_chars(string: &str) -> String {
    string.chars().filter(|c| c.is_ascii_digit()).collect()
}
