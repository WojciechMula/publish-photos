use db::Database;
use db::LabelConfig;
use egui::Color32;
use egui::KeyboardShortcut;
use std::collections::BTreeMap;

pub struct LabelEntry {
    pub label: String,
    pub shortcut: Option<KeyboardShortcut>,
    pub color: Color32,
    pub text_color: Color32,
    pub enabled: bool,
}

pub fn from_db(db: &Database) -> BTreeMap<String, LabelEntry> {
    let mut result = BTreeMap::<String, LabelEntry>::new();
    for config in db.labels.iter() {
        let label: LabelEntry = config.into();
        result.insert(label.label.clone(), label);
    }

    result
}

pub fn update_db(db: &mut Database, entries: &BTreeMap<String, LabelEntry>) {
    db.labels.clear();
    db.current_version.labels += 1;

    for entry in entries.values() {
        db.labels.push(entry.into());
    }
}

impl LabelEntry {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
            shortcut: None,
            color: Color32::BLACK,
            text_color: Color32::WHITE,
            enabled: true,
        }
    }
}

impl From<&LabelConfig> for LabelEntry {
    fn from(v: &LabelConfig) -> Self {
        use crate::colors::color_by_name;

        Self {
            label: v.label.clone(),
            shortcut: crate::keyboard::from_str(&v.shortcut).ok(),
            color: color_by_name(&v.color).unwrap_or(crate::colors::BLACK),
            text_color: color_by_name(&v.text_color).unwrap_or(crate::colors::WHITE),
            enabled: v.enabled,
        }
    }
}

impl From<&LabelEntry> for LabelConfig {
    fn from(v: &LabelEntry) -> Self {
        use crate::colors::color_name;
        use crate::keyboard::format_shortcut;

        let shortcut = if let Some(shortcut) = &v.shortcut {
            format_shortcut(&shortcut.logical_key, &shortcut.modifiers)
        } else {
            "".to_owned()
        };

        Self {
            label: v.label.clone(),
            shortcut,
            color: color_name(v.color).unwrap_or("").to_owned(),
            text_color: color_name(v.text_color).unwrap_or("").to_owned(),
            enabled: v.enabled,
        }
    }
}
