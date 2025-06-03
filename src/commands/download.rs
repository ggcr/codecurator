// src/commands/download.rs

use colored::Colorize;

use crate::downloader::download_repos;
use crate::source::parse_source;
use std::path::PathBuf;

pub async fn run(source: PathBuf, workers: usize) {
    // Read source file
    let uris: Vec<(String, String)> = parse_source(&source);
    if uris.is_empty() {
        eprintln!(
            "{} No valid URIs found in source file",
            "[WARNING]".truecolor(214, 143, 0)
        );
    }

    // Download
    download_repos(uris, workers)
        .await
        .expect("No content has been downloaded.");
}
