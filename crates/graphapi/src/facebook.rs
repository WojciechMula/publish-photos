use crate::Credentials;
use crate::FnResult;
use crate::Photo;
use crate::PublishEvent;
use crate::Query;
use crate::Sender;
use crate::mk_query;
use reqwest::blocking::Client;
use serde::Deserialize;

use crate::FB_URL;

pub(crate) fn publish_post(
    client: &mut Client,
    facebook: &Credentials,
    text: &str,
    photos: &[Photo],
    tx: Sender,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut query = Query::default();
    query.add("access_token", facebook.user_access_token.clone());
    query.add("message", text);

    for (num, photo) in photos.iter().enumerate() {
        let field = format!("attached_media[{num}]");
        let value = format!(r#"{{"media_fbid":"{}"}}"#, photo.facebook_id);
        query.add(field, value);
    }

    let req = client
        .post(format!("{FB_URL}/{}/feed", facebook.user_id))
        .query(&query.entries)
        .build()?;

    let buf = mk_query(client, req)?;
    let facebook_id = serde_json::from_slice::<FacebookId>(&buf)?;
    tx.send(PublishEvent::PublishedPostOnFacebook {
        fb_id: facebook_id.id.clone(),
    })?;

    Ok(())
}

pub(crate) fn upload_photos(
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
