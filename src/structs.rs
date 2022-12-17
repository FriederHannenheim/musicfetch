use std::path::PathBuf;

use id3::Tag;

use serde::Deserialize;
use serde_json::Value;

#[derive(Debug)]
pub struct Song {
    pub path: PathBuf,
    pub tag: Option<Tag>,
    pub song_metadata: SongMetadata,
}

#[derive(Debug, Deserialize, Default)]
pub struct SongMetadata {
    pub fulltitle: String,
    #[serde(rename = "track")]
    pub title: Option<String>,
    pub album: Option<String>,
    pub artist: Option<String>,
    #[serde(default, rename = "release_year")]
    pub year: Option<i32>,
    #[serde(skip)]
    pub genre: Option<String>,
    #[serde(skip)]
    pub track_no: Option<u32>,
    #[serde(skip)]
    pub total_tracks: Option<u32>,
    pub channel: Option<String>,
    pub description: Option<String>,
}

impl SongMetadata {
    pub fn apply(&mut self, album_metadata: AlbumMetadata) {
        self.album = Some(album_metadata.album_title);
        self.artist = Some(album_metadata.artist);
        self.year = Some(album_metadata.year);
        self.genre = album_metadata.genre;
    }
}

#[derive(Debug, Clone)]
pub struct AlbumMetadata {
    pub album_title: String,
    pub artist: String,
    pub year: i32,
    pub genre: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Playlist {
    pub title: String,
    pub entries: Vec<Value>,
}
