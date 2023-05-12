use std::sync::{Arc, Mutex};

use anyhow::{bail, Ok, Result};
use serde_json::Value;

use crate::modules::{album::AlbumModule, songcounter::SongcounterModule, tagui::TagUIModule, download::DownloadModule, tagger::TagModule, albumcover::AlbumCoverModule};

use self::{infocopy::InfocopyModule, jsonfetch::JsonfetchModule};

mod album;
mod download;
mod infocopy;
mod jsonfetch;
mod songcounter;
mod tagui;
mod tagger;
mod albumcover;
mod rename;

// TODO: Rework deps to take Vec<String> and panic if dependencies are not met / return if dependencies met
pub trait Module {
    fn name() -> String;

    fn deps() -> Vec<String>;

    fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()>;
}

macro_rules! match_module {
    (
        $match_var:ident,
        $(
            $module:ty
        )
        ,
        +
    ) => {
        {
            $(
                if $match_var == <$module>::name() {
                    return Ok((<$module>::deps, <$module>::run));
                }
            )*
            bail!("Error in config: no module named {}", $match_var)
        }
    };
}

pub fn get_module(
    name: &str,
) -> Result<(
    fn() -> Vec<String>,
    fn(Arc<Mutex<Value>>, Arc<Mutex<Value>>) -> Result<()>,
)> {
    match_module!(
        name,
        JsonfetchModule,
        InfocopyModule,
        AlbumModule,
        SongcounterModule,
        TagUIModule,
        DownloadModule,
        TagModule,
        AlbumCoverModule
    )
}
