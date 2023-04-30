use std::sync::{Arc, Mutex};

use anyhow::Result;
use phf::phf_map;
use serde_json::Value;

mod jsonfetch;
mod title_to_title;

pub const MODULES: phf::Map<&'static str, fn(Arc<Mutex<Value>>, Arc<Mutex<Value>>) -> Result<()>> = phf_map! {
    "jsonfetch" => jsonfetch::fetch_yt_dlp_json,
    "title_to_title" => title_to_title::title_to_title,
};
