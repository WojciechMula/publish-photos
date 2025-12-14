use crate::file_name;
use db::Database;
use db::Date;
use db::FileMetadata;
use db::Post;
use log::info;
use std::collections::BTreeSet;
use std::fs::read_dir;
use std::path::Path;
use std::path::PathBuf;

pub fn perform(
    rootdir: &Path,
    db: &mut Database,
) -> Result<usize, Box<dyn std::error::Error + 'static>> {
    let all_files = collect_paths(rootdir)?;
    let mut managed_files = collect_managed_paths(db);

    let mut count = 0;
    for path in all_files {
        if !managed_files.remove(&path) {
            info!("importing {}", path.display());
            let mut post = mk_post(&path);
            post.published = is_published(rootdir, &path);
            db.posts.push(post);
            count += 1;
        }
    }

    Ok(count)
}

fn is_published(rootdir: &Path, path: &Path) -> bool {
    let full_path = rootdir.join(path);
    is_published_aux(&full_path).unwrap_or_default()
}

fn is_published_aux(path: &Path) -> Option<bool> {
    let parent = path.parent()?;

    let name = path.file_name()?;
    let link = parent.join("facebook").join("published").join(name);

    Some(link.exists())
}

fn date_from_path(path: &Path) -> Date {
    for part in path.iter() {
        let Some(part) = part.to_str() else {
            continue;
        };

        if let Ok(date) = part.parse::<Date>() {
            return date;
        }
    }

    panic!(
        "{} does not contain date in form YYYY-MM-DD",
        path.display()
    );
}

fn mk_post(path: &Path) -> Post {
    let md = FileMetadata {
        rel_path: path.to_path_buf(),
        ..Default::default()
    };

    Post {
        files: vec![md],
        date: date_from_path(path),
        ..Default::default()
    }
}

fn collect_managed_paths(db: &Database) -> BTreeSet<PathBuf> {
    let mut result = BTreeSet::<PathBuf>::new();
    for post in db.posts.iter() {
        for entry in &post.files {
            result.insert(entry.rel_path.to_path_buf());
        }
    }

    result
}

fn collect_paths(
    rootdir: &Path,
) -> Result<BTreeSet<PathBuf>, Box<dyn std::error::Error + 'static>> {
    let mut files = BTreeSet::<PathBuf>::new();

    let mut stack: Vec<PathBuf> = vec![rootdir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path.to_path_buf());
                continue;
            }

            if !is_regular_file(&path) {
                continue;
            }

            if file_name(&path).ends_with("_small.jpg") {
                let p = strip_prefix(rootdir, &path);
                files.insert(p);
            }
        }
    }

    Ok(files)
}

fn is_regular_file(path: &Path) -> bool {
    !path.is_symlink() && path.is_file()
}

fn strip_prefix(prefix: &Path, path: &Path) -> PathBuf {
    let mut path_iter = path.iter();
    let mut prefix_iter = prefix.iter();

    loop {
        let Some(prefix) = prefix_iter.next() else {
            break;
        };

        let Some(part) = path_iter.next() else {
            break;
        };

        if prefix != part {
            break;
        }
    }

    path_iter.collect()
}
