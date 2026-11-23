use db::Database;
use db::Post;
use db::Species;
use std::env::args;
use std::path::Path;
use std::path::PathBuf;

type Error = Box<dyn std::error::Error + 'static>;
type Result = std::result::Result<(), Error>;

fn main() -> Result {
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

    let Some(category) = arg.next() else {
        help();
        return Ok(());
    };

    let dst_dir = Path::new("/dev/shm/species");
    if !dst_dir.exists() {
        println!("creating {}", dst_dir.display());
        std::fs::create_dir_all(dst_dir)?;
    } else {
        clean_directory(dst_dir)?;
    }

    let path = PathBuf::from(db_path);
    println!("loading {}", path.display());
    let db = Database::from_file(&path)?;

    let pred = |post: &&Post| -> bool { post.date.year == year && post.species_id.is_some() };

    let mut result = Vec::<Species>::new();
    for species in db.species.iter() {
        if !species.taxonomic_rank.is_species() {
            continue;
        }

        if !species.category.as_ref().is_some_and(|c| *c == category) {
            continue;
        }

        let species_id = Some(species.id());
        let exists = db
            .posts
            .iter()
            .filter(pred)
            .any(|post| post.species_id == species_id);

        if exists {
            result.push(species.clone());
        }
    }

    result.sort_by_key(|s| s.latin.clone());

    for (k, species) in result.iter().enumerate() {
        let species_id = Some(species.id());

        let name = species.latin.as_str();
        let name = name.to_lowercase();
        let name = name.replace(" ", "-");

        let letter = mk_letter(k);

        for (i, post) in db
            .posts
            .iter()
            .filter(pred)
            .filter(|post| post.species_id == species_id)
            .enumerate()
        {
            let src = &post.file.full_path;
            let dst = dst_dir.join(format!("{letter}-{:02}-{name}.jpg", i + 1));
            if !dst.exists() {
                println!("copying {} to {}", src.display(), dst.display());
                std::fs::copy(src, dst)?;
            }
        }
    }

    for (k, species) in result.iter().enumerate() {
        println!(
            "{} | {:30} | {:30} | {:30}",
            mk_letter(k),
            species.latin.as_str(),
            species.pl,
            species.en
        );
    }

    Ok(())
}

fn help() {
    println!("Usage:");
    println!();
    println!("listspecies <db-path> <year> <category>");
}

fn clean_directory(dst_dir: &Path) -> Result {
    for entry in dst_dir.read_dir()? {
        let entry = entry?;
        let path = entry.path();
        println!("removing {}", path.display());
        std::fs::remove_file(path)?;
    }

    Ok(())
}

fn mk_letter(k: usize) -> char {
    if k < 26 {
        (b'A' + k as u8) as char
    } else {
        'Z'
    }
}
