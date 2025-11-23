mod filter;
mod group;
mod modal_desc;
mod modal_publish;
mod modal_species;
mod modal_tags;
mod modal_view;

use filter::Filter;
use group::Group;
use modal_desc::Message as ModalDescriptionMessage;
use modal_desc::ModalDescription;
use modal_publish::Message as ModalPublishMessage;
use modal_publish::ModalPublish;
use modal_species::Message as ModalSpeciesMessage;
use modal_species::ModalSpecies;
use modal_tags::Message as ModalTagsMessage;
use modal_tags::ModalTags;
use modal_view::Message as ModalViewMessage;
use modal_view::ModalView;

use crate::application::Message as MainMessage;
use crate::application::MessageQueue as MainMessageQueue;
use crate::confirm::Confirm;
use crate::db::Database;
use crate::db::Date;
use crate::db::Month;
use crate::db::Post;
use crate::db::PostId;
use crate::db::Selector;
use crate::edit_details::EditDetails;
use crate::gui::add_image;
use crate::gui::add_image_with_tint;
use crate::gui::add_overlay;
use crate::gui::button;
use crate::gui::frame;
use crate::gui::icon_en;
use crate::gui::icon_pl;
use crate::gui::overlay_label;
use crate::gui::tag;
use crate::gui::tight_frame;
use crate::gui::OverlayLocation;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::style::Style;
use crate::ImageCounter;
use const_format::formatcp as fmt;
use egui::Align;
use egui::Button;
use egui::CentralPanel;
use egui::Color32;
use egui::Context;
use egui::Id;
use egui::Key;
use egui::KeyboardShortcut;
use egui::Label;
use egui::Layout;
use egui::Modifiers;
use egui::RichText;
use egui::ScrollArea;
use egui::Sense;
use egui::SidePanel;
use egui::TextEdit;
use egui::TopBottomPanel;
use egui::Ui;
use serde::Deserialize;
use serde::Serialize;
use std::cell::LazyCell;
use std::collections::BTreeMap;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_ADD;
use egui_material_icons::icons::ICON_CHECK;
use egui_material_icons::icons::ICON_CLOSE;
use egui_material_icons::icons::ICON_DELETE;
use egui_material_icons::icons::ICON_DIALOGS;
use egui_material_icons::icons::ICON_EDIT;
use egui_material_icons::icons::ICON_FULLSCREEN;
use egui_material_icons::icons::ICON_GRID_ON;
use egui_material_icons::icons::ICON_LIST;
use egui_material_icons::icons::ICON_UNDO;

const ID_PREFIX: &str = "tab-posts";

pub struct TabPosts {
    version: u64,
    view: Vec<PostId>,
    hovered: Option<PostId>,
    selected: Option<PostId>,
    scroll_to_selected: bool,
    grid_columns: isize,
    filter: Filter,
    inline_editors: BTreeMap<(PostId, Field), InlineEditor>,
    group: Option<Group>,
    label_width: f32,
    modal_window: ModalWindow,
    view_kind: ViewKind,

    keyboard_mapping: LazyCell<KeyboardMapping>,

    pub queue: MessageQueue,
}

#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
pub enum ViewKind {
    List,
    Grid,
}

impl ViewKind {
    const fn name(&self) -> &str {
        match self {
            Self::List => fmt!("{ICON_LIST} list"),
            Self::Grid => fmt!("{ICON_GRID_ON} grid"),
        }
    }
}

pub enum ModalWindow {
    None,
    ModalTags(Box<ModalTags>),
    ModalSpecies(Box<ModalSpecies>),
    ModalPublish(Box<ModalPublish>),
    ModalView(Box<ModalView>),
    ModalDescription(Box<ModalDescription>),
}

impl ModalWindow {
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

struct InlineEditor {
    pub text: String,
    pub id: Id,
}

pub type MessageQueue = VecDeque<Message>;

#[derive(Clone)]
pub enum Message {
    FocusItem(Id),
    EditDescription(PostId),
    EditTags(PostId),
    EditSpecies(PostId),
    View(PostId),
    Select(PostId),
    Publish(PostId),
    InlineEditStart {
        id: PostId,
        field: Field,
    },
    InlineEditChange {
        id: PostId,
        field: Field,
        value: String,
    },
    InlineSaveChange {
        id: PostId,
        field: Field,
    },
    InlineDiscardChanges {
        id: PostId,
        field: Field,
    },
    EditDetails(EditDetails),
    StartGrouping(PostId),
    AbortGrouping,
    AddToGroup(PostId),
    RemoveFromGroup(PostId),
    SaveGroup,

    ModalTags(ModalTagsMessage),
    ModalSpecies(ModalSpeciesMessage),
    ModalView(ModalViewMessage),
    ModalDescription(ModalDescriptionMessage),
    ModalPublish(ModalPublishMessage),
    CloseModal,
    Confirm(Confirm),

