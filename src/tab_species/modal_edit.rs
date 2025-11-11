use crate::application::Message as MainMessage;
use crate::confirm::Confirm;
use crate::confirm::ConfirmOption;
use crate::db::Database;
use crate::db::Latin;
use crate::db::Species;
use crate::db::SpeciesId;
use crate::gui::button;
use crate::gui::icon_en;
use crate::gui::icon_pl;
use crate::help;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::species_view::image;
use crate::style::Style;
use crate::tab_species::Message as TabMessage;
use crate::tab_species::MessageQueue as TabMessageQueue;
use const_format::formatcp as fmt;
use egui::Align;
use egui::CentralPanel;
use egui::Context;
use egui::Grid;
use egui::Key;
use egui::Layout;
use egui::TopBottomPanel;
use egui::Ui;
use std::cell::LazyCell;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_WARNING;

pub struct ModalEdit {
    original: Option<Species>,
    new: Species,
    can_save: Result<bool, String>,

    pub queue: MessageQueue,
    pub keyboard_mapping: LazyCell<KeyboardMapping>,
}

type MessageQueue = VecDeque<Message>;

#[derive(Clone)]
pub enum Message {
    SoftClose,
    SaveAndExit,
    CancelAndExit,
    ChangeLatin(Latin),
    ChangePolish(String),
    ChangeEnglish(String),
    ChangeWikipediaPl(String),
    ChangeWikipediaEn(String),
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::SoftClose => help::SOFT_CLOSE,
            Self::SaveAndExit => help::SAVE_AND_EXIT,
            Self::CancelAndExit => unreachable!(),
            Self::ChangeLatin(_) => unreachable!(),
            Self::ChangePolish(_) => unreachable!(),
            Self::ChangeEnglish(_) => unreachable!(),
            Self::ChangeWikipediaPl(_) => unreachable!(),
            Self::ChangeWikipediaEn(_) => unreachable!(),
        }
    }
}

impl From<Message> for TabMessage {
    fn from(val: Message) -> Self {
        Self::ModalEdit(val)
    }
}

impl ModalEdit {
    pub fn new() -> Self {
        Self {
            can_save: Ok(false),
            original: None,
            new: Species::default(),
            queue: MessageQueue::new(),
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        }
    }

    pub fn edit(id: SpeciesId, db: &Database) -> Self {
        let original = db.species_by_id(&id).unwrap().clone();
        let new = original.clone();

        Self {
            can_save: Ok(false),
            original: Some(original),
            new,
            queue: MessageQueue::new(),
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        }
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &mut Database,
        tab_queue: &mut TabMessageQueue,
    ) {
        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(style, db, msg, tab_queue);
        }

