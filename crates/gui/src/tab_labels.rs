use crate::colors::select_color;
use crate::keyboard::KeyboardMapping;
use crate::labels::LabelEntry;
use crate::widgets::label_button;
use crate::widgets::Shortcut;
use const_format::formatcp as fmt;
use db::Database;
use egui::Button;
use egui::CentralPanel;
use egui::Context;
use egui::Event;
use egui::Grid;
use egui::Key;
use egui::KeyboardShortcut;
use egui::ScrollArea;
use egui::Ui;
use std::collections::HashSet;

use egui_material_icons::icons::ICON_DELETE;

const ID_PREFIX: &str = "tab-labels";

#[derive(Default)]
pub struct TabLabels {
    pub keyboard_mapping: KeyboardMapping,
    cache: Vec<LabelEntry>,
    needs_sync: bool,
    new: String,
    wait_for_key: Option<usize>,
    taken_shortcuts: HashSet<KeyboardShortcut>,
    shortcut_error: Option<String>,
}

impl TabLabels {
    pub fn register_taken_shortcuts(&mut self, km: &KeyboardMapping) {
        for (key, list) in km.iter() {
            for (modifiers, _) in list.iter() {
                let shortcut = KeyboardShortcut::new(*modifiers, *key);
                self.taken_shortcuts.insert(shortcut);
            }
        }
    }

    pub fn update(&mut self, ctx: &Context, db: &mut Database) {
        self.refresh_cache(db);
        self.read_key(ctx);

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut self.new);
                let enabled = !self.new.trim().is_empty();
                let button = Button::new("➕Add new");
                if ui.add_enabled(enabled, button).clicked() {
                    self.cache.push(LabelEntry::new(&self.new));
                    self.new.clear();
                }
            });

            ui.separator();

            ScrollArea::vertical()
                .id_salt(fmt!("{ID_PREFIX}-scroll-area"))
                .auto_shrink(false)
                .show(ui, |ui| {
                    self.show_entries(ui);
                });
        });

        if self.needs_sync {
            crate::labels::update_db(db, &self.cache);
            self.needs_sync = false;
        }
    }

    fn show_entries(&mut self, ui: &mut Ui) {
        let shortcut_color = ui.visuals().strong_text_color();

        let mut to_remove: Option<usize> = None;

        Grid::new((ID_PREFIX, "grid"))
            .num_columns(3)
            .show(ui, |ui| {
                for (id, entry) in self.cache.iter_mut().enumerate() {
                    // column #1
                    ui.add(label_button(&entry.label, entry.color, entry.text_color));

                    // column #2
                    ui.horizontal(|ui| {
                        if let Some(shortcut) = entry.shortcut.as_ref() {
                            ui.add(Shortcut::from_shortcut(shortcut).with_color(shortcut_color));
                        }

                        match self.wait_for_key {
                            None => {
                                if ui.button("change").clicked() {
                                    self.shortcut_error = None;
                                    self.wait_for_key = Some(id);
                                }
                            }
                            Some(index) => {
                                if index == id {
                                    if let Some(error) = &self.shortcut_error {
                                        ui.label(error);
                                    } else {
                                        ui.label("press a key");
                                    }
                                }
                            }
                        }
                    });

                    // column #3
                    if select_color(ui, &format!("{ID_PREFIX}-{id}-bg"), &mut entry.color) {
                        self.needs_sync = true;
                    }

                    // column #4
                    if select_color(ui, &format!("{ID_PREFIX}-{id}-fg"), &mut entry.text_color) {
                        self.needs_sync = true;
                    }

                    // column #5
                    if ui.button(ICON_DELETE).clicked() {
                        to_remove = Some(id)
                    }

                    ui.end_row();
                }
            });

        if let Some(idx) = to_remove {
            self.cache.remove(idx);
            self.needs_sync = true;
        }
    }

    fn refresh_cache(&mut self, db: &Database) {
        if !self.cache.is_empty() {
            return;
        }

        self.cache = crate::labels::from_db(db);
    }

    fn read_key(&mut self, ctx: &Context) {
        let Some(id) = self.wait_for_key else {
            return;
        };

        let Some(shortcut) = ctx.input(|i| {
            for event in &i.events {
                if let Event::Key {
                    key,
                    modifiers,
                    pressed: true,
                    ..
                } = event
                {
                    return Some(KeyboardShortcut::new(*modifiers, *key));
                }
            }

            None
        }) else {
            return;
        };

        if shortcut.logical_key == Key::Escape {
            self.wait_for_key = None;
            return;
        }

        if shortcut.logical_key == Key::Delete {
            self.wait_for_key = None;
            self.cache[id].shortcut = None;
            self.needs_sync = true;
            return;
        }

        if self.taken_shortcuts.contains(&shortcut) {
            self.shortcut_error = Some("already taken".to_string());
            return;
        }

        for (k, entry) in self.cache.iter_mut().enumerate() {
            if k == id {
                entry.shortcut = Some(shortcut);
            } else if entry.shortcut == Some(shortcut) {
                entry.shortcut = None;
            }
        }

        self.wait_for_key = None;
        self.needs_sync = true;
    }
}
