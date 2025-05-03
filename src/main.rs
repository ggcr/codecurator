mod cli;
mod logger;

use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    thread, time,
};

use cli::Opt;
use colored::Colorize;
use logger::*;
use structopt::StructOpt;

fn parse_source(source: &PathBuf) -> Vec<(String, String)> {
    let input = fs::read_to_string(source).expect("Cannot read source");
    let mut valid_repos: Vec<(String, String)> = Vec::new();

    for line in serde_json::Deserializer::from_str(&input).into_iter::<&str>() {
        let repo = match line {
            Ok(line) => line,
            _ => continue,
        };
        let parts: Vec<&str> = repo.split("/").collect();
        if parts.len() == 2 {
            valid_repos.push((parts[0].to_owned(), parts[1].to_owned()));
        }
    }
    // for repo in &valid_repos {
    //     println!("{}/{}", repo.0, repo.1);
    // }
    valid_repos
}

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

fn process_source(source: &PathBuf) -> Option<Vec<PathBuf>> {
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
    if let Some(paths) = process_source(&source) {
        logger::log(
            Level::Info,
            format!("Downloaded {} repos onto `zip`", paths.len()).as_str(),
        );
    } else {
        panic!("Unable to download any repo from source file.")
    }
}
