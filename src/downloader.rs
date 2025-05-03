use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use bytes::Bytes;
use colored::Colorize;
use futures::StreamExt;
use thiserror::Error;
use tokio::time::sleep;

// To avoid the usage of Box<dyn Error>
// With this we catch both errors either from reqwest or from fs
#[derive(Debug, Error)]
enum DownloadError {
    #[error("Network error {0}")]
    Http(#[from] reqwest::Error),
    #[error("Filesystem error {0}")]
    Io(#[from] std::io::Error),
}

async fn fetch(url: String) -> Result<Bytes, DownloadError> {
    let resp = reqwest::get(url).await?.error_for_status()?;
    let content = resp.bytes().await?;
    Ok(content)
}

async fn store(filepath: String, content: Bytes) -> Result<PathBuf, DownloadError> {
    tokio::fs::write(&filepath, content).await?;
    let path = tokio::fs::canonicalize(&filepath).await?;
    Ok(path)
}

async fn download_repo(
    user: &String,
    repo: &String,
    branch: &str,
) -> Result<PathBuf, DownloadError> {
    // Download
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        user, repo, branch
    );
    let content = fetch(url).await?;
    // Store
    let filepath = format!("./zip/{}-{}.zip", user, repo);
    let zip_path = store(filepath, content).await?;
    Ok(zip_path)
}

pub async fn download_repos(uris: Vec<(String, String)>, workers: usize) -> Option<Vec<PathBuf>> {
    // Download & Write
    let destination_dir = Path::new("./zip/");
    fs::create_dir_all(destination_dir).ok()?;

    // We try first downloading from main, if err, we try on master
    let futures = futures::stream::iter(uris.into_iter().map(|(user, repo)| async move {
        let result = async {
            match download_repo(&user, &repo, "main").await {
                Ok(path) => Ok(path),
                Err(_) => {
                    sleep(Duration::from_secs(1)).await;
                    download_repo(&user, &repo, "master").await
                }
            }
        }
        .await;

        match result {
            Ok(path) => {
                println!("{} {}/{}", "[DOWNLOADED]".green(), user, repo);
                Some(path)
            }
            Err(e) => {
                println!("{} {}", "[ERROR]".red(), e);
                None
            }
        }
    }))
    .buffer_unordered(workers)
    .collect::<Vec<_>>()
    .await;

    // We accumulate the path on those successful downloads
    let files: Vec<PathBuf> = futures.into_iter().flatten().collect();

    if files.is_empty() {
        return None;
    }
    Some(files)
}
