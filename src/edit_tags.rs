use crate::db::Database;
use crate::db::TagGroupId;
use crate::db::TagList;
use crate::db::TranslatedTag;
use crate::db::TranslatedTagsView;

#[derive(Clone)]
pub enum Action {
    FromString(String),
    FromTagGroup(TagGroupId),
    AddTag(TranslatedTag),
    AddManyTags(TranslatedTagsView),
    RemoveTag(String),
    AssignTags(TagList),
}

impl Action {
    pub fn apply(self, tags: &mut TagList, db: &Database) -> Option<Action> {
        match self {
            Action::AssignTags(other) => {
                let prev = tags.clone();

                tags.0 = other.0;

                Some(Action::AssignTags(prev))
            }
            Action::AddTag(tag) => {
                let prev = tags.clone();
                if Self::add_tag(tags, tag) {
                    Some(Action::AssignTags(prev))
                } else {
                    None
                }
            }
            Action::FromString(string) => {
                let prev = tags.clone();
                for tag in string.split_whitespace().map(|tag| {
                    if let Some(stripped) = tag.strip_prefix("#") {
                        stripped
                    } else {
                        tag
                    }
                }) {
                    tags.add(tag.to_owned());
                    if let Some(trans) = db.tag_translations.translate(tag) {
                        tags.add(trans.clone());
                    }
                }

                if prev.len() != tags.len() {
                    Some(Action::AssignTags(prev))
                } else {
                    None
                }
            }
            Action::FromTagGroup(group_id) => {
                if let Some(group) = db.tag_groups.get(&group_id) {
                    let prev = tags.clone();
                    for tag in group.tags.iter() {
                        tags.add(tag.clone());
                    }

                    if prev.len() != tags.len() {
                        Some(Action::AssignTags(prev))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Action::RemoveTag(tag) => {
                let prev = tags.clone();

                if let Some(trans) = db.tag_translations.translate(&tag) {
                    tags.remove(trans);
                }

                tags.remove(&tag);

                if prev.len() != tags.len() {
                    Some(Action::AssignTags(prev))
                } else {
                    None
                }
            }
            Action::AddManyTags(mut list) => {
                let prev = tags.clone();
                while let Some(tag) = list.pop() {
                    Self::add_tag(tags, tag);
                }

                if prev.len() != tags.len() {
                    Some(Action::AssignTags(prev))
                } else {
                    None
                }
            }
        }
    }

    fn add_tag(tags: &mut TagList, tag: TranslatedTag) -> bool {
        match tag {
            TranslatedTag::Untranslated(s) => tags.add(s.to_owned()),
            TranslatedTag::Translation(trans) => {
                let en = tags.add(trans.en.clone());
                let pl = tags.add(trans.pl.clone());

                pl || en
            }
        }
    }
}
