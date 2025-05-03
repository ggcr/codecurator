use std::{
    fs,
    path::{Path, PathBuf},
    thread, time,
};

use bytes::Bytes;
use colored::Colorize;
use thiserror::Error;

use crate::logger::{self, Level};

// To avoid the usage of Box<dyn Error>
// With this we catch both errors either from reqwest or from fs
#[derive(Debug, Error)]
enum DownloadError {
    #[error("Network error {0}")]
    Http(#[from] reqwest::Error),
    #[error("Filesystem error {0}")]
    Io(#[from] std::io::Error),
}

fn fetch(url: String) -> Result<Bytes, DownloadError> {
    let resp = reqwest::blocking::get(url)?.error_for_status()?;
    let content = resp.bytes()?;
    Ok(content)
}

fn store(filepath: String, content: Bytes) -> Result<PathBuf, DownloadError> {
    // Write to disk
    fs::write(&filepath, content)?;
    let path = fs::canonicalize(&filepath)?;
    Ok(path)
}

fn download_repo(user: &String, repo: &String, branch: &str) -> Result<PathBuf, DownloadError> {
    // Download
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        user, repo, branch
    );
    let content = fetch(url)?;
    // Store
    let filepath = format!("./zip/{}-{}.zip", user, repo);
    let zip_path = store(filepath, content)?;
    Ok(zip_path)
}

pub fn download_repos(uris: Vec<(String, String)>) -> Option<Vec<PathBuf>> {
    // Download & Write
    let destination_dir = Path::new("./zip/");
    fs::create_dir_all(destination_dir).ok()?;

    // We try first downloading from main, if err, we try on master
    // We accumulate the path on those successful downloads
    let mut files: Vec<PathBuf> = Vec::new();
    for (user, repo) in &uris {
        let path = download_repo(user, repo, "main").or_else(|_| {
            thread::sleep(time::Duration::from_secs(1));
            download_repo(user, repo, "master")
        });
        match path {
            Ok(path) => {
                println!("{} {}/{}", "[DOWNLOADED]".green(), user, repo);
                files.push(path);
            }
            Err(e) => logger::log(Level::Error, e.to_string().as_str()),
        }
    }
    if files.is_empty() {
        return None;
    }
    Some(files)
}
