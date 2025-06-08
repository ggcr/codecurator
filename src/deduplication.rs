use std::io::Cursor;
use std::{fs::File, io::BufReader, path::PathBuf};

use polars::prelude::*;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Serialize;

use crate::error::ExactDedupError;
use crate::extractor::Record;

#[derive(Serialize)]
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

// Trait to convert to Polars DataFrame and back
trait DedupDataFrame {
    fn to_dataframe(&self) -> Result<DataFrame, ExactDedupError>;
    fn get_unique_md5(&self) -> Result<DataFrame, ExactDedupError>;
    fn get_unique_ids(&self) -> Result<Vec<String>, ExactDedupError>;
}

impl DedupDataFrame for Vec<DedupRecord> {
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

    println!("Computed {} MD5 hashes", hashes.len());

    let ids: Vec<String> = hashes.get_unique_ids().unwrap();

    println!("Found {} unique MD5 hashes", ids.len());
}
