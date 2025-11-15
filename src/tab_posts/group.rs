use crate::db::Database;
use crate::db::Latin;
use crate::db::PostId;
use crate::db::TagList;
use crate::ImageCounter;
use std::path::PathBuf;

pub struct Group {
    items: Vec<PostId>,
}

impl Group {
    pub fn new(id: &PostId) -> Self {
        Self { items: vec![*id] }
    }

    pub fn is_empty(&self) -> bool {
        self.items.len() <= 1
    }

    pub fn count(&self) -> ImageCounter {
        ImageCounter(self.items.len())
    }

    pub fn contains(&self, id: &PostId) -> bool {
        self.items.contains(id)
    }

    pub fn add(&mut self, id: &PostId) {
        if !self.items.contains(id) {
            self.items.push(*id);
        }
    }

    pub fn remove(&mut self, id: &PostId) {
        let Some(index) = self.items.iter().position(|entry| entry == id) else {
            return;
        };

        self.items.remove(index);
    }

    pub fn iter(&self) -> impl Iterator<Item = &PostId> {
        self.items.iter()
    }

    pub fn apply(self, db: &mut Database) {
        let mut pl = Vec::<String>::new();
        let mut en = Vec::<String>::new();
        let mut raw_tags = Vec::<String>::new();
        let mut files = Vec::<PathBuf>::new();
        let mut species: Option<Latin> = None;

        for id in &self.items {
            let post = db.post(id);
            if !post.pl.is_empty() {
                pl.push(post.pl.clone());
            }

            if !post.en.is_empty() {
                en.push(post.en.clone());
            }

            for tag in post.tags.iter() {
                raw_tags.push(tag.clone());
            }

            if species.is_none() && post.species.is_some() {
                species = post.species.clone();
            }

            for path in &post.files {
                files.push(path.to_path_buf());
            }
        }

        let pl = pl.join(" / ");
        let en = en.join(" / ");

        let mut tags = TagList::default();
        for tag in raw_tags {
            tags.add(tag);
        }

        let post = db.post_mut(&self.items[0]);
        post.files = files;
        post.pl = pl;
        post.en = en;
        post.tags = tags;
        post.species = species;

        for id in self.iter().skip(1) {
            db.drop_post(id);
        }

        db.refresh_all_records();
        db.current_version.posts += 1;
    }
}
