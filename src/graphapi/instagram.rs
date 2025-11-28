use crate::graphapi::mk_query;
use crate::graphapi::Credentials;
use crate::graphapi::FnResult;
use crate::graphapi::Photo;
use crate::graphapi::PublishEvent;
use crate::graphapi::Sender;
use crate::GraphApiCredentials;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;

use crate::graphapi::FB_URL;
use crate::graphapi::IG_URL;

pub fn publish(
    client: &mut Client,
    credentials: &GraphApiCredentials,
    text: &str,
    photos: &mut [Photo],
    tx: Sender,
) -> FnResult {
    // 1. Obtain Facebook FB_URLs for pictures
    collect_photo_urls(client, &credentials.facebook, photos)?;

    // 2. Create Instagram containers
    create_containers(client, &credentials.instagram, text, photos, tx.clone())?;

    // 3. Create Instagram post
    create_post(client, &credentials.instagram, photos, tx.clone())?;

    Ok(())
}

fn collect_photo_urls(
    client: &mut Client,
    facebook: &Credentials,
    photos: &mut [Photo],
) -> FnResult {
    for photo in photos.iter_mut() {
        let query = [
            ("access_token", facebook.user_access_token.clone()),
            ("fields", "source".to_owned()),
        ];

        let url = format!("{FB_URL}/{}", photo.facebook_id);
        let req = client.get(url).query(&query).build()?;
        let buf = mk_query(client, req)?;

        let json = serde_json::from_slice::<FacebookPhotoUrl>(&buf)?;
        photo.url = json.source;
    }

    Ok(())
}

fn create_containers(
    client: &mut Client,
    instagram: &Credentials,
    text: &str,
    photos: &mut [Photo],
    tx: Sender,
) -> FnResult {
    let is_carousel = photos.len() > 1;

    let mut query = HashMap::<String, String>::new();
    query.insert(
        "access_token".to_string(),
        instagram.user_access_token.clone(),
    );

    if is_carousel {
        query.insert("is_carousel_item".to_string(), "true".to_string());
    } else {
        query.insert("caption".to_string(), text.to_owned());
    }

    for photo in photos
        .iter_mut()
        .filter(|photo| photo.instagram_id.is_empty())
    {
        query.insert("image_url".to_string(), photo.url.clone());

        let url = format!("{IG_URL}/{}/media", instagram.user_id);
        let req = client.post(url).query(&query).build()?;
        let buf = mk_query(client, req)?;

        let json = serde_json::from_slice::<InstagramId>(&buf)?;
        photo.instagram_id = json.id;

        tx.send(PublishEvent::PublishedPhotoOnInstagram {
            path: photo.path.clone(),
            ig_id: photo.instagram_id.clone(),
        })?;
    }

    Ok(())
}

fn create_post(
    client: &mut Client,
    instagram: &Credentials,
    photos: &[Photo],
    tx: Sender,
) -> FnResult {
    let is_carousel = photos.len() > 1;

    if is_carousel {
        todo!()
    } else {
        let photo = &photos[0];

        let query = [
            ("access_token", instagram.user_access_token.clone()),
            ("creation_id", photo.instagram_id.clone()),
            ("fields", "permalink".to_owned()),
        ];

        let url = format!("{IG_URL}/{}/media_publish", instagram.user_id);
        let req = client.post(url).query(&query).build()?;
        let buf = mk_query(client, req)?;

        let json = serde_json::from_slice::<InstagramIdPermalink>(&buf)?;

        tx.send(PublishEvent::PublishedPostOnInstagram {
            ig_id: json.id,
            permalink: json.permalink,
        })?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct FacebookPhotoUrl {
    pub source: String,
}

#[derive(Deserialize)]
struct InstagramId {
    pub id: String,
}

#[derive(Deserialize)]
struct InstagramIdPermalink {
    pub id: String,
    pub permalink: String,
}
