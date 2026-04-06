use crate::Database;
use crate::Latin;
use crate::Post;
use crate::PostId;
use crate::TagList;
use crate::post::PublishedState;

#[derive(Clone)]
pub enum EditDetails {
    SetPublished(PostId, PublishedState),
    Example(PostId, bool),
    SetPolish(PostId, String),
    SetEnglish(PostId, String),
    SetTags(PostId, TagList),
    SetSpecies(PostId, Option<Latin>),
    SetSocialMediaLink(PostId, String, SocialMediaLink),
    Undo(PostId),
}

#[derive(Clone)]
pub enum SocialMediaLink {
    Facebook,
    Instagram,
}

impl EditDetails {
    const fn id(&self) -> PostId {
        match self {
            Self::SetPublished(id, _)
            | Self::Example(id, _)
            | Self::SetPolish(id, _)
            | Self::SetEnglish(id, _)
            | Self::SetTags(id, _)
            | Self::SetSpecies(id, _)
            | Self::SetSocialMediaLink(id, _, _)
            | Self::Undo(id) => *id,
        }
    }
}

pub fn apply(action: EditDetails, db: &mut Database) {
    match action {
        EditDetails::Undo(id) => {
            let post = db.post_mut(&id);
            if let Some(undo) = post.undo.pop() {
                let changed = apply_aux(undo, post).is_some();
                if changed {
                    post.refresh();
                    db.current_version.posts += 1;
                }
            }
        }

        _ => {
            let id = action.id();
            let post = db.post_mut(&id);
            let undo = apply_aux(action, post);
            if let Some(undo) = undo {
                post.undo.push(undo);
                post.refresh();
                db.current_version.posts += 1;
            }
        }
    }
}

pub fn apply_aux(action: EditDetails, post: &mut Post) -> Option<EditDetails> {
    match action {
        EditDetails::Undo(_) => unreachable!(),
        EditDetails::SetPublished(id, new_state) => {
            if post.published != new_state {
                let prev = post.published.clone();
                post.published = new_state;

                Some(EditDetails::SetPublished(id, prev))
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
        EditDetails::SetSocialMediaLink(id, url, sml) => {
            let prev = match sml {
                SocialMediaLink::Facebook => &post.social_media.facebook_url,
                SocialMediaLink::Instagram => &post.social_media.instagram_url,
            };

            if url != *prev {
                let prev = prev.clone();
                match sml {
                    SocialMediaLink::Facebook => post.social_media.facebook_url = url,
                    SocialMediaLink::Instagram => post.social_media.instagram_url = url,
                };

                Some(EditDetails::SetSocialMediaLink(id, prev, sml))
            } else {
                None
            }
        }
    }
}
