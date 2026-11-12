use clap::Parser;
use db::Database;
use std::collections::BTreeMap;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

#[derive(Parser)]
struct Options {
    /// Path to db.toml
    #[arg(value_name = "DB")]
    pub db: PathBuf,

    /// Path to renamed dir
    #[arg(value_name = "DIR")]
    pub dir: PathBuf,
}

type Renumber = BTreeMap<PathBuf, String>;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let opts = Options::parse();

    if !opts.dir.is_dir() {
        return Err(format!("{} is not a directory", opts.dir.display()).into());
    }

    let mut day2rename = BTreeMap::<String, Renumber>::new();
    for month in [
        "01", "02", "03", "04", "05", "06", "07", "08", "09", "10", "11", "12",
    ] {
        let rootdir = opts.dir.join(month);
        if !rootdir.is_dir() {
            continue;
        }
        for entry in rootdir.read_dir()? {
            let entry = entry?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let publish = path.join("publish");
            if !publish.is_dir() {
                println!("skipping {}", path.display());
                continue;
            }

            println!("loading {}", path.display());
            let lookup = renumber(&path.join("renumber.txt"));
            if lookup.is_empty() {
                panic!("lookup is empty");
            }
            let day = path.file_name().unwrap().to_string_lossy().to_string();

            day2rename.insert(day, lookup);
        }
    }

    let get_lookup = |path: &Path| -> Option<&Renumber> {
        for comp in path.components() {
            let Component::Normal(part) = comp else {
                continue;
            };

            if part.len() != "XXXX-XX-XX".len() {
                continue;
            }

            let part = part.to_string_lossy().to_string();
            if let Some(ren) = day2rename.get(&part) {
                return Some(ren);
            }
        }

        None
    };

    println!("loading {}", opts.db.display());
    let mut db = Database::from_file(&opts.db)?;

    let mut modified = false;
    for post in db.posts.iter_mut() {
        if let Some(lookup) = get_lookup(&post.file.rel_path) {
            let name = post.file.rel_path.file_name().unwrap().to_string_lossy();
            let name = name.strip_suffix("_small.jpg").unwrap();

            let key = if let Some(letter) = get_single_letter(&post.file.rel_path) {
                Path::new(&letter).join(name)
            } else {
                PathBuf::from(name)
            };

            let Some(new) = lookup.get(&key) else {
                println!("{}", post.file.rel_path.display());
                panic!("key now found {}", key.display());
            };

            // rel_path = "2011/04/2011-04-03/publish/P1234567.jpg"
            // p        = "2011/04/2011-04-03/publish"
            let p = post.file.rel_path.parent().unwrap();
            println!("{}", p.display());

            let name = format!("{new}_small.jpg");
            // p        = "2011/04/2011-04-03/publish/new_small.jpg"
            let p = p.join(name);

            println!(
                "updating {} => {}",
                post.file.rel_path.display(),
                p.display()
            );

            post.file.rel_path = strip_single_letter(&p);
            modified = true;
        }
    }

    if modified {
        println!("saving {}", opts.db.display());
        db.save(&opts.db)
    } else {
        Ok(())
    }
}

fn get_single_letter(p: &Path) -> Option<String> {
    for comp in p.components() {
        if let Component::Normal(part) = comp {
            if part.len() == 1 {
                return Some(part.to_string_lossy().to_string());
            }
        }
    }

    None
}

fn strip_single_letter(p: &Path) -> PathBuf {
    p.components()
        .filter(|comp| {
            if let Component::Normal(part) = comp {
                part.len() != 1
            } else {
                true
            }
        })
        .collect()
}

fn renumber(path: &Path) -> BTreeMap<PathBuf, String> {
    let mut r = BTreeMap::<PathBuf, String>::new();
    let s = std::fs::read_to_string(path).unwrap();

    let parent = Some(Path::new(""));
    for line in s.lines() {
        let (key, target) = line.split_once(' ').unwrap();

        let key = Path::new(key).with_extension("");
        let target = Path::new(target);
        assert_eq!(target.parent(), parent);

        let prev = r.insert(
            key,
            target.file_stem().unwrap().to_string_lossy().to_string(),
        );
        assert!(prev.is_none());
    }

    r
}
