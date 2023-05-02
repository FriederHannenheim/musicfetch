tui for tagging
man page
Allow splitting album videos

# Implement UI fully
- Total Tracks field
- Musicfetch banner

## Make the system more modular
### The Dict, or 2 Dicts
One dict with global values and one with values for each song. Maybe use Serde json values for the dict
Song dict will look like this:
```json
"songs": [
    {
        "yt_dlp": {
            ...
        },
        "songinfo": {
            "title": "Never Gonna Give You Up",
            "artist": "Rick Astley",
        }
    }
]
```
Global dict will look like this:
```json
"args": [
    ...
],
"config": [

]
```
Musicfetch directory in which the user can put multiple configs. Config can be selected using --config

### Ideas for modules:
- jsonfetch: Inserts the yt-dlp json for each song into the dict
- Yt-Title to Song Title
- Custom script: Runs a user-specified script which will get the dict passed as json in stdin and needs to return the dict as json in the stdout
- Song select: Remove certain songs that should not be downloaded
- Discogs / Musicbrainz metadata download

### Other Ideas
- You can do `--help <module>` to get a description for each module with their settings