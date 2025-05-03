mod cli;
mod downloader;
mod logger;
mod source;

use std::path::PathBuf;

use cli::Opt;
use colored::Colorize;
use downloader::download_repos;
use logger::*;
use structopt::StructOpt;

fn main() {
    let opts = Opt::from_args();
    let source: PathBuf = opts.source;

    if !source.exists() {
        panic!("Source does not exists");
    }

    if source.extension().expect("Could not read extension") != "jsonl" {
        panic!("Source is not a valid JSON lines file");
    }

    println!("Source: {}", source.to_string_lossy().blue());
    if let Some(paths) = download_repos(&source) {
        logger::log(
            Level::Info,
            format!("Downloaded {} repos onto `zip`", paths.len()).as_str(),
        );
    } else {
        panic!("Unable to download any repo from source file.")
    }
}
