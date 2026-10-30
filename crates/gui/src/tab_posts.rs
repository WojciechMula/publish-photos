mod filter;
mod modal_desc;
pub mod modal_publish;
mod modal_species;
mod modal_tags;
mod modal_view;

use filter::Filter;
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
use crate::clipboard::Clipboard;
use crate::clipboard::ClipboardKind;
use crate::clipboard::ClipboardValue;
use crate::confirm::Confirm;
use crate::gui::add_image;
use crate::gui::add_overlay;
use crate::gui::button;
use crate::gui::frame;
use crate::gui::icon_en;
use crate::gui::icon_pl;
use crate::gui::tag;
use crate::gui::tight_frame;
use crate::gui::OverlayLocation;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::labels::LabelEntry;
use crate::style::Style;
use crate::tab_posts::filter::render_selector;
use crate::tab_species::Message as TabSpeciesMessage;
use crate::widgets::checkmark;
use crate::widgets::clickable_label_button;
use crate::widgets::label_button;
use crate::widgets::HistoryInputAction;
use const_format::formatcp as fmt;
use db::edit_details::EditDetails;
use db::Database;
use db::Date;
use db::Month;
use db::Post;
use db::PostId;
use db::Selector;
use db::SpeciesId;
use db::SpeciesIndex;
use db::TagList;
use db::Year;
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
use std::collections::BTreeMap;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_BACKSPACE;
use egui_material_icons::icons::ICON_CHECK;
use egui_material_icons::icons::ICON_CLOSE;
use egui_material_icons::icons::ICON_CONTENT_COPY;
use egui_material_icons::icons::ICON_DIALOGS;
use egui_material_icons::icons::ICON_EDIT;
use egui_material_icons::icons::ICON_FULLSCREEN;
use egui_material_icons::icons::ICON_GRID_ON;
use egui_material_icons::icons::ICON_LIST;
use egui_material_icons::icons::ICON_UNDO;

const ID_PREFIX: &str = "tab-posts";

type GroupedView = Vec<(Selector, Vec<PostId>)>;

pub struct TabPosts {
    version: u64,
    view: Vec<PostId>,
    view_grouped: GroupedView,
    hovered: Option<PostId>,
    selected: Option<PostId>,
    scroll_to_selected: bool,
    grid_columns: isize,
    filter: Filter,
    inline_editors: BTreeMap<(PostId, Field), InlineEditor>,
    label_width: f32,
    modal_window: ModalWindow,
    view_kind: ViewKind,
    show_selectors: bool,
    selector_filter: String,
    selector_granurality: SelectorGranurality,

    keyboard_mapping: KeyboardMapping,
    labels_version: u64,
    enabled_labels_count: usize,
    labels: BTreeMap<String, LabelEntry>,

    pub queue: MessageQueue,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub enum ViewKind {
    List,
    Grid,
}

impl ViewKind {
    const ALL: [Self; 2] = [Self::List, Self::Grid];

    const fn name(&self) -> &str {
        match self {
            Self::List => fmt!("{ICON_LIST} list"),
            Self::Grid => fmt!("{ICON_GRID_ON} grid"),
        }
    }

    fn next(&self) -> Self {
        match self {
            Self::List => Self::Grid,
            Self::Grid => Self::List,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default, Copy, Debug)]
pub enum SelectorGranurality {
    Year,
    Month,
    #[default]
    Day,
}

impl SelectorGranurality {
    const ALL: [Self; 3] = [Self::Year, Self::Month, Self::Day];

