mod species;

use crate::application::Message as MainMessage;
use crate::confirm::Confirm;
use crate::confirm::ConfirmOption;
use crate::db::Database;
use crate::db::Latin;
use crate::db::PostId;
use crate::db::SpeciesId;
use crate::edit_details::EditDetails;
use crate::gui::add_image;
use crate::gui::button;
use crate::gui::icon_en;
use crate::gui::icon_pl;
use crate::help;
use crate::image_cache::ImageCache;
use crate::keyboard::KeyboardMapping;
use crate::search_box::SearchBox;
use crate::species_view::SpeciesViewAction;
use crate::style::Style;
use crate::tab_posts::Message as TabMessage;
use crate::tab_posts::MessageQueue as TabMessageQueue;
use crate::tab_posts::Post;
use const_format::formatcp as fmt;
use egui::CentralPanel;
use egui::Context;
use egui::Frame;
use egui::Key;
use egui::Label;
use egui::Layout;
use egui::ScrollArea;
use egui::SidePanel;
use egui::TopBottomPanel;
use egui::Ui;
use species::RecentSpecies;
use std::cell::LazyCell;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_MAGNIFICATION_LARGE;
use egui_material_icons::icons::ICON_MAGNIFICATION_SMALL;
use egui_material_icons::icons::ICON_WARNING;

const ID_PREFIX: &str = "modal-post-species";

pub struct ModalSpecies {
    id: PostId,
    new: Option<Latin>,
    original: Option<Latin>,
    recent_species: RecentSpecies,
    recent_species_version: u64,
    search_box: SearchBox,
    species_hovered: Option<SpeciesId>,
    species_thumbnail_size: ThumbnailSize,

    pub queue: MessageQueue,
    pub keyboard_mapping: LazyCell<KeyboardMapping>,
}

#[derive(Default, PartialEq, Eq, Clone, Copy)]
pub enum ThumbnailSize {
    #[default]
    Small,
    Large,
}

impl ThumbnailSize {
    const fn name(&self) -> &str {
        match self {
            Self::Small => fmt!("{ICON_MAGNIFICATION_SMALL} small"),
            Self::Large => fmt!("{ICON_MAGNIFICATION_LARGE} large"),
        }
    }

    const fn width(&self, style: &Style) -> f32 {
        match self {
            Self::Small => style.image.thumbnail_width,
            Self::Large => style.image.preview_width,
        }
    }
}

type MessageQueue = VecDeque<Message>;

#[derive(Clone)]
pub enum Message {
    RefreshView,
    SpeciesHovered(Option<SpeciesId>),
    SpeciesViewAction(SpeciesViewAction, SpeciesId),
    SetSpecies(SpeciesId),
    FilterByName(String),
    UnsetSpecies,
    SoftClose,
    SaveAndExit,
    CancelAndExit,
    FocusSearch,
    ThumbnailSize(ThumbnailSize),
}

impl Message {
    pub const fn name(&self) -> &str {
        match self {
            Self::RefreshView => unreachable!(),
            Self::SpeciesHovered(_) => unreachable!(),
            Self::SpeciesViewAction { .. } => unreachable!(),
            Self::SetSpecies(_) => unreachable!(),
            Self::FilterByName(_) => unreachable!(),
            Self::UnsetSpecies => unreachable!(),
            Self::SoftClose => help::SOFT_CLOSE,
            Self::SaveAndExit => help::SAVE_AND_EXIT,
            Self::CancelAndExit => unreachable!(),
            Self::FocusSearch => help::FOCUS_SEARCH,
            Self::ThumbnailSize(_) => unreachable!(),
        }
    }
}

impl From<Message> for TabMessage {
    fn from(val: Message) -> Self {
        Self::ModalSpecies(val)
    }
}

impl ModalSpecies {
    pub fn new(id: PostId, db: &Database) -> Self {
        let post = db.post(&id);
        let original = post.species.clone();
        let new = original.clone();
        let recent_species = RecentSpecies::new(id, db);
        let recent_species_version = db.current_version.species;

        let mut res = Self {
            id,
            new,
            original,
            recent_species,
            recent_species_version,
            search_box: SearchBox::new(fmt!("{ID_PREFIX}-phrase")),
            species_hovered: None,
            species_thumbnail_size: ThumbnailSize::Small,
            queue: MessageQueue::new(),
            keyboard_mapping: LazyCell::new(Self::create_mapping),
        };

        res.queue.push_back(Message::RefreshView);
        res
    }

