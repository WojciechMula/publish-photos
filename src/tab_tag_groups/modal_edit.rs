use crate::application::Message as MainMessage;
use crate::confirm::Confirm;
use crate::confirm::ConfirmOption;
use crate::db::Database;
use crate::db::Selector;
use crate::db::TagGroup;
use crate::edit_tags::Action;
use crate::gui::button;
use crate::help;
use crate::keyboard::KeyboardMapping;
use crate::select_tags::SelectTags;
use crate::select_tags::SelectTagsAction;
use crate::select_tags::TranslatedTagGroup;
use crate::style::Style;
use crate::tab_tag_groups::Message as TabMessage;
use crate::tab_tag_groups::MessageQueue as TabMessageQueue;
use crate::widgets::tag_button;
use const_format::formatcp as fmt;
use egui::Align;
use egui::CentralPanel;
use egui::Context;
use egui::Key;
use egui::Layout;
use egui::TopBottomPanel;
use egui::Ui;
use std::cell::LazyCell;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_WARNING;

const ID_PREFIX: &str = "model-edit-tag-group";

pub struct ModalEdit {
    title: String,

    name: String,
    select_tags: SelectTags,
    original: Option<TagGroup>,

    state: ModalState,
    label_width: f32,

    pub queue: MessageQueue,
    pub keyboard_mapping: LazyCell<KeyboardMapping>,
}

enum ModalState {
    NoChanges,
    Modified,
    NameError(String),
    TagError(String),
}

#[derive(Clone)]
pub enum Message {
    TagAction(SelectTagsAction),
    SetName(String),
    SaveAndExit,
    SoftClose,
    Undo,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::SaveAndExit => help::SAVE_AND_EXIT,
            Self::SoftClose => help::SOFT_CLOSE,
            Self::Undo => help::UNDO,
            _ => unreachable!(),
        }
    }
}

impl From<Message> for TabMessage {
    fn from(val: Message) -> Self {
        Self::ModalEdit(val)
    }
}

impl From<SelectTagsAction> for Message {
    fn from(val: SelectTagsAction) -> Self {
        Self::TagAction(val)
    }
}

impl From<Action> for Message {
    fn from(val: Action) -> Self {
        Self::TagAction(val.into())
    }
}

type MessageQueue = VecDeque<Message>;

impl Default for ModalEdit {
    fn default() -> Self {
        Self {
            title: String::new(),
            select_tags: SelectTags::new(),
            name: String::new(),
            original: None,
            queue: MessageQueue::new(),
            state: ModalState::NoChanges,
            label_width: 0.0,
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        }
    }
}

impl ModalEdit {
    pub fn new() -> Self {
        Self {
            title: "Create a new group".to_owned(),
            ..Default::default()
        }
    }

    pub fn edit(group: &TagGroup) -> Self {
        Self {
            title: "Editing group".to_owned(),
            select_tags: SelectTags::edit(&group.tags),
            name: group.name.clone(),
            original: Some(group.clone()),
            ..Default::default()
        }
    }

