use crate::db::Database;
use crate::db::PostId;
use crate::graphapi::Credentials;
use crate::graphapi::FacebookError;
use crate::graphapi::FacebookId;
use crate::graphapi::PublishEvent;
use crate::tab_posts::modal_publish::render_text;
use reqwest::blocking::Client;
use reqwest::blocking::Request;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

const URL: &str = "https://graph.facebook.com/v24.0";

pub type Receiver = std::sync::mpsc::Receiver<PublishEvent>;
pub type Sender = std::sync::mpsc::Sender<PublishEvent>;

struct Photo {
    path: PathBuf,
    fb_id: String,
}

pub fn publish_post(credentials: Credentials, id: &PostId, db: &Database) -> Receiver {
    let post = db.post(id);

    let text = render_text(post, db);
    let mut photos = Vec::<Photo>::new();

    for item in &post.files_meta {
        photos.push(Photo {
            path: item.full_path.clone(),
            fb_id: String::new(),
        });
    }

    for (photo_id, fb_id) in post.social_media.facebook_photo_ids.iter() {
        photos[*photo_id].fb_id = fb_id.clone();
    }

    let (tx, rx) = channel::<PublishEvent>();

    thread::spawn(
        move || match publish_post_thread_fn(credentials, text, photos, tx.clone()) {
            Ok(json) => tx.send(PublishEvent::PublishedPost { fb_id: json.id }),
            Err(err) => tx.send(PublishEvent::Error(err.to_string())),
        },
    );

    rx
}

fn publish_post_thread_fn(
    credentials: Credentials,
    text: String,
    mut photos: Vec<Photo>,
    tx: Sender,
) -> Result<FacebookId, Box<dyn std::error::Error>> {
    let client = Client::builder().build()?;

    for photo in &mut photos {
        println!("sending {}", photo.path.display());

        let form = reqwest::blocking::multipart::Form::new()
            .text("published", "false")
            .file("source", &photo.path)?;

        let query = [("access_token", credentials.user_access_token.as_str())];

        let url = format!("{URL}/{}/photos", credentials.user_id);
        let req = client.post(url).query(&query).multipart(form).build()?;

        let facebook_id = mk_query(&client, req)?;
        tx.send(PublishEvent::PublishedPhoto {
            path: photo.path.clone(),
            fb_id: facebook_id.id.clone(),
        })?;
        photo.fb_id = facebook_id.id;
    }

    let mut query = Vec::<(String, String)>::new();
    query.push(("access_token".to_owned(), credentials.user_access_token));
    query.push(("message".to_owned(), text));

    for (num, photo) in photos.iter().enumerate() {
        let field = format!("attached_media[{num}]");
        let value = format!(r#"{{"media_fbid":"{}"}}"#, photo.fb_id);
        query.push((field, value));
    }

    let req = client
        .post(format!("{URL}/{}/feed", credentials.user_id))
        .query(&query)
        .build()?;

    let facebook_id = mk_query(&client, req)?;

    println!("page published with id {}", facebook_id.id);

    Ok(facebook_id)
}

fn mk_query(client: &Client, req: Request) -> Result<FacebookId, Box<dyn std::error::Error>> {
    let mut resp = client.execute(req)?;

    let mut body = Vec::<u8>::new();
    resp.copy_to(&mut body)?;

    if let Ok(facebook_id) = serde_json::from_slice::<FacebookId>(&body) {
        Ok(facebook_id)
    } else if let Ok(msg) = serde_json::from_slice::<FacebookError>(&body) {
        Err(Box::new(msg.error))
    } else {
        println!("{resp:?}");

        let err = match String::from_utf8(body.clone()) {
            Ok(s) => s,
            Err(_) => format!("{:?}", body),
        };

        Err(err.into())
    }
}
