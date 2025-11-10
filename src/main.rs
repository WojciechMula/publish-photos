use clap::Parser;
use env_logger::Builder;
use log::error;
use log::info;
use photos::application::Application;
use photos::cmdline::Options;
use photos::db::Database;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut builder = Builder::from_default_env();

    builder.init();

    let opts = Options::parse();
    let path = opts.rootdir.join("db.toml");
    let mut db = if path.is_file() {
        Database::from_file(&path)?
    } else {
        info!(
            "File {} not found, trying to create a fresh database",
            path.display()
        );
        let mut db = Database::new(&path);
        let count = photos::sync_db::perform(&opts.rootdir, &mut db)?;
        if count == 0 {
            error!("No photos matching the program criteria was found");
            return Ok(());
        }

        db.refresh_all_records();
        db.mark_dirty();

        db
    };

    if opts.update_db {
        photos::sync_db::perform(&opts.rootdir, &mut db)?;
        db.refresh_all_records();
        db.mark_dirty();
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(false),
        ..Default::default()
    };

    Ok(eframe::run_native(
        "Publish photos",
        native_options,
        Box::new(|_cc| Ok(Box::new(Application::new(db)))),
    )?)
}
