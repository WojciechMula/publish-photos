use crate::application::Message;
use crate::application::MessageQueue;
use crate::db::Database;
use crate::gui::button;
use crate::keyboard::KeyboardMapping;
use crate::modal::ModalWindowTrait;
use crate::style::Style;
use crate::widgets::Shortcut;
use egui::Grid;
use egui::Key;
use egui::Modifiers;
use egui::ScrollArea;
use egui::Ui;

pub struct ModalKeyboard {
    keyboard_mapping: KeyboardMapping,
    mappings: Vec<KeyboardMappingHelp>,
}

type KeyboardMappingHelp = Vec<(Key, Modifiers, String)>;

impl Default for ModalKeyboard {
    fn default() -> Self {
        let keyboard_mapping = KeyboardMapping::default().key(Key::Escape, Message::CloseModal);

        Self {
            keyboard_mapping,
            mappings: Vec::new(),
        }
    }
}

impl ModalKeyboard {
    pub fn with_mapping(mut self, mapping: &KeyboardMapping) -> Self {
        self.mappings.push(mk_help(mapping));
        self
    }
}

fn mk_help(mapping: &KeyboardMapping) -> KeyboardMappingHelp {
    let mut result = KeyboardMappingHelp::new();
    for (key, bindings) in mapping.iter() {
        for (modifiers, message) in bindings {
            result.push((*key, *modifiers, message.name().to_owned()));
        }
    }

    fn weight(modifiers: &Modifiers) -> usize {
        let mut res = 0;
        if modifiers.alt {
            res += 1;
        }
        if modifiers.ctrl {
            res += 1;
        }
        if modifiers.shift {
            res += 1;
        }
        if modifiers.mac_cmd {
            res += 1;
        }
        if modifiers.command {
            res += 1;
        }

        res
    }

    fn sort_key(val: &(Key, Modifiers, String)) -> (Key, usize) {
        (val.0, weight(&val.1))
    }

    result.sort_by_key(sort_key);

    result
}

impl ModalWindowTrait for ModalKeyboard {
    fn update(&mut self, ui: &mut Ui, _style: &Style, _db: &Database, queue: &mut MessageQueue) {
        let shortcut_color = ui.visuals().strong_text_color();
        ScrollArea::vertical().show(ui, |ui| {
            for (id, mapping) in self.mappings.iter().enumerate() {
                if id > 0 {
                    ui.separator();
                }

                Grid::new(("keyboard-help-grid", id))
                    .num_columns(2)
                    .show(ui, |ui| {
                        for (key, modifiers, help) in mapping {
                            ui.add(
                                Shortcut::from_key_and_modifiers(*key, *modifiers)
                                    .with_color(shortcut_color),
                            );
                            ui.label(help);
                            ui.end_row();
                        }
                    });
            }

            ui.separator();

            ui.vertical_centered(|ui| {
                if button::close(ui) {
                    queue.push_back(Message::CloseModal);
                }
            });
        });
    }

    fn keyboard_mapping(&self) -> &KeyboardMapping {
        &self.keyboard_mapping
    }
}
