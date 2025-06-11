use std::collections::HashSet;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::{fs::File, io::BufReader, path::PathBuf};

use polars::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;

use crate::error::ExactDedupError;
use crate::extractor::{Record, write_repo_jsonl};

#[derive(Serialize)]
struct MD5Record {
    id: String,
    md5: String,
}

impl From<Record> for MD5Record {
    fn from(doc: Record) -> Self {
        let digest = md5::compute(&doc.text);
        Self {
            id: doc.id,
            md5: format!("{:?}", digest),
        }
    }
}

// Trait to convert to Polars DataFrame and back
trait DedupDataFrame {
    fn to_dataframe(&self) -> Result<DataFrame, ExactDedupError>;
    fn get_unique_md5(&self) -> Result<DataFrame, ExactDedupError>;
    fn get_unique_ids(&self) -> Result<Vec<String>, ExactDedupError>;
}

impl DedupDataFrame for Vec<MD5Record> {
    fn to_dataframe(&self) -> Result<DataFrame, ExactDedupError> {
        let json = serde_json::to_string(&self)?;
        let cursor = Cursor::new(json);
        let df = JsonReader::new(cursor).finish()?;
        Ok(df)
    }

    fn get_unique_md5(&self) -> Result<DataFrame, ExactDedupError> {
        let df = self.to_dataframe()?;
        let df_unique =
            df.unique_stable(Some(&["md5".to_string()]), UniqueKeepStrategy::Any, None)?;
        Ok(df_unique)
    }

    fn get_unique_ids(&self) -> Result<Vec<String>, ExactDedupError> {
        let df = self.get_unique_md5()?;
        let ids: Vec<String> = df
            .column("id")?
            .str()?
            .into_iter()
            .flatten()
            .map(|r| r.to_string())
            .collect();
        Ok(ids)
    }
}

fn read_records(jsonl_path: &PathBuf) -> Result<Vec<Record>, ExactDedupError> {
    let file = File::open(jsonl_path)?;
    let mut reader = BufReader::new(file);
    let mut records = Vec::new();

    loop {
        match jsonl::read::<_, Record>(&mut reader) {
            Ok(record) => records.push(record),
            Err(jsonl::ReadError::Eof) => break,
            Err(_) => continue,
        }
    }

    Ok(records)
}

fn write_records(
    jsonl_path: &PathBuf,
    ids: &HashSet<String>,
    dest_dir: &Path,
) -> Result<(), ExactDedupError> {
    let file = File::open(jsonl_path)?;
    let mut reader = BufReader::new(file);

    // Replace the root dir from /jsonl/ to /<dedup dir>/
    let name = match jsonl_path.file_stem() {
        Some(name) => format!("{}", name.display()),
        _ => {
            return Err(ExactDedupError::Validation {
                message: format!(
                    "Unable to parse filename of jsonl path {}",
                    jsonl_path.display()
                ),
            });
        }
    };

    let mut fc: i64 = 0;
    loop {
        match jsonl::read::<_, Record>(&mut reader) {
            Ok(record) => {
                if ids.contains(&record.id) {
                    match write_repo_jsonl(dest_dir, &name, &record, &fc) {
                        Ok(_) => {
                            fc += 1;
                            continue;
                        }
                        Err(e) => eprintln!("{}", e),
                    }
                }
            }
            Err(jsonl::ReadError::Eof) => break,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        }
    }
    Ok(())
}

pub fn exact_deduplication(jsonl_paths: &Vec<PathBuf>, ddest: &Path) {
    println!(
        "Starting Exact deduplication on {} files",
        jsonl_paths.len()
    );

    let hashes: Vec<MD5Record> = jsonl_paths
        .par_iter()
        .filter_map(|path| read_records(path).ok())
        .flatten()
        .map(MD5Record::from)
        .collect();

    println!("Loaded {} documents", hashes.len());

    let ids: Vec<String> = hashes.get_unique_ids().unwrap();
    let ids_hs: HashSet<String> = HashSet::from_iter(ids.iter().cloned());

    println!("Found {} unique documents", ids.len());

    let destination_dir = ddest;
    fs::create_dir_all(&destination_dir).expect("Unable to create deduplication dir");

    jsonl_paths
        .par_iter()
        .for_each(|path| match write_records(path, &ids_hs, destination_dir) {
            Ok(()) => (),
            Err(e) => {
                eprintln!("Error processing {}: {}", path.display(), e);
            }
        });

    println!("Exact dedup written to {}", destination_dir.display());
}

pub fn fuzzy_deduplication(jsonl_paths: &Vec<PathBuf>, ddest: &Path) {
    println!(
        "Starting Fuzzy deduplication on {} files",
        jsonl_paths.len()
    );

    let records: Vec<Record> = jsonl_paths
        .par_iter()
        .filter_map(|path| read_records(path).ok())
        .flatten()
        .collect();

    println!("Loaded {} documents", records.len());

    let destination_dir = ddest;
    fs::create_dir_all(&destination_dir).expect("Unable to create deduplication dir");
}