    const fn name(&self) -> &str {
        match self {
            Self::Year => "Year",
            Self::Month => "Month",
            Self::Day => "Day",
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
    StartPublishing(PostId),
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
        by_keyboard: bool,
    },
    InlineDiscardChanges {
        id: PostId,
        field: Field,
    },
    EditDetails(EditDetails),
    PasteTags(PostId, String),
    PasteSpecies(PostId, String),

    ModalTags(ModalTagsMessage),
    ModalSpecies(ModalSpeciesMessage),
    ModalView(ModalViewMessage),
    ModalDescription(ModalDescriptionMessage),
    ModalPublish(ModalPublishMessage),
    CloseModal,
    Confirm(Confirm),

    Copy(ClipboardValue),

    RefreshView,
    Hovered(Option<PostId>),
    PublishCurrent,
    EditDescriptionCurrent,
    EditTagsCurrent,
    EditSpeciesCurrent,
    ToggleLabelCurrent(String),
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
    FilterByMonth(Year, Month),
    SetViewKind(ViewKind),
    ToggleViewKind,
    SetGridColumns(isize),

    EditSpeciesDetails(SpeciesIndex),
    AddNewSpecies(String),
    SearchBoxAction(HistoryInputAction),
    ChangeSelector(Selector),
    ChangeSelectorFilter(String),
    ChangeSelectorGranurality(SelectorGranurality),
    ToggleSelectorList,
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
            Self::StartPublishing(..) => unreachable!(),
            Self::InlineEditStart { .. } => unreachable!(),
            Self::InlineEditChange { .. } => unreachable!(),
            Self::InlineSaveChange { .. } => unreachable!(),
            Self::InlineDiscardChanges { .. } => unreachable!(),
            Self::EditDetails(_) => unreachable!(),
            Self::PasteTags(..) => unreachable!(),
            Self::PasteSpecies(..) => unreachable!(),
            Self::ModalTags(msg) => msg.name(),
            Self::ModalSpecies(msg) => msg.name(),
            Self::ModalView(msg) => msg.name(),
            Self::ModalDescription(msg) => msg.name(),
            Self::ModalPublish(msg) => msg.name(),
            Self::CloseModal => unreachable!(),
            Self::Confirm(_) => unreachable!(),
            Self::Copy(..) => unreachable!(),
            Self::RefreshView => unreachable!(),
            Self::Hovered(_) => unreachable!(),
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
            Self::FilterByMonth(_, _) => unreachable!(),
            Self::SetViewKind(_) => unreachable!(),
            Self::ToggleViewKind => "switch between grid/list view",
            Self::SetGridColumns(_) => unreachable!(),
            Self::EditSpeciesDetails(_) => unreachable!(),
            Self::AddNewSpecies(_) => "add new species",
            Self::SearchBoxAction(_) => unreachable!(),
            Self::ToggleLabelCurrent(_) => "toggle label",
            Self::ChangeSelector(_) => unreachable!(),
            Self::ChangeSelectorFilter(_) => unreachable!(),
            Self::ChangeSelectorGranurality(_) => unreachable!(),
            Self::ToggleSelectorList => "toggle selector list",
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

impl From<ClipboardValue> for Message {
    fn from(val: ClipboardValue) -> Self {
        Self::Copy(val)
    }
}

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub enum Field {
    Polish,
    English,
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

    pub const EDIT_DESCRIPTION: KeyboardShortcut = key(Key::E);
    pub const EDIT_TAGS: KeyboardShortcut = key(Key::T);
    pub const EDIT_SPECIES: KeyboardShortcut = key(Key::S);
    pub const UNDO: KeyboardShortcut = ctrl(Key::Z);
    pub const PUBLISH: KeyboardShortcut = key(Key::P);
    pub const PREVIEW_1: KeyboardShortcut = key(Key::F);
    pub const PREVIEW_2: KeyboardShortcut = key(Key::V);
    pub const PREVIEW_3: KeyboardShortcut = key(Key::Space);
}

impl TabPosts {
    pub fn new() -> Self {
        let mut queue = MessageQueue::new();
        queue.push_back(Message::RefreshView);

        Self {
            view: Vec::new(),
            view_grouped: Vec::new(),
            hovered: None,
            selected: None,
            scroll_to_selected: false,
            grid_columns: 1,
            version: u64::MAX,
            filter: Filter::default(),
            queue,
            inline_editors: BTreeMap::new(),
            modal_window: ModalWindow::None,
            view_kind: ViewKind::List,
            show_selectors: true,
            selector_filter: String::new(),
            selector_granurality: SelectorGranurality::default(),
            label_width: 0.0,
            keyboard_mapping: Self::create_mapping(),
            labels_version: u64::MAX,
            labels: BTreeMap::new(),
            enabled_labels_count: 0,
        }
    }

    #[allow(clippy::option_map_unit_fn)]
    pub fn load(&mut self, _db_id: &str, storage: &dyn eframe::Storage) {
        self.filter.load(storage);
        eframe::get_value(storage, fmt!("{ID_PREFIX}-selected-post")).map(|v| self.selected = v);
        eframe::get_value(storage, fmt!("{ID_PREFIX}-view-kind")).map(|v| self.view_kind = v);
        eframe::get_value(storage, fmt!("{ID_PREFIX}-show-selectors"))
            .map(|v| self.show_selectors = v);
        eframe::get_value(storage, fmt!("{ID_PREFIX}-selector-granurality"))
            .map(|v| self.selector_granurality = v);
    }

    pub fn save(&self, _db_id: &str, storage: &mut dyn eframe::Storage) {
        self.filter.save(storage);
        eframe::set_value(storage, fmt!("{ID_PREFIX}-selected-post"), &self.selected);
        eframe::set_value(storage, fmt!("{ID_PREFIX}-view-kind"), &self.view_kind);
        eframe::set_value(
            storage,
            fmt!("{ID_PREFIX}-show-selectors"),
            &self.show_selectors,
        );
        eframe::set_value(
            storage,
            fmt!("{ID_PREFIX}-selector-granurality"),
            &self.selector_granurality,
        );
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &mut Database,
        main_queue: &mut MainMessageQueue,
        clipboard: &Clipboard,
    ) {
        db.refresh_caches();
        if self.version != db.current_version.posts {
            self.queue.push_back(Message::RefreshView);
            self.version = db.current_version.posts;
        }

        self.update_labels(db);

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
                self.draw(ctx, image_cache, style, db, &mut queue, clipboard);
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
            Message::PasteTags(post_id, tags_string) => {
                let mut tags = TagList::default();
                for tag in tags_string.split_whitespace() {
                    tags.add(tag.to_owned());
                }
                let edit_details = EditDetails::SetTags(post_id, tags);

                main_queue.push_back(edit_details.into());
            }
            Message::PasteSpecies(post_id, species_text) => {
                if let Ok(species_id) = SpeciesId::try_from(species_text) {
                    let edit_details = EditDetails::SetSpecies(post_id, Some(species_id));

                    main_queue.push_back(edit_details.into());
                }
            }
            Message::RefreshView => {
                let state = self.save_selected();
                self.view = self.filter.make_view(db);
                self.view_grouped = mk_grouped_view(&self.view, db, self.filter.get_current());
                self.restore_selected(state);
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
                let post = db.post(&id);
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
            Message::InlineSaveChange {
                id,
                field,
                by_keyboard,
            } => {
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

                if by_keyboard {
                    let post = db.post(&id);
                    match field {
                        Field::Polish => {
                            if post.en.is_empty() {
                                queue.push_back(Message::InlineEditStart {
                                    id,
                                    field: Field::English,
                                });
                            }
                        }
                        Field::English => {
                            if post.pl.is_empty() {
                                queue.push_back(Message::InlineEditStart {
                                    id,
                                    field: Field::Polish,
                                });
                            }
                        }
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
                let window = ModalView::new(id);
                self.modal_window = ModalWindow::ModalView(Box::new(window));
            }
            Message::Publish(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalPublish::new(id, db);
                self.modal_window = ModalWindow::ModalPublish(Box::new(window));
            }
            Message::StartPublishing(id) => {
                main_queue.push_back(MainMessage::StartPublishing(id));
            }
            Message::Hovered(post_id) => {
                self.hovered = post_id;
            }
            Message::Copy(val) => {
                main_queue.push_back(MainMessage::Copy(val));
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
            Message::ViewCurrent => {
                if let Some(id) = self.hovered {
                    queue.push_back(Message::View(id));
                }
            }
            Message::ToggleLabelCurrent(label) => {
                if let Some(id) = self.hovered {
                    let msg = EditDetails::ToggleLabel(id, label);
                    main_queue.push_back(msg.into());
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
                self.filter.set_current(Selector::ByDate(date));
                self.scroll_to_selected = true;
                queue.push_back(Message::RefreshView);
            }
            Message::FilterByMonth(year, month) => {
                self.filter.set_current(Selector::ByMonth(year, month));
                self.scroll_to_selected = true;
                queue.push_back(Message::RefreshView);
            }
            Message::SetViewKind(view_kind) => {
                self.view_kind = view_kind;
                self.scroll_to_selected = true;
                self.grid_columns = 1;
            }
            Message::ToggleViewKind => {
                queue.push_back(Message::SetViewKind(self.view_kind.next()));
            }
            Message::SetGridColumns(grid_columns) => {
                self.grid_columns = grid_columns;
            }
            Message::EditSpeciesDetails(id) => {
                main_queue.push_back(MainMessage::SelectTabSpecies);
                main_queue.push_back(TabSpeciesMessage::OpenModalEdit(id).into());
            }
            Message::AddNewSpecies(hint) => {
                main_queue.push_back(MainMessage::SelectTabSpecies);
                main_queue.push_back(TabSpeciesMessage::AddNewFrom(hint).into());
            }
            Message::SearchBoxAction(action) => {
                self.filter.search_box.update(action);
            }
            Message::ChangeSelector(selector) => {
                self.filter.set_current(selector);
                queue.push_back(Message::RefreshView);
            }
            Message::ChangeSelectorFilter(val) => {
                self.selector_filter = val;
            }
            Message::ChangeSelectorGranurality(val) => {
                self.selector_granurality = val;
            }
            Message::ToggleSelectorList => {
                self.show_selectors = !self.show_selectors;
            }
        }
    }

    fn save_selected(&self) -> Option<SelectedState> {
        let id = self.selected?;
        let position = self.view.iter().position(|post_id| *post_id == id)?;

        Some(SelectedState { id, position })
    }

    fn restore_selected(&mut self, state: Option<SelectedState>) {
        if let Some(state) = state {
            let selected_still_in_view = self.view.contains(&state.id);

            if selected_still_in_view {
                self.selected = Some(state.id);
            } else {
                match self.view.get(state.position) {
                    Some(post_id) => {
                        self.selected = Some(*post_id);
                    }
                    None => {
                        self.selected = self.view.last().copied();
                    }
                }
            }
        } else {
            self.selected = self.view.first().cloned();
        }
    }

    fn update_labels(&mut self, db: &Database) {
        if self.labels_version == db.current_version.labels {
            return;
        }

        self.keyboard_mapping = Self::create_mapping();
        self.labels.clear();
        self.enabled_labels_count = 0;

        for (_, entry) in crate::labels::from_db(db) {
            if let Some(shortcut) = entry.shortcut.as_ref() {
                if entry.enabled {
                    self.keyboard_mapping.add(
                        shortcut.logical_key,
                        shortcut.modifiers,
                        Message::ToggleLabelCurrent(entry.label.clone()).into(),
                    );
                    self.enabled_labels_count += 1;
                }
            }

            self.labels.insert(entry.label.clone(), entry);
        }

        self.labels_version = db.current_version.labels;
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
            .ctrl(Key::L, msg(Message::ToggleViewKind))
            .key(Key::L, msg(Message::ToggleSelectorList))
    }

    pub fn modal_opened(&self) -> bool {
        !matches!(self.modal_window, ModalWindow::None)
    }

    pub fn try_close_modal(&mut self) {
        match &mut self.modal_window {
            ModalWindow::ModalSpecies(window) => window.try_close(),
            ModalWindow::ModalTags(window) => window.try_close(),
            ModalWindow::ModalPublish(window) => window.try_close(),
            ModalWindow::ModalView(window) => window.try_close(),
            ModalWindow::ModalDescription(window) => window.try_close(),
            ModalWindow::None => (),
        }
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
        clipboard: &Clipboard,
    ) {
        TopBottomPanel::top(fmt!("{ID_PREFIX}-filter")).show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    self.filter.view(ui, db, queue);

                    ui.separator();

                    let mut val = self.view_kind;
                    for option in ViewKind::ALL {
                        ui.selectable_value(&mut val, option, option.name());
                    }

                    if val != self.view_kind {
                        queue.push_back(Message::SetViewKind(val));
                    }
                });

                if self.filter.is_extra_filter_enabled() {
                    ui.horizontal(|ui| {
                        self.filter.view_extra(ui, queue);
                    });
                }
            });

            if self.label_width == 0.0 {
                self.label_width = crate::gui::max_size(&["tags", "species", "labels"], ui);
            }
        });

        if self.show_selectors {
            SidePanel::left(fmt!("{ID_PREFIX}-list")).show(ctx, |ui| {
                self.draw_selectors(ui, db, queue);
            });
        }

        CentralPanel::default().show(ctx, |ui| {
            match self.view_kind {
                ViewKind::List => self.draw_main_list(ui, image_cache, style, db, queue, clipboard),
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
        clipboard: &Clipboard,
    ) {
        ScrollArea::both()
            .id_salt((ID_PREFIX, "scroll-main-list", self.filter.get_current()))
            .show(ui, |ui| {
                let mut hovered: Option<PostId> = None;
                for id in self.view.iter() {
                    let post = db.post(id);
                    if self.draw_post(ui, image_cache, style, post, db, queue, clipboard) {
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
        clipboard: &Clipboard,
    ) -> bool {
        let fill = self.fill(post, style);

        let resp = frame(ui, fill, |ui| {
            self.draw_post_inner(ui, image_cache, style, post, db, queue, clipboard);
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

    fn draw_selectors(&self, ui: &mut Ui, db: &Database, queue: &mut MessageQueue) {
        let mut filter = self.selector_filter.clone();

        ui.horizontal(|ui| {
            ui.set_max_width(200.0);
            ui.text_edit_singleline(&mut filter);

            let enabled = !filter.is_empty();
            let button = Button::new(ICON_BACKSPACE);
            if ui.add_enabled(enabled, button).clicked() {
                filter.clear();
            }
        });

        if filter != self.selector_filter {
            queue.push_back(Message::ChangeSelectorFilter(filter.clone()));
        }

        ui.separator();

        let mut selector_granurality = self.selector_granurality;

        ui.horizontal(|ui| {
            for dsg in SelectorGranurality::ALL {
                ui.selectable_value(&mut selector_granurality, dsg, dsg.name());
            }
        });

        if selector_granurality != self.selector_granurality {
            queue.push_back(Message::ChangeSelectorGranurality(selector_granurality));
        }

        ui.separator();

        ScrollArea::vertical()
            .id_salt((ID_PREFIX, "selectors"))
            .show(ui, |ui| {
                let mut current = self.filter.get_current();
                for selector in &db.picture_views.selectors {
                    if filter.is_empty() {
                        let valid = match selector_granurality {
                            SelectorGranurality::Year => matches!(selector, Selector::ByYear(_)),
                            SelectorGranurality::Month => {
                                matches!(selector, Selector::ByYear(_) | Selector::ByMonth(..))
                            }
                            SelectorGranurality::Day => true,
                        };

                        if !valid {
                            continue;
                        }
                    } else if !selector.as_iso_date().contains(&filter) {
                        continue;
                    }

                    render_selector(
                        ui,
                        db,
                        10.0,
                        self.filter.get_image_state(),
                        selector,
                        &mut current,
                    );
                }

                if current != self.filter.get_current() {
                    queue.push_back(Message::ChangeSelector(current));
                }
            });
    }

    fn post_context_menu(&self, ui: &mut Ui, post: &Post, queue: &mut MessageQueue) {
        let enabled = match self.filter.get_current() {
            Selector::ByDate(date) => date != post.date,
            _ => true,
        };
        let button = Button::new(format!("Show posts from {}", post.date));
        if ui.add_enabled(enabled, button).clicked() {
            queue.push_back(Message::FilterByDate(post.date));
        }

        let enabled = match self.filter.get_current() {
            Selector::ByMonth(year, month) => year != post.date.year || month != post.date.month,
            _ => true,
        };
        let button = Button::new(format!("Show posts from {}", post.date.month));
        if ui.add_enabled(enabled, button).clicked() {
            queue.push_back(Message::FilterByMonth(post.date.year, post.date.month));
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

        let enabled = post.is_dirty();
        let button =
            Button::new(fmt!("{ICON_UNDO} Undo")).shortcut_text(format_shortcut(shortcut::UNDO));
        if ui.add_enabled(enabled, button).clicked() {
            queue.push_back(EditDetails::Undo(post.id).into());
        }

        let enabled = !post.published.as_bool();
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

        if self.enabled_labels_count > 0 {
            ui.separator();
            ui.menu_button("Label...", |ui| {
                for entry in self.labels.values().filter(|entry| entry.enabled) {
                    if ui
                        .add(clickable_label_button(
                            &entry.label,
                            entry.color,
                            entry.text_color,
                        ))
                        .clicked()
                    {
                        let action = EditDetails::ToggleLabel(post.id, entry.label.clone());
                        queue.push_back(action.into());
                    }
                }
            });
        }
    }

    fn draw_image(&self, ui: &mut Ui, image_cache: &mut ImageCache, style: &Style, post: &Post) {
        let resp = add_image(
            ui,
            &post.file,
            image_cache,
            style.image.preview_width,
            style.image.radius,
        );

        if post.published.as_bool() {
            add_overlay(
                ui,
                &resp,
                OverlayLocation::TopLeft,
                style.image.overlay.margin,
                |ui| ui.add(checkmark(true, style.copied_mark)),
            );
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
        clipboard: &Clipboard,
    ) {
        ui.horizontal(|ui| {
            self.draw_image(ui, image_cache, style, post);

            ui.vertical(|ui| {
                ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
                    ui.horizontal(|ui| {
                        let label = format!("{}", post.file.rel_path.display());
                        if post.is_dirty() {
                            ui.colored_label(style.modified, RichText::new(label).heading());
                        } else {
                            ui.heading(label);
                        };
                        if button::copy(ui, true) {
                            let path = post.file.full_path.display().to_string();
                            queue.push_back(ClipboardValue::generic(path).into());
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

                    self.show_tags(ui, style, post, queue, clipboard);
                });

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_width(self.label_width);
                        ui.label("species");
                    });

                    self.show_species(ui, post, db, queue, clipboard);
                });

                ui.horizontal(|ui| {
                    ui.horizontal(|ui| {
                        ui.set_min_width(self.label_width);
                        ui.label("labels");
                    });

                    self.show_labels(ui, post);
                });

                if !post.social_media.facebook_url.is_empty() {
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {
                            ui.set_min_width(self.label_width);
                        });

                        Self::link(ui, "Facebook", &post.social_media.facebook_url, queue);
                    });
                }

                if !post.social_media.instagram_url.is_empty() {
                    ui.horizontal(|ui| {
                        ui.horizontal(|ui| {
                            ui.set_min_width(self.label_width);
                        });

                        Self::link(ui, "Instagram", &post.social_media.instagram_url, queue);
                    });
                }
            });
        });
    }

    fn link(ui: &mut Ui, title: &str, url: &str, queue: &mut MessageQueue) {
        ui.horizontal(|ui| {
            ui.label("published on");
            let resp = ui.hyperlink_to(title, url).on_hover_text(url);
            resp.context_menu(|ui| {
                if ui.button(fmt!("{ICON_CONTENT_COPY} Copy URL")).clicked() {
                    queue.push_back(ClipboardValue::generic(url.to_string()).into());
                }
            });
        });
    }

    fn show_tags(
        &self,
        ui: &mut Ui,
        style: &Style,
        post: &Post,
        queue: &mut MessageQueue,
        clipboard: &Clipboard,
    ) {
        ui.horizontal_wrapped(|ui| {
            if button::edit(ui) {
                queue.push_back(Message::EditTags(post.id));
            }

            let empty = post.tags.is_empty();
            if button::copy(ui, !empty) {
                queue.push_back(ClipboardValue::tags(post.tags_string.clone()).into());
            }

            if empty && clipboard.available(ClipboardKind::Tags) {
                let entries = clipboard.get(ClipboardKind::Tags);

                let resp = button::paste(ui, true);
                if resp.clicked() {
                    queue.push_back(Message::PasteTags(post.id, entries.last().unwrap().clone()));
                }

                resp.context_menu(|ui| {
                    for entry in entries.iter().rev() {
                        let button = Button::new(entry).truncate();
                        if ui.add(button).clicked() {
                            queue.push_back(Message::PasteTags(post.id, entry.clone()));
                        }
                    }
                });
            }

            for string in post.tags.iter() {
                ui.add(tag(string, style));
            }
        });
    }

    fn show_species(
        &self,
        ui: &mut Ui,
        post: &Post,
        db: &Database,
        queue: &mut MessageQueue,
        clipboard: &Clipboard,
    ) {
        ui.horizontal(|ui| {
            if ui.button(ICON_EDIT).clicked() {
                queue.push_back(Message::EditSpecies(post.id));
            }
            if button::copy(ui, post.species_id.is_some()) {
                let species_text = String::from(post.species_id.as_ref().unwrap());
                queue.push_back(ClipboardValue::species(species_text).into());
            }
            if let Some(species_id) = &post.species_id {
                let species = db.species_by_id(species_id).unwrap();
                let action = crate::species_view::singleline(ui, species);
                if let Some(action) = action {
                    queue.push_back(action.into());
                }

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

                if ui.button("Edit species").clicked() {
                    queue.push_back(Message::EditSpeciesDetails(species.index));
                }
            } else {
                if clipboard.available(ClipboardKind::Species) {
                    let entries = clipboard.get(ClipboardKind::Species);
                    let resp = button::paste(ui, true).on_hover_ui(|ui| {
                        ui.label(entries.last().unwrap());
                    });
                    if resp.clicked() {
                        queue.push_back(Message::PasteSpecies(
                            post.id,
                            entries.last().unwrap().clone(),
                        ));
                    }

                    if entries.len() > 1 {
                        resp.context_menu(|ui| {
                            for entry in entries.iter().rev() {
                                let button = Button::new(entry).truncate();
                                if ui.add(button).clicked() {
                                    queue.push_back(Message::PasteSpecies(post.id, entry.clone()));
                                }
                            }
                        });
                    }
                } else {
                    ui.label("—");
                }
            }
        });
    }

    fn show_labels(&self, ui: &mut Ui, post: &Post) {
        ui.horizontal_wrapped(|ui| {
            for label in post.labels.iter() {
                if let Some(entry) = self.labels.get(label) {
                    ui.add(label_button(&entry.label, entry.color, entry.text_color));
                } else {
                    ui.label(label);
                }
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
            .id_salt((ID_PREFIX, "scroll-main-grid", self.filter.get_current()))
            .show(ui, |ui| {
                let mut hovered: Option<PostId> = None;
                let width = ui.available_size().x;
                let n = (width / (style.image.preview_width + 8.0)) as isize;
                if n != self.grid_columns {
                    queue.push_back(Message::SetGridColumns(n));
                }

                for (selector, view) in self.view_grouped.iter() {
                    let mut it = view.iter();

                    ui.heading(format_selector(selector, view.len()));

                    let mut empty = false;
                    while !empty {
                        ui.horizontal(|ui| {
                            for _ in 0..n {
                                if let Some(id) = it.next() {
                                    let post = db.post(id);
                                    let fill = self.fill(post, style);

                                    let resp = tight_frame(ui, fill, |ui| {
                                        self.draw_image(ui, image_cache, style, post);
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
        } else if post.published.as_bool() {
            Some(style.published_post)
        } else {
            None
        }
    }
}

fn format_selector(selector: &Selector, count: usize) -> String {
    match selector {
        Selector::ByYear(year) => format!("{year} ({count} images)"),
        Selector::ByMonth(year, month) => format!("{month} {year} ({count} images)"),
        Selector::ByDate(date) => format!(
            "{}-{:02}-{:02} ({count} images)",
            date.year,
            date.month.as_u8(),
            date.day.as_u8()
        ),
    }
}

struct SelectedState {
    id: PostId,
    position: usize,
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
                result = Some(Message::InlineSaveChange {
                    id,
                    field,
                    by_keyboard: true,
                });
            } else if !changed {
                result = Some(Message::InlineDiscardChanges { id, field });
            }
        }

        if ui.button(ICON_CHECK).clicked() {
            result = Some(Message::InlineSaveChange {
                id,
                field,
                by_keyboard: false,
            });
        }
        if ui.button(ICON_CLOSE).clicked() {
            result = Some(Message::InlineDiscardChanges { id, field });
        }
    } else {
        if ui.button(ICON_EDIT).clicked() {
            result = Some(Message::InlineEditStart { id, field });
        }
        if button::copy(ui, !current.is_empty()) {
            match field {
                Field::Polish => {
                    result = Some(ClipboardValue::polish(current.to_owned()).into());
                }

                Field::English => {
                    result = Some(ClipboardValue::english(current.to_owned()).into());
                }
            };
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
        let position = position as isize + direction;

        let max = view.len() - 1;
        let id = position.clamp(0, max as isize);
        let id = id as usize;

        Some(view[id])
    } else {
        Some(view[0])
    }
}

fn mk_grouped_view(view: &[PostId], db: &Database, main: Selector) -> GroupedView {
    let mut res = GroupedView::new();

    if matches!(main, Selector::ByDate(_)) {
        res.push((main, view.to_owned()));
        return res;
    }

    let mut current_selector = Selector::ByYear(0);
    let mut group = Vec::<PostId>::new();
    for id in view {
        let post = db.post(id);
        let selector = match main {
            Selector::ByYear(_) => Selector::ByMonth(post.date.year, post.date.month),
            Selector::ByMonth(_, _) => Selector::ByDate(post.date),
            Selector::ByDate(_) => unreachable!(),
        };

        if selector != current_selector {
            if !group.is_empty() {
                let mut tmp = Vec::<PostId>::new();
                std::mem::swap(&mut tmp, &mut group);
                res.push((current_selector, tmp));
            }

            current_selector = selector;
        }

        group.push(*id);
    }

    if !group.is_empty() {
        res.push((current_selector, group));
    }

    res
}

fn format_shortcut(shortcut: KeyboardShortcut) -> String {
    crate::keyboard::format_shortcut(&shortcut.logical_key, &shortcut.modifiers)
}
