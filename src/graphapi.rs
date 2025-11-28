use serde::Deserialize;
use std::path::Path;
use std::path::PathBuf;

pub mod facebook;

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

#[derive(Deserialize, Debug, Clone)]
pub struct Credentials {
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub user_access_token: String,
}

impl Credentials {
    pub fn is_valid(&self) -> bool {
        !self.user_id.is_empty() && !self.user_access_token.is_empty()
    }
}

#[derive(Deserialize)]
struct FacebookId {
    pub id: String,
}

#[derive(Deserialize)]
struct FacebookPhotoUrl {
    pub source: String,
}

#[derive(Deserialize)]
struct FacebookError {
    pub error: FacebookErrorDetails,
}

#[derive(Deserialize, Debug)]
pub struct FacebookErrorDetails {
    #[serde(skip)]
    pub url: String,
    pub message: String,
    #[serde(rename(deserialize = "type"))]
    pub typ: String,
    pub code: u64,
    #[serde(rename(deserialize = "error_subcode"))]
    pub subcode: Option<u64>,
    pub fbtrace_id: String,
}

impl std::fmt::Display for FacebookErrorDetails {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}: {}", self.url, self.message)
    }
}

impl std::error::Error for FacebookErrorDetails {}

#[derive(Debug)]
pub enum PublishEvent {
    Error(String),
    PublishedPhotoOnFacebook { path: PathBuf, fb_id: String },
    PublishedPostOnFacebook { fb_id: String },
    PublishedPhotoOnInstagram { path: PathBuf, ig_id: String },
    PublishedPostOnInstagram { ig_id: String, permalink: String },
    Completed,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn case1() {
        let sample = br#"{"error":{"message":"Error validating access token: Session has expired on Tuesday, 25-Nov-25 15:00:00 PST. The current time is Wednesday, 26-Nov-25 14:19:00 PST.","type":"OAuthException","code":190,"error_subcode":463,"fbtrace_id":"AaX18KXfIMJ8i9bHfPJVjZq"}}"#;

        serde_json::from_slice::<FacebookError>(sample).unwrap();
    }

    #[test]
    fn case2() {
        let sample = br#"{"error":{"message":"(#200) Unpublished posts must be posted to a page as the page itself.","type":"OAuthException","code":200,"fbtrace_id":"Az_F0sqqSW4zxSe7NQrv7Wo"}}"#;

        serde_json::from_slice::<FacebookError>(sample).unwrap();
    }
}
