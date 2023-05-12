use crate::modules::download::DownloadModule;

use super::Module;

use regex::Regex;
use serde_json::Value;
use std::sync::{Arc, Mutex};

use anyhow::{Result, bail};

pub struct RenameModule;

impl Module for RenameModule {
    fn name() -> String {
        String::from("rename")
    }

    fn deps() -> Vec<String> {
        vec![DownloadModule::name()]
    }

    fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()> {
        let songs = songs.lock().unwrap();
        let songs = songs.as_array().unwrap();

        let global = global.lock().unwrap();
        let global = global.as_object().unwrap();
        let name_template = match &global["stages"]["rename"]["template"] {
            Value::String(template) => template.to_owned(),
            _ => {
                log::warn!("No rename template in config. Using default");
                String::from("%(title).%(ext)")
            }
        };

        for song in songs {

        }
        
        Ok(())
    }
}

fn get_filename_for_song(name_template: &str, song: &Value) -> Result<String> {
    let mut filename = name_template.to_owned();

    let re = Regex::new(r"%\((\w+)\)").unwrap();
    for caps in re.captures_iter(name_template) {
        let matched_string = &caps[0];
        let value = match song["songinfo"][&caps[1]].clone() {
            Value::Null => bail!("Field '{}' does not exist in songinfo struct", &caps[1]),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => s,
            Value::Array(_) => bail!("Field '{}' is an array", &caps[1]),
            Value::Object(_) => bail!("Field '{}' is an object", &caps[1]),
        };

        filename = filename.replace(matched_string, &value);
    }
    Ok(filename)
}


#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::get_filename_for_song;

    #[test]
    fn test_filename_creation() {
        let song = json!({
            "songinfo": {
                "title": "Test Song",
                "artist": "Testartist",
                "year": 1994,
            }
        });

        assert_eq!("Testartist - Test Song", get_filename_for_song("%(artist) - %(title)", &song).unwrap());
        assert_eq!("1994 Test Song", get_filename_for_song("%(year) %(title)", &song).unwrap());
        assert_eq!("lalala", get_filename_for_song("lalala", &song).unwrap());
    }

    #[test]
    fn test_filename_creation_error() {
        let song = json!({
            "songinfo": {
                "title": ["a", "b", "c"],
                "year": {
                    "a": "b"
                },
            }
        });

        assert!(get_filename_for_song("%(title)", &song).is_err());
        assert!(get_filename_for_song("%(album)", &song).is_err());
        assert!(get_filename_for_song("%(year)", &song).is_err());
    }
}