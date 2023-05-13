# Musicfetch
Musicfetch is a tool for downloading music from Youtube and other platforms. It allows you to add metadata to your music which will then be displayed in your music player of choice. 

![GitHub](https://img.shields.io/github/license/FriederHannenheim/Musicfetch?logo=gnu)

This is the code for the rework. Versions < v1.0 can be found on the branch `old`

## Supported Song metadata:
- Title
- Album Title
- Artist
- Year released
- Genre
- Track Number
- Total Tracks

## Compiling
Select the nightly Rust toolchain and enter `cargo build --release`. To install musicfetch enter `cargo install --path .`

## Usage
This is the branch for musicfetch >= v1.0. v1.0 was rebuilt from the ground up with massive changes under the hood. Right now it is in it's alpha stage, the core functionality is there but there are still things to be done to make it actually usable.

You can try it now by placing the `config_example.toml` under `/etc/musicfetch.toml` and invoking musicfetch with a link to an Album on youtube.

In the UI for tagging you can use '+' or '-' to increment or decrement the track number of the current song. You can also use number keys 1-9 to set it directly.

## Dependencies
- [yt-dlp](https://github.com/yt-dlp/yt-dlp)

## Errors Explained
`Error in module rename:: Song '' has no field '' or field is empty` - The rename module tried inserting a song field into the filename but it was empty. Try running musicfetch again and ensuring that all fields are set