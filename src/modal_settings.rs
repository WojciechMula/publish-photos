use crate::application::Message;
use crate::application::MessageQueue;
use crate::colors;
use crate::db::Database;
use crate::gui::button;
use crate::keyboard::KeyboardMapping;
use crate::modal::ModalWindowTrait;
use crate::style::Style;
use crate::widgets::color_box;
use const_format::formatcp as fmt;
use egui::Align;
use egui::CollapsingHeader;
use egui::Color32;
use egui::ComboBox;
use egui::Grid;
use egui::Id;
use egui::Key;
use egui::Layout;
use egui::Slider;
use egui::Ui;

use std::ops::RangeInclusive;

const ID_PREFIX: &str = "modal-settings";

pub struct ModalSettings {
    new: Style,
    original: Style,
    live_preview_id: Id,
    keyboard_mapping: KeyboardMapping,
}

impl ModalSettings {
    pub fn new(style: &Style) -> Self {
        let keyboard_mapping = KeyboardMapping::default().key(Key::Escape, Message::CloseModal);

        Self {
            new: style.clone(),
            original: style.clone(),
            live_preview_id: Id::new(fmt!("{ID_PREFIX}-live-preview")),
            keyboard_mapping,
        }
    }
}

impl ModalWindowTrait for ModalSettings {
    fn update(&mut self, ui: &mut Ui, style: &Style, _db: &Database, queue: &mut MessageQueue) {
        self.show_appearence(ui);
        self.show_image(ui);
        self.show_tags(ui);
        self.show_button(ui);

        let changed = self.new != self.original;

        ui.separator();

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            let mut flag = ui
                .ctx()
                .data_mut(|data| data.get_persisted(self.live_preview_id).unwrap_or_default());

            if ui.checkbox(&mut flag, "live preview").changed() {
                ui.ctx().data_mut(|data| {
                    data.insert_persisted(self.live_preview_id, flag);
                });
            }

            if flag && changed {
                queue.push_back(Message::SetStyle(self.new.clone()));
            }
        });

        ui.separator();

        ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
            if button::save(ui, changed, Some(style.button.save)) {
                queue.push_back(Message::SetStyle(self.new.clone()));
                queue.push_back(Message::CloseModal);
            }
            if button::cancel(ui) {
                queue.push_back(Message::SetStyle(self.original.clone()));
                queue.push_back(Message::CloseModal);
            }
        });
    }

    fn keyboard_mapping(&self) -> &KeyboardMapping {
        &self.keyboard_mapping
    }
}

impl ModalSettings {
    fn show_appearence(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Appearence")
            .id_salt(fmt!("{ID_PREFIX}-header-appearence"))
            .default_open(true)
            .show(ui, |ui| {
                Grid::new(fmt!("{ID_PREFIX}-grid-appearence"))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("modified post forground");
                        select_color(ui, "modified-post-fg", &mut self.new.modified);
                        ui.end_row();

                        ui.label("hovered background");
                        select_color(ui, "hovered-background", &mut self.new.hovered_frame);
                        ui.end_row();

                        ui.label("published post background");
                        select_color(
                            ui,
                            "published-post-background",
                            &mut self.new.published_post,
                        );
                        ui.end_row();

                        ui.label("selected post background");
                        select_color(ui, "selected-post-background", &mut self.new.selected_post);
                        ui.end_row();
                    });
            });
    }

    fn show_tags(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Tags")
            .id_salt(fmt!("{ID_PREFIX}-header-tags"))
            .default_open(true)
            .show(ui, |ui| {
                Grid::new(fmt!("{ID_PREFIX}-grid-tags"))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("active: forground");
                        select_color(ui, "tag_active_fg", &mut self.new.tag_active_fg);
                        ui.end_row();

                        ui.label("active: background");
                        select_color(ui, "tag_active_bg", &mut self.new.tag_active_bg);
                        ui.end_row();

                        ui.label("hovered: forground");
                        select_color(ui, "tag_hovered_fg", &mut self.new.tag_hovered_fg);
                        ui.end_row();

                        ui.label("hovered: background");
                        select_color(ui, "tag_hovered_bg", &mut self.new.tag_hovered_bg);
                        ui.end_row();

                        ui.label("highlighted: foreground");
                        select_color(ui, "tag_highlight_fg", &mut self.new.tag_highlight_fg);
                        ui.end_row();
                    });
            });
    }

    fn show_image(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Image")
            .id_salt(fmt!("{ID_PREFIX}-header-image"))
            .default_open(true)
            .show(ui, |ui| {
                Grid::new(fmt!("{ID_PREFIX}-grid-image"))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("preview width");
                        select_size(ui, &mut self.new.image.preview_width, 32..=800);
                        ui.end_row();

                        ui.label("thumbnail width");
                        select_size(ui, &mut self.new.image.thumbnail_width, 32..=800);
                        ui.end_row();

                        ui.label("corner radius");
                        select_size(ui, &mut self.new.image.radius, 0..=32);
                        ui.end_row();
                    });
            });
    }

    fn show_button(&mut self, ui: &mut Ui) {
        CollapsingHeader::new("Buttons")
            .id_salt(fmt!("{ID_PREFIX}-header-button"))
            .default_open(true)
            .show(ui, |ui| {
                Grid::new(fmt!("{ID_PREFIX}-grid-button"))
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("save");
                        select_color(ui, "button-save-bg", &mut self.new.button.save);
                        ui.end_row();

                        ui.label("remove");
                        select_color(ui, "button-remove-bg", &mut self.new.button.remove);
                        ui.end_row();

                        ui.label("publish");
                        select_color(ui, "button-publish-bg", &mut self.new.button.publish);
                        ui.end_row();

                        ui.label("save database");
                        select_color(
                            ui,
                            "button-save-database-bg",
                            &mut self.new.button.save_database,
                        );
                        ui.end_row();
                    });
            });
    }
}

fn select_size(ui: &mut Ui, current_value: &mut f32, range: RangeInclusive<usize>) {
    let before = *current_value as usize;
    let mut value = before;
    let slider = Slider::new(&mut value, range);
    if ui.add(slider).changed() {
        *current_value = value as f32;
    }
}

fn select_color(ui: &mut Ui, id: &str, color: &mut Color32) {
    ui.horizontal(|ui| {
        readonly_color(ui, *color);
        if let Some(new_color) = choose_color(ui, id, *color) {
            *color = new_color;
        }
    });
}

fn readonly_color(ui: &mut Ui, color: Color32) {
    ui.add(color_box(color, 2));
}

fn choose_color(ui: &mut Ui, id: &str, current_color: Color32) -> Option<Color32> {
    let mut selected_color = current_color;
    ComboBox::from_id_salt(id).show_ui(ui, |ui| {
        for (color, label) in colors::ALL {
            ui.horizontal(|ui| {
                readonly_color(ui, color);
                ui.selectable_value(&mut selected_color, color, label);
            });
        }
    });

    if selected_color != current_color {
        Some(selected_color)
    } else {
        None
    }
}
