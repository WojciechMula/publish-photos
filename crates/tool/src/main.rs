use db::Database;
use db::Post;
use std::env::args;
use std::path::Path;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    for arg in args().skip(1) {
        let path = PathBuf::from(arg);
        println!("loading {}", path.display());
        let db = Database::from_file(&path)?;

        fn pred(post: &Post) -> bool {
            post.labels.iter().any(|s| s == "wydruk") && post.date.year == 2025
        }

        let src_dir = Path::new("/mnt/ext7/photos-bak");
        let dst_dir = Path::new("/dev/shm/wydruk");
        for post in db.posts.iter().filter(|post| pred(post)) {
            for file in &post.files {
                let path = format!("{}", file.rel_path.display());
                let path = path.replace("_small", "");

                let dst = dst_dir.join(&mk_dest_stem(&path));
                if dst.exists() {
                    continue;
                }

                let src1 = src_dir.join(&path);
                let src2 = src1.with_extension("JPG");

                if src1.exists() {
                    println!("cp {} {}", src1.display(), dst.display());
                    let data = std::fs::read(src1)?;
                    std::fs::write(dst, data)?;
                } else if src2.exists() {
                    println!("cp {} {}", src2.display(), dst.display());
                    let data = std::fs::read(src2)?;
                    std::fs::write(dst, data)?;
                } else {
                    panic!("{}/{} not exists", src1.display(), src2.display());
                }
            }
        }
    }

    Ok(())
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
