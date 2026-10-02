use crate::widgets::HistoryInput;
use crate::widgets::HistoryInputAction;
use egui::Button;
use egui::Context;
use egui::Id;
use egui::Label;
use egui::Ui;

use egui_material_icons::icons::ICON_BACKSPACE;
use egui_material_icons::icons::ICON_SEARCH;

const ID_PREFIX: &str = "search-box-";

pub struct SearchBox {
    pub input: HistoryInput,
    pub state_key: String,
}

impl SearchBox {
    pub fn new(id: &str) -> Self {
        Self {
            input: HistoryInput::new(Id::new(id)).with_hint("search..."),
            state_key: format!("{ID_PREFIX}-{id}"),
        }
    }

    pub fn load(&mut self, storage: &dyn eframe::Storage) {
        if let Some(input) = eframe::get_value(storage, &self.state_key) {
            self.input = input;
        }
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, &self.state_key, &self.input);
    }

    pub fn persist(&self, ctx: &Context) {
        self.input.persist(ctx);
    }

    pub fn restore(&mut self, ctx: &Context) {
        self.input.restore(ctx);
    }

    pub fn phrase(&self) -> &String {
        &self.input.current
    }

    pub fn take_focus(&self, ctx: &Context) {
        self.input.take_focus(ctx);
    }

    pub fn update(&mut self, action: HistoryInputAction) -> bool {
        self.input.update(action)
    }

    pub fn show(&self, ui: &mut Ui) -> Option<HistoryInputAction> {
        let is_empty = self.phrase().is_empty();

        ui.add(Label::new(ICON_SEARCH).selectable(false));

        let ret = self.input.show(ui);

        let enabled = !is_empty;
        let button = Button::new(ICON_BACKSPACE);
        if ui.add_enabled(enabled, button).clicked() {
            Some(HistoryInputAction::Clear)
        } else {
            ret
        }
    }
}
