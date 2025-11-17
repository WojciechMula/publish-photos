use super::Message;
use super::ID_PREFIX;
use crate::db::Database;
use crate::db::Date;
use crate::db::PictureView;
use crate::db::Post;
use crate::db::PostId;
use crate::db::Selector;
use crate::file_stem;
use crate::gui::text_size;
use crate::search_box::SearchBox;
use crate::ImageCounter;
use const_format::formatcp as fmt;
use egui::ComboBox;
use egui::Ui;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_CALENDAR_MONTH;
use egui_material_icons::icons::ICON_PUBLIC;

pub struct Filter {
    image_state: ImageState,
    pub current: Selector,
    count: ImageCounter,
    pub search_box: SearchBox,

    icon_width: f32,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            image_state: ImageState::Unpublished,
            current: Selector::All,
            count: ImageCounter(0),
            search_box: SearchBox::new(fmt!("{ID_PREFIX}-phrase")),
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

    pub fn view(&mut self, ui: &mut Ui, db: &Database, queue: &mut VecDeque<Message>) {
        if self.icon_width == 0.0 {
            self.icon_width = text_size(ICON_CALENDAR_MONTH, ui).x;
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

        let (selected_text, mut current) = if let Some(view) = db.get_picture_view(&self.current) {
            let selected_text =
                format_selector(&self.current, count_pictures(view, &self.image_state));

            (selected_text, self.current)
        } else {
            ("".to_owned(), Selector::All)
        };

        ComboBox::from_id_salt("tab-images-filter-combo-box")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for selector in db.all_selectors().rev() {
                    let Some(view) = db.get_picture_view(selector) else {
                        continue;
                    };
                    let label = format_selector(selector, count_pictures(view, &self.image_state));

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
            ui.label(self.count.to_string());
        }
    }

    pub fn make_view(&mut self, phrase: &str, db: &Database) -> Vec<PostId> {
        let mut tmp = Vec::<(PostId, (Date, String))>::new();
        let fragments: Vec<&str> = phrase.split_whitespace().collect();
        for post in db
            .posts
            .iter()
            .filter(|post| self.image_state.matches(post))
            .filter(|post| self.current.matches(&post.date))
            .filter(|post| post.search_parts.matches_all(&fragments))
        {
            let item = (post.id, (post.date, file_stem(&post.files[0])));
            tmp.push(item);
        }

        self.count = ImageCounter(tmp.len());

        tmp.sort_by_key(|(_id, (date, stem))| (*date, stem.clone()));

        tmp.iter().map(|(id, _)| *id).collect()
    }
}

fn format_selector(selector: &Selector, count: usize) -> String {
    let label = match selector {
        Selector::All => format!("{ICON_PUBLIC} All images"),
        Selector::ByMonth(month) => format!("{ICON_CALENDAR_MONTH} {month}"),
        Selector::ByDate(date) => format!("{:02}-{:02}", date.month.as_u8(), date.day.as_u8()),
    };

    if count > 0 {
        format!("{label} ({count})")
    } else {
        label
    }
}

fn count_pictures(view: &PictureView, image_state: &ImageState) -> usize {
    match image_state {
        ImageState::Any => view.all.len(),
        ImageState::Unpublished => view.unpublished.len(),
        ImageState::Published => view.published.len(),
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
            Self::Published => post.published,
            Self::Unpublished => !post.published,
        }
    }
}
