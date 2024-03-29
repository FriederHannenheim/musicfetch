use std::cmp::Ordering;

use anyhow::{bail, Result};

use cursive::{views::SelectView, Cursive};
use serde_json::{Value, Map};

pub fn get_song_field(song: &Value, field: &str) -> Result<String> {
    let field_value_str = match &song["songinfo"][field] {
        Value::String(string) => string.to_owned(),
        Value::Number(number) => match number.as_i64().unwrap() {
            0 => String::new(),
            num => num.to_string(),
        },
        v => bail!(
            "Invalid Value type in songinfo field {}. Content: {:#}",
            field,
            v
        ),
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

pub fn merge_b_into_a(a: &mut Map<String, Value>, b: Map<String, Value>) {
    for (key, b_value) in b.into_iter() {
        if let Some(mut a_value) = a.get_mut(&key) {
            if let (Value::Object(a_obj), Value::Object(b_obj)) = (&mut a_value, &b_value) {
                merge_b_into_a(a_obj, b_obj.clone());
            } else {
                *a_value = b_value;
            }
        } else {
            a.insert(key, b_value);
        }
    }
}