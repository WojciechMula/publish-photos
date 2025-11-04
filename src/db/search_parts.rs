#[derive(Default, Clone, PartialEq, Eq)]
pub struct SearchParts {
    parts: Vec<String>,
}

impl SearchParts {
    pub fn clear(&mut self) {
        self.parts.clear();
    }

    pub fn add(&mut self, s: &str) {
        if !s.is_empty() {
            self.parts.push(s.to_lowercase());
        }
    }

    pub fn matches(&self, phrase: &str) -> bool {
        if phrase.is_empty() {
            return true;
        }

        self.parts.iter().any(|part| part.contains(phrase))
    }
}
