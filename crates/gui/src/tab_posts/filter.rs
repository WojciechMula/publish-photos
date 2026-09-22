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
use egui::ComboBox;
use egui::TextEdit;
use egui::Ui;
use query::Expr;
use serde::Deserialize;
use serde::Serialize;
use std::collections::VecDeque;

use egui_material_icons::icons::ICON_CALENDAR_MONTH;
use egui_material_icons::icons::ICON_FILTER_ALT;
use egui_material_icons::icons::ICON_PUBLIC;

pub struct Filter {
    pub search_box: SearchBox,
    pub filter: FilterState,

    icon_width: f32,
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
        if let Some(filter) = eframe::get_value(storage, fmt!("{ID_PREFIX}-filter")) {
            self.filter = filter;
            self.filter.refresh_query();
        }
        self.search_box.load(storage);
    }

    pub fn save(&self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, fmt!("{ID_PREFIX}-filter"), &self.filter);
        self.search_box.save(storage);
    }

    pub fn set_current(&mut self, selector: Selector) {
        self.filter.set_current(selector);
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

                if !self.filter.selector_history.is_empty() {
                    for selector in self.filter.selector_history.iter() {
                        render_selector(
                            ui,
                            db,
                            0.0,
                            &self.filter.image_state,
                            selector,
                            &mut current,
                        );
                    }

                    ui.separator();
                }

                for selector in &db.picture_views.selectors {
                    render_selector(
                        ui,
                        db,
                        self.icon_width,
                        &self.filter.image_state,
                        selector,
                        &mut current,
                    );
                }

                if current != self.filter.current {
                    self.set_current(current);
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

        if let Some(action) = self.search_box.show(ui) {
            queue.push_back(Message::SearchBoxAction(action));
        }

        if !self.filter.query_err.is_empty() {
            let color = ui.visuals().error_fg_color;
            ui.colored_label(color, &self.filter.query_err);
        }

        if self.filter.is_enabled() {
            ui.label(self.filter.count.to_string());
        }

        if self.filter.set_phrase(self.search_box.phrase()) {
            queue.push_back(Message::RefreshView);
        }
    }

    pub fn view_extra(&mut self, ui: &mut Ui, queue: &mut VecDeque<Message>) {
        ui.label("tags:");
        for val in TagState::ALL {
            let resp = ui.radio_value(&mut self.filter.tag_state, val, val.name());
            if resp.changed() {
                queue.push_back(Message::RefreshView);
            }
        }

        ui.separator();

        ui.label("except tags");
        let widget = TextEdit::singleline(&mut self.filter.except_tags_string)
            .hint_text("space-separated list of tags");
        if ui.add(widget).changed() {
            self.filter
                .except_tags
                .set_string(&self.filter.except_tags_string);
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
                self.filter.query.as_ref().is_none_or(|query| {
                    if let Some(latin) = &post.species {
                        if let Some(species) = db.species_by_latin(latin) {
                            query.matches_chain(post.search_parts.get(), species.search_parts.get())
                        } else {
                            query.matches(post.search_parts.get())
                        }
                    } else {
                        query.matches(post.search_parts.get())
                    }
                })
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

fn render_selector(
    ui: &mut Ui,
    db: &Database,
    icon_width: f32,
    image_state: &ImageState,
    selector: &Selector,
    current: &mut Selector,
) {
    let Some(view) = db.picture_views.views.get(selector) else {
        return;
    };
    let label = format_selector(selector, count_pictures(view, db, image_state));

    ui.horizontal(|ui| {
        match selector {
            Selector::ByYear(_) => {}
            Selector::ByMonth(_, _) => {
                ui.add_space(icon_width);
            }
            Selector::ByDate(_) => {
                ui.add_space(2.0 * icon_width);
            }
        }
        ui.selectable_value(current, *selector, label);
    });
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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub enum TagState {
    Any,
    Empty,
    Present,
}

impl TagState {
    pub const ALL: [Self; 3] = [Self::Any, Self::Empty, Self::Present];

    pub fn name(&self) -> &'static str {
        match self {
            Self::Any => "any",
            Self::Empty => "no tags",
            Self::Present => "has tags",
        }
    }

    fn matches(&self, post: &Post) -> bool {
        match self {
            Self::Any => true,
            Self::Empty => post.tags.is_empty(),
            Self::Present => !post.tags.is_empty(),
        }
    }
}

// --------------------------------------------------

#[derive(Default, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct ExceptTags(Vec<String>);

impl ExceptTags {
    fn set_string(&mut self, s: &str) {
        self.0.clear();

        for tag in s.split_whitespace() {
            let tag = tag.trim();
            if !tag.is_empty() {
                self.0.push(tag.to_owned());
            }
        }
    }

    fn matches(&self, post: &Post) -> bool {
        for tag in &self.0 {
            if post.tags.contains(tag) {
                return false;
            }
        }

        for tag in &self.0 {
            if post.labels.iter().any(|s| s == tag) {
                return false;
            }
        }

        true
    }
}

// --------------------------------------------------

#[derive(Serialize, Deserialize, Clone)]
pub struct FilterState {
    image_state: ImageState,
    pub extra: bool,
    tag_state: TagState,
    except_tags_string: String,
    except_tags: ExceptTags,
    pub current: Selector,
    phrase: String,
    selector_history: VecDeque<Selector>,

    #[serde(skip)]
    count: ImageCounter,

    #[serde(skip)]
    query: Option<Expr>,

    #[serde(skip)]
    query_err: String,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            image_state: ImageState::Unpublished,
            extra: false,
            tag_state: TagState::Any,
            except_tags_string: String::new(),
            except_tags: ExceptTags::default(),
            current: Selector::ByYear(0),
            count: ImageCounter(0),
            phrase: String::new(),
            selector_history: VecDeque::new(),
            query: None,
            query_err: String::new(),
        }
    }
}

impl FilterState {
    const SELECTOR_HISTORY_SIZE: usize = 10;

    fn refresh_query(&mut self) -> bool {
        if self.phrase.is_empty() {
            self.query = None;
            self.query_err.clear();

            return true;
        }

        match Expr::new(&self.phrase) {
            Ok(expr) => {
                self.query = Some(expr);
                self.query_err.clear();

                true
            }
            Err(err) => {
                self.query = None;
                self.query_err = err.to_string();

                false
            }
        }
    }

    fn set_phrase(&mut self, phrase: &str) -> bool {
        if phrase == self.phrase {
            return false;
        }

        self.phrase = phrase.trim().to_owned();
        self.refresh_query()
    }

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
            if !self.tag_state.matches(post) {
                return false;
            }

            if !self.except_tags.matches(post) {
                return false;
            }
        }

        true
    }

    fn set_current(&mut self, sel: Selector) {
        if let Some(pos) = self.selector_history.iter().position(|s| *s == sel) {
            self.selector_history.remove(pos);
        }

        self.selector_history.push_front(sel);
        while self.selector_history.len() > Self::SELECTOR_HISTORY_SIZE {
            self.selector_history.pop_back();
        }

        self.current = sel;
    }
}
