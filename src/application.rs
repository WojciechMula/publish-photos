use crate::confirm::Confirm;
use crate::confirm::ConfirmOption;
use crate::db::Database;
use crate::db::PostId;
use crate::edit_details::EditDetails;
use crate::graphapi::manager::SocialMediaPublisher;
use crate::graphapi::GraphApiCredentials;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::modal::ModalWindowTrait;
use crate::modal_errors::ModalErrors;
use crate::modal_keyboard::ModalKeyboard;
use crate::modal_settings::ModalSettings;
use crate::style::Style;
use crate::tab_posts::Message as TabPostsMessage;
use crate::tab_posts::TabPosts;
use crate::tab_species::Message as TabSpeciesMessage;
use crate::tab_species::TabSpecies;
use crate::tab_tag_groups::Message as TabTagGroupsMessage;
use crate::tab_tag_groups::TabTagGroups;
use crate::tab_tag_translations::Message as TabTagTranslationsMessage;
use crate::tab_tag_translations::TabTagTranslations;
use const_format::formatcp as fmt;
use eframe::egui::Context;
use egui::style::ScrollAnimation;
use egui::Align;
use egui::Button;
use egui::Id;
use egui::Key;
use egui::Layout;
use egui::Modal;
use egui::Sense;
use egui::TopBottomPanel;
use egui::ViewportCommand;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_HELP;
use egui_material_icons::icons::ICON_SETTINGS;
use egui_material_icons::icons::ICON_WARNING;

pub struct Application {
    db: Database,
    active_tab: Tab,

    species: TabSpecies,
    posts: TabPosts,
    tag_translations: TabTagTranslations,
    tag_groups: TabTagGroups,

    modal_window: Vec<Box<dyn ModalWindowTrait>>,
    can_close: bool,
    style: Style,

    queue: MessageQueue,
    initialized: bool,
    image_cache: ImageCache,
    sm_manager: Option<SocialMediaPublisher>,

    keyboard_mapping: KeyboardMapping,
}

pub type MessageQueue = VecDeque<Message>;

pub enum Message {
    TabPosts(TabPostsMessage),
    TabSpecies(TabSpeciesMessage),
    TabTagGroups(TabTagGroupsMessage),
    TabTagTranslations(TabTagTranslationsMessage),
    OpenModal(Box<dyn ModalWindowTrait>),
    EditDetails(EditDetails),
    StartPublishing(PostId),
    Copy(String),
    CloseModal,
    SaveDatabase,
    SetStyle(Style),
    Confirm(Confirm),
    ConfirmResult(Option<Box<Message>>),
    SoftClose,
    AllowClose,
    MaximizeWindow,
    SelectTabPosts,
    SelectTabSpecies,
    SelectTabTagTranslations,
    SelectTabTagGroup,
    SelectNextTab,
    SelectPrevTab,
    OpenHelp,
}

impl Message {
    pub const fn name(&self) -> &str {
        match &self {
            Self::TabPosts(msg) => msg.name(),
            Self::TabSpecies(msg) => msg.name(),
            Self::TabTagGroups(msg) => msg.name(),
            Self::TabTagTranslations(msg) => msg.name(),
            Self::OpenModal(_) => unreachable!(),
            Self::EditDetails(_) => unreachable!(),
            Self::StartPublishing(_) => unreachable!(),
            Self::Copy(_) => unreachable!(),
            Self::CloseModal => unreachable!(),
            Self::SaveDatabase => "save database",
            Self::SetStyle(_) => unreachable!(),
            Self::Confirm(_) => unreachable!(),
            Self::SoftClose => unreachable!(),
            Self::AllowClose => unreachable!(),
            Self::MaximizeWindow => unreachable!(),
            Self::SelectTabPosts => "select tab posts",
            Self::SelectTabSpecies => "select tab species",
            Self::SelectTabTagTranslations => "select tab tag translations",
            Self::SelectTabTagGroup => "select tab tag groups",
            Self::OpenHelp => "keyboard shortcuts help",
            Self::SelectNextTab => "select next tab",
            Self::SelectPrevTab => "select previous tab",
            Self::ConfirmResult(_) => unreachable!(),
        }
    }
}

