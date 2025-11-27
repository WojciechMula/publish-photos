use clap::Parser;
use env_logger::Builder;
use log::error;
use log::info;
use photos::application::Application;
use photos::cmdline::Options;
use photos::db::Database;
use photos::GraphApiCredentials;
use std::env::home_dir;
use std::path::Path;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let mut builder = Builder::from_default_env();
    builder.init();

    let opts = Options::parse();

    let mut graph_api_credentials: Option<GraphApiCredentials> = None;
    let socmedia = expand_homedir(&opts.socmedia);
    if socmedia.exists() {
        let gac = GraphApiCredentials::from_file(&socmedia)?;
        graph_api_credentials = Some(gac);
    }

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
        db.current_version.posts += 1;

        db
    };

    if opts.update_db {
        photos::sync_db::perform(&opts.rootdir, &mut db)?;
        db.refresh_all_records();
        db.current_version.posts += 1;
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(false),
        ..Default::default()
    };

    Ok(eframe::run_native(
        "Publish photos",
        native_options,
        Box::new(|_cc| Ok(Box::new(Application::new(db, graph_api_credentials)))),
    )?)
}

pub fn expand_homedir(path: &Path) -> PathBuf {
    match expand_homedir_aux(path) {
        None => PathBuf::from(path),
        Some(path) => path,
    }
}

fn expand_homedir_aux(path: &Path) -> Option<PathBuf> {
    let path = path.strip_prefix("~/").ok()?;
    let home = home_dir()?;

    Some(home.join(path))
}
