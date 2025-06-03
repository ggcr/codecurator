use std::path::PathBuf;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Command {
    Download {
        #[structopt(parse(from_os_str))]
        source: PathBuf,

        #[structopt(short, long, default_value = "16")]
        workers: usize,
    },
    Refresh {
        #[structopt(parse(from_os_str))]
        source: PathBuf,

        #[structopt(short, long, default_value = "32")]
        workers: usize,
    },
    Extract {
        #[structopt(parse(from_os_str))]
        source: PathBuf,

        #[structopt(short, long, default_value = "32")]
        workers: usize,
    },
}

// CLI Args
#[derive(Debug, StructOpt)]
#[structopt(name = "codecurator")]
pub struct Opt {
    // download, refresh, extract
    #[structopt(subcommand)]
    pub cmd: Command,
}
