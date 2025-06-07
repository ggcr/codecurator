// src/commands/extract.rs

use crate::config::ExtractionConfig;
use crate::source::parse_source_as_hashset;
use crate::{error::ExtractionError, extractor::extract_text};
use colored::Colorize;
use std::collections::HashSet;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use tokenizers::Tokenizer;
use yaml_rust::YamlLoader;

pub fn listdir(dir: &Path, match_extension: String) -> Result<Vec<PathBuf>, ExtractionError> {
    let mut files: Vec<PathBuf> = Vec::new();
    let dir_files = match fs::read_dir(dir) {
        Ok(dfs) => dfs,
        Err(e) => {
            return Err(ExtractionError::Validation {
                message: format!("Error while listing dir {}: {}", dir.display(), e),
            });
        }
    };
    for file in dir_files {
        let f = file?.path();
        if f.extension() == Some(OsStr::new(&match_extension)) {
            files.push(f.clone());
        }
    }
    if files.is_empty() {
        return Err(ExtractionError::Validation {
            message: String::from("Empty directory"),
        });
    }
    Ok(files)
}

fn read_linguist(path: &Path) -> Result<HashMap<String, String>, ExtractionError> {
    let fc = std::fs::read_to_string(path)?;
    let docs = YamlLoader::load_from_str(&fc)?;
    let doc = &docs[0];
    let mut ret: HashMap<String, String> = HashMap::new();
    for (_, v) in doc.as_hash().unwrap() {
        let Some(file_type) = v["type"].as_str() else {
            continue;
        };
        if file_type == "programming" {
            if let Some(ext_list) = v["extensions"].as_vec() {
                for ext in ext_list {
                    ret.insert(ext.clone().into_string().unwrap(), file_type.to_owned());
                }
            };
        }
    }
    if ret.is_empty() {
        return Err(ExtractionError::Validation {
            message: String::from("Linguist yml is empty"),
        });
    }
    Ok(ret)
}

fn filter_listdir_by_source(
    paths: &Vec<PathBuf>,
    source_hs: &HashSet<String>,
) -> Result<Vec<PathBuf>, ExtractionError> {
    let mut filtered = Vec::new();

    // We compare the provided source file with the local zips in disk

    for path in paths {
        if let Some(stem_os) = path.file_stem() {
            if let Some(stem) = stem_os.to_str() {
                let prefix = stem.split("_").next().unwrap_or(stem);
                if source_hs.contains(prefix) {
                    filtered.push(path.clone());
                }
            }
        }
    }
    if filtered.is_empty() {
        return Err(ExtractionError::Validation {
            message: String::from("0 Filtered repos"),
        });
    }
    Ok(filtered)
}

pub async fn run(ctx: &ExtractionConfig) {
    // List zip dir
    let paths = match listdir(&ctx.zip_dir, "zip".to_string()) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("{} {}", "[WARNING]".truecolor(214, 143, 0), e);
            return;
        }
    };

    // Filter zip files for those ennumerated in source file
    let repos_hs = parse_source_as_hashset(&ctx.source);
    let paths = match filter_listdir_by_source(&paths, &repos_hs) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("{} {}", "[WARNING]".truecolor(214, 143, 0), e);
            return;
        }
    };

    // Load tokenizer and linguist yaml
    let gpt2tokenizer = Tokenizer::from_pretrained("openai-community/gpt2", None)
        .expect("Failed to load the tokenizer");
    let linguist_file_types =
        read_linguist(&ctx.linguist_path).expect("Unable to read linguist languages yaml");

    // Extract
    let _ = extract_text(&ctx.jsonl_dir, paths, linguist_file_types, gpt2tokenizer);
}
