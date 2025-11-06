mod filter;
mod group;
mod modal_publish;
mod modal_species;
mod modal_tags;
mod modal_view;

use egui::Widget;
use filter::Filter;
use filter::ImageCounter;
use group::Group;
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
use crate::db::Post;
use crate::db::PostId;
use crate::edit_details::EditDetails;
use crate::gui::add_image;
use crate::gui::add_image_with_tint;
use crate::gui::add_overlay;
use crate::gui::button;
use crate::gui::frame;
use crate::gui::icon_en;
use crate::gui::icon_pl;
use crate::gui::tag;
use crate::gui::widget_size;
use crate::gui::OverlayLocation;
use crate::keyboard::KeyboardMapping;
use crate::style::Style;
use const_format::formatcp as fmt;
use eframe::emath::OrderedFloat;
use egui::Align;
use egui::Button;
use egui::CentralPanel;
use egui::Context;
use egui::Grid;
use egui::Id;
use egui::Key;
use egui::Label;
use egui::Layout;
use egui::RichText;
use egui::ScrollArea;
use egui::Sense;
use egui::SidePanel;
use egui::SizeHint;
use egui::TextEdit;
use egui::TopBottomPanel;
use egui::Ui;
use egui::Vec2;
use std::cell::LazyCell;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_ADD;
use egui_material_icons::icons::ICON_CHECK;
use egui_material_icons::icons::ICON_CLOSE;
use egui_material_icons::icons::ICON_DELETE;
use egui_material_icons::icons::ICON_EDIT;
use egui_material_icons::icons::ICON_UNDO;

const ID_PREFIX: &str = "tab-posts";

pub struct TabPosts {
    version: u64,
    view: Vec<PostId>,
    hovered: Option<PostId>,
    filter: Filter,
    inline_editors: BTreeMap<(PostId, Field), InlineEditor>,
    group: Option<Group>,
    scroll_delta: f32,
    scroll_item: f32,
    scroll_page: f32,
    scroll_everything: f32,
    modal_window: ModalWindow,

    overlay_size: HashMap<Overlay, Vec2>,
    keyboard_mapping: LazyCell<KeyboardMapping>,

    pub queue: MessageQueue,
}

pub enum ModalWindow {
    None,
    ModalTags(Box<ModalTags>),
    ModalSpecies(Box<ModalSpecies>),
    ModalPublish(Box<ModalPublish>),
    ModalView(Box<ModalView>),
}

impl ModalWindow {
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Overlay {
    Label(String),
    Button(String),
}

struct InlineEditor {
    pub text: String,
    pub id: Id,
}

pub type MessageQueue = VecDeque<Message>;

#[derive(Clone)]
pub enum Message {
    FocusItem(Id),
    EditTags(PostId),
    EditSpecies(PostId),
    View(PostId),
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
    OverlaySize(Overlay, Vec2),

    OpenModalPublish(PostId),
    OpenModalView(PostId),
    OpenModalTags(PostId),
    OpenModalSpecies(PostId),
    ModalTags(ModalTagsMessage),
    ModalSpecies(ModalSpeciesMessage),
    ModalView(ModalViewMessage),
    ModalPublish(ModalPublishMessage),
    CloseModal,
    Confirm(Confirm),

    Copy(String),

