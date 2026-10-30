use db::Database;
use std::env::args;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    for arg in args().skip(1) {
        let path = PathBuf::from(arg);
        println!("loading {}", path.display());
        let db = Database::from_file(&path)?;
        for post in db.posts.iter() {
            let path = &post.file.full_path;
            if !path.exists() {
                println!("{} not found", path.display());
            }

            if let Some(species_id) = &post.species_id {
                if db.species_by_id(species_id).is_none() {
                    println!("species {species_id} not found");
                }
            }
        }
    }

    Ok(())
}
