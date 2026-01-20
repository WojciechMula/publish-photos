use db::Database;
use db::PostId;
use db::render_text;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::blocking::Request;
use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::thread;

pub mod facebook;
pub mod instagram;
pub mod manager;

pub type Receiver = std::sync::mpsc::Receiver<PublishEvent>;
pub type Sender = std::sync::mpsc::Sender<PublishEvent>;
pub type ErrorType = Box<dyn std::error::Error>;
pub type FnResult = Result<(), ErrorType>;

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

struct Photo {
    path: PathBuf,
    url: String,
    facebook_id: String,
    instagram_id: String,
}

fn publish_post_thread_fn(
    credentials: GraphApiCredentials,
    create_fb_post: bool,
    create_ig_post: bool,
    text: String,
    mut photos: Vec<Photo>,
    tx: Sender,
) -> FnResult {
    let mut client = Client::builder().build()?;

    // 1. Upload photos to Facebook, if needed
    if create_fb_post || create_ig_post {
        facebook::upload_photos(&mut client, &credentials.facebook, &mut photos, tx.clone())?;
    }

    // 2. create Facebook post
    if create_fb_post {
        facebook::publish_post(
            &mut client,
            &credentials.facebook,
            &text,
            &photos,
            tx.clone(),
        )?;
    }

    // 3. create Instagram post(s)
    if create_ig_post {
        if photos.len() == 1 {
            instagram::publish_single_image(
                &mut client,
                &credentials,
                text,
                &mut photos,
                tx.clone(),
            )?;
        } else {
            instagram::publish_multiple_images(
                &mut client,
                &credentials,
                text,
                &mut photos,
                tx.clone(),
            )?;
        }
    }

    Ok(())
}

// --------------------------------------------------

#[derive(Deserialize, Debug, Clone)]
pub struct GraphApiCredentials {
    pub facebook: Credentials,
    pub instagram: Credentials,
}

impl GraphApiCredentials {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let contents = std::fs::read(path)?;
        let gac = toml::from_slice(&contents)?;

        Ok(gac)
    }
}

// --------------------------------------------------

#[derive(Deserialize, Debug, Clone)]
pub struct Credentials {
    pub user_id: String,
    pub user_access_token: String,
    pub endpoint: String,
}

impl Credentials {
    pub fn is_valid(&self) -> bool {
        !self.user_id.is_empty() && !self.user_access_token.is_empty()
    }
}

// --------------------------------------------------

#[derive(Deserialize)]
struct FacebookError {
    pub error: FacebookErrorDetails,
}

#[derive(Deserialize, Debug)]
pub struct FacebookErrorDetails {
    #[serde(skip)]
    pub url: String,
    pub message: String,
}

impl std::fmt::Display for FacebookErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}: {}", self.url, self.message)
    }
}

impl std::error::Error for FacebookErrorDetails {}

// --------------------------------------------------

#[derive(Debug)]
pub enum PublishEvent {
    Error(String),
    PublishedPhotoOnFacebook { path: PathBuf, fb_id: String },
    PublishedPostOnFacebook { fb_id: String },
    PublishedPhotoOnInstagram { path: PathBuf, ig_id: String },
    PublishedPostOnInstagram { ig_id: String, permalink: String },
    PublishedCarouselOnInstagram { ig_id: String },
    Completed,
}

// --------------------------------------------------

#[derive(Default)]
pub struct Query {
    pub entries: Vec<(String, String)>,
}

impl Query {
    pub fn add(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.entries.push((key.into(), value.into()));
    }
}

// --------------------------------------------------

pub fn mk_query(client: &Client, req: Request) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_config() {
        let path = Path::new("tests/sm.toml");

        _ = GraphApiCredentials::from_file(&path).unwrap();
    }

    #[test]
    fn parse_error_response_case1() {
        let sample = br#"{"error":{"message":"Error validating access token: Session has expired on Tuesday, 25-Nov-25 15:00:00 PST. The current time is Wednesday, 26-Nov-25 14:19:00 PST.","type":"OAuthException","code":190,"error_subcode":463,"fbtrace_id":"AaX18KXfIMJ8i9bHfPJVjZq"}}"#;

        serde_json::from_slice::<FacebookError>(sample).unwrap();
    }

    #[test]
    fn parse_error_response_case2() {
        let sample = br#"{"error":{"message":"(#200) Unpublished posts must be posted to a page as the page itself.","type":"OAuthException","code":200,"fbtrace_id":"Az_F0sqqSW4zxSe7NQrv7Wo"}}"#;

        serde_json::from_slice::<FacebookError>(sample).unwrap();
    }
}