    Copy(String),

    RefreshView,
    Hovered(Option<PostId>),
    StartGroupingCurrent,
    PublishCurrent,
    EditDescriptionCurrent,
    EditTagsCurrent,
    EditSpeciesCurrent,
    ViewCurrent,
    SelectNext,
    SelectPrev,
    SelectNextRow,
    SelectPrevRow,
    SelectNextRowsMany,
    SelectPrevRowsMany,
    SelectFirst,
    SelectLast,
    Undo,
    FocusSearch,
    FilterByDate(Date),
    FilterByMonth(Month),
    SetViewKind(ViewKind),
    SetGridColumns(isize),
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::EditDescription(_) => unreachable!(),
            Self::EditTags(_) => unreachable!(),
            Self::EditSpecies(_) => unreachable!(),
            Self::View(_) => unreachable!(),
            Self::Select(_) => unreachable!(),
            Self::Publish(_) => unreachable!(),
            Self::InlineEditStart { .. } => unreachable!(),
            Self::InlineEditChange { .. } => unreachable!(),
            Self::InlineSaveChange { .. } => unreachable!(),
            Self::InlineDiscardChanges { .. } => unreachable!(),
            Self::EditDetails(_) => unreachable!(),
            Self::StartGrouping(_) => unreachable!(),
            Self::AbortGrouping => unreachable!(),
            Self::AddToGroup(_) => unreachable!(),
            Self::RemoveFromGroup(_) => unreachable!(),
            Self::SaveGroup => unreachable!(),
            Self::ModalTags(msg) => msg.name(),
            Self::ModalSpecies(msg) => msg.name(),
            Self::ModalView(msg) => msg.name(),
            Self::ModalDescription(msg) => msg.name(),
            Self::ModalPublish(msg) => msg.name(),
            Self::CloseModal => unreachable!(),
            Self::Confirm(_) => unreachable!(),
            Self::Copy(_) => unreachable!(),
            Self::RefreshView => unreachable!(),
            Self::Hovered(_) => unreachable!(),
            Self::StartGroupingCurrent => "start grouping photos in the highlighted post",
            Self::PublishCurrent => "publish the highlighted post",
            Self::EditDescriptionCurrent => "edit description of the highlighted post",
            Self::EditTagsCurrent => "edit tags of the highlighted post",
            Self::EditSpeciesCurrent => "edit species of the highlighted post",
            Self::ViewCurrent => "fullscreen view of photos from the highlighted post",
            Self::SelectNext => "select the next post",
            Self::SelectPrev => "select the previous post",
            Self::SelectNextRow => "select the next post",
            Self::SelectPrevRow => "select the previous post",
            Self::SelectNextRowsMany => "move selection some position forward",
            Self::SelectPrevRowsMany => "move selection some position backward",
            Self::SelectFirst => "scroll to the beginning",
            Self::SelectLast => "scroll to the end",
            Self::Undo => "undo changes",
            Self::FocusSearch => "focus search bar",
            Self::FocusItem(_) => unreachable!(),
            Self::FilterByDate(_) => unreachable!(),
            Self::FilterByMonth(_) => unreachable!(),
            Self::SetViewKind(_) => unreachable!(),
            Self::SetGridColumns(_) => unreachable!(),
        }
    }
}

impl From<Message> for MainMessage {
    fn from(val: Message) -> Self {
        Self::TabPosts(val)
    }
}

