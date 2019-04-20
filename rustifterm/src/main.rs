mod client;
mod errors;

use std::path::Path;

use clap::{App, Arg};

use client::IFTerminalClient;
use errors::IFtResult;
use rustifzm;

fn main() -> IFtResult<()> {
    let matches = App::new("rustifterm")
        .version("0.0.1")
        .author("pierreyoda <pierreyoda@users.noreply.github.com>")
        .about("This terminal client for the rustifzm Z-machine interpreter allows to play classic Interactive Fiction games like Zork.")
        .arg(Arg::with_name("STORY")
            .help("The input story file to play.")
            .required(true)
            .index(1))
        .get_matches();

    let story_file_name = matches.value_of("STORY").unwrap();
    let story_file_path = Path::new(story_file_name);

    let client = IFTerminalClient::with_story_file(&story_file_path)?;
    Ok(())
}
