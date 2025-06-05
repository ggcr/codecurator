use std::{
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use crate::error::DownloadError;
use bytes::Bytes;
use colored::Colorize;
use futures::StreamExt;
use tokio::time::sleep;

async fn download_bytes(client: &reqwest::Client, url: &str) -> Result<Bytes, DownloadError> {
    let resp = client
        .get(url)
        .header(reqwest::header::USER_AGENT, "Rust CodeCurator")
        .send()
        .await?
        .error_for_status()?;
    let content = resp.bytes().await?;
    Ok(content)
}

async fn get_etag(client: &reqwest::Client, url: &str) -> Option<String> {
    let resp = client.head(url).send().await.ok()?;
    resp.headers()
        .get("etag")?
        .to_str()
        .ok()
        .map(|s| s.trim_matches('"').to_string())
}

async fn write_bytes_to_file(filepath: &str, content: Bytes) -> Result<PathBuf, DownloadError> {
    tokio::fs::write(&filepath, content).await?;
    let path = tokio::fs::canonicalize(&filepath).await?;
    Ok(path)
}

async fn are_equal(contents: Bytes, filepath: &str) -> Result<bool, DownloadError> {
    // Read filepath, we know it exists
    // still could be corrupted!
    let dest_content = tokio::fs::read(filepath).await?;
    Ok(contents == dest_content)
}

async fn download_repo_zip(
    user: &String,
    repo: &String,
    branch: &str,
) -> Result<PathBuf, DownloadError> {
    let client = reqwest::Client::new();

    // Define remote zip URL and local filepath
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        user, repo, branch
    );

    let mut filepath = format!("./zip/{}-{}.zip", user, &repo);

    // Fetch the latest commit SHA and check if we have it in local
    if let Some(etag) = get_etag(&client, &url).await {
        filepath = format!("./zip/{}-{}_{}.zip", user, repo, etag);
        if tokio::fs::try_exists(&filepath).await.unwrap_or(false) {
            return Ok(PathBuf::from(&filepath));
        }
    }

    let content = download_bytes(&client, &url).await?;

    // Check the checksum between content (Bytes) and the file in disk
    if tokio::fs::try_exists(&filepath).await.unwrap_or(false)
        && are_equal(content.clone(), &filepath).await.is_ok_and(|r| r)
    {
        return Ok(PathBuf::from(&filepath));
    }

    write_bytes_to_file(&filepath, content).await?;

    Ok(PathBuf::from(&filepath))
}

pub async fn download_repos(
    uris: Vec<(String, String)>,
    workers: usize,
) -> Result<Vec<PathBuf>, DownloadError> {
    // Download & Write
    let destination_dir = Path::new("./zip/");
    fs::create_dir_all(destination_dir)?;

    // We try first downloading from main, if err, we try on master
    let futures = futures::stream::iter(uris.into_iter().map(|(user, repo)| async move {
        let result = async {
            match download_repo_zip(&user, &repo, "main").await {
                Ok(path) => Ok(path),
                Err(_) => {
                    sleep(Duration::from_secs(1)).await;
                    download_repo_zip(&user, &repo, "master").await
                }
            }
        }
        .await;

        match result {
            Ok(path) => {
                println!("\t{}:  {}/{}", "Downloaded".green(), user, repo);
                Ok(path)
            }
            Err(e) => {
                println!("\t{}:  {}", "Error".red(), e);
                Err(e)
            }
        }
    }))
    .buffer_unordered(workers)
    .collect::<Vec<_>>()
    .await;

    // We accumulate the path on those successful downloads
    let files: Vec<PathBuf> = futures.into_iter().flatten().collect();

    if files.is_empty() {
        return Err(DownloadError::Validation {
            message: String::from("No repo was downloaded"),
        });
    }
    Ok(files)
}
