use chrono::DateTime;
use chrono::Local;
use chrono::Utc;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeSet;

mod database;
pub use database::Database;

mod date;
pub use date::Date;
pub use date::Day;
pub use date::Month;
pub use date::Year;
pub mod edit_details;
pub mod edit_tags;

mod post;
pub use post::FileMetadata;
pub use post::Post;
pub use post::PublishedState;
pub use post::SocialMediaState;
pub use post::render_text;

mod search_parts;
pub use search_parts::SearchParts;

mod species;
pub use species::Latin;
pub use species::Species;
pub use species::SpeciesId;

mod tag_group;
pub use tag_group::TagGroup;
pub use tag_group::TagGroupId;
pub use tag_group::TagGroupList;

mod tag_hints;
mod tag_list;
pub use tag_list::TagList;

mod tag_translations;
pub use tag_translations::TagTranslations;
pub use tag_translations::TranslatedTag;
pub use tag_translations::Translation;

mod label_list;
pub use label_list::LabelList;

mod label_config;
pub use label_config::LabelConfig;

mod picture_views;
pub use picture_views::PictureViews;
use picture_views::PictureViewsBuilder;

mod selector;
pub use selector::Selector;
pub use selector::SelectorLevel;

mod taxonomic_rank;
pub use taxonomic_rank::TaxonomicRank;

// --------------------------------------------------

pub type LocalDateTime = DateTime<Local>;
pub type UtcDateTime = DateTime<Utc>;

#[derive(Default, Clone)]
pub struct Version {
    pub posts: u64,
    pub species: u64,
    pub tag_groups: u64,
    pub tag_translations: u64,
    pub labels: u64,
}

// --------------------------------------------------

#[derive(Default, Serialize, Deserialize)]
pub struct PostList(Vec<Post>);

impl PostList {
    pub fn iter(&self) -> impl Iterator<Item = &Post> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Post> {
        self.0.iter_mut()
    }

    pub fn push(&mut self, item: Post) {
        self.0.push(item)
    }
}

#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct PostId(pub usize);

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct SpeciesIndex(pub usize);

pub enum Language {
    None,
    Polish,
    English,
}

// --------------------------------------------------

#[derive(Default, Clone, Debug)]
pub struct TranslatedTagsView(pub BTreeSet<TranslatedTag>);

impl TranslatedTagsView {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn pop(&mut self) -> Option<TranslatedTag> {
        self.0.pop_first()
    }

    pub fn iter(&self) -> impl Iterator<Item = &TranslatedTag> {
        self.0.iter()
    }

    pub fn add(&mut self, tag: TranslatedTag) {
        self.0.insert(tag);
    }
}
