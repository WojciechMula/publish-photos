mod date;
mod post;
mod search_parts;
mod species;
mod tag_group;
mod tag_list;
mod tag_translations;

pub use date::Date;
pub use date::Day;
pub use date::Month;
pub use post::Post;
pub use search_parts::SearchParts;
pub use species::Latin;
pub use species::Species;
pub use tag_group::TagGroup;
pub use tag_group::TagGroupId;
pub use tag_group::TagGroupList;
pub use tag_list::TagList;
pub use tag_translations::TagTranslations;
pub use tag_translations::TranslatedTag;
pub use tag_translations::Translation;

use crate::tag_hints::Builder;
use crate::tag_hints::TagHints;
use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize)]
pub struct Database {
    pub posts: PostList,
    pub tag_translations: TagTranslations,
    pub species: Vec<Species>,
    pub tag_groups: TagGroupList,

    #[serde(skip)]
    pub rootpath: PathBuf,

    #[serde(skip)]
    pub rootdir: PathBuf,

    #[serde(skip)]
    picture_views: BTreeMap<Selector, PictureView>,

    #[serde(skip)]
    tags_views: BTreeMap<Selector, TranslatedTagsView>,

    #[serde(skip)]
    pub tag_hints: TagHints,

    #[serde(skip)]
    pub version: u64,

    #[serde(skip)]
    dirty: Option<bool>,
}

#[derive(Default, Serialize, Deserialize)]
pub struct PostList(Vec<Post>);

impl PostList {
    pub fn iter(&self) -> impl Iterator<Item = &Post> {
        self.0.iter()
    }

    pub fn push(&mut self, item: Post) {
        self.0.push(item)
    }
}

#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize,
)]
pub struct PostId(pub usize);

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct SpeciesId(pub usize);

pub enum Language {
    None,
    Polish,
    English,
}

impl Database {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error + 'static>> {
        let contents = std::fs::read(path)?;
        let mut result: Self = toml::from_slice(&contents)?;

