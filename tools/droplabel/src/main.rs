use clap::Parser;
use db::Database;
use std::path::PathBuf;

#[derive(Parser)]
struct Options {
    /// Path to db.toml
    #[arg(value_name = "DB")]
    pub path: String,

    /// Label name
    #[arg(value_name = "LABEL")]
    pub label: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let opts = Options::parse();

    let path = PathBuf::from(opts.path);
    println!("loading {}", path.display());
    let mut db = Database::from_file(&path)?;

    let mut modified = false;
    for post in db.posts.iter_mut() {
        if post.labels.remove(&opts.label) {
            modified = true;
        }
    }

    if modified {
        println!("saving {}", path.display());
        db.save(&path)
    } else {
        Ok(())
    }
}
