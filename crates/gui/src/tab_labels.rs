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
use egui::ScrollArea;
use egui::Ui;

use egui_material_icons::icons::ICON_DELETE;

const ID_PREFIX: &str = "tab-labels";

#[derive(Default)]
pub struct TabLabels {
    new: String,
    pub keyboard_mapping: KeyboardMapping,
    cache: Vec<LabelEntry>,
    wait_for_key: Option<usize>,
    needs_sync: bool,
}

impl TabLabels {
    pub fn update(&mut self, ctx: &Context, db: &mut Database) {
        self.refresh_cache(db);

        if let Some(id) = self.wait_for_key {
            ctx.input(|i| {
                for event in &i.events {
                    if let Event::Key {
                        key,
                        modifiers,
                        pressed: true,
                        ..
                    } = event
                    {
                        if *key != Key::Escape {
                            self.cache[id].shortcut = Some((*key, *modifiers));
                            self.needs_sync = true;
                        }
                        self.wait_for_key = None;
                        break;
                    }
                }
            });
        }

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
                        if let Some((key, modifiers)) = entry.shortcut.as_ref() {
                            ui.add(
                                Shortcut::from_key_and_modifiers(*key, *modifiers)
                                    .with_color(shortcut_color),
                            );
                        }

                        if self.wait_for_key.is_none() {
                            if ui.button("change").clicked() {
                                self.wait_for_key = Some(id);
                            }
                        } else if self.wait_for_key == Some(id) {
                            ui.label("press a key");
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
}
