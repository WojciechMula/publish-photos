use crate::db::Database;
use crate::db::Latin;
use crate::db::Post;
use crate::db::PostId;
use crate::db::TagList;

#[derive(Clone)]
pub enum EditDetails {
    SetPublished(PostId),
    UnsetPublished(PostId),
    Example(PostId, bool),
    SetPolish(PostId, String),
    SetEnglish(PostId, String),
    SetTags(PostId, TagList),
    SetSpecies(PostId, Option<Latin>),
    Undo(PostId),
}

impl EditDetails {
    const fn id(&self) -> PostId {
        match self {
            Self::SetPublished(id)
            | Self::UnsetPublished(id)
            | Self::Example(id, _)
            | Self::SetPolish(id, _)
            | Self::SetEnglish(id, _)
            | Self::SetTags(id, _)
            | Self::SetSpecies(id, _)
            | Self::Undo(id) => *id,
        }
    }
}

pub fn apply(action: EditDetails, db: &mut Database) {
    match action {
        EditDetails::Undo(id) => {
            let post = db.post_mut(&id);
            if let Some(undo) = post.undo.pop() {
                let change_published_flag = matches!(
                    action,
                    EditDetails::SetPublished(_) | EditDetails::UnsetPublished(_)
                );
                let change_tags = matches!(action, EditDetails::SetTags(_, _));

                let changed = apply_aux(undo, post).is_some();
                if changed {
                    post.refresh();
                    db.mark_dirty();
                    if change_published_flag {
                        db.invalidate_picture_cache();
                        db.bump_version();
                    }
                    if change_tags {
                        db.invalidate_tags_cache();
                    }
                }
            }
        }

        _ => {
            let change_published_flag = matches!(
                action,
                EditDetails::SetPublished(_) | EditDetails::UnsetPublished(_)
            );
            let change_tags = matches!(action, EditDetails::SetTags(_, _));

            let id = action.id();
            let post = db.post_mut(&id);
            let undo = apply_aux(action, post);
            if let Some(undo) = undo {
                post.undo.push(undo);
                post.refresh();
                db.mark_dirty();
            }

            if change_published_flag {
                db.invalidate_picture_cache();
                db.bump_version();
            }
            if change_tags {
                db.invalidate_tags_cache();
            }
        }
    }
}

pub fn apply_aux(action: EditDetails, post: &mut Post) -> Option<EditDetails> {
    match action {
        EditDetails::Undo(_) => unreachable!(),
        EditDetails::SetPublished(id) => {
            if !post.published {
                post.published = true;

                Some(EditDetails::UnsetPublished(id))
            } else {
                None
            }
        }
        EditDetails::UnsetPublished(id) => {
            if post.published {
                post.published = false;

                Some(EditDetails::SetPublished(id))
            } else {
                None
            }
        }
        EditDetails::Example(id, flag) => {
            if post.is_example != flag {
                let prev = post.is_example;
                post.is_example = flag;

                Some(EditDetails::Example(id, prev))
            } else {
                None
            }
        }
        EditDetails::SetSpecies(id, maybe_species) => {
            if post.species != maybe_species {
                let prev = post.species.clone();
                post.species = maybe_species;

                Some(EditDetails::SetSpecies(id, prev))
            } else {
                None
            }
        }
        EditDetails::SetPolish(id, new) => {
            if post.pl != new {
                let prev = post.pl.clone();
                post.pl = new;

                Some(EditDetails::SetPolish(id, prev))
            } else {
                None
            }
        }
        EditDetails::SetEnglish(id, new) => {
            if post.en != new {
                let prev = post.en.clone();
                post.en = new;

                Some(EditDetails::SetEnglish(id, prev))
            } else {
                None
            }
        }
        EditDetails::SetTags(id, tags) => {
            if post.tags != tags {
                let prev = post.tags.clone();
                post.tags = tags;

                Some(EditDetails::SetTags(id, prev))
            } else {
                None
            }
        }
    }
}
