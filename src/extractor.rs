use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use std::io::BufReader;
use yaml_rust::YamlLoader;

// Things we need to do:
// 1. Unzip each file
// 2. Walk it
// 3. Read
// 4. Write

pub fn read_linguist(path: &Path) -> anyhow::Result<HashMap<String, String>> {
    let p = path.to_path_buf();
    let fc = std::fs::read_to_string(p)?;
    let docs = YamlLoader::load_from_str(&fc).unwrap();
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

pub fn extract_text(
    zip_paths: Vec<PathBuf>,
    file_types: HashMap<String, String>,
    workers: usize,
) -> Option<()> {
    // Download & Write
    let destination_dir = Path::new("./jsonl/");
    fs::create_dir_all(destination_dir).ok()?;

    for zip_path in zip_paths {
        // Open file
        let f = fs::File::open(&zip_path).ok()?;
        let reader = BufReader::new(f);
        // Zip Reader
        let mut zip = zip::ZipArchive::new(reader).ok()?;

        for i in 0..zip.len() {
            let file = zip.by_index(i).ok()?;
            // If we are in a file + it has extension
            let try_ext: Vec<&str> = file.name().splitn(2, ".").collect();
            if file.is_file() && (try_ext.len() == 2 && try_ext.last().is_some()) {
                println!(
                    "Filename: {} \t Ext: {}",
                    file.name(),
                    try_ext.last().unwrap()
                );
            }
        }
    }
    Some(())
}
