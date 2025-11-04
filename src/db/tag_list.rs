use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct TagList(pub Vec<String>);

impl TagList {
    #[inline]
    pub fn contains(&self, s: &String) -> bool {
        self.0.contains(s)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn add(&mut self, s: String) -> bool {
        if self.contains(&s) || s.is_empty() {
            return false;
        }

        self.0.push(s);
        true
    }

    pub fn remove(&mut self, s: &String) -> bool {
        if let Some(index) = self.0.iter().position(|item| item == s) {
            self.0.remove(index);
            return true;
        }

        false
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.0.iter()
    }

    pub fn as_str(&self) -> String {
        let mut result = String::new();
        for (id, tag) in self.iter().enumerate() {
            if id > 0 {
                result += " ";
            }

            result += tag;
        }

        result
    }
}
