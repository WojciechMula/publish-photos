use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LabelConfig {
    pub label: String,
    pub shortcut: String,
    pub color: String,
    pub text_color: String,
}
