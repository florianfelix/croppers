use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Cli {
    pub path: Option<PathBuf>,

    #[arg(short, long, default_value = "croppers")]
    pub app_id: String,

    #[arg(short, long, default_value_t = false)]
    pub fullscreen: bool,
}