    fn create_mapping() -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabTagGroups(TabMessage::ModalEdit(msg))
        }

        KeyboardMapping::default()
            .key(Key::Escape, msg(Message::SoftClose))
            .ctrl(Key::S, msg(Message::SaveAndExit))
            .ctrl(Key::Z, msg(Message::Undo))
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        style: &Style,
        db: &mut Database,
        tab_queue: &mut TabMessageQueue,
    ) {
        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(style, db, msg, tab_queue);
        }

        db.refresh_caches();

        if self.select_tags.available.is_empty() {
            let view = db.get_tags_view(&Selector::All);
            let group = TranslatedTagGroup::from_tags_view("", view.clone());
            self.select_tags.available.push(group);
        }

        let mut queue = MessageQueue::new();
        self.draw(ctx, style, &mut queue);

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }
    }

    fn handle_message(
        &mut self,
        style: &Style,
        db: &mut Database,
        msg: Message,
        tab_queue: &mut TabMessageQueue,
    ) {
        match msg {
            Message::TagAction(action) => {
                self.select_tags.update(action, db);
                self.validate(db);
            }
            Message::SetName(string) => {
                if self.name != string {
                    self.name = string;
                    self.validate(db);
                }
            }
            Message::SaveAndExit => {
                assert!(matches!(self.state, ModalState::Modified));
                if self.original.is_some() {
                    match db.update_group(self.mk_tag_group()) {
                        Ok(_) => tab_queue.push_back(TabMessage::CloseModal),
                        Err(_err) => (),
                    }
                } else {
                    match db.add_group(self.mk_tag_group()) {
                        Ok(_) => tab_queue.push_back(TabMessage::CloseModal),
                        Err(_err) => (),
                    }
                }
            }
            Message::SoftClose => {
                if self.is_modified() {
                    let msg: TabMessage = Message::SaveAndExit.into();
                    let save = ConfirmOption::new("Save and exit")
                        .with_message(msg.into())
                        .with_color(style.button.save);

                    let abort = ConfirmOption::new(fmt!("{ICON_WARNING} Abandon changes"))
                        .with_message(TabMessage::CloseModal.into())
                        .with_color(style.button.discard);

                    let cont = ConfirmOption::new("Continue").with_key(Key::Escape);

                    let confirm = Confirm::new("Tags got changed.", vec![abort, save, cont]);

                    tab_queue.push_back(TabMessage::Confirm(confirm));
                } else {
                    tab_queue.push_back(TabMessage::CloseModal);
                }
            }
            Message::Undo => {
                self.queue.push_back(SelectTagsAction::Undo.into());
            }
        }
    }

    fn mk_tag_group(&self) -> TagGroup {
        TagGroup {
            id: self
                .original
                .as_ref()
                .map(|group| group.id)
                .unwrap_or_default(),
            name: self.name.clone(),
            tags: self.select_tags.tags.clone(),
        }
    }

    fn validate(&mut self, db: &Database) {
        if self.name.is_empty() {
            self.state = ModalState::NameError("name cannot be empty".to_string());
            return;
        }

        let duplicated_name = db
            .tag_groups
            .iter()
            .filter(|group| group.name == self.name)
            .any(|other| {
                self.original
                    .as_ref()
                    .is_none_or(|group| group.id != other.id)
            });

        if duplicated_name {
            self.state = ModalState::NameError("name alrady used".to_string());
            return;
        }

        if self.select_tags.tags.is_empty() {
            self.state = ModalState::TagError("no tags".to_string());
            return;
        }

        if self.is_modified() {
            self.state = ModalState::Modified;
        } else {
            self.state = ModalState::NoChanges;
        }
    }

    fn draw(&mut self, ctx: &Context, style: &Style, queue: &mut MessageQueue) {
        TopBottomPanel::bottom(fmt!("{ID_PREFIX}-buttons")).show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                if self.label_width == 0.0 {
                    self.label_width = crate::gui::max_size(&["Tags:", "Name:"], ui);
                }

                let can_save = matches!(self.state, ModalState::Modified);
                if button::save(ui, can_save, Some(style.button.save)) {
                    queue.push_back(Message::SaveAndExit);
                }
                if button::cancel(ui) {
                    queue.push_back(Message::SoftClose);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.heading(&self.title);
                    ui.separator();
                    self.draw_current(ui, style, queue);
                    ui.separator();
                    self.draw_all_tags(ui, style, queue);
                });
            });
        });
    }

    fn draw_current(&self, ui: &mut Ui, style: &Style, queue: &mut MessageQueue) {
        let err_color = ui.visuals().error_fg_color;

        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.set_min_width(self.label_width);
                ui.label("Name:");
            });
            let mut name = self.name.clone();
            if ui.text_edit_singleline(&mut name).changed() {
                queue.push_back(Message::SetName(name));
            }

            if let ModalState::NameError(msg) = &self.state {
                ui.colored_label(err_color, msg);
            }
        });

        ui.horizontal_wrapped(|ui| {
            ui.horizontal(|ui| {
                ui.set_min_width(self.label_width);
                ui.label("Tags:");
            });
            if let ModalState::TagError(msg) = &self.state {
                ui.colored_label(err_color, msg);
            }
            for tag in self.select_tags.tags.iter() {
                if ui.add(tag_button(tag, "", style)).clicked() {
                    queue.push_back(Action::RemoveTag(tag.clone()).into());
                }
            }
        });
    }

    fn draw_all_tags(&self, ui: &mut Ui, style: &Style, queue: &mut MessageQueue) {
        if let Some(action) = self.select_tags.draw_controls(ui, style) {
            queue.push_back(action.into());
        }

        ui.separator();

        if let Some(action) = self.select_tags.draw_tags(ui, style) {
            queue.push_back(action.into());
        }
    }

    fn is_modified(&self) -> bool {
        if let Some(original) = &self.original {
            self.name != original.name || self.select_tags.tags != original.tags
        } else {
            !self.name.is_empty() || !self.select_tags.tags.is_empty()
        }
    }
}
