mod cli;
mod downloader;
mod extractor;
mod source;

use std::path::Path;
use std::path::PathBuf;

use cli::Opt;
use colored::Colorize;
use downloader::download_repos;
use extractor::extract_text;
use extractor::read_linguist;
use source::parse_source;
use structopt::StructOpt;
use tokenizers::tokenizer::{Result, Tokenizer};

#[tokio::main]
async fn main() {
    let opts = Opt::from_args();
    let source: PathBuf = opts.source;

    if !source.exists() {
        panic!("Source does not exists");
    }

    if source.extension().expect("Could not read extension") != "jsonl" {
        panic!("Source is not a valid JSON lines file");
    }

    println!("Source: {}", source.to_string_lossy().blue());

    // Read source file
    let uris: Vec<(String, String)> = parse_source(&source);
    if uris.is_empty() {
        eprintln!(
            "{} No valid URIs found in source file",
            "[WARNING]".truecolor(214, 143, 0)
        );
    }

    // Download
    let download_workers = 16;
    let paths = download_repos(uris, download_workers)
        .await
        .expect("No content has been downloaded.");

    // Extract
    let gpt2tokenizer = Tokenizer::from_pretrained("openai-community/gpt2", None)
        .expect("Failed to load the tokenier");
    let linguist_file_types = read_linguist(Path::new("vendor/languages.yml"))
        .expect("Unable to read linguist languages yaml");
    let extract_workers = 32;
    let paths = extract_text(paths, linguist_file_types, gpt2tokenizer, extract_workers);
}
