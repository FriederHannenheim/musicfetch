use std::{
    env,
    fs::{self, create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use anyhow::{bail, Context, Result};
use home::home_dir;
use log::info;
use serde_json::Value;

const DEFAULT_CONFIG: &[u8] = include_bytes!("../config/default.toml");

pub fn get_config(name: &str) -> Result<Value> {
    let dir = match get_config_dir() {
        Some(dir) => dir,
        None => {
            eprintln!("Failed to find musicfetch config dir. Creating default config...");
            create_default_config()?
        }
    };
    get_config_by_name(name, dir)
}

fn create_default_config() -> Result<PathBuf> {
    let mut config_dir = match env::var("XDG_CONFIG_HOME") {
        Ok(dir) => PathBuf::from(dir),
        Err(_) => match home_dir() {
            Some(mut dir) => {
                dir.push(".config");
                dir
            }
            None => bail!("Failed to find config directory"),
        },
    };
    config_dir.push("musicfetch");

    create_dir_all(&config_dir)?;

    let mut config_path = config_dir.clone();
    config_path.push("default.toml");

    let mut config_file = File::create(&config_path)?;

    config_file.write_all(DEFAULT_CONFIG)?;

    Ok(config_dir)
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
    get_user_config_dir().or_else(get_global_config_dir)
}

fn get_user_config_dir() -> Option<PathBuf> {
    if let Ok(config_dir) = env::var("XDG_CONFIG_HOME") {
        let mut path = PathBuf::from(config_dir);
        path.push("musicfetch");

        if path.is_absolute() && path.exists() {
            info!(
                "Found path using XDG_CONFIG_HOME: {}",
                path.to_string_lossy()
            );
            return Some(path);
        }
    }

    if let Some(mut path) = home_dir() {
        path.push(".config");
        path.push("musicfetch");

        if path.is_absolute() && path.exists() {
            info!(
                "Found path using home directory: {}",
                path.to_string_lossy()
            );
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
