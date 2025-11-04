use serde::Deserialize;
use serde::Serialize;
use std::cmp::Ordering;

#[derive(Default, Serialize, Deserialize)]
pub struct TagTranslations(pub Vec<Translation>);

impl TagTranslations {
    pub fn as_tag(&self, tag: &str) -> TranslatedTag {
        for trans in &self.0 {
            if trans.en == tag || trans.pl == tag {
                return TranslatedTag::Translation(trans.clone());
            }
        }

        TranslatedTag::Untranslated(tag.to_owned())
    }

    pub fn translate(&self, tag: &str) -> Option<&String> {
        for trans in &self.0 {
            if trans.en == tag {
                return Some(&trans.pl);
            }
            if trans.pl == tag {
                return Some(&trans.en);
            }
        }

        None
    }
}

// --------------------------------------------------

#[derive(Clone)]
pub enum TranslatedTag {
    Untranslated(String),
    Translation(Translation),
}

impl TranslatedTag {
    pub const fn base(&self) -> &String {
        match self {
            Self::Untranslated(tag) => tag,
            Self::Translation(trans) => &trans.en,
        }
    }
}

impl PartialOrd for TranslatedTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TranslatedTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.base().cmp(other.base())
    }
}

impl PartialEq for TranslatedTag {
    fn eq(&self, other: &Self) -> bool {
        self.base() == other.base()
    }
}

impl Eq for TranslatedTag {}

// --------------------------------------------------

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Translation {
    pub en: String,
    pub pl: String,
}