impl Clone for Message {
    fn clone(&self) -> Self {
        match self {
            Self::TabPosts(val) => Self::TabPosts(val.clone()),
            Self::TabSpecies(val) => Self::TabSpecies(val.clone()),
            Self::TabTagGroups(val) => Self::TabTagGroups(val.clone()),
            Self::TabTagTranslations(val) => Self::TabTagTranslations(val.clone()),
            Self::OpenModal(_) => unreachable!(),
            Self::EditDetails(val) => Self::EditDetails(val.clone()),
            Self::StartPublishing(val) => Self::StartPublishing(*val),
            Self::Copy(val) => Self::Copy(val.clone()),
            Self::CloseModal => Self::CloseModal,
            Self::SaveDatabase => Self::SaveDatabase,
            Self::SetStyle(val) => Self::SetStyle(val.clone()),
            Self::Confirm(val) => Self::Confirm(val.clone()),
            Self::SoftClose => Self::SoftClose,
            Self::AllowClose => Self::AllowClose,
            Self::MaximizeWindow => Self::MaximizeWindow,
            Self::SelectTabPosts => Self::SelectTabPosts,
            Self::SelectTabSpecies => Self::SelectTabSpecies,
            Self::SelectTabTagTranslations => Self::SelectTabTagTranslations,
            Self::SelectTabTagGroup => Self::SelectTabTagGroup,
            Self::OpenHelp => Self::OpenHelp,
            Self::SelectNextTab => Self::SelectNextTab,
            Self::SelectPrevTab => Self::SelectPrevTab,
            Self::ConfirmResult(val) => {
                Self::ConfirmResult(val.as_ref().map(|boxed| Box::new(*boxed.clone())))
            }
        }
    }
}

