// src/commands/extract.rs

use crate::commands::extract::listdir;
use crate::config::DedupeConfig;
use crate::deduplication::exact_deduplication;
use colored::Colorize;

pub async fn run(ctx: &DedupeConfig) {
    // List zip dir
    let paths = match listdir(&ctx.jsonl_dir, "jsonl".to_string()) {
        Ok(paths) => paths,
        Err(e) => {
            eprintln!("{} {}", "[WARNING]".truecolor(214, 143, 0), e);
            return;
        }
    };

    // Compute hashes per each document
    let ed_records = exact_deduplication(paths);
}
