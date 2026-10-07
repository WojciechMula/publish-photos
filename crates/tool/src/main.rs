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
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let opts = Options::parse();

    let src_dir = find_source_dir(Path::new("/mnt/"), opts.year)?;
    println!("found source for {}: {}", opts.year, src_dir.display());

    let path = PathBuf::from(opts.path);
    println!("loading {}", path.display());
    let db = Database::from_file(&path)?;

    let dst_dir = Path::new("/dev/shm/wydruk");

    let pred = |post: &&Post| -> bool {
        post.labels.iter().any(|s| s == "wydruk") && post.date.year == opts.year
    };

    for post in db.posts.iter().filter(pred) {
        for file in &post.files {
            let path_str = format!("{}", file.rel_path.display());
            let bare = path_str.replace("_small", "");

            let stem = mk_dest_stem(&bare);

            let processed = path_str.replace("_small", "_processed");
            let src3 = src_dir.join(&processed);
            if src3.exists() {
                let dst = dst_dir.join(&stem);
                if !dst.exists() {
                    println!("cp {} {}", src3.display(), dst.display());
                    let data = std::fs::read(&src3)?;
                    std::fs::write(dst, data)?;
                }

                continue;
            }

            let src1 = src_dir.join(&bare);
            if src1.exists() {
                let dst = dst_dir.join(&stem);
                if !dst.exists() {
                    println!("cp {} {}", src1.display(), dst.display());
                    let data = std::fs::read(&src1)?;
                    std::fs::write(dst, data)?;
                }

                continue;
            }

            let src2 = src1.with_extension("JPG");
            if src2.exists() {
                let dst = dst_dir.join(&stem).with_extension("JPG");
                if !dst.exists() {
                    println!("cp {} {}", src2.display(), dst.display());
                    let data = std::fs::read(&src2)?;
                    std::fs::write(dst, data)?;
                }

                continue;
            }

            panic!(
                "`{}`/`{}`/`{}` not found",
                src1.display(),
                src2.display(),
                src3.display()
            );
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
    let mut iter = path.split("/").skip(2).filter(|part| *part != "publish");

    for part in iter {
        if !res.is_empty() {
            res += "-";
        }
        res += &part;
    }

    res
}
