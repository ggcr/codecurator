use std::fs;
use std::path::PathBuf;

fn parse_line(line: &str) -> Option<(String, String)> {
    let repo = match line {
        Ok(line) => line,
        _ => continue,
    };
    let parts: Vec<&str> = repo.split("/").collect();
    if parts.len() == 2 {
        valid_repos.push((parts[0].to_owned(), parts[1].to_owned()));
    }
}

pub fn read_manifest(manifest: &PathBuf) -> Vec<(String, String)> {
    let input = fs::read_to_string(manifest).expect("Cannot read source");
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
    valid_repos
}
