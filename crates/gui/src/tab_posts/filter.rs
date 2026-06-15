use super::Message;
use super::ID_PREFIX;
use crate::file_stem;
use crate::gui::text_size;
use crate::search_box::SearchBox;
use crate::ImageCounter;
use const_format::formatcp as fmt;
use db::Database;
use db::Date;
use db::Post;
use db::PostId;
use db::Selector;
use db::Species;
use egui::ComboBox;
use egui::Ui;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_CALENDAR_MONTH;
use egui_material_icons::icons::ICON_CONTENT_COPY;
use egui_material_icons::icons::ICON_FILTER_ALT;
use egui_material_icons::icons::ICON_MENU;
use egui_material_icons::icons::ICON_PUBLIC;

pub struct Filter {
    pub search_box: SearchBox,
    pub filter: FilterState,

    icon_width: f32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct FilterState {
    image_state: ImageState,
    pub extra: bool,
    no_tags: bool,
    pub current: Selector,
    phrase: String,

    #[serde(skip)]
    count: ImageCounter,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            filter: FilterState::default(),
            search_box: SearchBox::new(fmt!("{ID_PREFIX}-phrase")),
            icon_width: 0.0,
        }
    }
}

impl Filter {
    pub fn load(&mut self, storage: &dyn eframe::Storage) {
        self.filter =
            eframe::get_value(storage, fmt!("{ID_PREFIX}-filter")).unwrap_or(self.filter.clone());
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, fmt!("{ID_PREFIX}-filter"), &self.filter);
    }

    pub fn set_current(&mut self, selector: Selector) {
        self.filter.current = selector;
    }

    pub fn get_current(&self) -> Selector {
        self.filter.current
    }

    pub fn is_extra_filter_enabled(&self) -> bool {
        self.filter.extra
    }

    pub fn view(&mut self, ui: &mut Ui, db: &Database, queue: &mut VecDeque<Message>) {
        if self.icon_width == 0.0 {
            self.icon_width = text_size(ICON_CALENDAR_MONTH, ui).x;
        }

        if db.picture_views.is_empty() {
            ui.label("no data");
            return;
        }

        if db.picture_views.get(self.filter.current).is_none() {
            self.filter.current = *db.picture_views.selectors.first().unwrap();
        }

        let options = [
            ImageState::Any,
            ImageState::Unpublished,
            ImageState::Published,
        ];

        for option in options {
            if ui
                .radio_value(&mut self.filter.image_state, option, option.name())
                .changed()
            {
                queue.push_back(Message::RefreshView);
            }
        }

        let selected_text = {
            let view = db.picture_views.views.get(&self.filter.current).unwrap();

            format_selector(
                &self.filter.current,
                count_pictures(view, db, &self.filter.image_state),
            )
        };

        ComboBox::from_id_salt("tab-images-filter-combo-box")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                let mut current = self.filter.current;

                for selector in &db.picture_views.selectors {
                    let Some(view) = db.picture_views.views.get(selector) else {
                        continue;
                    };
                    let label = format_selector(
                        selector,
                        count_pictures(view, db, &self.filter.image_state),
                    );

                    ui.horizontal(|ui| {
                        if matches!(selector, Selector::ByDate(_)) {
                            ui.add_space(self.icon_width);
                        }
                        ui.selectable_value(&mut current, *selector, label);
                    });
                }

                if current != self.filter.current {
                    self.filter.current = current;
                    queue.push_back(Message::RefreshView);
                }
            });

        if ui
            .toggle_value(&mut self.filter.extra, ICON_FILTER_ALT)
            .changed()
        {
            queue.push_back(Message::RefreshView);
        }

        ui.separator();

        if self.search_box.show(ui).is_some() {
            queue.push_back(Message::RefreshView);
        }

        self.filter.phrase = self.search_box.phrase(ui.ctx());

        if self.filter.is_enabled() {
            ui.label(self.filter.count.to_string());
            let resp = ui.button(ICON_MENU);
            resp.context_menu(|ui| {
                if ui
                    .button(fmt!("{ICON_CONTENT_COPY} Copy all paths"))
                    .clicked()
                {
                    queue.push_back(Message::CopyPaths);
                }
            });
        }
    }

    pub fn view_extra(&mut self, ui: &mut Ui, queue: &mut VecDeque<Message>) {
        if ui
            .checkbox(&mut self.filter.no_tags, "having no tags")
            .changed()
        {
            queue.push_back(Message::RefreshView);
        }
    }

    pub fn make_view(&mut self, db: &Database) -> Vec<PostId> {
        let mut tmp = Vec::<(PostId, (Date, String))>::new();
        for post in db
            .posts
            .iter()
            .filter(|post| self.filter.matches(post))
            .filter(|post| {
                if self.filter.post_matches_qs(post) {
                    return true;
                }

                let Some(latin) = &post.species else {
                    return false;
                };

                if let Some(species) = db.species_by_latin(latin) {
                    self.filter.species_matches_qs(species)
                } else {
                    false
                }
            })
        {
            let stem = file_stem(&post.files[0].rel_path);
            let item = (post.id, (post.date, stem));
            tmp.push(item);
        }

        self.filter.count = ImageCounter(tmp.len());

        tmp.sort_by_key(|(_id, (date, stem))| (*date, stem.clone()));

        tmp.iter().map(|(id, _)| *id).collect()
    }
}

fn format_selector(selector: &Selector, count: usize) -> String {
    let label = match selector {
        Selector::ByYear(year) => format!("{ICON_PUBLIC} {year}"),
        Selector::ByMonth(year, month) => format!("{ICON_CALENDAR_MONTH} {month} {year}"),
        Selector::ByDate(date) => format!("{:02}-{:02}", date.month.as_u8(), date.day.as_u8()),
    };

    if count > 0 {
        format!("{label} ({count})")
    } else {
        label
    }
}

fn count_pictures(view: &[PostId], db: &Database, image_state: &ImageState) -> usize {
    match image_state {
        ImageState::Any => view.len(),
        ImageState::Unpublished => view
            .iter()
            .filter(|post_id| db.post(post_id).is_unpublished())
            .count(),
        ImageState::Published => view
            .iter()
            .filter(|post_id| db.post(post_id).is_published())
            .count(),
    }
}

// --------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum ImageState {
    Any,
    Unpublished,
    Published,
}

impl ImageState {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Any => "show all",
            Self::Unpublished => "unpublished",
            Self::Published => "published",
        }
    }

    fn matches(&self, post: &Post) -> bool {
        match self {
            Self::Any => true,
            Self::Published => post.published.as_bool(),
            Self::Unpublished => !post.published.as_bool(),
        }
    }
}

// --------------------------------------------------

impl Default for FilterState {
    fn default() -> Self {
        Self {
            image_state: ImageState::Unpublished,
            extra: false,
            no_tags: false,
            current: Selector::ByYear(0),
            count: ImageCounter(0),
            phrase: String::new(),
        }
    }
}

impl FilterState {
    fn is_enabled(&self) -> bool {
        self.extra || !self.phrase.is_empty()
    }

    fn matches(&self, post: &Post) -> bool {
        if !self.image_state.matches(post) {
            return false;
        }

        if !self.current.matches(&post.date) {
            return false;
        }

        if self.extra {
            let no_tags = post.tags.is_empty();
            if self.no_tags != no_tags {
                return false;
            }
        }

        true
    }

    fn post_matches_qs(&self, post: &Post) -> bool {
        post.search_parts.matches(&self.phrase)
    }

    fn species_matches_qs(&self, species: &Species) -> bool {
        species.search_parts.matches(&self.phrase)
    }
}
