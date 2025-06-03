use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    io::{BufWriter, Read},
    path::{Path, PathBuf},
};

use serde::Serialize;
use std::io::BufReader;
use tokenizers::Tokenizer;
use uuid::Uuid;
use yaml_rust::YamlLoader;

#[derive(Serialize)]
struct Record {
    text: String,
    id: String,
    path: String,
    file_type: String,
    n_tokens: usize,
}

pub fn read_linguist(path: &Path) -> anyhow::Result<HashMap<String, String>> {
    let fc = std::fs::read_to_string(path)?;
    let docs = YamlLoader::load_from_str(&fc)?;
    let doc = &docs[0];
    let mut ret: HashMap<String, String> = HashMap::new();
    for (_, v) in doc.as_hash().unwrap() {
        let Some(file_type) = v["type"].as_str() else {
            continue;
        };
        if let Some(ext_list) = v["extensions"].as_vec() {
            for ext in ext_list {
                ret.insert(ext.clone().into_string().unwrap(), file_type.to_owned());
            }
        };
    }
    Ok(ret)
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

fn write_repo_jsonl(dest_dir: &Path, zip_path: &Path, r: &Record) -> Option<()> {
    let stem = zip_path.file_stem()?.to_str()?; // From OsStr → &str
    let base_name = stem.rsplit_once('_').map(|(left, _)| left).unwrap_or(stem);
    let mut jsonl_name = base_name.to_owned();
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

pub fn extract_text(
    zip_paths: Vec<PathBuf>,
    file_types: HashMap<String, String>,
    tokenizer: Tokenizer,
    workers: usize,
) -> Option<()> {
    let destination_dir = Path::new("./jsonl/").to_path_buf();
    fs::create_dir_all(&destination_dir).ok()?;

    let mut file_count = 0;
    for zip_path in zip_paths {
        let f = match fs::File::open(&zip_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Unable to open zip {}: {}", zip_path.display(), e);
                continue;
            }
        };
        let reader = BufReader::new(f);
        let mut zip = zip::ZipArchive::new(reader).ok()?;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).ok()?;
            // If we are in a file + it has extension
            let Some(ext) = parse_ext(&file) else {
                continue;
            };
            if file.is_file()
                && file.size() <= 2u64.pow(17) // 128KB
                && file_types.contains_key(&ext)
                && file_types.get(&ext).unwrap() == "programming"
            {
                // Parse file
                let Ok(r) = process_valid_file(&mut file, &tokenizer) else {
                    eprintln!("Error on {}", file.name());
                    continue;
                };
                // Write to JSONL
                let Some(_) = write_repo_jsonl(&destination_dir, &zip_path, &r) else {
                    continue;
                };
                file_count = file_count + 1;
            }
        }
    }
    println!("Total files processed = {}", file_count);
    if file_count <= 0 {
        return None;
    }
    Some(())
}