impl From<EditDetails> for Message {
    fn from(val: EditDetails) -> Self {
        Self::EditDetails(val)
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum Field {
    Polish,
    English,
}

impl Default for TabPosts {
    fn default() -> Self {
        let mut queue = MessageQueue::new();
        queue.push_back(Message::RefreshView);

        Self {
            view: Vec::new(),
            hovered: None,
            selected: None,
            scroll_to_selected: false,
            grid_columns: 1,
            version: 0,
            filter: Filter::default(),
            queue,
            inline_editors: BTreeMap::new(),
            modal_window: ModalWindow::None,
            view_kind: ViewKind::List,
            label_width: 0.0,
            group: None,
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        }
    }
}

mod shortcut {
    use super::*;

    const fn ctrl(key: Key) -> KeyboardShortcut {
        KeyboardShortcut {
            logical_key: key,
            modifiers: Modifiers::CTRL,
        }
    }

    const fn key(key: Key) -> KeyboardShortcut {
        KeyboardShortcut {
            logical_key: key,
            modifiers: Modifiers::NONE,
        }
    }

    pub const EDIT_DESCRIPTION: KeyboardShortcut = ctrl(Key::E);
    pub const EDIT_TAGS: KeyboardShortcut = ctrl(Key::T);
    pub const EDIT_SPECIES: KeyboardShortcut = ctrl(Key::S);
    pub const PUBLISH: KeyboardShortcut = ctrl(Key::P);
    pub const START_GROUPING: KeyboardShortcut = ctrl(Key::G);
    pub const PREVIEW_1: KeyboardShortcut = key(Key::F);
    pub const PREVIEW_2: KeyboardShortcut = key(Key::V);
    pub const PREVIEW_3: KeyboardShortcut = key(Key::Space);
}

impl TabPosts {
    pub fn load(&mut self, db_id: &str, storage: &dyn eframe::Storage) {
        self.filter.load(db_id, storage);
        if let Some(value) =
            eframe::get_value::<Option<PostId>>(storage, fmt!("{ID_PREFIX}-selected-post"))
        {
            self.selected = value;
        }
        if let Some(view_kind) = eframe::get_value(storage, fmt!("{ID_PREFIX}-view-kind")) {
            self.view_kind = view_kind;
        }
    }

    pub fn save(&self, db_id: &str, storage: &mut dyn eframe::Storage) {
        self.filter.save(db_id, storage);
        eframe::set_value(storage, fmt!("{ID_PREFIX}-selected-post"), &self.selected);
        eframe::set_value(storage, fmt!("{ID_PREFIX}-view-kind"), &self.view_kind);
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &mut Database,
        main_queue: &mut MainMessageQueue,
    ) {
        db.refresh_caches();
        if self.version != db.current_version.posts {
            self.queue.push_back(Message::RefreshView);
            self.version = db.current_version.posts;
        }

        let mut queue = MessageQueue::new();

        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(ctx, db, msg, &mut queue, main_queue);
        }

        if !queue.is_empty() {
            self.queue = queue;
            return;
        }

        match &mut self.modal_window {
            ModalWindow::None => {
                self.draw(ctx, image_cache, style, db, &mut queue);
            }
            ModalWindow::ModalTags(window) => {
                window.update(ctx, image_cache, style, db, &mut queue);
            }
            ModalWindow::ModalSpecies(window) => {
                window.update(ctx, image_cache, style, db, &mut queue);
            }
            ModalWindow::ModalPublish(window) => {
                window.update(ctx, image_cache, style, db, &mut queue);
            }
            ModalWindow::ModalView(window) => {
                window.update(ctx, image_cache, db, &mut queue);
            }
            ModalWindow::ModalDescription(window) => {
                window.update(ctx, image_cache, style, db, &mut queue);
            }
        }

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }

        self.scroll_to_selected = false;
    }

