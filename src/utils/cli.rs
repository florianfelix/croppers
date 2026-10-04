use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    // #[arg(short, long, value_name = "FILE")]
    pub path: Option<PathBuf>,
}
