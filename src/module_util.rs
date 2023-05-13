use serde_json::Value;
use anyhow::{Result, anyhow};


pub fn song_to_string(song: &Value) -> String {
    match song["songinfo"].get("title") {
        Some(song_name) => song_name,
        None => &song["yt_dlp"]["title"],
    }
    .as_str()
    .expect("Failed creating string from song")
    .to_owned()
}

pub fn get_songinfo_field<'a,T: ?Sized>(song: &'a Value, field: &str) -> Result<&'a T> 
    where &'a Value: TryInto<&'a T> 
{
    match song["songinfo"].get(field) {
        Some(v) => match v.try_into() {
            Ok(v) => Ok(v),
            Err(_) => Result::Err(anyhow!("Error getting song field '{}' for song '{}' field has wrong type", field, song_to_string(song)))
        },
        None => Result::Err(anyhow!("Error getting song field '{}' for song '{}' field does not exist", field, song_to_string(song)))
    }
}