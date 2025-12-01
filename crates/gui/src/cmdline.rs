use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct Options {
    #[arg(value_name = "DIR")]
    pub rootdir: PathBuf,

    #[arg(long)]
    pub update_db: bool,

    #[arg(value_name = "TOML")]
    #[arg(default_value = "~/.facebook")]
    pub socmedia: PathBuf,
}
