use cursive::{
    view::Nameable,
    views::{EditView, LinearLayout, TextView},
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
