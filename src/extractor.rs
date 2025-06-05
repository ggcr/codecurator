use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{BufWriter, Read},
    path::{Path, PathBuf},
    sync::Arc,
};

use colored::Colorize;
use rayon::prelude::*;
use serde::Serialize;
use std::io::BufReader;
use tokenizers::Tokenizer;
use uuid::Uuid;
use zip::ZipArchive;

#[derive(Serialize)]
struct Record {
    text: String,
    id: String,
    path: String,
    file_type: String,
    n_tokens: usize,
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

fn write_repo_jsonl(dest_dir: &Path, zip_name: &str, r: &Record) -> Option<()> {
    let mut jsonl_name = zip_name.to_owned();
    jsonl_name.push_str(".jsonl");
    let jsonl_path = dest_dir.join(jsonl_name);

    // To avoid over-writting on the next file of the repo
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&jsonl_path)
        .ok()?;

    let writer = BufWriter::new(file);
    jsonl::write(writer, r).ok()?;
    Some(())
}

//
// JSONL format: text, id, path, metadata
//
fn process_valid_file(
    file: &mut zip::read::ZipFile<'_, BufReader<fs::File>>,
    tokenizer: &Tokenizer,
) -> anyhow::Result<Record> {
    // Read file contents
    let mut text = String::new();
    file.read_to_string(&mut text)?;

    // Secondary fields: id, path
    let id = Uuid::new_v4().to_string();
    let path: String = file.name().to_string();

    // Metadata: file_type & tokenize
    let file_type = String::from("programming");
    let Ok(encoding) = tokenizer.encode(text.clone(), false) else {
        return Err(anyhow::Error::msg("Unable to tokenize"));
    };
    let n_tokens = encoding.len();

    Ok(Record {
        text,
        id,
        path,
        file_type,
        n_tokens,
    })
}

fn extract_zip(
    zip: &mut ZipArchive<BufReader<File>>,
    name: &str,
    file_types: &HashMap<String, String>,
    dest_dir: &Path,
    tokenizer: &Tokenizer,
) -> Option<i64> {
    let mut file_count = 0;
    for i in 0..zip.len() {
        let mut file = zip.by_index(i).ok()?;
        // If we are in a file + it has extension
        let Some(ext) = parse_ext(&file) else {
            continue;
        };
        if file.is_file()
                && file.size() <= 2u64.pow(17) // 128KB
                && file_types.contains_key(&ext)
        {
            // Parse file
            let r = match process_valid_file(&mut file, tokenizer) {
                Ok(r) => r,
                Err(_) => {
                    continue;
                }
            };
            // Write to JSONL
            let Some(_) = write_repo_jsonl(dest_dir, name, &r) else {
                continue;
            };
            file_count += 1;
        }
    }
    Some(file_count)
}

pub fn extract_text(
    zip_paths: Vec<PathBuf>,
    file_types: HashMap<String, String>,
    tokenizer: Tokenizer,
    _workers: usize,
) -> Option<()> {
    let destination_dir = PathBuf::from("./jsonl/");
    fs::create_dir_all(&destination_dir).ok()?;

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

            if let Some(count) = extract_zip(&mut zip, zip_name, &ft, &ddir, &tok) {
                println!("\t{}:  {}", "Extracted".green(), zip_name);
                return count;
            }
            0
        })
        .sum();

    println!("Total files processed = {}", total_files);
    if total_files <= 0 {
        return None;
    }
    Some(())
}
