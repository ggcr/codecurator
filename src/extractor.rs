use crate::error::ExtractionError;

use colored::Colorize;
use rayon::prelude::*;
use serde::Serialize;
use std::io::BufReader;
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read},
    path::{Path, PathBuf},
    sync::Arc,
};
use tokenizers::Tokenizer;
use uuid::Uuid;
use zip::ZipArchive;

#[derive(Serialize)]
struct Record {
    text: String,
    id: String,
    file_extension: String,
    category: String,
    file_path: String,
    size_in_bytes: u64,
    file_name: String,
    tokens: usize,
}

fn parse_ext(file: &zip::read::ZipFile<'_, BufReader<fs::File>>) -> Option<String> {
    let split_ext: Vec<&str> = file.name().splitn(2, ".").collect();
    if split_ext.len() == 2 {
        let mut ext = split_ext.last()?.to_string();
        ext.insert(0, '.');
        return Some(ext);
    }
    None
}

fn get_zip_name(zip_path: &Path) -> Option<&str> {
    let stem = zip_path.file_stem()?.to_str()?; // From OsStr → &str
    let base_name = stem.rsplit_once('_').map(|(left, _)| left).unwrap_or(stem);
    Some(base_name)
}

fn write_repo_jsonl(dest_dir: &Path, zip_name: &str, r: &Record) -> Result<(), ExtractionError> {
    let mut jsonl_name = zip_name.to_owned();
    jsonl_name.push_str(".jsonl");
    let jsonl_path = dest_dir.join(jsonl_name);

    // To avoid over-writting on the next file of the repo
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&jsonl_path)?;

    let writer = BufWriter::new(file);
    jsonl::write(writer, r)?;
    Ok(())
}

fn extract_path_metadata(
    file: &mut zip::read::ZipFile<'_, BufReader<fs::File>>,
) -> Option<(String, String)> {
    let path: PathBuf = file.enclosed_name()?;
    let filename = path.file_name()?;
    Some((path.display().to_string(), filename.display().to_string()))
}

//
// JSONL format: text, id, path, metadata
//
fn process_valid_file(
    file: &mut zip::read::ZipFile<'_, BufReader<fs::File>>,
    tokenizer: &Tokenizer,
    extension: String,
) -> Result<Record, ExtractionError> {
    // Read file contents
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    // Secondary fields: id, path
    let id = Uuid::new_v4().to_string();
    let Some((file_path, file_name)): Option<(String, String)> = extract_path_metadata(file) else {
        return Err(ExtractionError::Validation {
            message: format!(
                "Cannot safely extract path and filename from {}.",
                file.name()
            ),
        });
    };

    // Metadata: file_type & tokenize
    let file_type = String::from("programming");
    let Ok(encoding) = tokenizer.encode(text.clone(), false) else {
        return Err(ExtractionError::Tokenizer {
            message: format!("Unable to tokenize {}", file.name()),
        });
    };
    let n_tokens = encoding.len();

    Ok(Record {
        text,
        id,
        category: file_type,
        file_path,
        file_name,
        file_extension: extension,
        size_in_bytes: file.size(),
        tokens: n_tokens,
    })
}

fn extract_zip(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
    file_types: &HashMap<String, String>,
    dest_dir: &Path,
    tokenizer: &Tokenizer,
) -> Result<i64, ExtractionError> {
    let mut file_count = 0;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        // If we are in a file + it has extension
        let Some(ext) = parse_ext(&file) else {
            continue;
        };
        if file.is_file()
                && file.size() <= 2u64.pow(17) // 128KB
                && file_types.contains_key(&ext)
        {
            // Parse file
            let r = match process_valid_file(&mut file, tokenizer, ext) {
                Ok(r) => r,
                Err(_) => {
                    continue;
                }
            };
            // Write to JSONL
            let Ok(_) = write_repo_jsonl(dest_dir, name, &r) else {
                continue;
            };
            file_count += 1;
        }
    }
    Ok(file_count)
}

pub fn extract_text(
    zip_paths: Vec<PathBuf>,
    file_types: HashMap<String, String>,
    tokenizer: Tokenizer,
    _workers: usize,
) -> Result<(), ExtractionError> {
    let destination_dir = PathBuf::from("./jsonl/");
    fs::create_dir_all(&destination_dir)?;

    // Arc types for read-only on async
    let file_types = Arc::new(file_types);
    let tokenizer = Arc::new(tokenizer);
    let dest_dir = Arc::new(destination_dir);

    let total_files: i64 = zip_paths
        .par_iter()
        .map(|zip_path| {
            let ft = Arc::new(&file_types);
            let tok = Arc::new(&tokenizer);
            let ddir = Arc::clone(&dest_dir);

            let f = match fs::File::open(zip_path) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("Unable to open zip {}: {}", zip_path.display(), e);
                    return 0;
                }
            };

            let Some(zip_name) = get_zip_name(zip_path) else {
                return 0;
            };

            let reader = BufReader::new(f);
            let mut zip = zip::ZipArchive::new(reader).unwrap();

            if let Ok(count) = extract_zip(&mut zip, zip_name, &ft, &ddir, &tok) {
                println!("\t{}:  {}", "Extracted".green(), zip_name);
                return count;
            }
            0
        })
        .sum();

    println!("Total files processed = {}", total_files);
    // TODO: Make this error if and only if the original length is not 0
    if total_files <= 0 {
        return Err(ExtractionError::Validation {
            message: String::from("No repos were extracted."),
        });
    }
    Ok(())
}
