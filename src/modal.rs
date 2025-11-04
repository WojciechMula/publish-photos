use crate::application::MessageQueue;
use crate::db::Database;
use crate::keyboard::KeyboardMapping;
use crate::style::Style;
use egui::Ui;

pub trait ModalWindowTrait {
    fn update(&mut self, ui: &mut Ui, style: &Style, db: &Database, queue: &mut MessageQueue);

    fn keyboard_mapping(&self) -> &KeyboardMapping;
}
