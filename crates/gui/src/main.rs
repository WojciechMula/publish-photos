use clap::Parser;
use db::Database;
use env_logger::Builder;
use log::error;
use log::info;
use log::LevelFilter;
use photos::application::Application;
use photos::cmdline::Options;
use std::path::absolute;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut builder = Builder::from_default_env();
    builder.filter_level(LevelFilter::Info).init();

    let opts = Options::parse();
    let rootdir = match absolute(&opts.rootdir) {
        Ok(path) => path,
        Err(_) => opts.rootdir.to_path_buf(),
    };

    let path = rootdir.join("db.toml");
    let mut db = if path.is_file() {
        Database::from_file(&path)?
    } else {
        info!(
            "File {} not found, trying to create a fresh database",
            path.display()
        );
        let mut db = Database::new(&path);
        let count = photos::sync_db::perform(&rootdir, &mut db)?;
        if count == 0 {
            error!("No photos matching the program criteria was found");
            return Ok(());
        }

        info!("Imported {count} image(s)");

        db.refresh_all_records();
        db.current_version.posts += 1;

        db
    };

    if opts.update_db {
        photos::sync_db::perform(&rootdir, &mut db)?;
        db.refresh_all_records();
        db.current_version.posts += 1;
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
