use clap::Parser;
use db::Database;
use db::Post;
use std::path::Path;
use std::path::PathBuf;

type Error = Box<dyn std::error::Error + 'static>;

#[macro_export]
macro_rules! err {
    ($($arg:expr),*) => {
        Err(format!($($arg,)*).into())
    }
}

#[derive(Parser)]
struct Options {
    /// Path to db.toml
    #[arg(value_name = "DB", long)]
    pub path: String,

    /// Year
    #[arg(value_name = "YEAR", long)]
    pub year: u16,

    /// Tag name
    #[arg(value_name = "YEAR", long, default_value = "wydruk")]
    pub tag: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let opts = Options::parse();

    let src_dir = find_source_dir(Path::new("/mnt/"), opts.year)?;
    println!("found source for {}: {}", opts.year, src_dir.display());

    let path = PathBuf::from(opts.path);
    println!("loading {}", path.display());
    let db = Database::from_file(&path)?;

    let dst_dir = Path::new("/dev/shm/wydruk");

    let mut printed = false;
    for entry in dst_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if !printed {
            println!("cleaning {}", dst_dir.display());
            printed = true;
        }
        std::fs::remove_file(path)?;
    }

    let pred = |post: &&Post| -> bool {
        post.labels.iter().any(|s| *s == opts.tag) && post.date.year == opts.year
    };

    for post in db.posts.iter().filter(pred) {
        let path_str = format!("{}", post.file.rel_path.display());
        let bare = path_str.replace("_small", "");

        let stem = mk_dest_stem(&bare);

        let processed = path_str.replace("_small", "_processed");

        let candidates = [
            src_dir.join(&processed).with_extension("jpg"),
            src_dir.join(&processed).with_extension("JPG"),
            src_dir.join(&bare).with_extension("jpg"),
            src_dir.join(&bare).with_extension("JPG"),
        ];

        let mut found = false;
        for src in &candidates {
            if src.exists() {
                let dst = dst_dir.join(&stem);
                println!("cp {} {}", src.display(), dst.display());
                let data = std::fs::read(src)?;
                std::fs::write(dst, data)?;

                found = true;
                break;
            }
        }

        if !found {
            eprintln!("{path_str}: source JPEG for not found",);

            for (i, path) in candidates.iter().enumerate() {
                eprintln!("#{i}: {}", path.display());
            }
        }
    }

    Ok(())
}

fn find_source_dir(rootdir: &Path, year: u16) -> Result<PathBuf, Error> {
    let year = year.to_string();

    for entry in rootdir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        for subdir in ["photos", "photos-bak"] {
            let rootdir = path.join(subdir);
            let src = rootdir.join(&year);
            if src.is_dir() {
                return Ok(rootdir);
            }
        }
    }

    err!(
        "Cannot find `{}/**/photos{{-bak}}/{year}`",
        rootdir.display()
    )
}

fn mk_dest_stem(path: &str) -> String {
    let mut res = String::new();
    let iter = path.split("/").skip(2).filter(|part| *part != "publish");

    for part in iter {
        if !res.is_empty() {
            res += "-";
        }
        res += part;
    }

    res
}
