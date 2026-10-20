use db::Database;
use db::Month;
use db::Post;
use std::env::args;
use std::path::Path;
use std::path::PathBuf;

type Error = Box<dyn std::error::Error + 'static>;

#[macro_export]
macro_rules! err {
    ($($arg:expr),*) => {
        Err(format!($($arg,)*).into())
    }
}

const POSTS: usize = 5;

fn main() -> Result<(), Error> {
    let mut arg = args().skip(1);

    let Some(db_path) = arg.next() else {
        help();
        return Ok(());
    };

    let Some(year) = arg.next() else {
        help();
        return Ok(());
    };

    let year = year.parse::<u16>()?;

    let Some(label) = arg.next() else {
        help();
        return Ok(());
    };

    let (month, category, num) = parse_label(&label)?;

    let path = PathBuf::from(db_path);
    println!("loading {}", path.display());
    let db = Database::from_file(&path)?;

    let dst_dir = Path::new("/dev/shm/post");
    assert!(dst_dir.exists());
    assert!(dst_dir.is_dir());

    for entry in dst_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        println!("removing {}", path.display());
        std::fs::remove_file(path)?;
    }

    let prefix = format!("{:02}-{}", month.as_u8(), category.part());
    let pred = |post: &&Post| -> bool {
        post.date.month == month
            && post.date.year == year
            && post.labels.iter().any(|s| s.starts_with(&prefix))
    };

    let total = db.posts.iter().filter(pred).count();
    let count = total.div_ceil(POSTS);

    let mut result = format!(
        "{} w {} (cz. {num} z {count})",
        pl_name(&month),
        match category {
            Category::Yard => "ogrodzie",
            Category::BaryczValley => "Dolinie Baryczy",
        }
    );

    let pred = |post: &&Post| -> bool {
        post.date.month == month
            && post.date.year == year
            && post.labels.iter().any(|s| *s == label)
    };

    let mut n = 0;
    for post in db.posts.iter().filter(pred) {
        let Some(latin) = post.species.as_ref() else {
            continue;
        };

        let species = db.species_by_latin(latin).unwrap();

        if n == 0 {
            result += ": ";
        } else {
            result += ", ";
        }

        n += 1;

        if !species.pl.is_empty() {
            if let Some((first, _)) = species.pl.split_once(',') {
                result += &first.to_lowercase();
            } else {
                result += &species.pl.to_lowercase();
            }
        } else {
            if !species.taxonomic_rank.is_species() {
                result += species.taxonomic_rank.pl_name();
                result += " ";
            }
            result += &latin.as_str().to_lowercase();
        }

        for (k, file) in post.files.iter().enumerate() {
            let src = &file.full_path;
            let dst = dst_dir.join(format!("{n}-{k:02}.jpg"));

            println!("copying {} to {}", src.display(), dst.display());
            let buf = std::fs::read(src)?;
            std::fs::write(dst, buf)?;
        }
    }

    for post in db.posts.iter().filter(pred) {
        if post.species.is_some() {
            continue;
        }

        n += 1;

        for (k, file) in post.files.iter().enumerate() {
            let src = &file.full_path;
            let dst = dst_dir.join(format!("{n}-{k:02}.jpg"));

            println!("copying {} to {}", src.display(), dst.display());
            let buf = std::fs::read(src)?;
            std::fs::write(dst, buf)?;
        }
    }

    println!();
    println!("{result}");
    println!();

    Ok(())
}

fn help() {
    println!("Usage:");
    println!();
    println!("mkpost <db-path> <year> <label>");
}

#[derive(Debug)]
enum Category {
    Yard,
    BaryczValley,
}

impl Category {
    fn new(s: &str) -> Result<Self, Error> {
        match s {
            "yard" => Ok(Self::Yard),
            "barycz" => Ok(Self::BaryczValley),
            _ => err!("'{s}' is not a valid category"),
        }
    }

    fn part(&self) -> &'static str {
        match self {
            Self::Yard => "yard",
            Self::BaryczValley => "barycz",
        }
    }
}

fn parse_label(label: &str) -> Result<(Month, Category, usize), Error> {
    let mut part = label.split('-');

    let Some(s) = part.next() else {
        return err!("missing month number");
    };

    let k = s.parse::<usize>()?;
    let month = Month::new(k)?;

    let Some(s) = part.next() else {
        return err!("missing category name");
    };

    let category = Category::new(s)?;

    let Some(num) = part.next() else {
        return err!("missing post number");
    };

    let num = num.parse::<usize>()?;

    if part.next().is_some() {
        return err!("too many parts");
    }

    Ok((month, category, num))
}

fn pl_name(m: &Month) -> &'static str {
    match m.as_u8() {
        1 => "Styczeń",
        2 => "Luty",
        3 => "Marzec",
        4 => "Kwiecień",
        5 => "Maj",
        6 => "Czerwiec",
        7 => "Lipiec",
        8 => "Sierpień",
        9 => "Wrzesień",
        10 => "Październik",
        11 => "Listopad",
        12 => "Grudzień",
        _ => unreachable!(),
    }
}
