use crate::commands::extract::listdir;
use crate::config::DedupeConfig;
use crate::deduplication::{exact_deduplication, fuzzy_deduplication};
use crate::source::parse_source_as_hashset;
use colored::Colorize;

use super::extract::filter_listdir_by_source;

pub async fn run(ctx: &DedupeConfig) {
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
