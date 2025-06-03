mod cli;
mod downloader;
mod extractor;
mod source;

mod commands {
    pub mod download;
    pub mod extract;
    // pub mod refresh;
}

use cli::{Command, Opt};
use structopt::StructOpt;

#[tokio::main]
async fn main() {
    let opts = Opt::from_args();
    match opts.cmd {
        Command::Download { source, workers } => {
            commands::download::run(source, workers).await;
        }
        Command::Refresh { source, workers } => {
            commands::download::run(source, workers).await;
        }
        Command::Extract { source, workers } => {
            commands::extract::run(source, workers).await;
        }
    }
}
