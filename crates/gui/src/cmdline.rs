use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Options {
    /// Path to database file
    #[arg(value_name = "DB")]
    pub path: PathBuf,

    /// Search for new photos to publish
    #[arg(long)]
    pub update_db: bool,
}