impl From<EditDetails> for Message {
    fn from(act: EditDetails) -> Self {
        Message::EditDetails(act)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tab {
    Posts,
    Species,
    TagTranslations,
    TagGroups,
}

impl Tab {
    const fn name(&self) -> &str {
        match self {
            Self::Posts => "Posts",
            Self::Species => "Species",
            Self::TagTranslations => "Tag translations",
            Self::TagGroups => "Tag groups",
        }
    }

    const fn next(&self) -> Self {
        match self {
            Self::Posts => Self::Species,
            Self::Species => Self::TagTranslations,
            Self::TagTranslations => Self::TagGroups,
            Self::TagGroups => Self::Posts,
        }
    }

    const fn prev(&self) -> Self {
        match self {
            Self::Posts => Self::TagGroups,
            Self::Species => Self::Posts,
            Self::TagTranslations => Self::Species,
            Self::TagGroups => Self::TagTranslations,
        }
    }
}

impl Application {
    pub fn new(db: Database, credentials: Option<GraphApiCredentials>) -> Self {
        let mut queue = MessageQueue::new();
        queue.push_back(Message::MaximizeWindow);

        let sm_manager = credentials.map(SocialMediaPublisher::new);

        Self {
            db,
            active_tab: Tab::Posts,
            modal_window: Vec::new(),
            species: TabSpecies::default(),
            posts: TabPosts::new(),
            tag_translations: TabTagTranslations::default(),
            tag_groups: TabTagGroups::default(),
            initialized: false,
            image_cache: ImageCache::default(),
            style: Style::default(),
            queue,
            can_close: false,
            keyboard_mapping: Self::create_mapping(),
            sm_manager,
        }
    }

    fn create_mapping() -> KeyboardMapping {
        KeyboardMapping::default()
            .key(Key::F1, Message::OpenHelp)
            .key(Key::F2, Message::SelectTabPosts)
            .key(Key::F3, Message::SelectTabSpecies)
            .key(Key::F4, Message::SelectTabTagTranslations)
            .key(Key::F5, Message::SelectTabTagGroup)
            .ctrl(Key::S, Message::SaveDatabase)
            .ctrl(Key::ArrowRight, Message::SelectNextTab)
            .ctrl(Key::ArrowLeft, Message::SelectPrevTab)
    }

    fn keyboard(&mut self, ctx: &Context) {
        // 1. top-level modal window
        if let Some(window) = self.modal_window.last_mut() {
            if let Some(msg) = keyboard_action(ctx, window.keyboard_mapping()) {
                self.queue.push_back(msg);
            }
            return;
        }

        // 2. main winow
        if let Some(msg) = keyboard_action(ctx, &self.keyboard_mapping) {
            self.queue.push_back(msg);
            return;
        }

        // 3. active tab/modal inside the tab
        let keyboard_mapping = match self.active_tab {
            Tab::Posts => self.posts.get_keyboard_mapping(),
            Tab::Species => self.species.get_keyboard_mapping(),
            Tab::TagTranslations => &self.tag_translations.keyboard_mapping,
            Tab::TagGroups => self.tag_groups.get_keyboard_mapping(),
        };

        if let Some(msg) = keyboard_action(ctx, keyboard_mapping) {
            self.queue.push_back(msg);
        }
    }

    fn handle_message(&mut self, ctx: &Context, message: Message) {
        match message {
            Message::TabPosts(msg) => {
                self.posts.queue.push_back(msg);
            }
            Message::TabSpecies(msg) => {
                self.species.queue.push_back(msg);
            }
            Message::TabTagGroups(msg) => {
                self.tag_groups.queue.push_back(msg);
            }
            Message::TabTagTranslations(msg) => {
                self.tag_translations.queue.push_back(msg);
            }
            Message::EditDetails(action) => {
                crate::edit_details::apply(action, &mut self.db);
            }
            Message::StartPublishing(id) => {
                if let Some(sm_manager) = self.sm_manager.as_mut() {
                    sm_manager.publish(&id, &self.db);
                } else {
                    let action = EditDetails::SetPublished(id);
                    crate::edit_details::apply(action, &mut self.db);
                }
            }
            Message::CloseModal => {
                let _ = self.modal_window.pop();
            }
            Message::Confirm(confirm) => {
                let window: Box<dyn ModalWindowTrait> = Box::new(confirm);
                self.queue.push_back(Message::OpenModal(window));
            }
            Message::SaveDatabase => {
                let path = self.db.rootpath.to_path_buf();
                match self.db.save(&path) {
                    Err(err) => {
                        let confirm = Confirm::new(
                            format!("Cannot save {}: {}", path.display(), err),
                            vec![ConfirmOption::new("Close").with_key(Key::Escape)],
                        );
                        self.queue.push_back(Message::Confirm(confirm));
                    }
                    Ok(()) => {
                        self.db.mark_saved();
                    }
                }
            }
            Message::SoftClose => {
                if !self.modal_window.is_empty() {
                    return;
                }

                if self.posts.modal_opened() {
                    if self.active_tab == Tab::Posts {
                        self.posts.try_close_modal()
                    }
                    return;
                }

                if self.species.modal_opened() {
                    if self.active_tab == Tab::Species {
                        self.species.try_close_modal()
                    }
                    return;
                }

                if self.tag_groups.modal_opened() {
                    if self.active_tab == Tab::TagGroups {
                        self.tag_groups.try_close_modal()
                    }
                    return;
                }

                if let Some(sm_manager) = self.sm_manager.as_ref() {
                    if sm_manager.stats().active > 0 {
                        return;
                    }
                }
                if self.db.is_dirty() {
                    let opt1 = ConfirmOption::new(fmt!("{ICON_WARNING} Discard all changes!"))
                        .with_message(Message::AllowClose)
                        .with_color(self.style.button.discard);

                    let opt2 = ConfirmOption::new("Continue editing").with_key(Key::Escape);

                    let confirm = Confirm::new(
                        "Database got changed, do you really want to quit?",
                        vec![opt1, opt2],
                    );
                    self.queue.push_back(Message::Confirm(confirm));

                    return;
                }

                self.queue.push_back(Message::AllowClose);
            }
            Message::AllowClose => {
                self.can_close = true;
                ctx.send_viewport_cmd(ViewportCommand::Close);
            }
            Message::MaximizeWindow => {
                ctx.send_viewport_cmd(ViewportCommand::Maximized(true));
            }
            Message::Copy(text) => {
                ctx.copy_text(text);
            }
            Message::OpenModal(window) => {
                self.modal_window.push(window);
            }
            Message::SetStyle(style) => {
                self.style = style;
            }
            Message::SelectTabPosts => {
                self.active_tab = Tab::Posts;
            }
            Message::SelectTabSpecies => {
                self.active_tab = Tab::Species;
            }
            Message::SelectTabTagTranslations => {
                self.active_tab = Tab::TagTranslations;
            }
            Message::SelectTabTagGroup => {
                self.active_tab = Tab::TagGroups;
            }
            Message::SelectNextTab => {
                self.active_tab = self.active_tab.next();
            }
            Message::SelectPrevTab => {
                self.active_tab = self.active_tab.prev();
            }
            Message::OpenHelp => {
                let window = ModalKeyboard::default()
                    .with_mapping(&self.keyboard_mapping)
                    .with_mapping(match self.active_tab {
                        Tab::Posts => self.posts.get_keyboard_mapping(),
                        Tab::Species => self.species.get_keyboard_mapping(),
                        Tab::TagTranslations => &self.tag_translations.keyboard_mapping,
                        Tab::TagGroups => self.tag_groups.get_keyboard_mapping(),
                    });

                let window: Box<dyn ModalWindowTrait> = Box::new(window);
                self.queue.push_back(Message::OpenModal(window));
            }
            Message::ConfirmResult(mut val) => {
                if let Some(msg) = val.take() {
                    self.queue.push_back(*msg);
                }

                self.queue.push_back(Message::CloseModal);
            }
        }
    }

    fn load(&mut self, storage: &dyn eframe::Storage) {
        self.active_tab =
            eframe::get_value(storage, "main-active-tab").unwrap_or(self.active_tab.clone());
        self.style = eframe::get_value(storage, "main-style").unwrap_or(self.style.clone());

        let db_id = self.db.rootpath.display().to_string();

        self.posts.load(&db_id, storage);
    }
}

impl eframe::App for Application {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        if !self.initialized {
            egui_extras::install_image_loaders(ctx);
            egui_material_icons::initialize(ctx);
            if let Some(storage) = frame.storage() {
                self.load(storage);
            }
            self.initialized = true;

            ctx.style_mut(|style| style.scroll_animation = ScrollAnimation::none());
        }

        if let Some(sm_manager) = self.sm_manager.as_mut() {
            sm_manager.update(&mut self.db);
        }

        self.db.refresh_caches();

        self.keyboard(ctx);

        while let Some(msg) = self.queue.pop_front() {
            self.handle_message(ctx, msg);
        }

        if ctx.input(|i| i.viewport().close_requested()) {
            if !self.can_close {
                ctx.send_viewport_cmd(ViewportCommand::CancelClose);
                self.queue.push_back(Message::SoftClose);
            }
        }

        TopBottomPanel::top("main-window-controls").show(ctx, |ui| {
            ui.columns_const::<2, ()>(|[col1, col2]| {
                col1.horizontal(|ui| {
                    for tab in [
                        Tab::Posts,
                        Tab::Species,
                        Tab::TagTranslations,
                        Tab::TagGroups,
                    ] {
                        ui.selectable_value(&mut self.active_tab, tab.clone(), tab.name());
                    }

                    if let Some(sm_manager) = &self.sm_manager {
                        let stats = sm_manager.stats();
                        if stats.active > 0 {
                            ui.spinner();
                            ui.label(format!("in-progress: {}", stats.active));
                        }

                        if stats.failed > 0 {
                            let label = format!("failed: {}", stats.failed);
                            let button = Button::new(label).fill(self.style.error);
                            if ui.add(button).clicked() {
                                let window: Box<dyn ModalWindowTrait> =
                                    Box::new(ModalErrors::new(sm_manager.errors()));
                                self.queue.push_back(Message::OpenModal(window));
                            }
                        }
                    }
                });

                col2.with_layout(Layout::right_to_left(Align::TOP), |ui| {
                    if self.db.is_dirty() {
                        let button =
                            Button::new("Save database").fill(self.style.button.save_database);

                        if ui.add(button).clicked() {
                            self.queue.push_back(Message::SaveDatabase);
                        }
                    } else {
                        let button = Button::new("Save database").sense(Sense::empty());
                        ui.add(button);
                    }

                    if ui.button(ICON_SETTINGS).clicked() {
                        let window: Box<dyn ModalWindowTrait> =
                            Box::new(ModalSettings::new(&self.style));
                        self.queue.push_back(Message::OpenModal(window));
                    }

                    if ui.button(ICON_HELP).clicked() {
                        self.queue.push_back(Message::OpenHelp);
                    }
                });
            });
        });

        match self.active_tab {
            Tab::Posts => self.posts.update(
                ctx,
                &mut self.image_cache,
                &self.style,
                &mut self.db,
                &mut self.queue,
            ),
            Tab::Species => self.species.update(
                ctx,
                &mut self.image_cache,
                &self.style,
                &mut self.db,
                &mut self.queue,
            ),
            Tab::TagTranslations => self.tag_translations.update(ctx, &mut self.db),
            Tab::TagGroups => {
                self.tag_groups
                    .update(ctx, &self.style, &mut self.db, &mut self.queue)
            }
        }

        for (id, window) in self.modal_window.iter_mut().rev().enumerate() {
            let id = Id::new(("modal-window", id));
            let mut modal = Modal::new(id);
            modal.area = modal
                .area
                .default_height(ctx.content_rect().height() * 0.75);

            modal.show(ctx, |ui| {
                window.update(
                    ui,
                    &mut self.image_cache,
                    &self.style,
                    &self.db,
                    &mut self.queue,
                );
            });
        }

        self.image_cache.load_requested(ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, "main-active-tab", &self.active_tab);
        eframe::set_value(storage, "main-style", &self.style);

        let db_id = self.db.rootpath.display().to_string();

        self.posts.save(&db_id, storage);
    }
}

fn keyboard_action(ctx: &Context, keyboard_mapping: &KeyboardMapping) -> Option<Message> {
    if ctx.wants_keyboard_input() {
        ctx.input_mut(|input_mut| keyboard_mapping.lookup_only_combined(input_mut))
    } else {
        ctx.input_mut(|input_mut| keyboard_mapping.lookup(input_mut))
    }
}