    fn handle_message(
        &mut self,
        ctx: &Context,
        db: &mut Database,
        message: Message,
        queue: &mut MessageQueue,
        main_queue: &mut MainMessageQueue,
    ) {
        match message {
            Message::EditDetails(edit_details) => {
                main_queue.push_back(edit_details.into());
            }
            Message::RefreshView => {
                let phrase = self.filter.search_box.phrase(ctx);
                self.view = self.filter.make_view(&phrase, db);
            }
            Message::EditTags(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalTags::new(id, db);
                self.modal_window = ModalWindow::ModalTags(Box::new(window));
            }
            Message::EditSpecies(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalSpecies::new(id, db);
                self.modal_window = ModalWindow::ModalSpecies(Box::new(window));
            }
            Message::InlineEditStart { id, field } => {
                let post = db.post_mut(&id);
                let key = (post.id, field);
                let id = Id::new(("inline-editor", id, field));

                let editor = InlineEditor {
                    id,
                    text: match field {
                        Field::Polish => post.pl.clone(),
                        Field::English => post.en.clone(),
                    },
                };

                self.inline_editors.insert(key, editor);
                queue.push_back(Message::FocusItem(id));
            }
            Message::InlineEditChange { id, field, value } => {
                let editor = self.inline_editors.get_mut(&(id, field)).unwrap();

                editor.text = value;
            }
            Message::InlineSaveChange { id, field } => {
                let editor = self.inline_editors.remove(&(id, field)).unwrap();

                match field {
                    Field::Polish => {
                        let val = editor.text;

                        let msg = EditDetails::SetPolish(id, val);
                        main_queue.push_back(msg.into());
                    }
                    Field::English => {
                        let val = editor.text;

                        let msg = EditDetails::SetEnglish(id, val);
                        main_queue.push_back(msg.into());
                    }
                }
            }
            Message::InlineDiscardChanges { id, field } => {
                self.inline_editors.remove(&(id, field));
            }
            Message::Select(id) => {
                self.scroll_to_selected = self.selected != Some(id);
                self.selected = Some(id);
            }
            Message::View(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalView::new(id, db);
                self.modal_window = ModalWindow::ModalView(Box::new(window));
            }
            Message::Publish(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalPublish::new(id, db);
                self.modal_window = ModalWindow::ModalPublish(Box::new(window));
            }
            Message::Hovered(post_id) => {
                self.hovered = post_id;
            }
            Message::Copy(text) => {
                main_queue.push_back(MainMessage::Copy(text));
            }
            Message::StartGrouping(id) => {
                if self.group.is_none() {
                    self.group = Some(Group::new(&id));
                }
            }
            Message::AbortGrouping => {
                self.group = None;
            }
            Message::AddToGroup(id) => {
                if let Some(group) = self.group.as_mut() {
                    group.add(&id);
                }
            }
            Message::RemoveFromGroup(id) => {
                if let Some(group) = self.group.as_mut() {
                    group.remove(&id);
                }
            }
            Message::SaveGroup => {
                assert!(self.group.is_some());
                let group = self.group.take().unwrap();

                group.apply(db);
                queue.push_back(Message::RefreshView);
            }
            Message::EditDescription(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalDescription::new(id, db);
                self.modal_window = ModalWindow::ModalDescription(Box::new(window));
            }
            Message::CloseModal => {
                assert!(!self.modal_window.is_none());
                self.modal_window = ModalWindow::None;
            }
            Message::ModalTags(msg) => {
                if let ModalWindow::ModalTags(window) = &mut self.modal_window {
                    window.queue.push_back(msg);
                }
            }
            Message::ModalSpecies(msg) => {
                if let ModalWindow::ModalSpecies(window) = &mut self.modal_window {
                    window.queue.push_back(msg);
                }
            }
            Message::ModalView(msg) => {
                if let ModalWindow::ModalView(window) = &mut self.modal_window {
                    window.queue.push_back(msg);
                }
            }
            Message::ModalDescription(msg) => {
                if let ModalWindow::ModalDescription(window) = &mut self.modal_window {
                    window.queue.push_back(msg);
                }
            }
            Message::ModalPublish(msg) => {
                if let ModalWindow::ModalPublish(window) = &mut self.modal_window {
                    window.queue.push_back(msg);
                }
            }
            Message::Confirm(confirm) => {
                main_queue.push_back(MainMessage::Confirm(confirm));
            }
            Message::FocusSearch => {
                self.filter.search_box.take_focus(ctx);
            }
            Message::PublishCurrent => {
                if let Some(id) = self.hovered {
                    queue.push_back(Message::Publish(id));
                }
            }
            Message::Undo => {
                if let Some(id) = self.hovered {
                    queue.push_back(EditDetails::Undo(id).into());
                }
            }
            Message::EditDescriptionCurrent => {
                if let Some(id) = self.hovered {
                    queue.push_back(Message::EditDescription(id));
                }
            }
            Message::EditTagsCurrent => {
                if let Some(id) = self.hovered {
                    queue.push_back(Message::EditTags(id));
                }
            }
            Message::EditSpeciesCurrent => {
                if let Some(id) = self.hovered {
                    queue.push_back(Message::EditSpecies(id));
                }
            }
            Message::StartGroupingCurrent => {
                if let Some(id) = self.hovered {
                    queue.push_back(Message::StartGrouping(id));
                }
            }
            Message::ViewCurrent => {
                if let Some(id) = self.hovered {
                    queue.push_back(Message::View(id));
                }
            }
            Message::SelectNext => {
                let id = move_selection(&self.view, self.selected, 1);
                self.scroll_to_selected = self.selected != id;
                self.selected = id;
            }
            Message::SelectPrev => {
                let id = move_selection(&self.view, self.selected, -1);
                self.scroll_to_selected = self.selected != id;
                self.selected = id;
            }
            Message::SelectNextRow => {
                let id = move_selection(&self.view, self.selected, self.grid_columns);
                self.scroll_to_selected = self.selected != id;
                self.selected = id;
            }
            Message::SelectPrevRow => {
                let id = move_selection(&self.view, self.selected, -self.grid_columns);
                self.scroll_to_selected = self.selected != id;
                self.selected = id;
            }
            Message::SelectNextRowsMany => {
                let id = move_selection(&self.view, self.selected, 5 * self.grid_columns);
                self.scroll_to_selected = self.selected != id;
                self.selected = id;
            }
            Message::SelectPrevRowsMany => {
                let id = move_selection(&self.view, self.selected, -5 * self.grid_columns);
                self.scroll_to_selected = self.selected != id;
                self.selected = id;
            }
            Message::SelectFirst => {
                let id = self.view.first().copied();
                self.scroll_to_selected = self.selected != id;
                self.selected = id;
            }
            Message::SelectLast => {
                let id = self.view.last().copied();
                self.scroll_to_selected = self.selected != id;
                self.selected = id;
            }
            Message::FocusItem(id) => {
                ctx.memory_mut(|mem| mem.request_focus(id));
            }
            Message::FilterByDate(date) => {
                self.filter.current = Selector::ByDate(date);
                self.scroll_to_selected = true;
                queue.push_back(Message::RefreshView);
            }
            Message::FilterByMonth(month) => {
                self.filter.current = Selector::ByMonth(month);
                self.scroll_to_selected = true;
                queue.push_back(Message::RefreshView);
            }
            Message::SetViewKind(view_kind) => {
                self.view_kind = view_kind;
                self.scroll_to_selected = true;
                self.grid_columns = 1;
            }
            Message::SetGridColumns(grid_columns) => {
                self.grid_columns = grid_columns;
            }
        }
    }

