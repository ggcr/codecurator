mod cli;
mod downloader;
mod source;

use std::path::PathBuf;

use cli::Opt;
use colored::Colorize;
use downloader::download_repos;
use source::parse_source;
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let opts = Opt::from_args();
    let source: PathBuf = opts.source;

    if !source.exists() {
        panic!("Source does not exists");
    }

    if source.extension().expect("Could not read extension") != "jsonl" {
        panic!("Source is not a valid JSON lines file");
    }

    println!("Source: {}", source.to_string_lossy().blue());

    // Read source file
    let uris: Vec<(String, String)> = parse_source(&source);
    if uris.is_empty() {
        eprintln!(
            "{} No valid URIs found in source file",
            "[WARNING]".truecolor(214, 143, 0)
        );
    }

    // Download
    let download_workers = 10;
    if let Some(paths) = download_repos(uris, download_workers).await {
        println!(
            "{} Downloaded {} repos onto `zip`",
            "[INFO]".cyan(),
            paths.len(),
        );
    } else {
        panic!("Unable to download any repo from source file.")
    }
}
