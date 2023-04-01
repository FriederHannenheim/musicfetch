use cursive::{
    view::{Nameable, Resizable},
    views::{DummyView, EditView, LinearLayout, ResizedView, TextView},
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

pub fn get_song_metadata_layout(first_title: &str) -> LinearLayout {
    LinearLayout::vertical()
        .child(ResizedView::with_fixed_height(
            3,
            TextView::new(first_title).center().with_name("title_text"),
        ))
        .child(DummyView.fixed_height(1))
        .child(TextView::new("Title"))
        .child(EditView::new().with_name("title"))
        .child(TextView::new("Album"))
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