        result.rootpath = path.to_path_buf();
        result.rootdir = match path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => unreachable!("path to a file always has parent"),
        };

        result.refresh_all_records();
        result.version = 1;

        Ok(result)
    }

    pub fn new(path: &Path) -> Self {
        let rootpath = path.to_path_buf();
        let rootdir = match path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => unreachable!("path to a file always has parent"),
        };
        let version = 1;

        let mut result = Self {
            rootpath,
            rootdir,
            version,
            ..Self::default()
        };

        result.refresh_all_records();

        result
    }

    pub fn save(&mut self, path: &Path) -> Result<(), Box<dyn std::error::Error + 'static>> {
        let contents = toml::to_string(self)?;

        std::fs::write(path, &contents)?;

        self.mark_saved();

        Ok(())
    }

    pub fn add_species(&mut self, data: &Species) {
        let id = self.species.len();

        let mut entry = Species {
            id: SpeciesId(id),
            ..Species::default()
        };
        entry.update(data);
        entry.refresh();

        self.species.push(entry);
        self.mark_dirty();
    }

    pub fn update_species(&mut self, data: &Species) {
        let entry = &mut self.species[data.id.0];
        if entry.update(data) {
            entry.refresh();
            self.mark_dirty();
        }
    }

    pub fn new_tag(&mut self) -> usize {
        let id = self.tag_translations.0.len();

        self.tag_translations.0.push(Translation::default());
        self.mark_dirty();

        id
    }

    pub fn mark_posts_dirty(&mut self) {
        self.dirty = None;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = Some(true);
    }

    pub fn is_dirty(&mut self) -> bool {
        if let Some(flag) = &self.dirty {
            return *flag;
        }

        let flag = self.check_for_dirty_posts();
        self.dirty = Some(flag);

        flag
    }

    fn check_for_dirty_posts(&self) -> bool {
        self.posts.iter().any(|entry| entry.is_dirty())
    }

    pub fn mark_saved(&mut self) {
        self.posts.0.iter_mut().for_each(|entry| entry.undo.clear());
        self.dirty = Some(false);
    }

    pub fn post(&self, id: &PostId) -> &Post {
        self.posts.0.get(id.0).unwrap()
    }

    pub fn post_mut(&mut self, id: &PostId) -> &mut Post {
        self.posts.0.get_mut(id.0).unwrap()
    }

    pub fn all_selectors(&self) -> impl DoubleEndedIterator<Item = &Selector> {
        self.picture_views.keys()
    }

    pub fn species_by_latin(&self, key: &Latin) -> Option<&Species> {
        self.species.iter().find(|species| species.latin == *key)
    }

    pub fn species_by_id(&self, id: &SpeciesId) -> Option<&Species> {
        self.species.get(id.0)
    }

    pub fn find_examples(&self, key: Latin) -> Vec<String> {
        let mut res = Vec::<String>::new();
        let key = Some(key);

        for post in self
            .posts
            .iter()
            .filter(|post| post.is_example)
            .filter(|post| post.species == key)
        {
            for uri in &post.uris {
                res.push(uri.clone());
            }
        }

        res
    }

    pub fn add_group(&mut self, group: TagGroup) -> Result<(), String> {
        self.tag_groups.add(group)?;
        self.invalidate_tags_cache();
        self.mark_dirty();

        Ok(())
    }

    pub fn update_group(&mut self, group: TagGroup) -> Result<(), String> {
        if let Some(existing) = self.tag_groups.get_mut(&group.id) {
            if existing.update(group) {
                self.invalidate_tags_cache();
                self.mark_dirty();
            }

            Ok(())
        } else {
            Err(format!("cannot find tag group {:?}", group.id))
        }
    }

    pub fn move_group_up(&mut self, id: &TagGroupId) {
        if self.tag_groups.move_up(id) {
            self.mark_dirty();
        }
    }

    pub fn move_group_down(&mut self, id: &TagGroupId) {
        if self.tag_groups.move_down(id) {
            self.mark_dirty();
        }
    }

    pub fn get_picture_view(&self, selector: &Selector) -> Option<&PictureView> {
        self.picture_views.get(selector)
    }

    pub fn get_tags_view(&self, selector: &Selector) -> &TranslatedTagsView {
        self.tags_views.get(selector).unwrap()
    }

    pub fn refresh_picture_views(&mut self) {
        if !self.picture_views.is_empty() {
            return;
        }

        for post in self.posts.iter() {
            let all = Selector::All;
            let month = Selector::ByMonth(post.date.month);
            let date = Selector::ByDate(post.date);

            for key in [all, month, date] {
                let view = self.picture_views.entry(key).or_default();

                update_view(view, post);
            }
        }
    }

    pub fn drop_post(&mut self, id: &PostId) {
        let Some(index) = self.posts.iter().position(|post| post.id == *id) else {
            return;
        };

        self.posts.0.remove(index);
    }

    pub fn refresh_caches(&mut self) {
        self.refresh_picture_views();
        self.refresh_tags_views();
        self.refresh_tag_hints();
    }

    fn refresh_tags_views(&mut self) {
        if !self.tags_views.is_empty() {
            return;
        }

        let all = Selector::All;
        let mut view_all = TranslatedTagsView::default();
        for trans in self.tag_translations.0.iter() {
            view_all.0.insert(TranslatedTag::Translation(trans.clone()));
        }

        for group in self.tag_groups.iter() {
            for tag in group.tags.iter() {
                view_all.0.insert(self.tag_translations.as_tag(tag));
            }
        }

        let mut tmp = Vec::<TranslatedTag>::new();
        for picture in self.posts.iter() {
            tmp.clear();
            tmp.reserve(picture.tags.len());
            for tag in picture.tags.iter() {
                tmp.push(self.tag_translations.as_tag(tag));
            }

            let date = Selector::ByDate(picture.date);
            let view = self.tags_views.entry(date).or_default();

            for tag in &tmp {
                view.0.insert(tag.clone());
            }

            let month = Selector::ByMonth(picture.date.month);
            let view = self.tags_views.entry(month).or_default();

            for tag in &tmp {
                view.0.insert(tag.clone());
                view_all.0.insert(tag.clone());
            }
        }

        self.tags_views.insert(all, view_all);
    }

    pub fn refresh_tag_hints(&mut self) {
        if !self.tag_hints.is_empty() {
            return;
        }

        let mut builder = Builder::default();
        for post in self.posts.0.iter() {
            builder.record_occurances(&post.tags);
        }

        self.tag_hints = builder.capture();
    }

    pub fn refresh_all_records(&mut self) {
        for (id, entry) in self.posts.0.iter_mut().enumerate() {
            entry.id = PostId(id);
            entry.full_paths.clear();
            entry.uris.clear();
            for path in &entry.files {
                let full_path = self.rootdir.join(path);
                let uri = format!("file://{}", full_path.display());
                entry.full_paths.push(full_path);
                entry.uris.push(uri);
            }

            entry.refresh();
        }

        for (id, entry) in self.species.iter_mut().enumerate() {
            entry.id = SpeciesId(id);

            entry.refresh();
        }
    }

    pub fn invalidate_caches(&mut self) {
        self.invalidate_picture_cache();
        self.invalidate_tags_cache();
    }

    pub fn invalidate_picture_cache(&mut self) {
        self.picture_views.clear();
    }

    pub fn invalidate_tags_cache(&mut self) {
        self.tags_views.clear();
        self.tag_hints.clear();
    }

    pub fn bump_version(&mut self) {
        self.version += 1;
    }
}

// --------------------------------------------------

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Deserialize, Serialize)]
pub enum Selector {
    ByDate(Date),
    ByMonth(Month),
    All,
}

impl Selector {
    pub fn matches(&self, v: &Date) -> bool {
        match self {
            Self::ByDate(date) => date == v,
            Self::ByMonth(month) => *month == v.month,
            Self::All => true,
        }
    }

    fn key(&self) -> (u8, u8) {
        match self {
            Self::All => (0, 0),
            Self::ByMonth(month) => (month.as_u8(), 0),
            Self::ByDate(date) => (date.month.as_u8(), date.day.as_u8()),
        }
    }
}

impl Ord for Selector {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key().cmp(&other.key())
    }
}

impl PartialOrd for Selector {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// --------------------------------------------------

#[derive(Default)]
pub struct PictureView {
    pub all: Vec<PostId>,
    pub published: Vec<PostId>,
    pub unpublished: Vec<PostId>,
}

fn update_view(view: &mut PictureView, picture: &Post) {
    view.all.push(picture.id);

    if picture.published {
        view.published.push(picture.id);
    } else {
        view.unpublished.push(picture.id);
    }
}

// --------------------------------------------------

#[derive(Default, Clone)]
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
