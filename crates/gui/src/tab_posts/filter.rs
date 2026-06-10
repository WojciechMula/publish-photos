use super::post_filter::PostFilter;
use super::Message;
use super::ID_PREFIX;
use crate::file_stem;
use crate::gui::text_size;
use crate::search_box::SearchBox;
use crate::style::Style;
use crate::ImageCounter;
use const_format::formatcp as fmt;
use db::Database;
use db::Date;
use db::Post;
use db::PostId;
use db::Selector;
use egui::ComboBox;
use egui::Ui;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_CALENDAR_MONTH;
use egui_material_icons::icons::ICON_CONTENT_COPY;
use egui_material_icons::icons::ICON_MENU;
use egui_material_icons::icons::ICON_PUBLIC;

pub struct Filter {
    image_state: ImageState,
    pub current: Selector,
    count: ImageCounter,
    pub search_box: SearchBox,
    pub post_filter: crate::Result<PostFilter>,

    icon_width: f32,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            image_state: ImageState::Unpublished,
            current: Selector::ByYear(0),
            count: ImageCounter(0),
            search_box: SearchBox::new(fmt!("{ID_PREFIX}-phrase")),
            post_filter: Ok(PostFilter::default()),
            icon_width: 0.0,
        }
    }
}

impl Filter {
    pub fn load(&mut self, db_id: &str, storage: &dyn eframe::Storage) {
        self.image_state =
            eframe::get_value(storage, fmt!("{ID_PREFIX}-image-state")).unwrap_or(self.image_state);

        let key = format!("{db_id}-{ID_PREFIX}-current");
        self.current = eframe::get_value(storage, &key).unwrap_or(self.current);
    }

    pub fn save(&self, db_id: &str, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, fmt!("{ID_PREFIX}-image-state"), &self.image_state);

        let key = format!("{db_id}-{ID_PREFIX}-current");
        eframe::set_value(storage, &key, &self.current);
    }

    pub fn view(
        &mut self,
        ui: &mut Ui,
        style: &Style,
        db: &Database,
        queue: &mut VecDeque<Message>,
    ) {
        if self.icon_width == 0.0 {
            self.icon_width = text_size(ICON_CALENDAR_MONTH, ui).x;
        }

        if db.picture_views.is_empty() {
            ui.label("no data");
            return;
        }

        if db.picture_views.get(self.current).is_none() {
            self.current = *db.picture_views.selectors.first().unwrap();
        }

        let options = [
            ImageState::Any,
            ImageState::Unpublished,
            ImageState::Published,
        ];

        for option in options {
            if ui
                .radio_value(&mut self.image_state, option, option.name())
                .changed()
            {
                queue.push_back(Message::RefreshView);
            }
        }

        let selected_text = {
            let view = db.picture_views.views.get(&self.current).unwrap();

            format_selector(&self.current, count_pictures(view, db, &self.image_state))
        };

        ComboBox::from_id_salt("tab-images-filter-combo-box")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                let mut current = self.current;

                for selector in &db.picture_views.selectors {
                    let Some(view) = db.picture_views.views.get(selector) else {
                        continue;
                    };
                    let label =
                        format_selector(selector, count_pictures(view, db, &self.image_state));

                    ui.horizontal(|ui| {
                        if matches!(selector, Selector::ByDate(_)) {
                            ui.add_space(self.icon_width);
                        }
                        ui.selectable_value(&mut current, *selector, label);
                    });
                }

                if current != self.current {
                    self.current = current;
                    queue.push_back(Message::RefreshView);
                }
            });

        ui.separator();

        if self.search_box.show(ui).is_some() {
            queue.push_back(Message::RefreshView);
        }

        if !self.search_box.phrase(ui.ctx()).is_empty() {
            match &self.post_filter {
                Ok(_) => {
                    ui.label(self.count.to_string());
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
                Err(err) => {
                    ui.colored_label(style.error, err.to_string());
                }
            }
        }
    }

    pub fn make_view(&mut self, phrase: &str, db: &Database) -> Vec<PostId> {
        self.post_filter = PostFilter::new(phrase);
        let Ok(post_filter) = &self.post_filter else {
            return vec![];
        };

        let mut tmp = Vec::<(PostId, (Date, String))>::new();
        for post in db
            .posts
            .iter()
            .filter(|post| self.image_state.matches(post))
            .filter(|post| self.current.matches(&post.date))
            .filter(|post| {
                if post_filter.matches_post(post) {
                    return true;
                }

                let Some(latin) = &post.species else {
                    return false;
                };

                if let Some(species) = db.species_by_latin(latin) {
                    post_filter.matches_species(species)
                } else {
                    false
                }
            })
        {
            let stem = file_stem(&post.files[0].rel_path);
            let item = (post.id, (post.date, stem));
            tmp.push(item);
        }

        self.count = ImageCounter(tmp.len());

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
