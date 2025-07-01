use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use colored::Colorize;
use tokenizers::Tokenizer;
use yaml_rust::{Yaml, YamlLoader};

use crate::config::{DedupeConfig, DownloadConfig, ExtractionConfig};
use crate::deduplication::{exact_deduplication, fuzzy_deduplication};
use crate::downloader::download_repos;
use crate::source::parse_source;
use crate::source::parse_source_as_hashset;
use crate::{error::ExtractionError, extractor::extract_text};

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

fn read_linguist(path: &Path) -> Result<Yaml, ExtractionError> {
    let fc = std::fs::read_to_string(path)?;
    let docs = YamlLoader::load_from_str(&fc)?;
    let doc = &docs[0];
    Ok(doc.clone())
}

fn get_ext_ft(doc: &Yaml) -> Result<HashMap<String, String>, ExtractionError> {
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

fn get_ext_pl(
    yaml_doc: &Yaml,
    langs: &[String],
) -> Result<HashMap<String, String>, ExtractionError> {
    let doc = yaml_doc.as_hash().unwrap();
    let mut union_exts: HashMap<String, String> = HashMap::new();
    for lang in langs {
        let lang_key = Yaml::from_str(lang);
        if let Some(v) = doc.get(&lang_key) {
            if let Some(file_type) = v["type"].as_str() {
                if let Some(ext_list) = v["extensions"].as_vec() {
                    for ext in ext_list {
                        union_exts.insert(ext.clone().into_string().unwrap(), file_type.to_owned());
                    }
                };
            };
        } else {
            eprintln!(
                "{} Language {} is not defined in linguist",
                "[WARNING]".truecolor(214, 143, 0),
                lang
            );
        }
    }
    if union_exts.is_empty() {
        return get_ext_ft(yaml_doc);
    }
    Ok(union_exts)
}

pub fn filter_listdir_by_source(
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

pub async fn download(ctx: &DownloadConfig) {
    // read source file
    let uris: Vec<(String, String)> = parse_source(&ctx.source);
    if uris.is_empty() {
        eprintln!(
            "{} No valid URIs found in source file",
            "[WARNING]".truecolor(214, 143, 0)
        );
    }

    // Download
    download_repos(uris, &ctx.zip_dir, &ctx.user_agent, ctx.workers)
        .await
        .expect("No content has been downloaded.");
}

pub async fn extract(ctx: &ExtractionConfig) {
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
    let linguist_file =
        read_linguist(&ctx.linguist_path).expect("Unable to read linguist languages yaml");

    let ext_file_types = match &ctx.languages {
        Some(langs) => get_ext_pl(&linguist_file, langs)
            .expect("Unable to get programming file types extensions from yaml"),
        None => get_ext_ft(&linguist_file)
            .expect("Unable to get programming file types extensions from yaml"),
    };

    // Extract
    let _ = extract_text(&ctx.jsonl_dir, paths, ext_file_types, gpt2tokenizer);
}

pub async fn dedupe(ctx: &DedupeConfig) {
    // List zip dir
    let paths = match listdir(&ctx.jsonl_dir, "jsonl".to_string()) {
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

    exact_deduplication(&paths, &ctx.exact_dedup_dir);

    // List zip dir
    let paths = match listdir(&ctx.exact_dedup_dir, "jsonl".to_string()) {
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
    fuzzy_deduplication(&paths, &ctx.dest_dir);
}
