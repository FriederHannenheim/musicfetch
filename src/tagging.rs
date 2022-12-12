use std::error::Error;

use regex::Regex;

use id3::{Tag, TagLike, Version};
use id3::frame::{Picture, PictureType};

use dialoguer::{Input, Confirm};

use crate::structs::Song;

pub fn add_metadata(mut song: Song, cover: Option<Picture>) -> Result<(), Box<dyn Error>> {
    song.artist = song.artist.split(",").next().unwrap_or("").to_string();

    loop {
        metadata_prompt(&mut song)?;
        if Confirm::new().with_prompt("Metadata correct?").default(true).interact()? {
            break;
        }
    }
    

    let mut tag = Tag::new();
    tag.set_title(&song.title);
    tag.set_album(&song.album);
    tag.set_year(song.release_year.unwrap());
    tag.set_artist(&song.artist);
    tag.set_genre(&song.genre);
    tag.set_track(song.track_no.unwrap());
    tag.set_total_tracks(song.total_tracks.unwrap());

    if let Some(picture) = cover {
        tag.add_frame(picture);
    }
    
    println!("Adding metadata to {}", &song.filename);
    tag.write_to_path(&song.filename, Version::Id3v23).expect("Writing Id3 tag failed");

    Ok(())
}

pub fn fetch_cover_image(url: &str) -> Picture {
    println!("Downloading cover image...");

    let resp = minreq::get(url).send().expect("Sending http request for cover image failed");
    let mime_type = get_mime_type(url).expect("Failed to find file extension in cover url. Make sure it is a valid image url");

    Picture {
        mime_type: mime_type,
        picture_type: PictureType::CoverFront,
        description: "Cover".to_owned(),
        data: resp.as_bytes().into(),
    }
}

// TODO: check if file extension is an image
fn get_mime_type(url: &str) -> Option<String> {
    let re = Regex::new(r"\.(\w{3,4})(?:$|\?)").unwrap();
    let captures = re.captures(url)?;
    let file_extension = captures.get(1)?.as_str();

    Some(format!("image/{}", file_extension))
}

fn metadata_prompt(song: &mut Song) -> Result<(), Box<dyn Error>> {
    println!("");
    song.title = Input::new()
        .with_prompt("Title")
        .with_initial_text(&song.title)
        .interact_text()?;
    song.album = Input::new()
        .with_prompt("Album")
        .with_initial_text(&song.album)
        .allow_empty(true)
        .interact_text()?;
    song.artist = Input::new()
        .with_prompt("Artist")
        .with_initial_text(&song.artist)
        .interact_text()?;
    song.release_year = Some(Input::new()
        .with_prompt("Year")
        .with_initial_text(to_string_or_empty(song.release_year))
        .interact_text()?);
    song.genre = Input::new()
        .with_prompt("Genre")
        .with_initial_text(&song.genre)
        .allow_empty(true)
        .interact_text()?;
    song.track_no = Some(Input::new()
        .with_prompt("Track No.")
        .with_initial_text(to_string_or_empty(song.track_no))
        .interact_text()?);
    song.total_tracks = Some(Input::new()
        .with_prompt("Total Tracks")
        .with_initial_text(to_string_or_empty(song.total_tracks))
        .interact_text()?);

    Ok(())
}

fn to_string_or_empty<T: ToString>(option: Option<T>) -> String {
    if let Some(value) = option {
        value.to_string()
    } else {
        String::new()
    }
}