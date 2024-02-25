use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the config file
    #[arg(short, long, value_name = "FILE")]
    pub config_path: String,
}
