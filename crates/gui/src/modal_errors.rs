use crate::application::Message;
use crate::application::MessageQueue;
use crate::clipboard::ClipboardKind;
use crate::gui::add_image;
use crate::gui::button;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::modal::ModalWindowTrait;
use crate::style::Style;
use const_format::formatcp as fmt;
use db::Database;
use db::PostId;
use egui::Button;
use egui::Key;
use egui::Label;
use egui::ScrollArea;
use egui::Ui;
use graphapi::manager::SocialMediaErrorList;
use graphapi::SocialMediaError;

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
                show_errors(ui, id, errors, image_cache, style, db, queue);
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

fn show_errors(
    ui: &mut Ui,
    id: &PostId,
    errors: &[SocialMediaError],
    image_cache: &mut ImageCache,
    style: &Style,
    db: &Database,
    queue: &mut MessageQueue,
) {
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
            let button = Button::new(fmt!("{ICON_REPLAY} Retry")).fill(style.button.publish);
            if ui.add(button).clicked() {
                queue.push_back(Message::StartPublishing(*id));
            }

            for err in errors {
                ui.horizontal(|ui| {
                    if button::copy(ui, true) {
                        queue.push_back(Message::Copy(ClipboardKind::Generic, err2string(err)));
                    }

                    fn add_label(ui: &mut Ui, label: &String) {
                        let label = Label::new(label).wrap();
                        ui.add(label);
                    }

                    match err {
                        SocialMediaError::String(msg) => {
                            add_label(ui, msg);
                        }
                        SocialMediaError::FacebookError(err) => {
                            ui.vertical(|ui| {
                                add_label(ui, &err.message);

                                if let Some(msg) = &err.error_user_title {
                                    add_label(ui, msg);
                                }

                                if let Some(msg) = &err.error_user_msg {
                                    add_label(ui, msg);
                                }

                                add_label(ui, &err.url);
                            });
                        }
                    }
                });
            }
        });
    });
}

fn err2string(sme: &SocialMediaError) -> String {
    match sme {
        SocialMediaError::String(msg) => msg.clone(),
        SocialMediaError::FacebookError(fb) => {
            let mut multiline = String::new();
            multiline += &format!("{}\n", fb.message);
            if let Some(msg) = &fb.error_user_title {
                multiline += &format!("{msg}\n");
            }
            if let Some(msg) = &fb.error_user_msg {
                multiline += &format!("{msg}\n");
            }
            multiline += &fb.url;

            multiline
        }
    }
}
