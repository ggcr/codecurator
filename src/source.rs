use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SourceError {
    #[error("failed to open source file: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid JSON line: {0}")]
    InvalidJsonLine(#[from] serde_json::Error),
    #[error("source line does not contain user / repo format: {0}")]
    MalformedLine(String),
    #[error("parsed source file is empty")]
    Empty(String),
}

pub struct Repo {
    user: String,
    name: String,
}

fn parse_line(line: &str) -> Result<Repo, SourceError> {
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

pub fn parse_source(source: &PathBuf) -> Result<Vec<(String, String)>, SourceError> {
    let input = fs::read_to_string(source)?;

    let mut valid_repos: Vec<(String, String)> = Vec::new();
    for line in serde_json::Deserializer::from_str(&input).into_iter::<&str>() {
        // Check that the retrieved line with serde is valid
        let valid_line = line.map_err(SourceError::InvalidJsonLine)?;
        if let Ok(repo) = parse_line(valid_line) {
            valid_repos.push((repo.user, repo.name));
        } else {
            eprintln!(
                "Source {:?} has a malformed line: {:?}",
                source.display().to_string(),
                valid_line
            );
        }
    }

    if valid_repos.is_empty() {
        return Err(SourceError::Empty(source.display().to_string()));
    }

    Ok(valid_repos)
}

// TODO(cristian): Remove this and do the logic of transforming a Vec to a HashSet outside of this scope
// Implement To trait to convert between vec and hashset
pub fn parse_source_as_hashset(source: &PathBuf) -> HashSet<String> {
    let input = fs::read_to_string(source).expect("Unable to read source file");
    let mut valid_repos: HashSet<String> = HashSet::new();
    for line in serde_json::Deserializer::from_str(&input).into_iter::<&str>() {
        // Check that the retrieved line with serde is valid
        let valid_line = line.map_err(SourceError::InvalidJsonLine).unwrap();
        if let Ok(repo) = parse_line(valid_line) {
            valid_repos.insert(format!("{}-{}", repo.user, repo.name));
        } else {
            eprintln!(
                "Source {:?} has a malformed line: {:?}",
                source.display().to_string(),
                valid_line
            );
        }
    }
    valid_repos
}
