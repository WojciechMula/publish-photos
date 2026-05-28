use db::Database;
use std::env::args;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    for arg in args().skip(1) {
        let path = PathBuf::from(arg);
        println!("loading {}", path.display());
        let db = Database::from_file(&path)?;
        for post in db.posts.iter() {
            for file in &post.files {
                if !file.full_path.exists() {
                    println!("{} not found", file.full_path.display());
                }
            }

            if let Some(name) = &post.species {
                if db.species_by_latin(name).is_none() {
                    println!("species {name} not found");
                }
            }
        }
    }

    Ok(())
}
