use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread, time,
};

use colored::Colorize;

use crate::{
    logger::{self, Level},
    source::parse_source,
};

fn download_repo(user: &String, repo: &String, branch: &str) -> Result<PathBuf, Box<dyn Error>> {
    let url = format!(
        "https://github.com/{}/{}/archive/refs/heads/{}.zip",
        user, repo, branch
    );
    let resp = reqwest::blocking::get(&url)?.error_for_status()?;
    let content = resp.bytes()?;

    // Write to disk
    let filename = format!("./zip/{}-{}.zip", user, repo);
    fs::write(&filename, content)?;
    let abspath = fs::canonicalize(&filename)?;

    Ok(abspath)
}

pub fn download_repos(source: &PathBuf) -> Option<Vec<PathBuf>> {
    // Read
    let uris: Vec<(String, String)> = parse_source(source);
    if uris.is_empty() {
        logger::log(Level::Warn, "No valid URIs found in source file");
        return None;
    }

    // Download & Write
    let destination_dir = Path::new("./zip/");
    fs::create_dir_all(destination_dir).ok()?;

    // We try first downloading from main, if err, we try on master
    // We accumulate the path of those successfull downloads
    let mut files: Vec<PathBuf> = Vec::new();
    for (user, repo) in &uris {
        if let Ok(path) = download_repo(user, repo, "main") {
            println!("{} {}", "[DOWNLOADED]".green(), path.display());
            files.push(path);
        } else {
            thread::sleep(time::Duration::from_secs(1));
            match download_repo(user, repo, "master") {
                Ok(path) => {
                    println!("{} {}", "[DOWNLOADED]".green(), path.display());
                    files.push(path);
                }
                Err(e) => logger::log(Level::Error, e.to_string().as_str()),
            }
        }
    }
    if files.is_empty() {
        return None;
    }
    Some(files)
}
