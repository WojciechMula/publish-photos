use clap::Parser;
use db::Database;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
struct Options {
    /// Path to db.toml
    #[arg(value_name = "DB")]
    pub db: PathBuf,

    /// Path to directory
    #[arg(value_name = "SRC-DIR")]
    pub src: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let opts = Options::parse();

    println!("loading {}", opts.db.display());
    let mut db = Database::from_file(&opts.db)?;

    let mut modified = false;
    for post in db.posts.iter_mut() {
        let parent = post.file.rel_path.parent().expect("no parent dir");
        let src_dir = opts.src.join(parent).join("500px");
        if !src_dir.is_dir() {
            continue;
        }

        let stem = post.file.rel_path.file_stem().expect("missing file name");
        let stem = format!("{}", stem.display());
        let stem = stem.replace("_small", "");

        if any_exists(&src_dir, &stem) {
            if post.labels.add("500px") {
                modified = true;
            }
        }

        let published = src_dir.join("published");
        if !published.is_dir() {
            continue;
        }

        if any_exists(&published, &stem) {
            if post.labels.add("500px-published") {
                modified = true;
            }
        }
    }

    if modified {
        println!("saving {}", opts.db.display());
        db.save(&opts.db)
    } else {
        Ok(())
    }
}

fn any_exists(src_dir: &Path, stem: &str) -> bool {
    let processed = format!("{stem}_processed");

    let candidates = [
        src_dir.join(&processed).with_extension("jpg"),
        src_dir.join(&processed).with_extension("JPG"),
        src_dir.join(stem).with_extension("jpg"),
        src_dir.join(stem).with_extension("JPG"),
    ];

    candidates.iter().any(|path| path.exists())
}