    fn create_mapping() -> KeyboardMapping {
        const fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(msg)
        }

        KeyboardMapping::default()
            .key(Key::Slash, msg(Message::FocusSearch))
            .shortcut(shortcut::PUBLISH, msg(Message::PublishCurrent))
            .key(shortcut::PUBLISH.logical_key, msg(Message::PublishCurrent))
            .ctrl(Key::Z, msg(Message::Undo))
            .shortcut(
                shortcut::EDIT_DESCRIPTION,
                msg(Message::EditDescriptionCurrent),
            )
            .key(
                shortcut::EDIT_DESCRIPTION.logical_key,
                msg(Message::EditDescriptionCurrent),
            )
            .shortcut(shortcut::EDIT_TAGS, msg(Message::EditTagsCurrent))
            .key(
                shortcut::EDIT_TAGS.logical_key,
                msg(Message::EditTagsCurrent),
            )
            .shortcut(shortcut::EDIT_SPECIES, msg(Message::EditSpeciesCurrent))
            .key(
                shortcut::EDIT_SPECIES.logical_key,
                msg(Message::EditSpeciesCurrent),
            )
            .shortcut(shortcut::START_GROUPING, msg(Message::StartGroupingCurrent))
            .shortcut(shortcut::PREVIEW_1, msg(Message::ViewCurrent))
            .shortcut(shortcut::PREVIEW_2, msg(Message::ViewCurrent))
            .shortcut(shortcut::PREVIEW_3, msg(Message::ViewCurrent))
            .key(Key::ArrowDown, msg(Message::SelectNextRow))
            .key(Key::ArrowUp, msg(Message::SelectPrevRow))
            .key(Key::ArrowLeft, msg(Message::SelectPrev))
            .key(Key::ArrowRight, msg(Message::SelectNext))
            .ctrl(Key::ArrowDown, msg(Message::SelectNextRowsMany))
            .ctrl(Key::ArrowUp, msg(Message::SelectPrevRowsMany))
            .key(Key::PageDown, msg(Message::SelectNextRowsMany))
            .key(Key::PageUp, msg(Message::SelectPrevRowsMany))
            .key(Key::Home, msg(Message::SelectFirst))
            .key(Key::End, msg(Message::SelectLast))
    }

    pub fn get_keyboard_mapping(&self) -> &KeyboardMapping {
        match &self.modal_window {
            ModalWindow::ModalSpecies(window) => &window.keyboard_mapping,
            ModalWindow::ModalTags(window) => &window.keyboard_mapping,
            ModalWindow::ModalPublish(window) => &window.keyboard_mapping,
            ModalWindow::ModalView(window) => &window.keyboard_mapping,
            ModalWindow::ModalDescription(window) => &window.keyboard_mapping,
            ModalWindow::None => &self.keyboard_mapping,
        }
    }

    fn draw(
        &mut self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        TopBottomPanel::top(fmt!("{ID_PREFIX}-filter")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.filter.view(ui, db, queue);

                ui.separator();

                let mut val = self.view_kind.clone();
                for option in [ViewKind::List, ViewKind::Grid] {
                    ui.selectable_value(&mut val, option.clone(), option.name());
                }

                if val != self.view_kind {
                    queue.push_back(Message::SetViewKind(val));
                }
            });

            if self.label_width == 0.0 {
                self.label_width = crate::gui::max_size(&["tags", "species"], ui);
            }
        });

        if let Some(group) = &self.group {
            SidePanel::left(fmt!("{ID_PREFIX}-group"))
                .resizable(false)
                .min_width(style.image.preview_width)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(group.count().to_string());
                        if button::cancel(ui) {
                            queue.push_back(Message::AbortGrouping);
                        }

                        let enabled = !group.is_empty();
                        let background = Some(style.button.save);
                        if button::save(ui, enabled, background) {
                            queue.push_back(Message::SaveGroup);
                        }
                    });

                    ScrollArea::vertical()
                        .id_salt(fmt!("{ID_PREFIX}-group-scroll"))
                        .show(ui, |ui| {
                            self.show_group(ui, image_cache, style, db, queue);
                        });
                });
        }

        CentralPanel::default().show(ctx, |ui| {
            match self.view_kind {
                ViewKind::List => self.draw_main_list(ui, image_cache, style, db, queue),
                ViewKind::Grid => self.draw_grid(ui, image_cache, style, db, queue),
            };
        });
    }

    fn draw_main_list(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        ScrollArea::both()
            .id_salt(fmt!("{ID_PREFIX}-scroll-main"))
            .show(ui, |ui| {
                let mut hovered: Option<PostId> = None;
                for id in self.view.iter() {
                    if let Some(group) = &self.group {
                        if group.contains(id) {
                            continue;
                        }
                    }

                    let post = db.post(id);
                    if self.draw_post(ui, image_cache, style, post, db, queue) {
                        hovered = Some(*id);
                    }
                }

                if hovered != self.hovered {
                    queue.push_back(Message::Hovered(hovered));
                }
            });
    }

    fn draw_post(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        post: &Post,
        db: &Database,
        queue: &mut MessageQueue,
    ) -> bool {
        let fill = self.fill(post, style);

        let resp = frame(ui, fill, |ui| {
            self.draw_post_inner(ui, image_cache, style, post, db, queue);
        });

        if self.scroll_to_selected && self.selected == Some(post.id) {
            ui.scroll_to_rect(resp.rect, Some(Align::Center));
        }

        if resp.double_clicked() {
            queue.push_back(Message::View(post.id));
        }

        if resp.clicked() {
            queue.push_back(Message::Select(post.id));
        }

        resp.context_menu(|ui| self.post_context_menu(ui, post, queue));

        resp.contains_pointer()
    }

    fn post_context_menu(&self, ui: &mut Ui, post: &Post, queue: &mut MessageQueue) {
        let enabled = match &self.filter.current {
            Selector::ByDate(date) => *date != post.date,
            _ => true,
        };
        let button = Button::new(format!("Show posts from {}", post.date));
        if ui.add_enabled(enabled, button).clicked() {
            queue.push_back(Message::FilterByDate(post.date));
        }

        let enabled = match &self.filter.current {
            Selector::ByMonth(month) => *month != post.date.month,
            _ => true,
        };
        let button = Button::new(format!("Show posts from {}", post.date.month));
        if ui.add_enabled(enabled, button).clicked() {
            queue.push_back(Message::FilterByMonth(post.date.month));
        }

        ui.separator();

        let button = Button::new(fmt!("{ICON_DIALOGS} Edit description"))
            .shortcut_text(format_shortcut(shortcut::EDIT_DESCRIPTION));
        if ui.add(button).clicked() {
            queue.push_back(Message::EditDescription(post.id));
        }

        let button = Button::new(fmt!("{ICON_DIALOGS} Edit tags"))
            .shortcut_text(format_shortcut(shortcut::EDIT_TAGS));
        if ui.add(button).clicked() {
            queue.push_back(Message::EditTags(post.id));
        }

        let button = Button::new(fmt!("{ICON_DIALOGS} Edit species"))
            .shortcut_text(format_shortcut(shortcut::EDIT_SPECIES));
        if ui.add(button).clicked() {
            queue.push_back(Message::EditSpecies(post.id));
        }

        let enabled = !post.published;
        let button = Button::new(fmt!("{ICON_DIALOGS} Publish post"))
            .shortcut_text(format_shortcut(shortcut::PUBLISH));
        if ui.add_enabled(enabled, button).clicked() {
            queue.push_back(Message::Publish(post.id));
        }

        let button = Button::new(fmt!("{ICON_FULLSCREEN} Show fullscreen"))
            .shortcut_text(format_shortcut(shortcut::PREVIEW_1));
        if ui.add(button).clicked() {
            queue.push_back(Message::View(post.id));
        }

        ui.separator();

        let button = Button::new("Start photos grouping")
            .shortcut_text(format_shortcut(shortcut::START_GROUPING));
        if ui.add(button).clicked() {
            queue.push_back(Message::StartGrouping(post.id));
        }
    }

    fn draw_image(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        post: &Post,
        queue: &mut MessageQueue,
    ) {
        let resp = add_image(
            ui,
            &post.files_meta[0],
            image_cache,
            style.image.preview_width,
            style.image.radius,
        );

        let n = post.files.len();
        if n > 1 {
            add_overlay(
                ui,
                &resp,
                OverlayLocation::BottomRight,
                style.image.overlay.margin,
                |ui| {
                    let count = ImageCounter(n);
                    let label = count.to_string();

                    ui.add(overlay_label(label, style))
                },
            );
        }

        if post.published {
            add_overlay(
                ui,
                &resp,
                OverlayLocation::TopLeft,
                style.image.overlay.margin,
                |ui| {
                    let text = RichText::new(ICON_CHECK).color(style.copied_mark);
                    let label = Label::new(text).selectable(false);
                    ui.add(label)
                },
            );
        }

        if self.group.is_some() {
            let resp = add_overlay(
                ui,
                &resp,
                OverlayLocation::TopLeft,
                style.image.overlay.margin,
                |ui: &mut Ui| {
                    let label = fmt!("{ICON_ADD} Add to group");
                    let button = Button::new(label).fill(style.button.save);

                    ui.add(button)
                },
            );

            if resp.clicked() {
                queue.push_back(Message::AddToGroup(post.id));
            }
        }
    }

    fn draw_post_inner(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        post: &Post,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        ui.horizontal(|ui| {
            self.draw_image(ui, image_cache, style, post, queue);

            ui.vertical(|ui| {
                ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        if post.is_dirty() {
                            let label = format!("{}", post.files[0].display());
                            ui.colored_label(style.modified, RichText::new(label).heading());
                        } else {
                            let label = format!("{}", post.files[0].display());
                            ui.heading(label);
                        };

                        if !post.published {
                            if ui.button("Publish").clicked() {
                                queue.push_back(Message::Publish(post.id));
                            }
                        }

                        if post.is_dirty() {
                            if ui.button(fmt!("{ICON_UNDO} Undo")).clicked() {
                                queue.push_back(EditDetails::Undo(post.id).into());
                            }
                        }
                    });
                });

                let inline_editor = self.inline_editors.get(&(post.id, Field::Polish));

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_width(self.label_width);
                        icon_pl(ui);
                    });

                    ui.horizontal(|ui| {
                        let msg = inline_edit(ui, post.id, &post.pl, Field::Polish, inline_editor);
                        if let Some(msg) = msg {
                            queue.push_back(msg);
                        }
                    });
                });

                let inline_editor = self.inline_editors.get(&(post.id, Field::English));

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_width(self.label_width);
                        icon_en(ui);
                    });

                    ui.horizontal(|ui| {
                        let msg = inline_edit(ui, post.id, &post.en, Field::English, inline_editor);
                        if let Some(msg) = msg {
                            queue.push_back(msg);
                        }
                    });
                });

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_width(self.label_width);
                        ui.label("tags");
                    });

                    self.show_tags(ui, style, post, queue);
                });

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_width(self.label_width);
                        ui.label("species");
                    });

                    self.show_species(ui, post, db, queue);
                });
            });
        });
    }

    fn show_tags(&self, ui: &mut Ui, style: &Style, post: &Post, queue: &mut MessageQueue) {
        ui.horizontal_wrapped(|ui| {
            if button::edit(ui) {
                queue.push_back(Message::EditTags(post.id));
            }
            if button::copy(ui, !post.tags.is_empty()) {
                queue.push_back(Message::Copy(post.tags_string.clone()));
            }
            for string in post.tags.iter() {
                ui.add(tag(string, style));
            }
        });
    }

    fn show_group(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        let Some(group) = &self.group else {
            return;
        };

        for (entry_id, id) in group.iter().enumerate() {
            let post = db.post(id);
            for (photo_id, meta) in post.files_meta.iter().enumerate() {
                if entry_id == 0 && photo_id == 0 {
                    add_image(
                        ui,
                        meta,
                        image_cache,
                        style.image.preview_width,
                        style.image.radius,
                    );
                } else if photo_id > 0 {
                    add_image_with_tint(
                        ui,
                        meta,
                        image_cache,
                        style.image.preview_width,
                        style.image.radius,
                        style.image.inactive,
                    );
                } else {
                    let resp = add_image(
                        ui,
                        meta,
                        image_cache,
                        style.image.preview_width,
                        style.image.radius,
                    );

                    let resp = add_overlay(
                        ui,
                        &resp,
                        OverlayLocation::BottomRight,
                        style.image.overlay.margin,
                        |ui| {
                            let label = fmt!("{ICON_DELETE} Remove from group");
                            let button = Button::new(label).fill(style.button.remove);

                            ui.add(button)
                        },
                    );
                    if resp.clicked() {
                        queue.push_back(Message::RemoveFromGroup(*id));
                    }
                }
            }
        }
    }

    fn show_species(&self, ui: &mut Ui, post: &Post, db: &Database, queue: &mut MessageQueue) {
        ui.horizontal(|ui| {
            if ui.button(ICON_EDIT).clicked() {
                queue.push_back(Message::EditSpecies(post.id));
            }
            if button::copy(ui, post.species.is_some()) {
                let latin = post.species.as_ref().unwrap().clone();
                queue.push_back(Message::Copy(latin.into()));
            }
            if let Some(latin) = &post.species {
                let species = db.species_by_latin(latin).unwrap();
                crate::species_view::singleline(ui, species);

                if post.is_example {
                    if ui.button("🗙 Not a good example").clicked() {
                        let msg = EditDetails::Example(post.id, false);
                        queue.push_back(msg.into());
                    }
                } else {
                    if ui.button("✔ Set as an example").clicked() {
                        let msg = EditDetails::Example(post.id, true);
                        queue.push_back(msg.into());
                    }
                }
            } else {
                ui.label("—");
            }
        });
    }

    fn draw_grid(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        ScrollArea::both()
            .id_salt(fmt!("{ID_PREFIX}-scroll-main"))
            .show(ui, |ui| {
                let mut hovered: Option<PostId> = None;
                let width = ui.available_size().x;
                let n = (width / (style.image.preview_width + 8.0)) as isize;
                if n != self.grid_columns {
                    queue.push_back(Message::SetGridColumns(n));
                }

                let mut it = self.view.iter().filter(|id| {
                    if let Some(group) = &self.group {
                        !group.contains(id)
                    } else {
                        true
                    }
                });

                let mut empty = false;
                while !empty {
                    ui.horizontal(|ui| {
                        for _ in 0..n {
                            if let Some(id) = it.next() {
                                let post = db.post(id);
                                let fill = self.fill(post, style);

                                let resp = tight_frame(ui, fill, |ui| {
                                    self.draw_image(ui, image_cache, style, post, queue);
                                });

                                if self.scroll_to_selected && self.selected == Some(post.id) {
                                    ui.scroll_to_rect(resp.rect, Some(Align::Center));
                                }

                                if resp.contains_pointer() {
                                    hovered = Some(*id);
                                }
                                if resp.clicked() {
                                    queue.push_back(Message::Select(post.id));
                                }
                                if resp.double_clicked() {
                                    queue.push_back(Message::View(post.id));
                                }
                                resp.context_menu(|ui| self.post_context_menu(ui, post, queue));
                            } else {
                                empty = true;
                                break;
                            }
                        }
                    });
                }

                if hovered != self.hovered {
                    queue.push_back(Message::Hovered(hovered));
                }
            });
    }

    fn fill(&self, post: &Post, style: &Style) -> Option<Color32> {
        if self.selected == Some(post.id) {
            Some(style.selected_post)
        } else if self.hovered == Some(post.id) {
            Some(style.hovered_frame)
        } else if post.published {
            Some(style.published_post)
        } else {
            None
        }
    }
}

