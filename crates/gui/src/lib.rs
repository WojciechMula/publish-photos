pub mod application;
pub mod cmdline;
mod colors;
mod confirm;
mod gui;
mod help;
mod image_cache;
mod image_counter;
mod keyboard;
mod modal;
mod modal_errors;
mod modal_keyboard;
mod modal_settings;
mod search_box;
mod select_tags;
mod species_view;
mod style;
pub mod sync_db;
mod tab_posts;
mod tab_species;
mod tab_tag_groups;
mod tab_tag_translations;
mod widgets;

pub use image_counter::ImageCounter;

// --------------------------------------------------

use chrono::DateTime;
use chrono::Local;
use chrono::Utc;

pub type LocalDateTime = DateTime<Local>;
pub type UtcDateTime = DateTime<Utc>;

// --------------------------------------------------

use std::path::Path;

pub fn file_name(path: &Path) -> String {
    let Some(stem) = path.file_name() else {
        return String::new();
    };

    if let Some(utf8stem) = stem.to_str() {
        utf8stem.to_owned()
    } else {
        String::new()
    }
}

pub fn file_stem(path: &Path) -> String {
    let Some(stem) = path.file_stem() else {
        return String::new();
    };

    if let Some(utf8stem) = stem.to_str() {
        utf8stem.to_owned()
    } else {
        String::new()
    }
}
