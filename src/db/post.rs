use super::Date;
use super::Latin;
use super::PostId;
use super::SearchParts;
use super::TagList;
use crate::edit_details::EditDetails;
use crate::jpeg::ImageSize;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct Post {
    pub published: bool,
    pub files: Vec<FileMetadata>,
    pub date: Date,
    #[serde(default)]
    pub pl: String,
    #[serde(default)]
    pub en: String,
    pub tags: TagList,
    pub species: Option<Latin>,
    #[serde(default)]
    pub is_example: bool,
    #[serde(default)]
    pub social_media: SocialMediaState,

    // runtime parameters
    #[serde(skip)]
    pub id: PostId,

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

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct FileMetadata {
    pub rel_path: PathBuf,
    pub image_size: Option<ImageSize>,
    pub facebook_id: String,
    pub instagram_id: String,

    #[serde(skip)]
    pub uri: String,

    #[serde(skip)]
    pub full_path: PathBuf,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SocialMediaState {
    pub facebook_post_id: String,
    pub instagram_post_id: String,
    #[serde(default)]
    pub instagram_permalink: String,
}

impl SocialMediaState {
    pub fn facebook_url(&self) -> String {
        format!("https://facebook.com/{}", self.facebook_post_id)
    }
}
