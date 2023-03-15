# Musicfetch
Musicfetch is a YouTube music downloader which allows you to interactively tag your music. This project is engineered to fit my needs but it might just fit yours too. This tool probably isn't for you if you are an Audiophile. Songs are downloaded from Youtube as a 133 kb/s opus and then recoded to mp3. Most people (including me) won't notice but if you will don't bother with this tool (or open a PR/Suggest another way to get songs with better quality). You can also download from Bandcamp, then the songs will be 128 kb/s mp3's and won't need to be reencoded. 
### Description
- Uses Id3v2.3 tags because 2.4 isn't supported anywhere
    - Because of this multiple artists/genres aren't supported
- Id3 frames supported:
    - Title
    - Album
    - Artist
    - Year
    - Genre
    - Track No.
    - Total Tracks

This crate uses many experimental features because I like living on the edge and thus can only compiled with the nightly toolchain.

## Notes
- ID3v1 tags will be automatically upgraded to ID3v2

## Usage
    Usage: musicfetch [OPTIONS] <URL|--files <FILES>...|--yt-dlp-json <FILE>>
    
    Arguments:
      [URL]  url of a song or a album playlist
    
    Options:
      -f, --files <FILES>...         Instead of downloading, tag these local files
      -j, --yt-dlp-json <FILE>       Path to read yt-dlp json from or "-" for stdin
      -c, --cover-url <COVER_URL>    url for the cover image
      -a, --album                    Enable album mode. Artist, Album, Year, Genre will be queried at the start and set for all tracks. Track Number and Total Tracks will be set automatically
      -o, --output-dir <OUTPUT_DIR>  [default: ./]
          --no-rename                Don't rename songs
          --rename                   Rename songs to their titles [default]
      -h, --help                     Print help information
      -V, --version                  Print version information

## Dependencies
- yt-dlp
## Common Errors:
#### Error splitting the argument list: Option not found
Your version of ffmpeg lacks the movflags option. Update ffmpeg.
This can also happen if your IDE is installed as a Flatpak and you're running the command inside the IDE terminal.
