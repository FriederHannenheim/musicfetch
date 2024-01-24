use std::sync::{Arc, Mutex};

use anyhow::{bail, Ok, Result};
use serde_json::Value;

mod album;
mod albumcover;
mod download;
mod infocopy;
mod jsonfetch;
mod rename;
mod songcounter;
mod tag_files;
mod tagui;

type ModuleRunFunction = fn(Arc<Mutex<Value>>, Arc<Mutex<Value>>) -> Result<()>;

pub struct ModuleStruct {
    pub deps: Vec<String>,
    pub run_function: ModuleRunFunction,
}

#[macro_export]
macro_rules! define_module {
    ($name:expr, $run_function:expr, [$($dependency:expr),*]) => {
        pub const MODULE_NAME: &str = $name;

        pub fn module_info() -> ModuleStruct {
            ModuleStruct {
                deps: vec![$(
                    String::from($dependency),
                )*],
                run_function: $run_function,
            }
        }
    };
}

pub fn get_module(name: &str) -> Result<ModuleStruct> {
    Ok(match name {
        jsonfetch::MODULE_NAME => jsonfetch::module_info(),
        infocopy::MODULE_NAME => infocopy::module_info(),
        download::MODULE_NAME => download::module_info(),
        songcounter::MODULE_NAME => songcounter::module_info(),
        albumcover::MODULE_NAME => albumcover::module_info(),
        album::MODULE_NAME => album::module_info(),
        tagui::MODULE_NAME => tagui::module_info(),
        rename::MODULE_NAME => rename::module_info(),
        tag_files::MODULE_NAME => tag_files::module_info(),

        module_name => bail!("No module named {module_name}"),
    })
}
