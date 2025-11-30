mod modal_edit;

use modal_edit::Message as ModalEditMessage;
use modal_edit::ModalEdit;

use crate::application::Message as MainMessage;
use crate::application::MessageQueue as MainMessageQueue;
use crate::confirm::Confirm;
use crate::db::Database;
use crate::db::SpeciesId;
use crate::help;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::search_box::SearchBox;
use crate::species_view::SortOrder;
use crate::species_view::SpeciesList;
use crate::species_view::SpeciesViewAction;
use crate::style::Style;
use const_format::formatcp as fmt;
use egui::CentralPanel;
use egui::Context;
use egui::Key;
use egui::ScrollArea;
use egui::Ui;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_ARROW_DOWNWARD;
use egui_material_icons::icons::ICON_ARROW_UPWARD;
use egui_material_icons::icons::ICON_FORMAT_LIST_NUMBERED;
use egui_material_icons::icons::ICON_SORT_BY_ALPHA;

pub struct TabSpecies {
    list: SpeciesList,
    search_box: SearchBox,
    modal_window: ModalWindow,

    pub queue: MessageQueue,
    pub keyboard_mapping: KeyboardMapping,
}

pub type MessageQueue = VecDeque<Message>;

pub enum ModalWindow {
    None,
    ModalEdit(Box<ModalEdit>),
}

impl ModalWindow {
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone)]
pub enum Message {
    ModalEdit(ModalEditMessage),
    OpenModalEdit(SpeciesId),
    AddNew,
    EditCurrent,
    Edit(SpeciesId),
    FilterByName(String),
    SortBy(SortOrder),
    RefreshView,
    Hovered(Option<SpeciesId>),
    Confirm(Confirm),
    CloseModal,
    FocusSearch,
    SpeciesViewAction(SpeciesViewAction, SpeciesId),
    SelectPrevExample,
    SelectNextExample,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::ModalEdit(msg) => msg.name(),
            Self::OpenModalEdit(_) => unreachable!(),
            Self::AddNew => "add new species",
            Self::EditCurrent => "edit highlighted species",
            Self::Edit(_) => unreachable!(),
            Self::FilterByName(_) => unreachable!(),
            Self::SortBy(_) => unreachable!(),
            Self::RefreshView => unreachable!(),
            Self::Hovered(_) => unreachable!(),
            Self::Confirm(_) => unreachable!(),
            Self::CloseModal => unreachable!(),
            Self::FocusSearch => help::FOCUS_SEARCH,
            Self::SpeciesViewAction(..) => unreachable!(),
            Self::SelectPrevExample => "select the previous example from the list",
            Self::SelectNextExample => "select the next example from the list",
        }
    }
}

impl From<Message> for MainMessage {
    fn from(val: Message) -> Self {
        Self::TabSpecies(val)
    }
}

impl Default for TabSpecies {
    fn default() -> Self {
        let mut res = Self {
            list: SpeciesList::default().with_width(300.0),
            queue: MessageQueue::new(),
            search_box: SearchBox::new("tab-species-search"),
            keyboard_mapping: Self::create_mapping(),
            modal_window: ModalWindow::None,
        };

        res.queue.push_back(Message::RefreshView);

        res
    }
}

impl TabSpecies {
    pub fn update(
        &mut self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &mut Database,
        main_queue: &mut MainMessageQueue,
    ) {
        self.list.image_width = style.image.preview_width;

        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(ctx, db, msg, main_queue);
        }

