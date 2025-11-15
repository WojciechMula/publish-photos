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
use std::collections::HashMap;
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
    saved_version: Version,

    #[serde(skip)]
    pub current_version: Version,

    #[serde(skip)]
    cache_versions: CacheVersion,
}

#[derive(Default, Clone)]
pub struct Version {
    pub posts: u64,
    pub species: u64,
    pub tag_groups: u64,
    pub tag_translations: u64,
}

struct CacheVersion {
    picture_views: u64,
    species_examples: u64,
    tags_views_posts: u64,
    tags_views_tag_translations: u64,
    tags_views_tag_groups: u64,
    tag_hints: u64,
}

impl Default for CacheVersion {
    fn default() -> Self {
        Self {
            picture_views: u64::MAX,
            species_examples: u64::MAX,
            tags_views_posts: u64::MAX,
            tags_views_tag_translations: u64::MAX,
            tags_views_tag_groups: u64::MAX,
            tag_hints: u64::MAX,
        }
    }
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

#[derive(Default, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Debug)]
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

        Ok(result)
    }

    pub fn new(path: &Path) -> Self {
        let rootpath = path.to_path_buf();
        let rootdir = match path.parent() {
            Some(dir) => dir.to_path_buf(),
            None => unreachable!("path to a file always has parent"),
        };

        let mut result = Self {
            rootpath,
            rootdir,
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
        self.current_version.species += 1;
    }

    pub fn update_species(&mut self, data: &Species) {
        let entry = &mut self.species[data.id.0];
        if entry.update(data) {
            entry.refresh();
            self.current_version.species += 1;
        }
    }

    pub fn new_tag(&mut self) -> usize {
        let id = self.tag_translations.0.len();

        self.tag_translations.0.push(Translation::default());
        self.current_version.tag_translations += 1;

        id
    }

    pub fn is_dirty(&mut self) -> bool {
        if self.current_version.species != self.saved_version.species {
            return true;
        }

        if self.current_version.tag_translations != self.saved_version.tag_translations {
            return true;
        }

        if self.current_version.tag_groups != self.saved_version.tag_groups {
            return true;
        }

        if self.current_version.posts != self.saved_version.posts {
            return self.check_for_dirty_posts();
        }

        false
    }

    fn check_for_dirty_posts(&self) -> bool {
        self.posts.iter().any(|entry| entry.is_dirty())
    }

    pub fn mark_saved(&mut self) {
        self.posts.0.iter_mut().for_each(|entry| entry.undo.clear());
        self.saved_version = self.current_version.clone();
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

    pub fn species_mut_by_latin(&mut self, key: &Latin) -> Option<&mut Species> {
        self.species
            .iter_mut()
            .find(|species| species.latin == *key)
    }

    pub fn species_by_id(&self, id: &SpeciesId) -> Option<&Species> {
        self.species.get(id.0)
    }

    pub fn species_mut_by_id(&mut self, id: &SpeciesId) -> Option<&mut Species> {
        self.species.get_mut(id.0)
    }

    pub fn add_group(&mut self, group: TagGroup) -> Result<(), String> {
        self.tag_groups.add(group)?;
        self.current_version.tag_groups += 1;

        Ok(())
    }

    pub fn update_group(&mut self, group: TagGroup) -> Result<(), String> {
        if let Some(existing) = self.tag_groups.get_mut(&group.id) {
            if existing.update(group) {
                self.current_version.tag_groups += 1;
            }

            Ok(())
        } else {
            Err(format!("cannot find tag group {:?}", group.id))
        }
    }

    pub fn move_group_up(&mut self, id: &TagGroupId) {
        if self.tag_groups.move_up(id) {
            self.current_version.tag_groups += 1;
        }
    }

    pub fn move_group_down(&mut self, id: &TagGroupId) {
        if self.tag_groups.move_down(id) {
            self.current_version.tag_groups += 1;
        }
    }

    pub fn get_picture_view(&self, selector: &Selector) -> Option<&PictureView> {
        self.picture_views.get(selector)
    }

    pub fn get_tags_view(&self, selector: &Selector) -> &TranslatedTagsView {
        self.tags_views.get(selector).unwrap()
    }

    pub fn refresh_picture_views(&mut self) {
        if self.cache_versions.picture_views == self.current_version.posts {
            return;
        }

        self.cache_versions.picture_views = self.current_version.posts;

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

    fn refresh_species_examples(&mut self) {
        if self.cache_versions.species_examples == self.current_version.posts {
            return;
        }

        self.cache_versions.species_examples = self.current_version.posts;

        let mut tmp = HashMap::<Latin, Vec<String>>::new();
        for post in self
            .posts
            .iter()
            .filter(|post| post.is_example)
            .filter(|post| post.species.is_some())
        {
            let species = post.species.as_ref().unwrap();
            tmp.entry(species.clone())
                .and_modify(|list| {
                    for uri in &post.uris {
                        list.push(uri.clone());
                    }
                })
                .or_insert_with(|| post.uris.clone());
        }

        for species in self.species.iter_mut() {
            if let Some(examples) = tmp.remove(&species.latin) {
                species.examples = examples;
            } else {
                species.examples.clear();
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
        self.refresh_species_examples();
    }

    fn refresh_tags_views(&mut self) {
        let cv = &mut self.cache_versions;
        if cv.tags_views_posts == self.current_version.posts
            && cv.tags_views_tag_translations == self.current_version.tag_translations
            && cv.tags_views_tag_groups == self.current_version.tag_groups
        {
            return;
        }

        cv.tags_views_posts = self.current_version.posts;
        cv.tags_views_tag_translations = self.current_version.tag_translations;
        cv.tags_views_tag_groups = self.current_version.tag_groups;

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
        if self.cache_versions.tag_hints == self.current_version.posts {
            return;
        }

        self.cache_versions.tag_hints = self.current_version.posts;

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

        self.refresh_species_examples();
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
