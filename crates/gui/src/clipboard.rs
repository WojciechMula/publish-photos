#[derive(Copy, Clone)]
pub enum ClipboardKind {
    Polish,
    English,
    Tags,
    Species,
    Generic,
}

#[derive(Default)]
pub struct Clipboard {
    polish: Vec<String>,
    english: Vec<String>,
    tags: Vec<String>,
    species: Vec<String>,
    generic: Vec<String>,
}

impl Clipboard {
    pub fn copy(&mut self, kind: ClipboardKind, val: String) {
        if let Some(last) = self.get(kind).last() {
            if *last == val {
                return;
            }
        }

        self.get_mut(kind).push(val);
    }

    pub fn available(&self, kind: ClipboardKind) -> bool {
        self.get(kind).last().is_some()
    }

    pub fn get(&self, kind: ClipboardKind) -> &Vec<String> {
        match kind {
            ClipboardKind::Polish => &self.polish,
            ClipboardKind::English => &self.english,
            ClipboardKind::Tags => &self.tags,
            ClipboardKind::Species => &self.species,
            ClipboardKind::Generic => &self.generic,
        }
    }

    pub fn get_mut(&mut self, kind: ClipboardKind) -> &mut Vec<String> {
        match kind {
            ClipboardKind::Polish => &mut self.polish,
            ClipboardKind::English => &mut self.english,
            ClipboardKind::Tags => &mut self.tags,
            ClipboardKind::Species => &mut self.species,
            ClipboardKind::Generic => &mut self.generic,
        }
    }
}
