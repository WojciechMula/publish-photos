use egui::Button;
use egui::Context;
use egui::Id;
use egui::Label;
use egui::TextEdit;
use egui::Ui;

use egui_material_icons::icons::ICON_BACKSPACE;
use egui_material_icons::icons::ICON_SEARCH;

pub struct SearchBox {
    pub id: Id,
}

impl SearchBox {
    pub fn new(id: &str) -> Self {
        Self { id: Id::new(id) }
    }

    pub fn phrase(&self, ctx: &Context) -> String {
        ctx.data_mut(|data| data.get_persisted(self.id).unwrap_or_default())
    }

    pub fn take_focus(&self, ctx: &Context) {
        ctx.memory_mut(|mem| mem.request_focus(self.id));
    }

    pub fn show(&self, ui: &mut Ui) -> Option<String> {
        let mut phrase = self.phrase(ui.ctx());
        let prev = phrase.clone();

        ui.add(Label::new(ICON_SEARCH).selectable(false));
        ui.add(
            TextEdit::singleline(&mut phrase)
                .id(self.id)
                .hint_text("search..."),
        );

        let enabled = !prev.is_empty();
        let button = Button::new(ICON_BACKSPACE);
        if ui.add_enabled(enabled, button).clicked() {
            phrase.clear();
        }

        phrase = phrase.trim().to_lowercase();
        if phrase != prev {
            ui.ctx()
                .data_mut(|data| data.insert_persisted(self.id, phrase.clone()));
            Some(phrase)
        } else {
            None
        }
    }
}
