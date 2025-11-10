use crate::application::Message as MainMessage;
use crate::db::Database;
use crate::db::Translation;
use crate::gui::icon_en;
use crate::gui::icon_pl;
use crate::keyboard::KeyboardMapping;
use crate::search_box::SearchBox;
use egui::CentralPanel;
use egui::Context;
use egui::Key;
use egui::ScrollArea;
use egui::Ui;
use std::cell::LazyCell;
use std::collections::VecDeque;

pub struct TabTagTranslations {
    search_box: SearchBox,

    pub queue: MessageQueue,
    pub keyboard_mapping: LazyCell<KeyboardMapping>,
}

#[derive(Clone)]
pub enum Message {
    AddNew,
    ChangePolish { id: usize, text: String },
    ChangeEnglish { id: usize, text: String },
    FocusSearch,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::AddNew => "add new translation",
            Self::FocusSearch => "focus search box",
            _ => "",
        }
    }
}

impl From<Message> for MainMessage {
    fn from(val: Message) -> Self {
        Self::TabTagTranslations(val)
    }
}

type MessageQueue = VecDeque<Message>;

impl Default for TabTagTranslations {
    fn default() -> Self {
        Self {
            queue: MessageQueue::new(),
            search_box: SearchBox::new("tab-tags-search"),
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        }
    }
}

impl TabTagTranslations {
    pub fn update(&mut self, ctx: &Context, db: &mut Database) {
        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(ctx, db, msg);
        }

        let mut queue = MessageQueue::new();
        self.draw(ctx, db, &mut queue);

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }
    }

    fn create_mapping() -> KeyboardMapping {
        KeyboardMapping::default()
            .key(Key::Slash, Message::FocusSearch.into())
            .ctrl(Key::Slash, Message::FocusSearch.into())
    }

    fn handle_message(&mut self, ctx: &Context, db: &mut Database, message: Message) {
        match message {
            Message::AddNew => {
                db.new_tag();
                db.mark_dirty();
            }
            Message::ChangePolish { id, text } => {
                db.tag_translations.0[id].pl = text;
                db.mark_dirty();
            }
            Message::ChangeEnglish { id, text } => {
                db.tag_translations.0[id].en = text;
                db.mark_dirty();
            }
            Message::FocusSearch => {
                self.search_box.take_focus(ctx);
            }
        }
    }

    fn draw(&self, ctx: &Context, db: &Database, queue: &mut MessageQueue) {
        CentralPanel::default().show(ctx, |ui| {
            self.draw_list(ui, db, queue);
        });
    }

    fn draw_list(&self, ui: &mut Ui, db: &Database, queue: &mut MessageQueue) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.search_box.show(ui);

                ui.separator();

                if ui.button("➕Add new").clicked() {
                    queue.push_back(Message::AddNew);
                }
            });

            ui.separator();

            self.draw_translations(ui, db, queue);
        });
    }

    fn draw_translations(&self, ui: &mut Ui, db: &Database, queue: &mut MessageQueue) {
        ScrollArea::vertical()
            .id_salt("scroll-area-tags")
            .auto_shrink(false)
            .show(ui, |ui| {
                let phrase = self.search_box.phrase(ui.ctx());
                for (id, trans) in db.tag_translations.0.iter().enumerate() {
                    if self.filter(&phrase, trans) {
                        self.draw_translation(ui, id, trans, queue);
                    }
                }
            });
    }

    fn filter(&self, phrase: &str, trans: &Translation) -> bool {
        phrase.is_empty() | trans.pl.contains(phrase) | trans.en.contains(phrase)
    }

    fn draw_translation(
        &self,
        ui: &mut Ui,
        id: usize,
        trans: &Translation,
        queue: &mut MessageQueue,
    ) {
        ui.horizontal(|ui| {
            icon_en(ui);

            let mut en = trans.en.clone();
            if ui.text_edit_singleline(&mut en).changed() {
                queue.push_back(Message::ChangeEnglish { id, text: en });
            }

            icon_pl(ui);

            let mut pl = trans.pl.clone();
            if ui.text_edit_singleline(&mut pl).changed() {
                queue.push_back(Message::ChangePolish { id, text: pl });
            }
        });
    }
}
