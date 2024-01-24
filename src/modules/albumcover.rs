use std::sync::{Arc, Mutex};

use anyhow::Result;
use lofty::{MimeType, Picture, PictureType, TagExt};
use serde_json::Value;

use crate::{
    define_module,
    modules::{tag_files::get_song_tag, ModuleStruct},
};

define_module!("albumcover", run, []);

fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
    let Some(cover_url) = get_cover_url(global) else {
        // No cover url was given, return
        return Ok(());
    };

    let resp = minreq::get(cover_url)
        .send()
        .expect("Sending http request for cover image failed");
    let kind = infer::get(resp.as_bytes()).expect("Filetype for cover not found");

    let picture = Picture::new_unchecked(
        PictureType::CoverFront,
        MimeType::from_str(kind.mime_type()),
        None,
        resp.into_bytes(),
    );

    let songs = songs.lock().unwrap();
    let songs = songs.as_array().unwrap();

    for song in songs {
        let mut tag = get_song_tag(song)?;

        tag.push_picture(picture.clone());

        tag.save_to_path(song["songinfo"]["path"].as_str().unwrap())?;
    }

    Ok(())
}

fn get_cover_url(global: Arc<Mutex<Value>>) -> Option<String> {
    let global = global.lock().unwrap();
    let global = global.as_object().unwrap();

    global["args"]["cover_url"].as_str().map(|c| c.to_owned())
}

