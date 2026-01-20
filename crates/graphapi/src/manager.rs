use crate::GraphApiCredentials;
use crate::PublishEvent;
use crate::Receiver;
use crate::SocialMediaError;
use crate::publish_post;
use chrono::Local;
use db::Database;
use db::FileMetadata;
use db::Post;
use db::PostId;
use std::collections::HashMap;
use std::path::Path;
use std::sync::mpsc::TryRecvError;

pub struct SocialMediaPublisher {
    credentials: GraphApiCredentials,
    ongoing: HashMap<PostId, Entry>,
}

struct Entry {
    active: bool,
    receiver: Receiver,
    errors: Vec<SocialMediaError>,
}

pub type SocialMediaErrorList = Vec<(PostId, Vec<SocialMediaError>)>;

#[derive(Default)]
pub struct SocialMediaPublisherStats {
    pub active: usize,
    pub failed: usize,
}

impl SocialMediaPublisher {
    pub fn new(credentials: GraphApiCredentials) -> Self {
        Self {
            credentials,
            ongoing: HashMap::new(),
        }
    }

    pub fn publish(&mut self, id: &PostId, db: &Database) {
        if let Some(entry) = self.ongoing.get(id)
            && entry.active
        {
            return;
        }

        let post = db.post(id);
        if post.published {
            return;
        }

        let entry = Entry {
            active: true,
            receiver: publish_post(self.credentials.clone(), id, db),
            errors: Vec::new(),
        };

        self.ongoing.insert(*id, entry);
    }

    pub fn stats(&self) -> SocialMediaPublisherStats {
        let mut res = SocialMediaPublisherStats::default();
        for entry in self.ongoing.values() {
            if entry.active {
                res.active += 1;
            }
            if !entry.errors.is_empty() {
                res.failed += 1;
            }
        }

        res
    }

    pub fn errors(&self) -> SocialMediaErrorList {
        let mut res = SocialMediaErrorList::new();

        for (id, entry) in self.ongoing.iter() {
            if !entry.errors.is_empty() {
                res.push((*id, entry.errors.clone()));
            }
        }

        res
    }

    pub fn update(&mut self, db: &mut Database) {
        for (id, entry) in self.ongoing.iter_mut().filter(|(_, entry)| entry.active) {
            Self::update_single(id, entry, db);
        }

        self.ongoing.retain(|_, entry| !entry.is_completed());
    }

    fn update_single(id: &PostId, entry: &mut Entry, db: &mut Database) {
        match entry.receiver.try_recv() {
            Ok(res) => match res {
                PublishEvent::Error(err) => {
                    entry.add_error(err);
                    entry.active = false;
                }
                PublishEvent::PublishedPhotoOnFacebook { path, fb_id } => {
                    let post = db.post_mut(id);

                    if let Some(meta) = get_file_metadate_mut_by_full_path(post, &path) {
                        meta.facebook_id = fb_id;
                        db.current_version.photos += 1;
                    } else {
                        let msg = format!("internal error: path not found {}", path.display());
                        entry.add_error(SocialMediaError::String(msg));
                    }
                }
                PublishEvent::PublishedPhotoOnInstagram { path, ig_id } => {
                    let post = db.post_mut(id);

                    if let Some(meta) = get_file_metadate_mut_by_full_path(post, &path) {
                        meta.instagram_id = ig_id;
                        db.current_version.photos += 1;
                    } else {
                        let msg = format!("internal error: path not found {}", path.display());
                        entry.add_error(SocialMediaError::String(msg));
                    }
                }
                PublishEvent::PublishedPostOnFacebook { fb_id } => {
                    let post = db.post_mut(id);
                    post.social_media.facebook_post_id = fb_id;
                    post.social_media.facebook_added_at = Some(Local::now());
                    db.current_version.posts += 1;
                    db.current_version.photos += 1;
                }
                PublishEvent::PublishedPostOnInstagram { ig_id, permalink } => {
                    let post = db.post_mut(id);
                    post.social_media.instagram_post_id = ig_id;
                    post.social_media.instagram_permalink = permalink;
                    post.social_media.instagram_added_at = Some(Local::now());

                    db.current_version.posts += 1;
                    db.current_version.photos += 1;
                }
                PublishEvent::PublishedCarouselOnInstagram { ig_id } => {
                    let post = db.post_mut(id);
                    post.social_media.instagram_carousel_id = ig_id;

                    db.current_version.posts += 1;
                    db.current_version.photos += 1;
                }
                PublishEvent::Completed => {
                    let post = db.post_mut(id);
                    post.published = true;
                    db.current_version.posts += 1;
                    db.current_version.photos += 1;
                    entry.active = false;
                }
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                entry.active = false;
            }
        }
    }
}

impl Entry {
    fn add_error(&mut self, err: SocialMediaError) {
        self.errors.push(err);
    }

    fn is_completed(&self) -> bool {
        !self.active && self.errors.is_empty()
    }
}

#[inline]
fn get_file_metadate_mut_by_full_path<'a>(
    post: &'a mut Post,
    rel_path: &Path,
) -> Option<&'a mut FileMetadata> {
    post.files
        .iter_mut()
        .find(|meta| meta.full_path == rel_path)
}
