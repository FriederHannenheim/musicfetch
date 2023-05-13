# !!! REMOVE ALL TODOS BEFORE RELEASING REWORK !!!

man page

remove nightly features so it can be compiled with the stable toolchain

Musicfetch banner in tag ui

Musicfetch directory in which the user can put multiple configs. Config can be selected using --config. Configs are able to inherit from eachother. That way configs which only change the settings for a module won't have to include a lot of boilerplate.

### Ideas for modules:
- Custom script: Runs a user-specified script which will get the dict passed as json in stdin and needs to return the dict as json in the stdout
- Song select: Remove certain songs that should not be downloaded
- Discogs / Musicbrainz metadata download

### Other Ideas
- You can do `--help <module>` to get a description for each module with their settings

### Output
Download files to output folder as yt_id.extension and rename in rename module