use clap::Parser;
use db::Database;
use env_logger::Builder;
use log::info;
use log::LevelFilter;
use photos::application::Application;
use photos::cmdline::Options;
use photos::err;
use std::path::absolute;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info).init();

    let opts = Options::parse();
    let path = absolute(&opts.path).unwrap_or_else(|_| opts.path.to_path_buf());
    let rootdir = match path.parent() {
        Some(path) => path,
        None => return err!("'{}' doesn't have parent directory", path.display()),
    };

    let mut db = if path.is_file() {
        Database::from_file(&path)?
    } else {
        info!(
            "File {} not found, trying to create a fresh database",
            path.display()
        );
        let mut db = Database::new(&path);
        let count = photos::sync_db::perform(rootdir, &mut db)?;
        if count == 0 {
            return err!("No photos matching the program criteria was found");
        }

        info!("Imported {count} image(s)");

        db.refresh_all_records();
        db.current_version.posts += 1;

        db
    };

    if opts.update_db {
        let new = photos::sync_db::perform(rootdir, &mut db)?;
        db.refresh_all_records();
        db.force_refresh_caches();
        if new > 0 {
            db.current_version.photos += 1;
        }
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(false),
        ..Default::default()
    };

    Ok(eframe::run_native(
        &format!("Publish photos: {}", path.display()),
        native_options,
        Box::new(|_cc| Ok(Box::new(Application::new(db)))),
    )?)
}