    RefreshView,
    RequestImage(PostId),
    Hovered(Option<PostId>),
    UpdateScrollAmounts {
        scroll_item: f32,
        scroll_page: f32,
        scroll_everything: f32,
    },
    StartGroupingCurrent,
    PublishCurrent,
    EditTagsCurrent,
    EditSpeciesCurrent,
    ViewCurrent,
    ScrollDown,
    ScrollUp,
    ScrollPageDown,
    ScrollPageUp,
    ScrollManyPagesDown,
    ScrollManyPagesUp,
    ScrollHome,
    ScrollEnd,
    Undo,
    FocusSearch,
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::EditTags(_) => unreachable!(),
            Self::EditSpecies(_) => unreachable!(),
            Self::View(_) => unreachable!(),
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
            Self::OverlaySize(_, _) => unreachable!(),
            Self::OpenModalPublish(_) => unreachable!(),
            Self::OpenModalView(_) => unreachable!(),
            Self::OpenModalTags(_) => unreachable!(),
            Self::OpenModalSpecies(_) => unreachable!(),
            Self::ModalTags(msg) => msg.name(),
            Self::ModalSpecies(msg) => msg.name(),
            Self::ModalView(msg) => msg.name(),
            Self::ModalPublish(msg) => msg.name(),
            Self::CloseModal => unreachable!(),
            Self::Confirm(_) => unreachable!(),
            Self::Copy(_) => unreachable!(),
            Self::RefreshView => unreachable!(),
            Self::RequestImage(_) => unreachable!(),
            Self::Hovered(_) => unreachable!(),
            Self::UpdateScrollAmounts { .. } => unreachable!(),
            Self::StartGroupingCurrent => "start grouping photos in the highlighted post",
            Self::PublishCurrent => "publish the highlighted post",
            Self::EditTagsCurrent => "edit tags of the highlighted post",
            Self::EditSpeciesCurrent => "edit view of the highlighted post",
            Self::ViewCurrent => "fullscreen view of photos from the highlighted post",
            Self::ScrollDown => "slightly scroll list down",
            Self::ScrollUp => "slightly scroll list up",
            Self::ScrollPageDown => "scroll list down",
            Self::ScrollPageUp => "scroll list down",
            Self::ScrollManyPagesDown => "quick scroll list down",
            Self::ScrollManyPagesUp => "quick scroll list up",
            Self::ScrollHome => "scroll to the beginning",
            Self::ScrollEnd => "scroll to the end",
            Self::Undo => "undo changes",
            Self::FocusSearch => "focus search bar",
            Self::FocusItem(_) => unreachable!(),
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

#[derive(Clone, Copy, Ord, PartialOrd, Eq, PartialEq, Debug)]
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
            version: 0,
            filter: Filter::default(),
            queue,
            inline_editors: BTreeMap::new(),
            scroll_delta: 0.0,
            scroll_item: 100.0,
            scroll_page: 500.0,
            scroll_everything: 0.0,
            modal_window: ModalWindow::None,
            group: None,
            overlay_size: HashMap::new(),
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        }
    }
}

impl TabPosts {
    pub fn load(&mut self, db_id: &str, storage: &dyn eframe::Storage) {
        self.filter.load(db_id, storage);
    }

    pub fn save(&self, db_id: &str, storage: &mut dyn eframe::Storage) {
        self.filter.save(db_id, storage);
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        style: &Style,
        db: &mut Database,
        main_queue: &mut MainMessageQueue,
    ) {
        db.refresh_caches();
        if self.version != db.version {
            self.queue.push_back(Message::RefreshView);
            self.version = db.version;
        }

        let mut queue = MessageQueue::new();

        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(ctx, db, msg, &mut queue, main_queue);
        }

