use std::collections::HashSet;
use std::path::PathBuf;
use std::{fs, process};

use colored::Colorize;
use serde_json::Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("failed to open source file: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON line: {0}")]
    InvalidJsonLine(#[from] serde_json::Error),
    #[error("source line does not contain user / repo format: {0}")]
    MalformedLine(String),
    // TODO: Set this in, return a Result and let the callee (main) do the warnings
    // #[error("source file is empty")]
    // Empty(String),
}

pub struct Repo {
    user: String,
    name: String,
}

fn parse_line(line: Result<&str, Error>) -> Result<Repo, SourceError> {
    // Check that the retrieved line with serde is valid
    let line = line.map_err(SourceError::InvalidJsonLine)?;
    // Attempt to retrieve Username & Repo
    let split: Vec<String> = line.split("/").map(str::to_owned).collect();
    if split.len() != 2 {
        return Err(SourceError::MalformedLine(line.into()));
    }
    let user = split.first().expect("Unable to extract username from repo");
    let name = split.get(1).expect("Unable to extract name from repo");
    Ok(Repo {
        user: user.to_owned(),
        name: name.to_owned(),
    })
}

pub fn parse_source(source: &PathBuf) -> Vec<(String, String)> {
    let input = match fs::read_to_string(source) {
        Ok(data) => data,
        Err(e) => {
            eprintln!(
                "{} Cannot read source file '{}': {}",
                "[ERROR]".red().bold(),
                source.display(),
                e
            );
            process::exit(1);
        }
    };

    let mut valid_repos: Vec<(String, String)> = Vec::new();
    for line in serde_json::Deserializer::from_str(&input).into_iter::<&str>() {
        if let Ok(repo) = parse_line(line) {
            valid_repos.push((repo.user, repo.name));
        }
    }

    if valid_repos.is_empty() {
        eprintln!(
            "{} No valid URIs found in source file '{}'",
            "[WARNING]".truecolor(214, 143, 0),
            source.display()
        );
        process::exit(1);
    }

    valid_repos
}

pub fn parse_source_as_hashset(source: &PathBuf) -> HashSet<String> {
    let input = fs::read_to_string(source).expect("Cannot read source");
    let mut valid_repos: HashSet<String> = HashSet::new();
    for line in serde_json::Deserializer::from_str(&input).into_iter::<&str>() {
        if let Ok(repo) = parse_line(line) {
            valid_repos.insert(format!("{}-{}", repo.user, repo.name));
        }
    }
    valid_repos
}
