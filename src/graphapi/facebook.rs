use crate::db::Database;
use crate::db::PostId;
use crate::graphapi::FacebookError;
use crate::graphapi::FacebookId;
use crate::graphapi::FacebookPhotoUrl;
use crate::graphapi::PublishEvent;
use crate::tab_posts::modal_publish::render_text;
use crate::GraphApiCredentials;
use reqwest::blocking::Client;
use reqwest::blocking::Request;
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

const FB_URL: &str = "https://graph.facebook.com/v24.0";
const IG_URL: &str = "https://graph.instagram.com/v24.0";

pub type Receiver = std::sync::mpsc::Receiver<PublishEvent>;
pub type Sender = std::sync::mpsc::Sender<PublishEvent>;

struct Photo {
    path: PathBuf,
    url: String,
    facebook_id: String,
    instagram_id: String,
}

pub fn publish_post(credentials: GraphApiCredentials, id: &PostId, db: &Database) -> Receiver {
    let post = db.post(id);

    let text = render_text(post, db);
    let mut photos = Vec::<Photo>::new();

    for entry in &post.files {
        photos.push(Photo {
            path: entry.full_path.clone(),
            url: String::new(),
            facebook_id: entry.facebook_id.clone(),
            instagram_id: entry.instagram_id.clone(),
        });
    }

    let create_fb_post = post.social_media.facebook_post_id.is_empty();
    let create_ig_post =
        post.social_media.instagram_post_id.is_empty() && credentials.instagram.is_valid();

    let (tx, rx) = channel::<PublishEvent>();

    thread::spawn(move || {
        match publish_post_thread_fn(
            credentials,
            create_fb_post,
            create_ig_post,
            text,
            photos,
            tx.clone(),
        ) {
            Ok(_) => tx.send(PublishEvent::Completed),
            Err(err) => tx.send(PublishEvent::Error(err.to_string())),
        }
    });

    rx
}

fn publish_post_thread_fn(
    credentials: GraphApiCredentials,
    create_fb_post: bool,
    create_ig_post: bool,
    text: String,
    mut photos: Vec<Photo>,
    tx: Sender,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    // 1. Upload photos to Facebook, if needed
    if create_fb_post || create_ig_post {
        let facebook = &credentials.facebook;
        for photo in photos
            .iter_mut()
            .filter(|photo| photo.facebook_id.is_empty())
        {
            let form = reqwest::blocking::multipart::Form::new()
                .text("published", "false")
                .file("source", &photo.path)?;

            let query = [("access_token", facebook.user_access_token.as_str())];

            let url = format!("{FB_URL}/{}/photos", facebook.user_id);
            let req = client.post(url).query(&query).multipart(form).build()?;

            let buf = mk_query(&client, req)?;
            let facebook_id = serde_json::from_slice::<FacebookId>(&buf)?;

            tx.send(PublishEvent::PublishedPhotoOnFacebook {
                path: photo.path.clone(),
                fb_id: facebook_id.id.clone(),
            })?;
            photo.facebook_id = facebook_id.id;
        }
    }

    // 2. create Facebook post
    if create_fb_post {
        let facebook = &credentials.facebook;

        let mut query = Vec::<(String, String)>::new();
        query.push((
            "access_token".to_string(),
            facebook.user_access_token.clone(),
        ));
        query.push(("message".to_string(), text.clone()));

        for (num, photo) in photos.iter().enumerate() {
            let field = format!("attached_media[{num}]");
            let value = format!(r#"{{"media_fbid":"{}"}}"#, photo.facebook_id);
            query.push((field, value));
        }

        let req = client
            .post(format!("{FB_URL}/{}/feed", facebook.user_id))
            .query(&query)
            .build()?;

        let buf = mk_query(&client, req)?;
        let facebook_id = serde_json::from_slice::<FacebookId>(&buf)?;
        tx.send(PublishEvent::PublishedPostOnFacebook {
            fb_id: facebook_id.id.clone(),
        })?;
    }

    if create_ig_post {
        let is_carousel = photos.len() > 1;

        let instagram = &credentials.instagram;
        let facebook = &credentials.facebook;

        // 2a. Obtain Facebook FB_URLs for pictures
        for photo in photos.iter_mut() {
            let query = [
                ("access_token", facebook.user_access_token.clone()),
                ("fields", "source".to_owned()),
            ];

            let url = format!("{FB_URL}/{}", photo.facebook_id);
            let req = client.get(url).query(&query).build()?;
            let buf = mk_query(&client, req)?;

            let json = serde_json::from_slice::<FacebookPhotoUrl>(&buf)?;
            photo.url = json.source;
        }

        // 2b. Create Instagram containers
        let mut query = HashMap::<String, String>::new();
        query.insert(
            "access_token".to_string(),
            instagram.user_access_token.clone(),
        );

        if is_carousel {
            query.insert("is_carousel_item".to_string(), "true".to_string());
        } else {
            query.insert("caption".to_string(), text.clone());
        }

        for photo in photos
            .iter_mut()
            .filter(|photo| photo.instagram_id.is_empty())
        {
            query.insert("image_url".to_string(), photo.url.clone());

            let url = format!("{IG_URL}/{}/media", instagram.user_id);
            let req = client.post(url).query(&query).build()?;
            let buf = mk_query(&client, req)?;

            let json = serde_json::from_slice::<FacebookId>(&buf)?;
            photo.instagram_id = json.id;

            tx.send(PublishEvent::PublishedPhotoOnInstagram {
                path: photo.path.clone(),
                ig_id: photo.instagram_id.clone(),
            })?;
        }

        // 2c. Create Instagram post
        if is_carousel {
            todo!()
        } else {
            let photo = &mut photos[0];

            let query = [
                ("access_token", instagram.user_access_token.clone()),
                ("creation_id", photo.instagram_id.clone()),
                ("fields", "permalink".to_owned()),
            ];

            let url = format!("{IG_URL}/{}/media_publish", instagram.user_id);
            let req = client.post(url).query(&query).build()?;
            let buf = mk_query(&client, req)?;

            let json = serde_json::from_slice::<ResponseIdPermalink>(&buf)?;

            tx.send(PublishEvent::PublishedPostOnInstagram {
                ig_id: json.id,
                permalink: json.permalink,
            })?;
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct ResponseIdPermalink {
    pub id: String,
    pub permalink: String,
}

fn mk_query(client: &Client, req: Request) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut resp = client.execute(req)?;

    let mut body = Vec::<u8>::new();
    resp.copy_to(&mut body)?;

    if resp.status() == StatusCode::OK {
        return Ok(body);
    }

    if let Ok(mut msg) = serde_json::from_slice::<FacebookError>(&body) {
        msg.error.url = resp.url().to_string();
        Err(Box::new(msg.error))
    } else {
        let err = match String::from_utf8(body.clone()) {
            Ok(s) => s,
            Err(_) => format!("{:?}", body),
        };

        Err(err.into())
    }
}
