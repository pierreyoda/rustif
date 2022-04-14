mod client;
mod errors;

use std::path::{Path, PathBuf};

use clap::Parser;

use client::IFTerminalClient;
use errors::IFtResult;
use rustifzm;

#[derive(Debug, Parser)]
#[clap(
    author = "pierreyoda <pierreyoda@users.noreply.github.com>",
    version = "0.0.1",
    about = "This terminal client for the rustifzm Z-machine interpreter allows to play classic Interactive Fiction games like Zork."
)]
struct Args {
    #[clap(parse(from_os_str), help = "The input story file to help.")]
    story_file: PathBuf,
}

fn main() -> IFtResult<()> {
    let args = Args::parse();

    let story_file_name = args.story_file;
    let story_file_path = Path::new(&story_file_name);

    let mut client = IFTerminalClient::with_story_file(story_file_path)?;
    client.run()
}