        match &mut self.modal_window {
            ModalWindow::None => {
                self.draw(ctx, style, db, &mut queue);
            }
            ModalWindow::ModalTags(window) => {
                window.update(ctx, style, db, &mut queue);
            }
            ModalWindow::ModalSpecies(window) => {
                window.update(ctx, style, db, &mut queue);
            }
            ModalWindow::ModalPublish(window) => {
                window.update(ctx, style, db, &mut queue);
            }
            ModalWindow::ModalView(window) => {
                window.update(ctx, db, &mut queue);
            }
        }

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }

        self.scroll_delta = 0.0;
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
            Message::RequestImage(id) => {
                let post = db.post_mut(&id);
                for uri in &post.uris {
                    let _ = ctx.try_load_image(uri, SizeHint::Scale(OrderedFloat(1.0)));
                }

                post.loaded = true;
            }
            Message::EditTags(id) => {
                queue.push_back(Message::OpenModalTags(id));
            }
            Message::EditSpecies(id) => {
                queue.push_back(Message::OpenModalSpecies(id));
            }
            Message::InlineEditStart { id, field } => {
                let post = db.post_mut(&id);
                let key = (post.id, field);
                let id = Id::new(format!("inline-editor-{id:?}-{field:?}"));

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
            Message::View(id) => {
                queue.push_back(Message::OpenModalView(id));
            }
            Message::Publish(id) => {
                queue.push_back(Message::OpenModalPublish(id));
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
            Message::OverlaySize(overlay, size) => {
                self.overlay_size.insert(overlay, size);
            }
            Message::UpdateScrollAmounts {
                scroll_item,
                scroll_page,
                scroll_everything,
            } => {
                self.scroll_item = scroll_item;
                self.scroll_page = scroll_page;
                self.scroll_everything = scroll_everything;
            }
            Message::OpenModalTags(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalTags::new(id, db);
                self.modal_window = ModalWindow::ModalTags(Box::new(window));
            }
            Message::OpenModalSpecies(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalSpecies::new(id, db);
                self.modal_window = ModalWindow::ModalSpecies(Box::new(window));
            }
            Message::OpenModalPublish(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalPublish::new(id, db);
                self.modal_window = ModalWindow::ModalPublish(Box::new(window));
            }
            Message::OpenModalView(id) => {
                assert!(self.modal_window.is_none());
                let window = ModalView::new(&id, db);
                self.modal_window = ModalWindow::ModalView(Box::new(window));
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
                    self.queue.push_back(Message::Publish(id));
                }
            }
            Message::Undo => {
                if let Some(id) = self.hovered {
                    self.queue.push_back(EditDetails::Undo(id).into());
                }
            }
            Message::EditTagsCurrent => {
                if let Some(id) = self.hovered {
                    self.queue.push_back(Message::EditTags(id));
                }
            }
            Message::EditSpeciesCurrent => {
                if let Some(id) = self.hovered {
                    self.queue.push_back(Message::EditSpecies(id));
                }
            }
            Message::StartGroupingCurrent => {
                if let Some(id) = self.hovered {
                    self.queue.push_back(Message::StartGrouping(id));
                }
            }
            Message::ViewCurrent => {
                if let Some(id) = self.hovered {
                    self.queue.push_back(Message::View(id));
                }
            }
            Message::ScrollDown => {
                self.scroll_delta = -self.scroll_item;
            }
            Message::ScrollUp => {
                self.scroll_delta = self.scroll_item;
            }
            Message::ScrollPageDown => {
                self.scroll_delta = -self.scroll_page;
            }
            Message::ScrollPageUp => {
                self.scroll_delta = self.scroll_page;
            }
            Message::ScrollManyPagesDown => {
                self.scroll_delta = -5.0 * self.scroll_page;
            }
            Message::ScrollManyPagesUp => {
                self.scroll_delta = 5.0 * self.scroll_page;
            }
            Message::ScrollHome => {
                self.scroll_delta = self.scroll_everything;
            }
            Message::ScrollEnd => {
                self.scroll_delta = -self.scroll_everything;
            }
            Message::FocusItem(id) => {
                ctx.memory_mut(|mem| mem.request_focus(id));
            }
        }
    }

    fn create_mapping() -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(msg)
        }

        KeyboardMapping::default()
            .key(Key::Slash, msg(Message::FocusSearch))
            .ctrl(Key::P, msg(Message::PublishCurrent))
            .key(Key::P, msg(Message::PublishCurrent))
            .ctrl(Key::Z, msg(Message::Undo))
            .ctrl(Key::T, msg(Message::EditTagsCurrent))
            .key(Key::T, msg(Message::EditTagsCurrent))
            .ctrl(Key::S, msg(Message::EditSpeciesCurrent))
            .key(Key::S, msg(Message::EditSpeciesCurrent))
            .ctrl(Key::G, msg(Message::StartGroupingCurrent))
            .key(Key::F, msg(Message::ViewCurrent))
            .key(Key::V, msg(Message::ViewCurrent))
            .key(Key::Space, msg(Message::ViewCurrent))
            .key(Key::ArrowDown, msg(Message::ScrollDown))
            .key(Key::ArrowUp, msg(Message::ScrollUp))
            .ctrl(Key::ArrowDown, msg(Message::ScrollPageDown))
            .ctrl(Key::ArrowUp, msg(Message::ScrollPageUp))
            .key(Key::PageDown, msg(Message::ScrollPageDown))
            .key(Key::PageUp, msg(Message::ScrollPageUp))
            .ctrl(Key::PageDown, msg(Message::ScrollManyPagesDown))
            .ctrl(Key::PageUp, msg(Message::ScrollManyPagesUp))
            .key(Key::Home, msg(Message::ScrollHome))
            .key(Key::End, msg(Message::ScrollEnd))
    }

    pub fn get_keyboard_mapping(&self) -> &KeyboardMapping {
        match &self.modal_window {
            ModalWindow::ModalSpecies(window) => &window.keyboard_mapping,
            ModalWindow::ModalTags(window) => &window.keyboard_mapping,
            ModalWindow::ModalPublish(window) => &window.keyboard_mapping,
            ModalWindow::ModalView(window) => &window.keyboard_mapping,
            ModalWindow::None => &self.keyboard_mapping,
        }
    }

    fn draw(&mut self, ctx: &Context, style: &Style, db: &Database, queue: &mut MessageQueue) {
        TopBottomPanel::top(fmt!("{ID_PREFIX}-filter")).show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.filter.view(ui, db, queue);
            });
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
                            self.show_group(ui, style, db, queue);
                        });
                });
        }

        CentralPanel::default().show(ctx, |ui| {
            self.draw_main_list(ui, style, db, queue);
        });
    }

    fn draw_main_list(&self, ui: &mut Ui, style: &Style, db: &Database, queue: &mut MessageQueue) {
        let mut count = 0;
        let resp = ScrollArea::both()
            .id_salt(fmt!("{ID_PREFIX}-scroll-main"))
            .show(ui, |ui| {
                ui.scroll_with_delta(Vec2::new(0.0, self.scroll_delta));

                let mut hovered: Option<PostId> = None;
                for id in self.view.iter() {
                    if let Some(group) = &self.group {
                        if group.contains(id) {
                            continue;
                        }
                    }

                    let post = db.post(id);
                    if self.draw_post(ui, style, post, db, queue) {
                        hovered = Some(*id);
                    }

                    count += 1;
                }

                if hovered != self.hovered {
                    queue.push_back(Message::Hovered(hovered));
                }
            });

        let scroll_page = resp.inner_rect.height();
        let scroll_everything = resp.content_size.y;
        let scroll_item = if count > 0 {
            scroll_everything / count as f32
        } else {
            0.0
        };

        if scroll_item != self.scroll_item
            || scroll_page != self.scroll_page
            || scroll_everything != self.scroll_everything
        {
            queue.push_back(Message::UpdateScrollAmounts {
                scroll_item,
                scroll_page,
                scroll_everything,
            });
        }
    }

    fn draw_post(
        &self,
        ui: &mut Ui,
        style: &Style,
        post: &Post,
        db: &Database,
        queue: &mut MessageQueue,
    ) -> bool {
        let fill = if self.hovered == Some(post.id) {
            Some(style.hovered_frame)
        } else if post.published {
            Some(style.published_frame)
        } else {
            None
        };

        let resp = frame(ui, fill, |ui| {
            self.draw_post_inner(ui, style, post, db, queue);
        });

        if resp.double_clicked() {
            queue.push_back(Message::View(post.id));
        }

        resp.contains_pointer()
    }

    fn draw_post_inner(
        &self,
        ui: &mut Ui,
        style: &Style,
        post: &Post,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        ui.horizontal(|ui| {
            let uri = if post.loaded {
                post.uris[0].clone()
            } else {
                "file:///dev/null".to_owned()
            };

            let resp = add_image(ui, uri, style.image.preview_width, style.image.radius);
            if !post.loaded {
                let intersect = ui.clip_rect().intersect(resp.rect);
                if intersect.is_positive() {
                    queue.push_back(Message::RequestImage(post.id));
                }
            }

            let n = post.files.len();
            if n > 1 {
                let count = ImageCounter(n);
                let label = count.to_string();
                let widget = image_count(&label, style);
                let overlay_id = Overlay::Label(label);

                if let Some(size) = self.overlay_size.get(&overlay_id) {
                    add_overlay(
                        ui,
                        resp.clone(),
                        OverlayLocation::BottomRight,
                        *size,
                        style.image.overlay.margin,
                        widget,
                    );
                } else {
                    let size = widget_size(ui, widget);
                    queue.push_back(Message::OverlaySize(overlay_id, size));
                }
            }

            if self.group.is_some() {
                let label = fmt!("{ICON_ADD} Add to group");
                let overlay_id = Overlay::Button(label.to_owned());
                let button = Button::new(label).fill(style.button.save);

                if let Some(size) = self.overlay_size.get(&overlay_id) {
                    let resp = add_overlay(
                        ui,
                        resp,
                        OverlayLocation::TopLeft,
                        *size,
                        style.image.overlay.margin,
                        button,
                    );

                    if resp.clicked() {
                        queue.push_back(Message::AddToGroup(post.id));
                    }
                } else {
                    let size = widget_size(ui, button);
                    queue.push_back(Message::OverlaySize(overlay_id, size));
                }
            }

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

                Grid::new(("image-details", post.id))
                    .num_columns(2)
                    .show(ui, |ui| {
                        let inline_editor = self.inline_editors.get(&(post.id, Field::Polish));

                        icon_pl(ui);

                        ui.horizontal(|ui| {
                            let msg =
                                inline_edit(ui, post.id, &post.pl, Field::Polish, inline_editor);
                            if let Some(msg) = msg {
                                queue.push_back(msg);
                            }
                        });
                        ui.end_row();

                        let inline_editor = self.inline_editors.get(&(post.id, Field::English));

                        icon_en(ui);

                        ui.horizontal(|ui| {
                            let msg =
                                inline_edit(ui, post.id, &post.en, Field::English, inline_editor);
                            if let Some(msg) = msg {
                                queue.push_back(msg);
                            }
                        });
                        ui.end_row();

                        ui.label("tags");
                        self.show_tags(ui, style, post, queue);
                        ui.end_row();

                        ui.label("species");
                        self.show_species(ui, post, db, queue);
                        ui.end_row();
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

    fn show_group(&self, ui: &mut Ui, style: &Style, db: &Database, queue: &mut MessageQueue) {
        let Some(group) = &self.group else {
            return;
        };

        for (entry_id, id) in group.iter().enumerate() {
            let post = db.post(id);
            for (photo_id, uri) in post.uris.iter().enumerate() {
                if entry_id == 0 && photo_id == 0 {
                    add_image(
                        ui,
                        uri.clone(),
                        style.image.preview_width,
                        style.image.radius,
                    );
                } else if photo_id > 0 {
                    add_image_with_tint(
                        ui,
                        uri.clone(),
                        style.image.preview_width,
                        style.image.radius,
                        style.image.inactive,
                    );
                } else {
                    let resp = add_image(
                        ui,
                        uri.clone(),
                        style.image.preview_width,
                        style.image.radius,
                    );

                    let label = fmt!("{ICON_DELETE} Remove from group");
                    let button = Button::new(label).fill(style.button.remove);
                    let overlay_id = Overlay::Label(label.to_owned());

                    if let Some(size) = self.overlay_size.get(&overlay_id) {
                        let resp = add_overlay(
                            ui,
                            resp,
                            OverlayLocation::BottomRight,
                            *size,
                            style.image.overlay.margin,
                            button,
                        );
                        if resp.clicked() {
                            queue.push_back(Message::RemoveFromGroup(*id));
                        }
                    } else {
                        let size = widget_size(ui, button);
                        queue.push_back(Message::OverlaySize(overlay_id, size));
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

        let label = Label::new(current).selectable(false).sense(Sense::CLICK);
        if ui.add(label).clicked() {
            result = Some(Message::InlineEditStart { id, field });
        }
    }

    result
}

fn image_count(label: &String, style: &Style) -> impl Widget {
    use crate::widgets::Label;

    let mut widget = Label::new(label);
    widget.padding = 3.0;
    widget.rounding = 5.0;
    widget.color = style.image.overlay.fg;
    widget.background = style.image.overlay.bg;

    widget
}
