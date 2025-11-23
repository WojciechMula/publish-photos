use crate::application::Message as MainMessage;
use crate::confirm::Confirm;
use crate::confirm::ConfirmOption;
use crate::db::Database;
use crate::db::PostId;
use crate::edit_details::EditDetails;
use crate::gui::add_image;
use crate::gui::button;
use crate::gui::icon_en;
use crate::gui::icon_pl;
use crate::help;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::style::Style;
use crate::tab_posts::Message as TabMessage;
use crate::tab_posts::MessageQueue as TabMessageQueue;
use const_format::formatcp as fmt;
use egui::Align;
use egui::CentralPanel;
use egui::Context;
use egui::Key;
use egui::Layout;
use egui::ScrollArea;
use egui::SidePanel;
use egui::TextEdit;
use egui::TopBottomPanel;
use egui_material_icons::icons::ICON_WARNING;
use std::cell::LazyCell;
use std::collections::VecDeque;

const ID_PREFIX: &str = "post-description";

pub struct ModalDescription {
    id: PostId,
    new: Description,
    original: Description,

    pub queue: MessageQueue,
    pub keyboard_mapping: LazyCell<KeyboardMapping>,
}

type MessageQueue = VecDeque<Message>;

#[derive(Clone, PartialEq, Eq)]
struct Description {
    pl: String,
    en: String,
}

#[derive(Clone)]
pub enum Message {
    SoftClose,
    SaveAndExit,
    CancelAndExit,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::SoftClose => help::SOFT_CLOSE,
            Self::SaveAndExit => help::SAVE_AND_EXIT,
            Self::CancelAndExit => unreachable!(),
        }
    }
}

impl From<Message> for TabMessage {
    fn from(val: Message) -> Self {
        Self::ModalDescription(val)
    }
}

impl ModalDescription {
    pub fn new(id: PostId, db: &Database) -> Self {
        let post = db.post(&id);

        let original = Description {
            pl: post.pl.clone(),
            en: post.en.clone(),
        };
        let new = original.clone();

        Self {
            id,
            new,
            original,
            queue: MessageQueue::new(),
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        }
    }

    fn create_mapping() -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(TabMessage::ModalDescription(msg))
        }

        KeyboardMapping::default().key(Key::Escape, msg(Message::SoftClose))
    }

    fn handle_message(&mut self, msg: Message, style: &Style, tab_queue: &mut TabMessageQueue) {
        match msg {
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

                    let confirm = Confirm::new("The tags got changed.", vec![abort, save, cont]);

                    tab_queue.push_back(TabMessage::Confirm(confirm));
                } else {
                    tab_queue.push_back(TabMessage::CloseModal);
                }
            }
            Message::CancelAndExit => {
                tab_queue.push_back(TabMessage::CloseModal);
            }
            Message::SaveAndExit => {
                tab_queue.push_back(TabMessage::CloseModal);
                if self.new.pl != self.original.pl {
                    let msg = EditDetails::SetPolish(self.id, self.new.pl.clone());
                    tab_queue.push_back(msg.into());
                }
                if self.new.en != self.original.en {
                    let msg = EditDetails::SetEnglish(self.id, self.new.en.clone());
                    tab_queue.push_back(msg.into());
                }
            }
        }
    }

    fn is_modified(&self) -> bool {
        self.new != self.original
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
            self.handle_message(msg, style, tab_queue);
        }

        let post = db.post(&self.id);

        SidePanel::left(fmt!("{ID_PREFIX}-left"))
            .resizable(false)
            .show(ctx, |ui| {
                ScrollArea::vertical()
                    .id_salt(fmt!("{ID_PREFIX}-pictures-scroll"))
                    .show(ui, |ui| {
                        for meta in &post.files_meta {
                            add_image(
                                ui,
                                meta,
                                image_cache,
                                style.image.preview_width,
                                style.image.radius,
                            );
                        }
                    });
            });

        TopBottomPanel::bottom(fmt!("{ID_PREFIX}-buttons")).show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if button::save(ui, self.is_modified(), Some(style.button.save)) {
                    self.queue.push_back(Message::SaveAndExit);
                }
                if button::cancel(ui) {
                    self.queue.push_back(Message::SoftClose);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    icon_pl(ui);

                    let edit = TextEdit::multiline(&mut self.new.pl)
                        .hint_text("polski opis")
                        .desired_width(f32::INFINITY);

                    ui.add(edit);
                });

                ui.horizontal(|ui| {
                    icon_en(ui);

                    let edit = TextEdit::multiline(&mut self.new.en)
                        .hint_text("English description")
                        .desired_width(f32::INFINITY);

                    ui.add(edit);
                });
            });
        });
    }
}
