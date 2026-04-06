mod species;
mod trans;

use db::Database;
use std::env::args;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut databases = Vec::<Database>::new();
    for arg in args().skip(1) {
        let path = PathBuf::from(arg);
        println!("loading {}", path.display());
        let db = Database::from_file(&path)?;
        databases.push(db);
    }

    species::update_species(&mut databases);
    trans::merge(&mut databases);

    for db in databases.iter_mut().filter(|db| db.is_dirty()) {
        println!("saving {}", db.rootpath.display());
        let path = db.rootpath.clone();
        db.save(&path)?;
    }

    Ok(())
}
