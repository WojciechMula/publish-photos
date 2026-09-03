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
    pub id: Id,
    pub input: HistoryInput,
    pub state_key: String,
}

impl SearchBox {
    pub fn new(id: &str) -> Self {
        Self {
            id: Id::new(id),
            input: HistoryInput::default().with_hint("search..."),
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
        self.input.persist(ctx, self.id);
    }

    pub fn restore(&mut self, ctx: &Context) {
        self.input.restore(ctx, self.id);
    }

    pub fn phrase(&self) -> &String {
        &self.input.current_text
    }

    pub fn take_focus(&self, ctx: &Context) {
        ctx.memory_mut(|mem| mem.request_focus(self.id));
    }

    pub fn update(&mut self, ctx: &Context, action: HistoryInputAction) {
        self.input.update(ctx, self.id, action);
    }

    pub fn show(&self, ui: &mut Ui) -> HistoryInputAction {
        let prev = self.input.current_text.clone();

        ui.add(Label::new(ICON_SEARCH).selectable(false));

        let ret = self.input.show(ui, self.id);

        let enabled = !prev.is_empty();
        let button = Button::new(ICON_BACKSPACE);
        if ui.add_enabled(enabled, button).clicked() {
            HistoryInputAction::Clear
        } else {
            ret
        }
    }
}
