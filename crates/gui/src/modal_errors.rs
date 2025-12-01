use crate::application::Message;
use crate::application::MessageQueue;
use crate::gui::add_image;
use crate::gui::button;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::modal::ModalWindowTrait;
use crate::style::Style;
use const_format::formatcp as fmt;
use db::Database;
use egui::Button;
use egui::Key;
use egui::Label;
use egui::ScrollArea;
use egui::Ui;
use graphapi::manager::SocialMediaErrorList;

use egui_material_icons::icons::ICON_REPLAY;

pub struct ModalErrors {
    errors: SocialMediaErrorList,
    keyboard_mapping: KeyboardMapping,
}

impl ModalErrors {
    pub fn new(errors: SocialMediaErrorList) -> Self {
        let keyboard_mapping = KeyboardMapping::default().key(Key::Escape, Message::CloseModal);

        Self {
            keyboard_mapping,
            errors,
        }
    }
}

impl ModalWindowTrait for ModalErrors {
    fn update(
        &mut self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        ScrollArea::vertical().show(ui, |ui| {
            for (id, errors) in &self.errors {
                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        let post = db.post(id);
                        add_image(
                            ui,
                            &post.files[0],
                            image_cache,
                            style.image.thumbnail_width,
                            style.image.radius,
                        );
                    });

                    ui.vertical(|ui| {
                        let button =
                            Button::new(fmt!("{ICON_REPLAY} Retry")).fill(style.button.publish);
                        if ui.add(button).clicked() {
                            queue.push_back(Message::StartPublishing(*id));
                        }

                        for msg in errors {
                            ui.horizontal(|ui| {
                                if button::copy(ui, true) {
                                    queue.push_back(Message::Copy(msg.clone()));
                                }
                                let label = Label::new(msg).wrap();
                                ui.add(label);
                            });
                        }
                    });
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
