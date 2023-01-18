use std::error::Error;

use id3::frame::Picture;
use id3::{Tag, TagLike, Version};

use dialoguer::{Confirm, Input};

use crate::structs::{Song, SongMetadata};
use crate::Args;

pub fn tag_song(
    mut song: Song,
    cover: Option<Picture>,
    settings: &Args,
) -> Result<Song, Box<dyn Error>> {
    let mut tag = song.tag.unwrap_or(Tag::new());

    add_metadata_to_tag(&song.song_metadata, &mut tag);

    if !settings.yes {
        tag_with_input(&mut tag, &song.path.display().to_string())?;
    }

    if let Some(picture) = cover {
        tag.add_frame(picture);
    }

    tag.write_to_path(&song.path, Version::Id3v23)
        .expect("Writing Id3 tag failed");

    song.tag = Some(tag);

    Ok(song)
}

fn add_metadata_to_tag(metadata: &SongMetadata, tag: &mut Tag) {
    if let Some(title) = &metadata.title {
        tag.set_title(title);
    }
    if let Some(album) = &metadata.album {
        tag.set_album(album);
    }
    if let Some(artist) = &metadata.artist {
        tag.set_artist(artist);
    }
    if let Some(year) = metadata.year {
        tag.set_year(year);
    }
    if let Some(genre) = &metadata.genre {
        tag.set_genre(genre);
    }

    // If the tag already has track_no/total_tracks ignore the generated one
    if let Some(track_no) = metadata.track_no && !tag.track().is_some() { tag.set_track(track_no); }
    if let Some(total_tracks) = metadata.total_tracks && !tag.total_tracks().is_some() { tag.set_total_tracks(total_tracks); }
}

fn tag_with_input(tag: &mut Tag, path: &str) -> Result<(), Box<dyn Error>> {
    loop {
        println!("\n{}", path);
        metadata_prompt(tag)?;
        if Confirm::new()
            .with_prompt("Metadata correct?")
            .default(true)
            .interact()?
        {
            break;
        }
    }
    Ok(())
}

fn metadata_prompt(tag: &mut Tag) -> Result<(), Box<dyn Error>> {
    let title: String = prompt("Title", false, tag.title().unwrap_or_default().to_owned())?;
    let album: String = prompt("Album", false, tag.album().unwrap_or_default().to_owned())?;
    let artist: String = prompt("Artist", false, tag.artist().unwrap_or_default().to_owned())?;
    let year: i32 = prompt("Year", false, to_string_or_empty(tag.year()))?;
    let genre: String = prompt("Genre", true, tag.genre().unwrap_or_default().to_owned())?;
    let track: u32 = prompt("Track No.", false, to_string_or_empty(tag.track()))?;
    let total_tracks: u32 = prompt(
        "Total Tracks",
        false,
        to_string_or_empty(tag.total_tracks()),
    )?;
    tag.set_title(title);
    tag.set_album(album);
    tag.set_artist(artist);
    tag.set_year(year);
    tag.set_genre(genre);
    tag.set_track(track);
    tag.set_total_tracks(total_tracks);

    Ok(())
}

fn to_string_or_empty<T: ToString>(option: Option<T>) -> String {
    if let Some(value) = option {
        value.to_string()
    } else {
        String::new()
    }
}

pub fn prompt<T: std::fmt::Display + Clone + std::str::FromStr>(
    prompt: &str,
    allow_empty: bool,
    initial_text: String,
) -> Result<T, std::io::Error>
where
    <T as std::str::FromStr>::Err: std::fmt::Display,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let mut input = Input::new();
    input
        .with_prompt(prompt)
        .allow_empty(allow_empty)
        .with_initial_text(initial_text);
    input.interact_text()
}
