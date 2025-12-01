use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Options {
    /// Directory containaing db.toml file
    #[arg(value_name = "DIR")]
    pub rootdir: PathBuf,

    /// Search for new photos to publish
    #[arg(long)]
    pub update_db: bool,

    /// Location of social media credentials
    #[arg(value_name = "TOML")]
    #[arg(default_value = "~/.facebook")]
    pub socmedia: PathBuf,

    /// Do not publish anything using Facebook/Instagram API
    #[arg(default_value_t = false)]
    #[arg(long)]
    pub disable_socmedia: bool,
}
