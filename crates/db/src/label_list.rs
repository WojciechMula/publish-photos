use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct LabelList(Vec<String>);

impl LabelList {
    pub fn toggle(&mut self, label: String) {
        let existing = self.0.iter().position(|s| label == *s);
        if let Some(existing) = existing {
            self.0.remove(existing);
        } else {
            self.0.push(label);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    pub fn add(&mut self, label: &str) -> bool {
        if self.0.iter().any(|s| s == label) {
            return false;
        }

        self.0.push(label.to_owned());
        true
    }

    pub fn remove(&mut self, label: &str) -> bool {
        if let Some(pos) = self.0.iter().position(|s| label == s) {
            self.0.remove(pos);
            true
        } else {
            false
        }
    }
}
