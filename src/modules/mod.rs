use std::sync::{Arc, Mutex};

use anyhow::Result;
use phf::phf_map;
use serde_json::Value;

use self::{infocopy::Infocopy, jsonfetch::Jsonfetch};

mod infocopy;
mod jsonfetch;

pub trait Module {
    fn name() -> String;

    fn deps() -> Vec<String>;

    fn run(global: Arc<Mutex<Value>>, songs: Arc<Mutex<Value>>) -> Result<()>;
}

macro_rules! methods {
    ($module:ty) => {
        <$module>::name() => (<$module>::deps, <$module>::run),
    };
}

pub fn get_module(
    name: &str,
) -> (
    fn() -> Vec<String>,
    fn(Arc<Mutex<Value>>, Arc<Mutex<Value>>) -> Result<()>,
) {
    match name {
        methods!(Jsonfetch)
        module => panic!("Module not found {module}"),
    }
}
/*
pub const MODULES: phf::Map<
    &'static str,
> = phf_map! {
    Jsonfetch::name() => (Jsonfetch::deps, Jsonfetch::run)
    // methods!(Jsonfetch),
    // methods!(Infocopy)
};
*/
