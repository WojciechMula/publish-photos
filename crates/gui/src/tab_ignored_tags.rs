use crate::keyboard::KeyboardMapping;
use crate::style::Style;
use crate::widgets::tag_button;
use db::Database;
use egui::Button;
use egui::CentralPanel;
use egui::Context;
use egui::ScrollArea;
use egui::Ui;

use egui_material_icons::icons::ICON_DELETE;

#[derive(Default)]
pub struct TabIgnoredTags {
    new: String,
    pub keyboard_mapping: KeyboardMapping,
}

impl TabIgnoredTags {
    pub fn update(&mut self, ctx: &Context, style: &Style, db: &mut Database) {
        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical()
                .id_salt("scroll-area-ignored-tags")
                .auto_shrink(false)
                .show(ui, |ui| {
                    self.aux(ui, style, db);
                });
        });
    }

    fn aux(&mut self, ui: &mut Ui, style: &Style, db: &mut Database) {
        ui.horizontal(|ui| {
            ui.text_edit_singleline(&mut self.new);
            let enabled = !self.new.trim().is_empty();
            let button = Button::new("➕Add new");
            if ui.add_enabled(enabled, button).clicked() {
                db.new_ignored_tag(self.new.clone());
                self.new.clear();
            }
        });

        ui.separator();

        let mut remove: Option<String> = None;
        for tag in db.ignored_tags.iter() {
            ui.horizontal(|ui| {
                if ui.button(ICON_DELETE).clicked() {
                    remove = Some(tag.clone());
                }

                ui.add(tag_button(tag, "", style));
            });
        }

        if let Some(tag) = remove {
            db.remove_ignored_tag(&tag);
        }
    }
}
