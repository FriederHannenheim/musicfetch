use std::sync::{Arc, Mutex};

use anyhow::{bail, Ok, Result};
use serde_json::Value;

use crate::modules::{album::Album, songcounter::Songcounter, tagui::TagUI};

use self::{infocopy::Infocopy, jsonfetch::Jsonfetch};

mod album;
mod infocopy;
mod jsonfetch;
mod songcounter;
mod tagui;

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
    match_module!(name, Jsonfetch, Infocopy, Album, Songcounter, TagUI)
}
