use super::SearchParts;
use super::SpeciesId;
use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Latin(String);

impl Latin {
    pub fn contains(&self, s: &str) -> bool {
        self.0.contains(s)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<Latin> for String {
    fn from(val: Latin) -> Self {
        val.0
    }
}

impl From<&Latin> for String {
    fn from(val: &Latin) -> Self {
        val.0.clone()
    }
}

impl<'a> From<&'a Latin> for &'a String {
    fn from(val: &'a Latin) -> Self {
        &val.0
    }
}

impl From<String> for Latin {
    fn from(val: String) -> Self {
        Self(val)
    }
}

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Species {
    pub latin: Latin,
    pub pl: String,
    pub wikipedia_pl: String,
    pub en: String,
    pub wikipedia_en: String,
    pub category: Option<String>,

    #[serde(skip)]
    pub id: SpeciesId,

    #[serde(skip)]
    pub search_parts: SearchParts,

    #[serde(skip)]
    pub examples: Vec<String>,

    #[serde(skip)]
    pub current_example: usize,
}

impl Species {
    pub fn update(&mut self, other: &Self) -> bool {
        let mut changed = false;

        if self.latin != other.latin {
            self.latin = other.latin.clone();
            changed = true;
        }

        if self.pl != other.pl {
            self.pl = other.pl.clone();
            changed = true;
        }

        if self.wikipedia_pl != other.wikipedia_pl {
            self.wikipedia_pl = other.wikipedia_pl.clone();
            changed = true;
        }

        if self.en != other.en {
            self.en = other.en.clone();
            changed = true;
        }

        if self.wikipedia_en != other.wikipedia_en {
            self.wikipedia_en = other.wikipedia_en.clone();
            changed = true;
        }

        if self.category != other.category {
            self.category = other.category.clone();
            changed = true;
        }

        if changed {
            self.refresh();
        }

        changed
    }

    pub fn refresh(&mut self) {
        let items = [&self.latin.0, &self.pl, &self.en];

        self.search_parts.clear();
        for item in items {
            self.search_parts.add(item);
        }

        if let Some(category) = &self.category {
            self.search_parts.add(category);
        }
    }

    pub fn next_example(&mut self) {
        let n = self.examples.len();
        if n == 0 {
            return;
        }

        self.current_example += 1;
        if self.current_example >= n {
            self.current_example = 0;
        }
    }

    pub fn prev_example(&mut self) {
        let n = self.examples.len();
        if n == 0 {
            return;
        }

        if self.current_example > 0 {
            self.current_example -= 1;
        } else {
            self.current_example = n - 1;
        }
    }
}
