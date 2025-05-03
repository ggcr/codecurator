use std::path::PathBuf;

use structopt::StructOpt;

// CLI Args
#[derive(Debug, StructOpt)]
pub struct Opt {
    pub source: PathBuf,
}
