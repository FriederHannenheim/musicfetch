use std::{env, path::PathBuf, fs};

use log::info;
use serde_json::Value;
use anyhow::{Result, bail, Context};



pub fn get_config(name: &str) -> Result<Value> {
    let Some(dir) = get_config_dir() else {
        bail!("Failed finding configuration directory");
    };
    get_config_by_name(name, dir)
}

fn get_config_by_name(name: &str, dir: PathBuf) -> Result<Value> {
    let mut file_path = dir;
    file_path.push(format!("{}.toml", name));

    let file_path_string = file_path.clone().to_string_lossy().to_string();

    let config_string = fs::read_to_string(file_path).context(file_path_string)?;
    let config = toml::from_str::<Value>(&config_string)?;

    Ok(config)
}

fn get_config_dir() -> Option<PathBuf> {
    get_user_config_dir()
        .or_else(get_global_config_dir)
}

fn get_user_config_dir() -> Option<PathBuf> {
    if let Ok(config_dir) = env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(config_dir);
        path.push("musicfetch");

        if path.is_absolute() && path.exists() {
            info!("Found path using XDG_CONFIG_HOME: {}", path.to_string_lossy());
            return Some(path);
        }
    }

    if let Ok(home_dir) = env::var("HOME") {
        let mut path = PathBuf::from(home_dir);
        path.push(".config");
        path.push("musicfetch");

        if path.is_absolute() && path.exists() {
            info!("Found path using home directory: {}", path.to_string_lossy());
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
