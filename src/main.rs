mod cli;
mod config;
mod downloader;
mod error;
mod extractor;
mod source;

mod commands {
    pub mod download;
    pub mod extract;
    // pub mod refresh;
}

use cli::{Command, Opt};
use config::{DownloadConfig, ExtractionConfig};
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let opts = Opt::from_args();
    println!("{:?}", opts);
    match opts.cmd {
        Command::Download { .. } => {
            let config = DownloadConfig::from_cli(&opts.cmd);
            commands::download::run(&config).await;
        }
        Command::Extract { .. } => {
            let config = ExtractionConfig::from_cli(&opts.cmd);
            commands::extract::run(&config).await;
        }
    }
}
