use std::{env, path::PathBuf, fs};

use serde_json::Value;
use anyhow::{Result, bail};



pub fn get_config(name: &str) -> Result<Value> {
    let Some(dir) = get_config_dir() else {
        bail!("Failed finding configuration directory");
    };
    get_config_by_name(name, &dir)
}

fn get_config_by_name(name: &str, dir: &PathBuf) -> Result<Value> {
    let file_path = dir.with_file_name(format!("{}.toml", name));

    let config_string = fs::read_to_string(file_path)?;
    let config = toml::from_str::<Value>(&config_string)?;

    Ok(config)
}

fn get_config_dir() -> Option<PathBuf> {
    get_user_config_dir()
        .and(get_global_config_dir())
}

fn get_user_config_dir() -> Option<PathBuf> {
    if let Ok(config_dir) = env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(config_dir);
        path.push("musicfetch");

        if path.is_absolute() && path.exists() {
            return Some(path);
        }
    }

    if let Ok(home_dir) = env::var("HOME") {
        let mut path = PathBuf::from(home_dir);
        path.push(".config");
        path.push("musicfetch");

        if path.is_absolute() && path.exists() {
            return Some(path);
        }
    }

    log::info!("Failed to find user config directory. Using global config");
    None
}

fn get_global_config_dir() -> Option<PathBuf> {
    let path = PathBuf::from("/etc/musicfetch");

    if path.exists() {
        return Some(path);
    }    

    let path = PathBuf::from("/usr/share/musicfetch");

    if path.exists() {
        return Some(path);
    }

    None
}

/// Merges b into a, values in b override values in a
fn merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                }
                else {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            } 

            return;
        }
    }

    *a = b;
}