        let mut queue = MessageQueue::new();
        self.draw(ctx, image_cache, style, db, &mut queue);

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }
    }

    fn create_mapping() -> KeyboardMapping {
        KeyboardMapping::default()
            .key(
                Key::Escape,
                MainMessage::TabSpecies(Message::SoftClose.into()),
            )
            .ctrl(Key::S, MainMessage::TabSpecies(Message::SaveAndExit.into()))
    }

    fn handle_message(
        &mut self,
        style: &Style,
        db: &mut Database,
        message: Message,
        tab_queue: &mut TabMessageQueue,
    ) {
        match message {
            Message::SoftClose => {
                if self.is_modified() {
                    let msg: TabMessage = Message::SaveAndExit.into();
                    let save = ConfirmOption::new("Save and exit")
                        .with_message(msg.into())
                        .with_color(style.button.save);

                    let msg: TabMessage = Message::CancelAndExit.into();
                    let abort = ConfirmOption::new(fmt!("{ICON_WARNING} Abandon changes"))
                        .with_message(msg.into())
                        .with_color(style.button.discard);

                    let cont = ConfirmOption::new("Continue").with_key(Key::Escape);

                    let actions = if self.can_save.is_ok() {
                        vec![abort, save, cont]
                    } else {
                        vec![abort, cont]
                    };

                    let confirm = Confirm::new("The species data got changed.", actions);

                    tab_queue.push_back(TabMessage::Confirm(confirm));
                } else {
                    tab_queue.push_back(TabMessage::CloseModal);
                }
            }
            Message::CancelAndExit => {
                tab_queue.push_back(TabMessage::CloseModal);
            }
            Message::SaveAndExit => {
                if self.original.is_some() {
                    db.update_species(&self.new);
                } else {
                    db.add_species(&self.new);
                }
                tab_queue.push_back(TabMessage::RefreshView);
                tab_queue.push_back(TabMessage::CloseModal);
            }
            Message::ChangeLatin(text) => {
                self.new.latin = text;
                self.validate(db);
            }
            Message::ChangePolish(text) => {
                self.new.pl = text;
                self.validate(db);
            }
            Message::ChangeEnglish(text) => {
                self.new.en = text;
                self.validate(db);
            }
            Message::ChangeWikipediaPl(text) => {
                self.new.wikipedia_pl = text;
                self.validate(db);
            }
            Message::ChangeWikipediaEn(text) => {
                self.new.wikipedia_en = text;
                self.validate(db);
            }
        }
    }

    fn validate(&mut self, db: &Database) {
        if self.new.latin.is_empty() {
            self.can_save = Err("name cannot be empty".to_string());
            return;
        }

        let duplicated_name = db
            .species
            .iter()
            .filter(|species| species.latin == self.new.latin)
            .any(|other| {
                self.original
                    .as_ref()
                    .is_none_or(|species| species.id != other.id)
            });

        if duplicated_name {
            self.can_save = Err("species alrady defined".to_string());
            return;
        }

        self.can_save = Ok(self.is_modified());
    }

    fn draw(
        &self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        TopBottomPanel::bottom("modal-species-edit-bottom").show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if button::save(ui, self.is_modified(), Some(style.button.save)) {
                    queue.push_back(Message::SaveAndExit);
                }
                if button::cancel(ui) {
                    queue.push_back(Message::SoftClose);
                }

                if let Err(msg) = &self.can_save {
                    let color = ui.visuals().error_fg_color;
                    ui.colored_label(color, msg);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                image(
                    ui,
                    image_cache,
                    style,
                    db,
                    &self.new,
                    style.image.preview_width,
                );

                ui.vertical(|ui| self.draw_details(ui, queue));
            });
        });
    }

    fn draw_details(&self, ui: &mut Ui, queue: &mut MessageQueue) {
        Grid::new("species-details").num_columns(2).show(ui, |ui| {
            ui.label("latin");
            ui.horizontal(|ui| {
                if let Some(val) = edit(ui, (&self.new.latin).into()) {
                    queue.push_back(Message::ChangeLatin(val.into()));
                }
            });

            ui.end_row();

            icon_pl(ui);
            ui.horizontal(|ui| {
                if let Some(val) = edit(ui, &self.new.pl) {
                    queue.push_back(Message::ChangePolish(val));
                }
            });
            ui.end_row();

            ui.label("wiki");
            ui.horizontal(|ui| {
                if let Some(val) = edit(ui, &self.new.wikipedia_pl) {
                    queue.push_back(Message::ChangeWikipediaPl(val));
                }
            });
            ui.end_row();

            icon_en(ui);
            ui.horizontal(|ui| {
                if let Some(val) = edit(ui, &self.new.en) {
                    queue.push_back(Message::ChangeEnglish(val));
                }
            });
            ui.end_row();

            ui.label("wiki");
            ui.horizontal(|ui| {
                if let Some(val) = edit(ui, &self.new.wikipedia_en) {
                    queue.push_back(Message::ChangeWikipediaEn(val));
                }
            });
            ui.end_row();
        });
    }

    fn is_modified(&self) -> bool {
        if let Some(original) = &self.original {
            self.new != *original
        } else {
            true
        }
    }
}

fn edit(ui: &mut Ui, curr: &String) -> Option<String> {
    let mut val = curr.clone();
    let changed = ui.text_edit_singleline(&mut val).changed();

    if changed && *curr != val {
        Some(val)
    } else {
        None
    }
}
