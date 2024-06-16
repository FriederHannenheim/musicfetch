use crate::{
    define_module,
    module_util::{get_songinfo_field, song_to_string},
    modules::{self, ModuleStruct},
};

use fs_extra::file::{self, CopyOptions};
use log::info;
use regex::Regex;
use serde_json::Value;
use std::{
    fs::create_dir_all,
    path::PathBuf,
    str::FromStr,
    sync::{Arc, Mutex},
};

use anyhow::{bail, Result};

define_module!("rename", run, [modules::download::MODULE_NAME]);

fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
    let mut songs = songs.lock().unwrap();
    let songs = songs.as_array_mut().unwrap();

    let global = global.lock().unwrap();
    let global = global.as_object().unwrap();
    let name_template = match &global["config"]["module"]["rename"]["template"] {
        Value::String(template) => template.to_owned(),
        _ => {
            log::warn!("No rename template in config. Using default");
            String::from("%(title).%(ext)")
        }
    };

    for song in songs {
        let ext = get_songinfo_field::<String>(song, "path")?
            .split('.')
            .last()
            .expect("Song path has no file extension");
        song["songinfo"]["ext"] = Value::from(ext);

        let filename = get_path_for_song(&name_template, song)?;

        let old_path = PathBuf::from_str(get_songinfo_field(song, "path")?)?;

        let new_path = PathBuf::from(filename);

        if new_path.is_absolute() && new_path.parent().is_some() {
            let mut dir = new_path.clone();
            dir.pop();

            create_dir_all(dir)?;
        }

        info!("renaming file to {}", new_path.display());
        file::move_file(old_path, &new_path, &CopyOptions::new())?;

        song["songinfo"]["path"] = Value::from(
            new_path
                .to_str()
                .unwrap_or_else(|| panic!("Filepath for '{}' is not valid utf-8", song)),
        );
    }

    Ok(())
}

fn get_path_for_song(path_template: &str, song: &Value) -> Result<String> {
    let mut path = path_template.to_owned();

    let re = Regex::new(r"%\((\w+)\)").unwrap();
    for caps in re.captures_iter(path_template) {
        let matched_string = &caps[0];
        let mut value = match song["songinfo"][&caps[1]].clone() {
            Value::Null => bail!(
                "Song '{}' has no field '{}' or field is empty",
                song_to_string(song),
                &caps[1]
            ),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s,
            Value::Array(_) => bail!("Field '{}' is an array", &caps[1]),
            Value::Object(_) => bail!("Field '{}' is an object", &caps[1]),
        };
        value = value.replace('/', "_");

        path = path.replace(matched_string, &value);
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::get_path_for_song;

    #[test]
    fn test_filename_creation() {
        let song = json!({
            "songinfo": {
                "title": "Test Song",
                "artist": "Testartist",
                "year": 1994,
            }
        });

        assert_eq!(
            "Testartist - Test Song",
            get_path_for_song("%(artist) - %(title)", &song).unwrap()
        );
        assert_eq!(
            "1994 Test Song",
            get_path_for_song("%(year) %(title)", &song).unwrap()
        );
        assert_eq!("lalala", get_path_for_song("lalala", &song).unwrap());
    }
}
