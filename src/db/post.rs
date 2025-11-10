use super::Date;
use super::Latin;
use super::PostId;
use super::SearchParts;
use super::TagList;
use crate::edit_details::EditDetails;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct Post {
    pub published: bool,
    pub files: Vec<PathBuf>,
    pub date: Date,
    #[serde(default)]
    pub pl: String,
    #[serde(default)]
    pub en: String,
    pub tags: TagList,
    pub species: Option<Latin>,
    #[serde(default)]
    pub is_example: bool,

    // runtime parameters
    #[serde(skip)]
    pub id: PostId,

    #[serde(skip)]
    pub uris: Vec<String>,

    #[serde(skip)]
    pub full_paths: Vec<PathBuf>,

    #[serde(skip)]
    pub loaded: bool,

    #[serde(skip)]
    pub undo: Vec<EditDetails>,

    #[serde(skip)]
    pub tags_string: String,

    #[serde(skip)]
    pub search_parts: SearchParts,
}

impl Post {
    pub fn is_dirty(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn refresh(&mut self) {
        self.tags_string = self.tags.as_str();

        self.search_parts.clear();
        self.search_parts.add(&self.pl);
        self.search_parts.add(&self.en);
        for tag in self.tags.iter() {
            self.search_parts.add(&format!("#{tag}"));
        }
    }
}
