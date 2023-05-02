tui for tagging
man page
Allow splitting album videos

# Implement UI fully
- Total Tracks field
- Musicfetch banner

# Ideas
- Tokio, one thread for downloading/tagging etc and one for UI

## Make the system more modular
### The Dict, or 2 Dicts
One dict with global values and one with values for each song. Maybe use Serde json values for the dict
Song dict will look like this:
```json
"songs": [
    {
        "yt_dlp_values": {
            ...
        },
        "tag_values": {
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
]
```
### Workflow
- Central Dict with values is passed to all modules
- First there will be a list of urls in the dict
- Then there may be a module that will download the yt-dlp jsons for all songs
- Then a downloader module which also writes the filenames in the dict
- Some more modules to edit the data
- A tagging module which writes the tags to the files
### Alternate Workflow
- First the json downloader module will do it's thing
- Since we now have all information the user can edit the data in the tui
- after that the downloading of songs / tagging can happen
this way the user doesn't have to wait for too long to edit

### Ideas for modules:
- jsonfetch: Inserts the yt-dlp json for each song into the dict
- title-to-title: Yt-Title to Song Title
- Custom script: Runs a user-specified script which will get the dict passed as json in stdin and needs to return the dict as json in the stdout
- Song select: Remove certain songs that should not be downloaded
- rename: renames the files
- tagger

### Parralelizing
Problem: I want modules to be able to run in parallel but some modules require information from other modules

#### Solution 1: Stages
Each module will have dependencies on modules that need to run before they run. The user can specify which modules he wants to run in stages. It would look like this in the config file:
```toml
[stages]
stage1 = ["jsonfetch"]
stage2 = ["title-to-title"]
stage3 = ["tagtui", "downloader"]
stage4 = ["tagger"]
```
##### Pros:
- Easy to configure & probably easier to implement
- We can check the configurations at runtime and tell the user if any module dependencies aren't met
##### Cons:
- Each stage needs to be completed before the next stage can start
    - While the downloader could theoretically start downloading right after the jsonfetch module has completed, it needs to wait for title-to-tile to finish
        - Will probably be a non-issue since most modules don't take that long
#### Solution 2: Figure out which modules can run at runtime
Each module will run once it's dependencies are met, which means there will be no time wasted waiting
##### Pros:
- No wasted waiting time
##### Cons:
- Harder to implement
- Probably harder to configure
If two modules edit the same data, one needs to directly or indirectly depend on the other, otherwhise the order in which they are run could change from run to run.
That means, if there are multiple inbuilt modules which modify the same data there needs to be a preconfigured dependency tree so that they run in the same order every time. If the user now wants to edit the order they will run in he needs to rewrite the dependencies for all these modules.