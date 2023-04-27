use std::error::Error;
use std::fmt::Display;

use cursive::theme::Theme;
use cursive::view::{Nameable, Resizable};
use cursive::views::{
    Button, Dialog, DummyView, EditView, LinearLayout, ResizedView, ScrollView, SelectView,
    TextView,
};
use cursive::{Cursive, CursiveExt};
use lofty::{Accessor, Picture, Tag, TagExt};

use dialoguer::{Confirm, Input};

use crate::structs::{Song, SongMetadata};
use crate::tui::get_song_metadata_layout;
use crate::Args;

pub fn tag_song(
    mut song: Song,
    cover: Option<Picture>,
    settings: &Args,
) -> Result<Song, Box<dyn Error>> {
    let mut tag = song.tag;

    add_metadata_to_tag(&song.song_metadata, &mut tag);

    if !settings.yes {
        tag_with_input(&mut tag, &song.path.display().to_string())?;
    }

    if let Some(cover) = cover {
        tag.push_picture(cover);
    };

    tag.save_to_path(&song.path)
        .expect("Writing Id3 tag failed");

    song.tag = tag;

    Ok(song)
}

pub fn add_metadata_to_tag(metadata: &SongMetadata, tag: &mut Tag) {
    if let Some(title) = &metadata.title {
        tag.set_title(title.clone());
    }
    if let Some(album) = &metadata.album {
        tag.set_album(album.clone());
    }
    if let Some(artist) = &metadata.artist {
        tag.set_artist(artist.clone());
    }
    if let Some(year) = metadata.year {
        tag.set_year(year);
    }
    if let Some(genre) = &metadata.genre {
        tag.set_genre(genre.clone());
    }

    // If the tag already has track_no/total_tracks ignore the generated one
    if let Some(track_no) = metadata.track_no && !tag.track().is_some() { tag.set_track(track_no); }
    if let Some(total_tracks) = metadata.total_tracks && !tag.track_total().is_some() { tag.set_track_total(total_tracks); }
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
    let title: String = prompt("Title", false, tag.title().unwrap_or_default().to_string())?;
    let album: String = prompt("Album", false, tag.album().unwrap_or_default().to_string())?;
    let artist: String = prompt(
        "Artist",
        false,
        tag.artist().unwrap_or_default().to_string(),
    )?;
    let year: u32 = prompt("Year", false, to_string_or_empty(tag.year()))?;
    let genre: String = prompt("Genre", true, tag.genre().unwrap_or_default().to_string())?;
    let track: u32 = prompt("Track No.", false, to_string_or_empty(tag.track()))?;
    let total_tracks: u32 = prompt("Total Tracks", false, to_string_or_empty(tag.track_total()))?;
    tag.set_title(title);
    tag.set_album(album);
    tag.set_artist(artist);
    tag.set_year(year);
    tag.set_genre(genre);
    tag.set_track(track);
    tag.set_track_total(total_tracks);

    Ok(())
}

fn to_string_or_empty<T: ToString>(option: Option<T>) -> String {
    if let Some(value) = option {
        value.to_string()
    } else {
        String::new()
    }
}

fn prompt<T: std::fmt::Display + Clone + std::str::FromStr>(
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

macro_rules! set_content_for_field {
    ($siv:expr, $field:expr, $content:expr) => {
        $siv.call_on_name($field, |v: &mut EditView| {
            v.set_content($content);
        })
        .expect(&format!("No cursive field with name '{}'", $field));
    };
}

fn set_cursive_fields_for_song(s: &mut Cursive, song: &Song) {
    s.call_on_name("title_text", |v: &mut TextView| {
        v.set_content(&String::from(song));
    })
    .unwrap();
    set_content_for_field!(
        s,
        "title",
        song.song_metadata.title.clone().unwrap_or_default()
    );
    set_content_for_field!(
        s,
        "album",
        song.song_metadata.album.clone().unwrap_or_default()
    );
    set_content_for_field!(
        s,
        "artist",
        song.song_metadata.artist.clone().unwrap_or_default()
    );
    set_content_for_field!(s, "year", song.song_metadata.year.unwrap_string());
    set_content_for_field!(
        s,
        "genre",
        song.song_metadata.genre.clone().unwrap_or_default()
    );
    set_content_for_field!(s, "track", song.song_metadata.track_no.unwrap_string());
}

pub fn tag_songs_tui(songs: &mut Vec<Song>) {
    let mut siv = Cursive::default();

    siv.set_theme(Theme::terminal_default());

    let cloned = songs.clone();
    let filenames = cloned.iter().map(|f| String::from(f));

    let song_selection = SelectView::new()
        .with_all(
            filenames
                .zip(songs.clone().into_iter())
                .map(|(label, song)| {
                    (
                        format!("{} {}", song.song_metadata.track_no.unwrap_string(), label),
                        song,
                    )
                }),
        )
        .on_select(|s, song: &Song| {
            set_cursive_fields_for_song(s, song);
        })
        .with_name("songlist");

    let scroll_view = ScrollView::new(song_selection).fixed_width(32);

    siv.add_layer(Dialog::around(
        LinearLayout::vertical()
            .child(TextView::new("Edit Tags").center())
            .child(DummyView.fixed_height(1))
            .child(
                LinearLayout::horizontal()
                    .child(scroll_view)
                    .child(DummyView.fixed_width(1))
                    .child(ResizedView::with_fixed_width(
                        32,
                        get_song_metadata_layout(&songs[0]),
                    )),
            )
            .child(
                LinearLayout::horizontal()
                    .child(DummyView.full_width())
                    .child(TextView::new("Total Tracks:"))
                    .child(DummyView.fixed_width(11))
                    .child(
                        EditView::new()
                            .content(
                                songs[0].song_metadata.total_tracks.unwrap_string()
                            )
                            .on_edit(|siv, text, _cursor| {
                                let total_tracks = text
                                    .chars()
                                    .filter(|c| c.is_ascii_digit())
                                    .fold(String::new(), |x, y| x + &y.to_string());
                                
                                siv.call_on_name("total_tracks", |view: &mut EditView| {
                                    view.set_content(&total_tracks);
                                });
                                
                                siv.call_on_name("songlist", |v: &mut SelectView<Song>| {
                                    for (_lbl, song) in v.iter_mut() {
                                        song.song_metadata.total_tracks = total_tracks.parse::<u32>().ok();
                                    }
                                });
                            })
                            .with_name("total_tracks")
                            .fixed_width(8)
                    )
                    .fixed_width(65),
            )
            .child(Button::new("Save", |siv| siv.quit())),
    ));

    siv.run_crossterm()
        .expect("TUI initialization failed. Try using another Terminal");
}

pub trait UnwrapString {
    /// Returns the value as a string or "" if None
    fn unwrap_string(self) -> String;
}

impl<T> UnwrapString for Option<T>
where
    T: Display,
{
    fn unwrap_string(self) -> String {
        let unwrapped = if let Some(val) = self {
            val.to_string()
        } else {
            String::from("")
        };
        unwrapped
    }
}
