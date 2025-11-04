use crate::db::Database;
use crate::db::PostId;
use crate::db::Selector;
use crate::db::Tag;
use crate::db::TagsView;
use std::collections::HashSet;

#[derive(Default)]
pub struct FrequentTags {
    pub global: TagsView,
    pub month: TagsView,
    pub day: TagsView,
}

impl FrequentTags {
    pub fn new(id: PostId, db: &Database) -> Self {
        let mut used = HashSet::<String>::new();
        let post = db.post(&id);

        let by_date = db.get_tags_view(&Selector::ByDate(post.date)).clone();
        let by_month = db.get_tags_view(&Selector::ByMonth(post.date.month));
        let by_month = by_month.0.difference(&by_date.0);

        ()
    }
}
