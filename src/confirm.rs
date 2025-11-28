use crate::application::Message;
use crate::application::MessageQueue;
use crate::db::Database;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::modal::ModalWindowTrait;
use crate::style::Style;
use egui::Align;
use egui::Button;
use egui::Color32;
use egui::Key;
use egui::Layout;
use egui::Ui;

#[derive(Clone)]
pub struct Confirm {
    pub text: String,
    pub options: Vec<ConfirmOption>,
    keyboard_mapping: KeyboardMapping,
}

impl Confirm {
    pub fn new(text: impl Into<String>, options: Vec<ConfirmOption>) -> Self {
        assert!(!options.is_empty());

        let keyboard_mapping = Self::make_kbd_mapping(&options);

        Self {
            text: text.into(),
            options,
            keyboard_mapping,
        }
    }

    fn make_kbd_mapping(options: &[ConfirmOption]) -> KeyboardMapping {
        let mut keyboard_mapping = KeyboardMapping::default();
        for opt in options {
            let Some(key) = opt.key.as_ref() else {
                continue;
            };

            if let Some(msg) = opt.message.as_ref() {
                let boxed = Box::new(msg.clone());
                keyboard_mapping = keyboard_mapping.key(*key, Message::ConfirmResult(Some(boxed)));
            } else {
                keyboard_mapping = keyboard_mapping.key(*key, Message::ConfirmResult(None));
            }
        }

        keyboard_mapping
    }
}

impl ModalWindowTrait for Confirm {
    fn update(
        &mut self,
        ui: &mut Ui,
        _image_cache: &mut ImageCache,
        _style: &Style,
        _db: &Database,
        queue: &mut MessageQueue,
    ) {
        ui.heading(&self.text);
        ui.separator();

        let mut option_id: Option<usize> = None;
        ui.with_layout(Layout::right_to_left(Align::TOP), |ui| {
            for (id, opt) in self.options.iter().enumerate().rev() {
                let mut button = Button::new(&opt.label);
                if let Some(bg_color) = &opt.color {
                    button = button.fill(*bg_color);
                }

                if ui.add(button).clicked() {
                    option_id = Some(id);
                }
            }
        });

        if let Some(option_id) = option_id {
            if let Some(message) = self.options[option_id].message.take() {
                queue.push_back(Message::ConfirmResult(Some(Box::new(message))));
            } else {
                queue.push_back(Message::ConfirmResult(None));
            }
        }
    }

    fn keyboard_mapping(&self) -> &KeyboardMapping {
        &self.keyboard_mapping
    }
}

#[derive(Clone)]
pub struct ConfirmOption {
    pub message: Option<Message>,
    pub label: String,
    pub key: Option<Key>,
    pub color: Option<Color32>,
}

impl ConfirmOption {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            message: None,
            key: None,
            color: None,
        }
    }

    pub fn with_key(mut self, key: Key) -> Self {
        self.key = Some(key);
        self
    }

    pub fn with_message(mut self, message: Message) -> Self {
        self.message = Some(message);
        self
    }

    pub fn with_color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }
}
