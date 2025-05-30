use std::fs;
use std::path::PathBuf;

use itertools::Itertools;
use serde_json::Error;

fn parse_line(line: Result<&str, Error>) -> Option<(&str, &str)> {
    match line {
        Ok(line) => line.splitn(2, "/").collect_tuple(),
        _ => None,
    }
}

pub fn parse_source(source: &PathBuf) -> Vec<(String, String)> {
    let input = fs::read_to_string(source).expect("Cannot read source");
    let mut valid_repos: Vec<(String, String)> = Vec::new();
    for line in serde_json::Deserializer::from_str(&input).into_iter::<&str>() {
        if let Some((user, repo)) = parse_line(line) {
            valid_repos.push((user.to_owned(), repo.to_owned()));
        }
    }
    valid_repos
}