        let mut queue = MessageQueue::new();
        match &mut self.modal_window {
            ModalWindow::None => self.draw(ctx, image_cache, style, db, &mut queue),
            ModalWindow::ModalEdit(window) => {
                window.update(ctx, image_cache, style, db, &mut queue);
            }
        }

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }
    }

    fn create_mapping() -> KeyboardMapping {
        KeyboardMapping::default()
            .key(Key::Slash, Message::FocusSearch.into())
            .ctrl(Key::Slash, Message::FocusSearch.into())
            .ctrl(Key::F, Message::FocusSearch.into())
            .key(Key::E, Message::EditCurrent.into())
            .ctrl(Key::E, Message::EditCurrent.into())
            .ctrl(Key::N, Message::AddNew.into())
            .key(Key::ArrowRight, Message::SelectPrevExample.into())
            .key(Key::ArrowLeft, Message::SelectNextExample.into())
    }

    pub fn modal_opened(&self) -> bool {
        !matches!(self.modal_window, ModalWindow::None)
    }

    pub fn try_close_modal(&mut self) {
        match &mut self.modal_window {
            ModalWindow::None => (),
            ModalWindow::ModalEdit(window) => window.try_close(),
        }
    }

    pub fn get_keyboard_mapping(&self) -> &KeyboardMapping {
        match &self.modal_window {
            ModalWindow::None => &self.keyboard_mapping,
            ModalWindow::ModalEdit(window) => &window.keyboard_mapping,
        }
    }

    fn handle_message(
        &mut self,
        ctx: &Context,
        db: &mut Database,
        message: Message,
        main_queue: &mut MainMessageQueue,
    ) {
        match message {
            Message::RefreshView => {
                self.list.refresh_view(db);
            }
            Message::FilterByName(phrase) => {
                self.list.set_filter(phrase, db);
            }
            Message::SortBy(sort_order) => {
                self.list.set_sort_order(sort_order, db);
            }
            Message::AddNew => {
                assert!(self.modal_window.is_none());
                let window = ModalEdit::new();
                self.modal_window = ModalWindow::ModalEdit(Box::new(window));
            }
            Message::EditCurrent => {
                if let Some(id) = self.list.hovered {
                    self.queue.push_back(Message::OpenModalEdit(id));
                }
            }
            Message::Edit(id) => {
                self.queue.push_back(Message::OpenModalEdit(id));
            }
            Message::Hovered(hovered) => {
                self.list.hovered = hovered;
            }
            Message::ModalEdit(msg) => {
                if let ModalWindow::ModalEdit(window) = &mut self.modal_window {
                    window.queue.push_back(msg);
                }
            }
            Message::OpenModalEdit(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalEdit::edit(id, db);
                self.modal_window = ModalWindow::ModalEdit(Box::new(window));
            }
            Message::CloseModal => {
                assert!(!self.modal_window.is_none());
                self.modal_window = ModalWindow::None;
            }
            Message::Confirm(confirm) => {
                main_queue.push_back(MainMessage::Confirm(confirm));
            }
            Message::FocusSearch => {
                self.search_box.take_focus(ctx);
            }
            Message::SpeciesViewAction(action, id) => {
                if let Some(species) = db.species_mut_by_id(&id) {
                    match action {
                        SpeciesViewAction::SelectNext => species.next_example(),
                        SpeciesViewAction::SelectPrev => species.prev_example(),
                    }
                }
            }
            Message::SelectPrevExample => {
                if let Some(id) = self.list.hovered.as_ref() {
                    if let Some(species) = db.species_mut_by_id(id) {
                        species.prev_example();
                    }
                }
            }
            Message::SelectNextExample => {
                if let Some(id) = self.list.hovered.as_ref() {
                    if let Some(species) = db.species_mut_by_id(id) {
                        species.next_example();
                    }
                }
            }
        }
    }

    pub fn draw(
        &self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        CentralPanel::default().show(ctx, |ui| {
            self.draw_main(ui, image_cache, style, db, queue);
        });
    }

    pub fn draw_main(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        ui.horizontal(|ui| {
            if let Some(filter) = self.search_box.show(ui) {
                queue.push_back(Message::FilterByName(filter));
            }

            ui.separator();

            if ui.button("➕AddNew new").clicked() {
                queue.push_back(Message::AddNew);
            }

            ui.separator();

            if ui.button(sort_order_label(&self.list.sort_order)).clicked() {
                queue.push_back(Message::SortBy(next_sort_order(&self.list.sort_order)));
            }
        });

        ui.separator();

        ScrollArea::vertical()
            .auto_shrink(false)
            .id_salt("tab-species-scroll")
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let resp = self.list.render(ui, image_cache, style, db);
                    if let Some(id) = resp.hovered {
                        queue.push_back(Message::Hovered(id));
                    }
                    if let Some(id) = resp.double_clicked {
                        queue.push_back(Message::Edit(id));
                    }
                    if let Some((action, id)) = resp.species_view_action {
                        queue.push_back(Message::SpeciesViewAction(action, id));
                    }
                });
            });
    }
}

const fn next_sort_order(sort_order: &SortOrder) -> SortOrder {
    match sort_order {
        SortOrder::DateAddedAsc => SortOrder::DateAddedDesc,
        SortOrder::DateAddedDesc => SortOrder::LatinAsc,
        SortOrder::LatinAsc => SortOrder::LatinDesc,
        SortOrder::LatinDesc => SortOrder::PolishAsc,
        SortOrder::PolishAsc => SortOrder::PolishDesc,
        SortOrder::PolishDesc => SortOrder::EnglishAsc,
        SortOrder::EnglishAsc => SortOrder::EnglishDesc,
        SortOrder::EnglishDesc => SortOrder::DateAddedAsc,
    }
}

const fn sort_order_label(sort_order: &SortOrder) -> &str {
    match sort_order {
        SortOrder::DateAddedAsc => {
            fmt!("{ICON_FORMAT_LIST_NUMBERED}{ICON_ARROW_DOWNWARD} add date")
        }
        SortOrder::DateAddedDesc => fmt!("{ICON_FORMAT_LIST_NUMBERED}{ICON_ARROW_UPWARD} add date"),
        SortOrder::LatinAsc => fmt!("{ICON_SORT_BY_ALPHA}{ICON_ARROW_DOWNWARD} Latin"),
        SortOrder::LatinDesc => fmt!("{ICON_SORT_BY_ALPHA}{ICON_ARROW_UPWARD} Latin"),
        SortOrder::PolishAsc => fmt!("{ICON_SORT_BY_ALPHA}{ICON_ARROW_DOWNWARD} Polish"),
        SortOrder::PolishDesc => fmt!("{ICON_SORT_BY_ALPHA}{ICON_ARROW_UPWARD} Polish"),
        SortOrder::EnglishAsc => fmt!("{ICON_SORT_BY_ALPHA}{ICON_ARROW_DOWNWARD} English"),
        SortOrder::EnglishDesc => fmt!("{ICON_SORT_BY_ALPHA}{ICON_ARROW_UPWARD} English"),
    }
}
