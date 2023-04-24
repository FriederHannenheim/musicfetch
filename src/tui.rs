use crate::structs::Song;
use crate::tagging::UnwrapString;
use cursive::{
    direction::Direction,
    view::{Nameable, Resizable},
    views::{DummyView, EditView, LinearLayout, ResizedView, SelectView, TextView},
    Cursive, View,
};

pub fn get_album_metadata_layout() -> LinearLayout {
    LinearLayout::vertical()
        .child(TextView::new("Album Title"))
        .child(EditView::new().with_name("album"))
        .child(TextView::new("Artist"))
        .child(EditView::new().with_name("artist"))
        .child(TextView::new("Year"))
        .child(
            EditView::new()
                .on_edit(|s, t, _| {
                    s.call_on_name("year", |view: &mut EditView| {
                        view.set_content(
                            t.chars()
                                .filter(|c| c.is_ascii_digit())
                                .take(4)
                                .fold(String::new(), |x, y| x + &y.to_string()),
                        );
                    });
                })
                .with_name("year"),
        )
        .child(TextView::new("Genre"))
        .child(EditView::new().with_name("genre"))
}

macro_rules! set_song_data {
    ($siv:expr, $field:ident, $content:expr) => {
        $siv.call_on_name("songlist", |v: &mut SelectView<Song>| {
            let Some(selected) = v.selected_id() else { return; };
            let Some((_label, song)) = v.get_item_mut(selected) else { return; };
            song.song_metadata.$field = $content;
        })
        .unwrap();
    };
}

pub fn get_song_metadata_layout(first_song: &Song) -> LinearLayout {
    LinearLayout::vertical()
        .child(ResizedView::with_fixed_height(
            3,
            TextView::new(&String::from(first_song))
                .center()
                .with_name("title_text"),
        ))
        .child(DummyView.fixed_height(1))
        .child(TextView::new("Title"))
        .child(
            EditView::new()
                .content(first_song.song_metadata.title.clone().unwrap_or_default())
                .on_edit(|siv, text, _cursor| {
                    set_song_data!(siv, title, Some(text.to_string()));
                    // Refresh here to show title changes in the list
                    refresh_songlist(siv);
                })
                .with_name("title"),
        )
        .child(TextView::new("Album"))
        .child(
            EditView::new()
                .content(first_song.song_metadata.album.clone().unwrap_or_default())
                .on_edit(|siv, text, _cursor| {
                    set_song_data!(siv, album, Some(text.to_string()));
                })
                .with_name("album"),
        )
        .child(TextView::new("Artist"))
        .child(
            EditView::new()
                .content(first_song.song_metadata.artist.clone().unwrap_or_default())
                .on_edit(|siv, text, _cursor| {
                    set_song_data!(siv, artist, Some(text.to_string()));
                })
                .with_name("artist"),
        )
        .child(TextView::new("Year"))
        .child(
            EditView::new()
                .on_edit(|siv, t, _| {
                    let year = t
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .take(4)
                        .fold(String::new(), |x, y| x + &y.to_string());
                    siv.call_on_name("year", |view: &mut EditView| {
                        view.set_content(&year);
                    });
                    set_song_data!(siv, year, year.parse::<u32>().ok());
                })
                .content(first_song.song_metadata.year.clone().unwrap_string())
                .with_name("year"),
        )
        .child(TextView::new("Genre"))
        .child(
            EditView::new()
                .content(first_song.song_metadata.genre.clone().unwrap_or_default())
                .on_edit(|siv, text, _cursor| {
                    set_song_data!(siv, genre, Some(text.to_string()));
                })
                .with_name("genre"),
        )
        .child(DummyView.fixed_height(1))
        .child(
            LinearLayout::horizontal()
                .child(TextView::new("Track No:").max_width(9))
                .child(DummyView.full_width())
                .child(
                    EditView::new()
                        .content(first_song.song_metadata.track_no.clone().unwrap_string())
                        .on_edit(|siv, text, _cursor| {
                            let track = text
                                .chars()
                                .filter(|c| c.is_ascii_digit())
                                .fold(String::new(), |x, y| x + &y.to_string());
                            siv.call_on_name("track", |view: &mut EditView| {
                                view.set_content(&track);
                            });
                            set_song_data!(siv, track_no, track.parse::<u32>().ok());
                            refresh_songlist(siv);
                        })
                        .on_submit(|siv, _text| {
                            siv.focus_name("songlist").unwrap();
                        })
                        .with_name("track")
                        .fixed_width(8),
                )
                .full_width(),
        )
}

fn refresh_songlist(siv: &mut Cursive) {
    siv.call_on_name("songlist", |v: &mut SelectView<Song>| {
        // Get the currently selected song
        let sel = v
            .selection()
            .expect("Either the list of songs is empty or something has gone horribly wrong");

        v.sort_by(|s, t| {
            s.song_metadata
                .track_no
                .unwrap_or(u32::MAX)
                .cmp(&t.song_metadata.track_no.unwrap_or(u32::MAX))
        });
        for (label, song) in v.iter_mut() {
            label.remove_spans(0..1);
            label.compact();
            label.append(format!(
                "{} {}",
                song.song_metadata.track_no.unwrap_string(),
                String::from(&song.to_owned())
            ));
        }

        // Find the position of the edited song after the sort
        let pos = v
            .iter()
            .position(|(_itm, song)| song.path == sel.as_ref().path);
        // If the song is found, select it. If not give focus to the SelectView
        if let Some(pos) = pos {
            v.set_selection(pos);
        } else {
            v.take_focus(Direction::none()).unwrap();
        }
    });
}
