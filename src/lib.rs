pub mod application;
pub mod cmdline;
mod colors;
mod confirm;
pub mod db;
mod edit_details;
mod edit_tags;
mod gui;
mod help;
mod keyboard;
mod modal;
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
mod tag_hints;
mod widgets;

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
