use db::Database;
use db::LabelConfig;
use egui::Color32;
use egui::Key;
use egui::Modifiers;

pub struct LabelEntry {
    pub label: String,
    pub shortcut: Option<(Key, Modifiers)>,
    pub color: Color32,
    pub text_color: Color32,
}

pub fn from_db(db: &Database) -> Vec<LabelEntry> {
    let mut result = Vec::<LabelEntry>::with_capacity(db.labels.len());
    for config in db.labels.iter() {
        result.push(config.into());
    }

    result
}

pub fn update_db(db: &mut Database, entries: &[LabelEntry]) {
    db.labels.clear();
    db.current_version.labels += 1;

    for entry in entries {
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
        }
    }
}

impl From<&LabelEntry> for LabelConfig {
    fn from(v: &LabelEntry) -> Self {
        use crate::colors::color_name;
        use crate::keyboard::format_shortcut;

        let shortcut = if let Some((key, modifiers)) = &v.shortcut {
            format_shortcut(key, modifiers)
        } else {
            "".to_owned()
        };

        Self {
            label: v.label.clone(),
            shortcut,
            color: color_name(v.color).unwrap_or("").to_owned(),
            text_color: color_name(v.text_color).unwrap_or("").to_owned(),
        }
    }
}