fn inline_edit(
    ui: &mut Ui,
    id: PostId,
    current: &str,
    field: Field,
    inline: Option<&InlineEditor>,
) -> Option<Message> {
    let mut result: Option<Message> = None;
    if let Some(inline) = inline {
        let mut value = inline.text.clone();
        let edit = TextEdit::singleline(&mut value).id(inline.id);
        let resp = ui.add(edit);
        let changed = value != current;
        if resp.changed() {
            result = Some(Message::InlineEditChange { id, field, value });
        }

        if resp.lost_focus() {
            if ui.input(|input| input.key_pressed(Key::Enter)) {
                result = Some(Message::InlineSaveChange { id, field });
            } else if !changed {
                result = Some(Message::InlineDiscardChanges { id, field });
            }
        }

        if ui.button(ICON_CHECK).clicked() {
            result = Some(Message::InlineSaveChange { id, field });
        }
        if ui.button(ICON_CLOSE).clicked() {
            result = Some(Message::InlineDiscardChanges { id, field });
        }
    } else {
        if ui.button(ICON_EDIT).clicked() {
            result = Some(Message::InlineEditStart { id, field });
        }
        if button::copy(ui, !current.is_empty()) {
            result = Some(Message::Copy(current.to_owned()));
        }

        let mut label = if current.is_empty() {
            let color = ui.visuals().weak_text_color();
            Label::new(RichText::new("<click to edit>").color(color))
        } else {
            Label::new(current)
        };

        label = label.selectable(false).sense(Sense::CLICK);
        if ui.add(label).clicked() {
            result = Some(Message::InlineEditStart { id, field });
        }
    }

    result
}

fn move_selection(view: &[PostId], selected: Option<PostId>, direction: isize) -> Option<PostId> {
    if view.is_empty() {
        return None;
    }

    let Some(selected) = selected else {
        return Some(view[0]);
    };

    if let Some(position) = view.iter().position(|post_id| *post_id == selected) {
        let max = view.len() - 1;
        let id = clamp(position as isize + direction, 0, max as isize);
        let id = id as usize;

        Some(view[id])
    } else {
        Some(view[0])
    }
}

fn clamp(val: isize, min: isize, max: isize) -> isize {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

fn format_shortcut(shortcut: KeyboardShortcut) -> String {
    crate::keyboard::format_shortcut(&shortcut.logical_key, &shortcut.modifiers)
}