    pub fn update(
        &mut self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &mut Database,
        tab_queue: &mut TabMessageQueue,
    ) {
        if self.recent_species_version != db.current_version.species {
            self.recent_species = RecentSpecies::new(self.id, db);
            self.recent_species_version = db.current_version.species;
            self.queue.push_back(Message::RefreshView);
        }

        while let Some(message) = self.queue.pop_front() {
            self.handle_message(ctx, message, style, db, tab_queue);
        }

        let mut queue = MessageQueue::new();

        self.draw(ctx, image_cache, style, db, tab_queue, &mut queue);

        while let Some(msg) = queue.pop_front() {
            self.queue.push_back(msg);
        }
    }

    fn create_mapping() -> KeyboardMapping {
        fn msg(msg: Message) -> MainMessage {
            MainMessage::TabPosts(msg.into())
        }

        KeyboardMapping::default()
            .key(Key::Escape, msg(Message::SoftClose))
            .key(Key::Slash, msg(Message::FocusSearch))
            .ctrl(Key::Slash, msg(Message::FocusSearch))
            .ctrl(Key::S, msg(Message::SaveAndExit))
    }

    fn handle_message(
        &mut self,
        ctx: &Context,
        message: Message,
        style: &Style,
        db: &mut Database,
        tab_queue: &mut TabMessageQueue,
    ) {
        match message {
            Message::SpeciesHovered(hovered) => {
                let views = [
                    &mut self.recent_species.day,
                    &mut self.recent_species.month,
                    &mut self.recent_species.remaining,
                ];

                for view in views {
                    view.hovered = hovered;
                }
            }
            Message::RefreshView => {
                let views = [
                    &mut self.recent_species.day,
                    &mut self.recent_species.month,
                    &mut self.recent_species.remaining,
                ];

                for view in views {
                    view.image_width = self.species_thumbnail_size.width(style);
                    view.set_filter(self.search_box.phrase(ctx), db);
                    view.refresh_view(db);
                }
            }
            Message::FilterByName(value) => {
                let views = [
                    &mut self.recent_species.day,
                    &mut self.recent_species.month,
                    &mut self.recent_species.remaining,
                ];

                for view in views {
                    view.set_filter(value.clone(), db);
                    view.refresh_view(db);
                }
            }
            Message::ThumbnailSize(ts) => {
                let views = [
                    &mut self.recent_species.day,
                    &mut self.recent_species.month,
                    &mut self.recent_species.remaining,
                ];

                self.species_thumbnail_size = ts;
                for view in views {
                    view.image_width = self.species_thumbnail_size.width(style);
                }
            }
            Message::SetSpecies(id) => {
                if let Some(species) = db.species_by_id(&id) {
                    self.new = Some(species.latin.clone());
                }
            }
            Message::UnsetSpecies => {
                self.new = None;
            }
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

                    let confirm = Confirm::new("The species got changed.", vec![abort, save, cont]);

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
                if self.is_modified() {
                    let msg = EditDetails::SetSpecies(self.id, self.new.clone());
                    tab_queue.push_back(msg.into());
                }
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
        }
    }

