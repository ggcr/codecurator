// src/commands/download.rs

use colored::Colorize;

use crate::config::DownloadConfig;
use crate::downloader::download_repos;
use crate::source::parse_source;

pub async fn run(ctx: &DownloadConfig) {
    // Read source file
    let uris: Vec<(String, String)> = parse_source(&ctx.source);
    if uris.is_empty() {
        eprintln!(
            "{} No valid URIs found in source file",
            "[WARNING]".truecolor(214, 143, 0)
        );
    }

    // Download
    download_repos(uris, &ctx.zip_dir, &ctx.user_agent, ctx.workers)
        .await
        .expect("No content has been downloaded.");
}
