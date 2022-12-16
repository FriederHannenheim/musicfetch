use std::error::Error;

use id3::{Tag, TagLike, Version};
use id3::frame::Picture;

use dialoguer::{Input, Confirm};

use crate::structs::{Song, SongMetadata};

pub fn tag_song(mut song: Song, cover: Option<Picture>) -> Result<Song, Box<dyn Error>> {
    let mut tag = song.tag.unwrap_or(Tag::new());

    add_metadata_to_tag(&song.song_metadata, &mut tag);

    if let Some(picture) = cover {
        tag.add_frame(picture);
    }

    loop {
        println!("\nMetadata for {}", &song.path.display());
        metadata_prompt(&mut tag)?;
        if Confirm::new().with_prompt("Metadata correct?").default(true).interact()? {
            break;
        }
    }
    
    tag.write_to_path(&song.path, Version::Id3v23).expect("Writing Id3 tag failed");

    song.tag = Some(tag);

    Ok(song)
}

fn add_metadata_to_tag(metadata: &SongMetadata, tag: &mut Tag) {
    if let Some(title) = &metadata.title { tag.set_title(title); }
    if let Some(album) = &metadata.album { tag.set_album(album); }
    if let Some(artist) = &metadata.artist { tag.set_artist(artist); }
    if let Some(year) = metadata.year { tag.set_year(year); }
    if let Some(genre) = &metadata.genre { tag.set_genre(genre); }

    // If the tag already has track_no/total_tracks ignore the generated one
    if let Some(track_no) = metadata.track_no && !tag.track().is_some() { tag.set_track(track_no); }
    if let Some(total_tracks) = metadata.total_tracks && !tag.total_tracks().is_some() { tag.set_total_tracks(total_tracks); }
}

fn metadata_prompt(tag: &mut Tag) -> Result<(), Box<dyn Error>> {
    let title: String = 
        Input::new()
            .with_prompt("Title")
            .with_initial_text(tag.title().unwrap_or(""))
            .interact_text()?;
    let album: String = 
        Input::new()
            .with_prompt("Album")
            .with_initial_text(tag.album().unwrap_or(""))
            .allow_empty(true)
            .interact_text()?;
    let artist: String = 
        Input::new()
            .with_prompt("Artist")
            .with_initial_text(tag.artist().unwrap_or(""))
            .interact_text()?;
    let year: i32 = 
        Input::new()
            .with_prompt("Year")
            .with_initial_text(to_string_or_empty(tag.year()))
            .interact_text()?;
    let genre: String = 
        Input::new()
            .with_prompt("Genre")
            .with_initial_text(tag.genre().unwrap_or(""))
            .allow_empty(true)
            .interact_text()?;
    let track: u32 = 
        Input::new()
            .with_prompt("Track No.")
            .with_initial_text(to_string_or_empty(tag.track()))
            .interact_text()?;
    let total_tracks: u32 = 
        Input::new()
            .with_prompt("Total Tracks")
            .with_initial_text(to_string_or_empty(tag.total_tracks()))
            .interact_text()?;
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