    fn draw(
        &self,
        ctx: &Context,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        tab_queue: &mut TabMessageQueue,
        queue: &mut MessageQueue,
    ) {
        SidePanel::left(fmt!("{ID_PREFIX}-left"))
            .resizable(false)
            .show(ctx, |ui| {
                self.draw_header(ui, image_cache, style, db, tab_queue, queue);
            });

        TopBottomPanel::bottom(fmt!("{ID_PREFIX}-bottom")).show(ctx, |ui| {
            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                if button::save(ui, self.is_modified(), Some(style.button.save)) {
                    queue.push_back(Message::SaveAndExit);
                }
                if button::cancel(ui) {
                    queue.push_back(Message::SoftClose);
                }
            });
        });

        CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.horizontal(|ui| {
                    if let Some(phrase) = self.search_box.show(ui) {
                        queue.push_back(Message::FilterByName(phrase));
                    }

                    ui.separator();

                    let mut val = self.species_thumbnail_size;
                    for option in [ThumbnailSize::Small, ThumbnailSize::Large] {
                        ui.selectable_value(&mut val, option, option.name());
                    }

                    if val != self.species_thumbnail_size {
                        queue.push_back(Message::ThumbnailSize(val));
                    }
                });

                ui.separator();

                self.draw_species(ui, image_cache, style, db, queue);
            });
        });
    }

    fn draw_header(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        tab_queue: &mut TabMessageQueue,
        queue: &mut MessageQueue,
    ) {
        let post = db.post(&self.id);

        self.view_species(ui, post, db, tab_queue, queue);

        ui.separator();

        if !post.pl.is_empty() || !post.en.is_empty() {
            if !post.pl.is_empty() {
                ui.horizontal(|ui| {
                    let label = Label::new(&post.pl).truncate();
                    icon_pl(ui);
                    ui.add(label);
                });
            }

            if !post.en.is_empty() {
                ui.horizontal(|ui| {
                    let label = Label::new(&post.en).truncate();
                    icon_en(ui);
                    ui.add(label);
                });
            }

            ui.separator();
        }

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
    }

    fn view_species(
        &self,
        ui: &mut Ui,
        post: &Post,
        db: &Database,
        tab_queue: &mut TabMessageQueue,
        queue: &mut MessageQueue,
    ) {
        let Some(selected_species) = &self.new else {
            ui.label("no species selected");
            return;
        };

        if let Some(species) = db.species_by_latin(selected_species) {
            ui.horizontal(|ui| {
                if ui.button("🗙 Clear").clicked() {
                    queue.push_back(Message::UnsetSpecies);
                }

                ui.heading("Selected species")
            });

            Frame::new()
                .inner_margin(5.0)
                .outer_margin(5.0)
                .fill(ui.visuals().faint_bg_color)
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        crate::species_view::format_latin(ui, species);
                    });

                    ui.horizontal(|ui| {
                        crate::species_view::format_pl(ui, species);
                    });

                    ui.horizontal(|ui| {
                        crate::species_view::format_en(ui, species);
                    });
                });

            if post.is_example {
                if ui.button("🗙 Not a good example").clicked() {
                    let msg = EditDetails::Example(self.id, false);
                    tab_queue.push_back(msg.into());
                }
            } else {
                if ui.button("✔ Set as an example").clicked() {
                    let msg = EditDetails::Example(self.id, true);
                    tab_queue.push_back(msg.into());
                }
            }
        }
    }

    fn draw_species(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        ScrollArea::vertical()
            .id_salt(fmt!("{ID_PREFIX}-species-scroll"))
            .show(ui, |ui| {
                self.draw_species_aux(ui, image_cache, style, db, queue);
            });
    }

    fn draw_species_aux(
        &self,
        ui: &mut Ui,
        image_cache: &mut ImageCache,
        style: &Style,
        db: &Database,
        queue: &mut MessageQueue,
    ) {
        let items = [
            ("This day", &self.recent_species.day),
            ("This month", &self.recent_species.month),
            ("All else", &self.recent_species.remaining),
        ];

        ui.style_mut().interaction.selectable_labels = false;

        let mut hovered_set = false;
        let mut hovered_species: Option<SpeciesId> = None;
        for (label, collection) in items {
            if collection.is_empty() {
                continue;
            }

            ui.label(label);

            let resp = collection.render(ui, image_cache, style, db);
            if let Some(hovered) = resp.hovered {
                hovered_set = true;
                if hovered.is_some() {
                    hovered_species = hovered;
                }
            }
            if let Some(clicked) = resp.clicked {
                queue.push_back(Message::SetSpecies(clicked));
            }
            if let Some((action, id)) = resp.species_view_action {
                queue.push_back(Message::SpeciesViewAction(action, id));
            }
        }

        if hovered_set && hovered_species != self.species_hovered {
            queue.push_back(Message::SpeciesHovered(hovered_species));
        }
    }

    fn is_modified(&self) -> bool {
        self.new != self.original
    }
}
