use crate::graphapi::mk_query;
use crate::graphapi::Credentials;
use crate::graphapi::FnResult;
use crate::graphapi::Photo;
use crate::graphapi::PublishEvent;
use crate::graphapi::Sender;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::graphapi::FB_URL;

pub fn publish_post(
    client: &mut Client,
    facebook: &Credentials,
    text: &str,
    photos: &[Photo],
    tx: Sender,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut query = Vec::<(String, String)>::new();
    query.push((
        "access_token".to_string(),
        facebook.user_access_token.clone(),
    ));
    query.push(("message".to_string(), text.to_owned()));

    for (num, photo) in photos.iter().enumerate() {
        let field = format!("attached_media[{num}]");
        let value = format!(r#"{{"media_fbid":"{}"}}"#, photo.facebook_id);
        query.push((field, value));
    }

    let req = client
        .post(format!("{FB_URL}/{}/feed", facebook.user_id))
        .query(&query)
        .build()?;

    let buf = mk_query(client, req)?;
    let facebook_id = serde_json::from_slice::<FacebookId>(&buf)?;
    tx.send(PublishEvent::PublishedPostOnFacebook {
        fb_id: facebook_id.id.clone(),
    })?;

    Ok(())
}

pub fn upload_photos(
    client: &mut Client,
    facebook: &Credentials,
    photos: &mut [Photo],
    tx: Sender,
) -> FnResult {
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

        let buf = mk_query(client, req)?;
        let facebook_id = serde_json::from_slice::<FacebookId>(&buf)?;

        tx.send(PublishEvent::PublishedPhotoOnFacebook {
            path: photo.path.clone(),
            fb_id: facebook_id.id.clone(),
        })?;
        photo.facebook_id = facebook_id.id;
    }

    Ok(())
}

#[derive(Deserialize)]
struct FacebookId {
    pub id: String,
}
