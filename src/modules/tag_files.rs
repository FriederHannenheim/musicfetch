use std::sync::{Mutex, Arc};

use lofty::{read_from_path, TaggedFileExt, Tag, Accessor, TagExt};
use serde_json::Value;
use crate::{modules::download::DownloadModule, module_util::song_to_string};
use super::Module;
use anyhow::Result;


pub struct TagModule;

impl Module for TagModule {
    fn name() -> String {
        String::from("tag_files")
    }

    fn deps() -> Vec<String> {
        vec![DownloadModule::name()]
    }

    fn run(_global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        let songs = songs.lock().unwrap();
        let songs = songs.as_array().unwrap();

        for song in songs {
            tag_song(song)?;
        }

        Ok(())
    }
}

// TODO: Option to wipe old tags
fn tag_song(song: &Value) -> Result<()> {
    let mut tag = get_song_tag(song)?;

    if let Value::String(title) = &song["songinfo"]["title"] {
        tag.set_title(title.clone());
    }
    if let Value::String(album) = &song["songinfo"]["album"] {
        tag.set_album(album.clone());
    }
    if let Value::String(artist) = &song["songinfo"]["artist"] {
        tag.set_artist(artist.clone());
    }
    if let Value::String(genre) = &song["songinfo"]["genre"] {
        tag.set_genre(genre.clone());
    }
    if let Value::Number(year) = &song["songinfo"]["year"] {
        tag.set_year(year.as_u64().expect("Year is not a u64 int") as u32);
    }
    if let Value::Number(track_no) = &song["songinfo"]["track_no"] {
        tag.set_track(track_no.as_u64().expect("Track No. is not a u64 int") as u32);
    }
    if let Value::Number(total_tracks) = &song["songinfo"]["total_tracks"] {
        tag.set_track_total(total_tracks.as_u64().expect("Total Tracks is not a u64 int") as u32);
    }

    tag.save_to_path(song["songinfo"]["path"].as_str().unwrap())?;

    Ok(())
}

pub fn get_song_tag(song: &Value) -> Result<Tag> {
    let mut tagged_file = read_from_path(song["songinfo"]["path"].as_str().expect(&format!("song '{}' has no path", song_to_string(song))))?;
            
    let tag = match tagged_file.primary_tag_mut() {
        Some(primary_tag) => primary_tag,
        None => {
            if let Some(first_tag) = tagged_file.first_tag_mut() {
                first_tag
            } else {
                let tag_type = tagged_file.primary_tag_type();
        
                eprintln!("WARN: No tags found, creating a new tag of type `{tag_type:?}`");
                tagged_file.insert_tag(Tag::new(tag_type));
        
                tagged_file.primary_tag_mut().unwrap()
            }
        },
    }.to_owned();
    Ok(tag)
}