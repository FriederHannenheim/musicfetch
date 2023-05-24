use std::path::PathBuf;

use lexopt::prelude::*;
use serde::Serialize;

const HELP: &str = r#"
musicfetch

Usage:
    musicfetch <url>...
    musicfetch (-c | --cover_url) <cover_url> <url>
    musicfetch (-o | --output_dir) <output_dir> <url>
    musicfetch -? | -h | --help

Options:
    -? -h --help        Show this help
    -v --version        Print version and exit
    -c --cover_url      Specify the url of the cover that should be added to the songs
    -o --output_dir     Specify the directory the songs should be downloaded to
    -C --config         Use the config with this name
"#;

#[derive(Default, Serialize)]
pub struct Args {
    urls: Vec<String>,
    cover_url: Option<String>,
    output_dir: Option<PathBuf>,
    pub config: Option<String>,
}

pub fn parse_args() -> Result<Args, lexopt::Error> {
    let mut urls = vec![];
    let mut cover_url: Option<String> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut config: Option<String> = None;

    let mut parser = lexopt::Parser::from_env();
    while let Some(arg) = parser.next()? {
        match arg {
            Short('c') | Long("cover_url") => {
                cover_url = Some(parser.value()?.parse()?);
            }
            Short('o') | Long("output_dir") => {
                output_dir = Some(parser.value()?.parse()?);
            }
            Value(val) => {
                urls.push(
                    val.into_string()
                        .expect("Argument with invalid UTF-8 passed."),
                );
            }
            Short('C') | Long("config") => {
                config = Some(parser.value()?.parse()?);
            }
            Short('h') | Short('?') | Long("help") => {
                println!("{}", HELP);
                std::process::exit(0);
            }
            Short('v') | Long("version") => {
                println!("{}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            _ => return Err(arg.unexpected()),
        }
    }
    Ok(Args {
        urls,
        cover_url,
        output_dir,
        config
    })
}
