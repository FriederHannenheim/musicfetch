# Musicfetch
Musicfetch is a YouTube music downloader which allows you to interactively tag your music
## Usage
    musicfetch VIDEO_URL [COVER_URL]
## Dependencies
- yt-dlp
## Common Errors:
#### Error splitting the argument list: Option not found
Your version of ffmpeg lacks the movflags option. Update ffmpeg.
This can also happen if your IDE is installed as a Flatpak and you're running the command inside the IDE terminal.