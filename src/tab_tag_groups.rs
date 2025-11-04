use crate::application::Message as MainMessage;
use crate::application::MessageQueue as MainMessageQueue;
use crate::confirm::Confirm;
use crate::db::Database;
use crate::db::TagGroup;
use crate::db::TagGroupId;
use crate::gui::tag;
use crate::keyboard::KeyboardMapping;
use crate::style::Style;
use const_format::formatcp as fmt;
use egui::Button;
use egui::CentralPanel;
use egui::Context;
use egui::Key;
use egui::ScrollArea;
use egui::Ui;
use std::cell::LazyCell;
use std::collections::VecDeque;

mod modal_edit;
use modal_edit::Message as ModalEditMessage;
use modal_edit::ModalEdit;

use egui_material_icons::icons::ICON_ARROW_DOWNWARD;
use egui_material_icons::icons::ICON_ARROW_UPWARD;
use egui_material_icons::icons::ICON_EDIT;

const ID_PREFIX: &str = "tab-tag-groups";

pub struct TabTagGroups {
    modal_window: ModalWindow,

    pub queue: MessageQueue,
    pub keyboard_mapping: LazyCell<KeyboardMapping>,
}

#[derive(Clone)]
pub enum Message {
    AddNew,
    Edit(TagGroupId),
    MoveUp(TagGroupId),
    MoveDown(TagGroupId),
    CloseModal,
    ModalEdit(ModalEditMessage),
    Confirm(Confirm),
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::AddNew => "add new group",
            Self::CloseModal => "close window",
            Self::ModalEdit(msg) => msg.name(),
            _ => unreachable!(),
        }
    }
}

impl From<Message> for MainMessage {
    fn from(val: Message) -> Self {
        Self::TabTagGroups(val)
    }
}

enum ModalWindow {
    None,
    ModalEdit(Box<ModalEdit>),
}

impl ModalWindow {
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

type MessageQueue = VecDeque<Message>;

impl Default for TabTagGroups {
    fn default() -> Self {
        Self {
            queue: MessageQueue::new(),
            keyboard_mapping: LazyCell::new(Self::create_mapping),
            modal_window: ModalWindow::None,
        }
    }
}

impl TabTagGroups {
    pub fn update(
        &mut self,
        ctx: &Context,
        style: &Style,
        db: &mut Database,
        main_queue: &mut MainMessageQueue,
    ) {
        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(db, msg, main_queue);
        }

        db.refresh_caches();

        let mut queue = MessageQueue::new();
        match &mut self.modal_window {
            ModalWindow::None => {
                self.draw(ctx, style, db, &mut queue);
            }
            ModalWindow::ModalEdit(window) => {
                window.update(ctx, style, db, &mut queue);
            }
        }

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }
    }

    fn handle_message(
        &mut self,
        db: &mut Database,
        msg: Message,
        main_queue: &mut MainMessageQueue,
    ) {
        match msg {
            Message::AddNew => {
                let window = ModalEdit::new();
                self.modal_window = ModalWindow::ModalEdit(Box::new(window));
            }
            Message::Edit(id) => {
                assert!(self.modal_window.is_none());

                if let Some(group) = db.tag_groups.get(&id) {
                    let window = ModalEdit::edit(group);
                    self.modal_window = ModalWindow::ModalEdit(Box::new(window));
                }
            }
            Message::MoveUp(id) => {
                db.move_group_up(&id);
            }
            Message::MoveDown(id) => {
                db.move_group_down(&id);
            }
            Message::CloseModal => {
                assert!(!self.modal_window.is_none());
                self.modal_window = ModalWindow::None;
            }
            Message::ModalEdit(msg) => {
                if let ModalWindow::ModalEdit(window) = &mut self.modal_window {
                    window.queue.push_back(msg);
                }
            }
            Message::Confirm(confirm) => {
                main_queue.push_back(MainMessage::Confirm(confirm));
            }
        }
    }

    fn create_mapping() -> KeyboardMapping {
        KeyboardMapping::default().ctrl(Key::N, Message::AddNew.into())
    }

    pub fn get_keyboard_mapping(&self) -> &KeyboardMapping {
        match &self.modal_window {
            ModalWindow::None => &self.keyboard_mapping,
            ModalWindow::ModalEdit(window) => &window.keyboard_mapping,
        }
    }

    fn draw(&self, ctx: &Context, style: &Style, db: &Database, queue: &mut MessageQueue) {
        CentralPanel::default().show(ctx, |ui| {
            ui.vertical(|ui| {
                self.draw_header(ui, queue);

                ui.separator();

                self.draw_list(ui, style, db, queue);
            });
        });
    }

    fn draw_header(&self, ui: &mut Ui, queue: &mut MessageQueue) {
        ui.horizontal(|ui| {
            if ui.button("➕Add new").clicked() {
                queue.push_back(Message::AddNew);
            }
        });
    }

    fn draw_list(&self, ui: &mut Ui, style: &Style, db: &Database, queue: &mut MessageQueue) {
        ScrollArea::vertical()
            .id_salt(fmt!("{ID_PREFIX}-scroll"))
            .auto_shrink(false)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let n = db.tag_groups.len();
                    for (id, group) in db.tag_groups.iter().enumerate() {
                        let is_first = id == 0;
                        let is_last = (id + 1) == n;

                        self.draw_group(ui, style, group, is_first, is_last, queue);
                        ui.separator();
                    }
                });
            });
    }

    fn draw_group(
        &self,
        ui: &mut Ui,
        style: &Style,
        group: &TagGroup,
        is_first: bool,
        is_last: bool,
        queue: &mut MessageQueue,
    ) {
        ui.horizontal(|ui| {
            ui.heading(&group.name);
            if ui.button(fmt!("{ICON_EDIT} Edit")).clicked() {
                queue.push_back(Message::Edit(group.id));
            }

            let button = Button::new(fmt!("{ICON_ARROW_UPWARD}"));
            if ui.add_enabled(!is_first, button).clicked() {
                queue.push_back(Message::MoveUp(group.id));
            }

            let button = Button::new(fmt!("{ICON_ARROW_DOWNWARD}"));
            if ui.add_enabled(!is_last, button).clicked() {
                queue.push_back(Message::MoveDown(group.id));
            }
        });

        ui.horizontal_wrapped(|ui| {
            for string in group.tags.iter() {
                ui.add(tag(string, style));
            }
        });
    }
}
