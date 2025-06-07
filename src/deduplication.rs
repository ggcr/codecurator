use std::io::BufRead;
use std::{fs::File, io::BufReader, path::PathBuf};

use colored::Colorize;
use itertools::Dedup;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::error::ExactDedupError;
use crate::extractor::Record;

struct DedupRecord {
    id: String,
    md5: String,
}

impl From<Record> for DedupRecord {
    fn from(doc: Record) -> Self {
        let digest = md5::compute(&doc.text);
        Self {
            id: doc.id,
            md5: format!("{:?}", digest),
        }
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

pub fn exact_deduplication(jsonl_paths: Vec<PathBuf>) {
    println!(
        "Starting Exact deduplication on {} repos",
        jsonl_paths.len()
    );

    let hashes: Vec<DedupRecord> = jsonl_paths
        .par_iter()
        .filter_map(|path| read_records(path).ok())
        .flatten()
        .map(DedupRecord::from)
        .collect();

    println!("Computed MD5 hashes on {} files", hashes.len());
}
