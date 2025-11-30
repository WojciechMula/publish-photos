use crate::graphapi::mk_query;
use crate::graphapi::Credentials;
use crate::graphapi::ErrorType;
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

pub fn publish_single_image(
    client: &mut Client,
    credentials: &GraphApiCredentials,
    text: String,
    photos: &mut [Photo],
    tx: Sender,
) -> FnResult {
    assert_eq!(photos.len(), 1);

    // 1. Obtain Facebook FB_URLs for pictures
    collect_photo_urls(client, &credentials.facebook, photos)?;

    // 2. Create Instagram container for image
    create_photo_container(
        client,
        &credentials.instagram,
        text,
        &mut photos[0],
        tx.clone(),
    )?;

    let id = photos[0].instagram_id.clone();

    // 3. Publish Instagram post
    create_post(client, &credentials.instagram, id, tx.clone())
}

pub fn publish_multiple_images(
    client: &mut Client,
    credentials: &GraphApiCredentials,
    text: String,
    photos: &mut [Photo],
    tx: Sender,
) -> FnResult {
    assert!(photos.len() > 1);

    // 1. Obtain Facebook FB_URLs for pictures
    collect_photo_urls(client, &credentials.facebook, photos)?;

    // 2. Create Instagram containers
    create_photo_containers(client, &credentials.instagram, photos, tx.clone())?;

    // 3. Create Instagram container for multiple images
    let id = create_carousel_container(client, &credentials.instagram, text, photos, tx.clone())?;

    // 4. Publish post
    create_post(client, &credentials.instagram, id, tx.clone())
}

// --------------------------------------------------

fn collect_photo_urls(
    client: &mut Client,
    facebook: &Credentials,
    photos: &mut [Photo],
) -> FnResult {
    let query = [
        ("access_token", facebook.user_access_token.clone()),
        ("fields", "source".to_owned()),
    ];

    for photo in photos.iter_mut() {
        let url = format!("{FB_URL}/{}", photo.facebook_id);
        let req = client.get(url).query(&query).build()?;
        let buf = mk_query(client, req)?;

        let json = serde_json::from_slice::<FacebookPhotoUrl>(&buf)?;
        photo.url = json.source;
    }

    Ok(())
}

fn create_photo_container(
    client: &mut Client,
    instagram: &Credentials,
    text: String,
    photo: &mut Photo,
    tx: Sender,
) -> FnResult {
    if !photo.instagram_id.is_empty() {
        return Ok(());
    }

    let query = [
        ("access_token", instagram.user_access_token.clone()),
        ("caption", text),
        ("image_url", photo.url.clone()),
    ];

    let url = format!("{IG_URL}/{}/media", instagram.user_id);
    let req = client.post(url).query(&query).build()?;
    let buf = mk_query(client, req)?;

    let json = serde_json::from_slice::<InstagramId>(&buf)?;
    photo.instagram_id = json.id;

    tx.send(PublishEvent::PublishedPhotoOnInstagram {
        path: photo.path.clone(),
        ig_id: photo.instagram_id.clone(),
    })?;

    Ok(())
}

fn create_photo_containers(
    client: &mut Client,
    instagram: &Credentials,
    photos: &mut [Photo],
    tx: Sender,
) -> FnResult {
    let mut query = HashMap::<String, String>::new();
    query.insert("access_token".into(), instagram.user_access_token.clone());
    query.insert("is_carousel_item".into(), "true".into());

    for photo in photos
        .iter_mut()
        .filter(|photo| photo.instagram_id.is_empty())
    {
        query.insert("image_url".into(), photo.url.clone());

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

fn create_carousel_container(
    client: &mut Client,
    instagram: &Credentials,
    text: String,
    photos: &mut [Photo],
    tx: Sender,
) -> Result<String, ErrorType> {
    let mut children = String::new();
    for (id, photo) in photos.iter().enumerate() {
        if id > 0 {
            children += ",";
        }

        children += &format!("\"{}\"", photo.instagram_id);
    }

    let query = [
        ("access_token", instagram.user_access_token.clone()),
        ("media_type", "CAROUSEL".to_owned()),
        ("children", format!("[{children}]")),
        ("caption", text),
    ];

    let url = format!("{IG_URL}/{}/media", instagram.user_id);
    let req = client.post(url).query(&query).build()?;
    let buf = mk_query(client, req)?;

    let json = serde_json::from_slice::<InstagramId>(&buf)?;

    tx.send(PublishEvent::PublishedCarouselOnInstagram {
        ig_id: json.id.clone(),
    })?;

    Ok(json.id)
}

fn create_post(client: &mut Client, instagram: &Credentials, id: String, tx: Sender) -> FnResult {
    let query = [
        ("access_token", instagram.user_access_token.clone()),
        ("creation_id", id),
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
