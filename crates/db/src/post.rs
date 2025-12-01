use crate::Database;
use crate::Date;
use crate::Latin;
use crate::LocalDateTime;
use crate::PostId;
use crate::SearchParts;
use crate::TagList;
use crate::edit_details::EditDetails;
use jpeg::ImageSize;
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
    pub instagram_carousel_id: String,
    #[serde(default)]
    pub facebook_added_at: Option<LocalDateTime>,
    #[serde(default)]
    pub instagram_added_at: Option<LocalDateTime>,
    #[serde(default)]
    pub instagram_permalink: String,
}

impl SocialMediaState {
    pub fn facebook_url(&self) -> String {
        format!("https://facebook.com/{}", self.facebook_post_id)
    }
}

// --------------------------------------------------

const PL_EMOJI: &str = "🇵🇱";
const EN_EMOJI: &str = "🇬🇧";

pub fn render_text(post: &Post, db: &Database) -> String {
    let mut f = Builder::default();

    if !post.pl.is_empty() {
        f.writeln(format!("{PL_EMOJI} {}", post.pl));
    }

    if !post.en.is_empty() {
        f.writeln(format!("{EN_EMOJI} {}", post.en));
    }

    if post.species.is_some() && !f.is_empty() {
        f.newline();

        let latin = post.species.as_ref().unwrap();
        let species = db.species_by_latin(latin).unwrap();
        let latin = latin.as_str();

        let pl = format_species(PL_EMOJI, &species.pl);
        let en = format_species(EN_EMOJI, &species.en);
        f.writeln(match (pl, en) {
            (None, None) => latin.to_owned(),
            (Some(pl), None) => format!("{latin} ({pl})"),
            (None, Some(en)) => format!("{latin} ({en})"),
            (Some(pl), Some(en)) => format!("{latin} ({pl} {en})"),
        });
    }

    if f.is_empty() && post.species.is_some() {
        let latin = post.species.as_ref().unwrap();
        let species = db.species_by_latin(latin).unwrap();

        if !species.pl.is_empty() {
            f.writeln(format!(
                "{PL_EMOJI} {} ({})",
                species.pl,
                &species.latin.as_str()
            ));
            if !species.en.is_empty() {
                f.writeln(format!("{EN_EMOJI} {}", species.en));
            }
        } else if !species.en.is_empty() {
            f.writeln(format!(
                "{EN_EMOJI} {} ({})",
                species.en,
                species.latin.as_str()
            ));
        } else {
            f.writeln(species.latin.as_str().to_string());
        }
    }

    if !f.is_empty() {
        f.newline()
    }

    for (id, tag) in post.tags.iter().enumerate() {
        if id > 0 {
            f.write(format!(" #{tag}"));
        } else {
            f.write(format!("#{tag}"));
        }
    }

    f.buf
}

fn format_species(emoji: &str, name: &str) -> Option<String> {
    if name.is_empty() {
        return None;
    }

    Some(format!("{emoji} {name}"))
}

#[derive(Default)]
struct Builder {
    buf: String,
}

impl Builder {
    fn writeln(&mut self, s: String) {
        self.buf.push_str(&s);
        self.buf.push('\n');
    }

    fn write(&mut self, s: String) {
        self.buf.push_str(&s);
    }

    fn newline(&mut self) {
        self.buf.push('\n');
    }

    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
}
