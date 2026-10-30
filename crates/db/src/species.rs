use super::FileMetadata;
use super::SearchParts;
use super::SpeciesIndex;
use super::TaxonomicRank;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Display;
use std::fmt::Error;
use std::fmt::Formatter;

#[derive(Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Species {
    pub latin: Latin,
    pub taxonomic_rank: TaxonomicRank,
    pub pl: String,
    pub wikipedia_pl: String,
    pub insektarium_pl: String,
    pub en: String,
    pub wikipedia_en: String,
    pub category: Option<String>,

    #[serde(skip)]
    pub index: SpeciesIndex,

    #[serde(skip)]
    pub search_parts: SearchParts,

    #[serde(skip)]
    pub examples: Vec<FileMetadata>,

    #[serde(skip)]
    pub current_example: usize,
}

#[derive(Default, Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Latin(String);

#[derive(Default, Serialize, Deserialize, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(into = "String", try_from = "String")]
pub struct SpeciesId {
    pub latin: Latin,
    pub taxonomic_rank: TaxonomicRank,
}

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

impl From<&str> for Latin {
    fn from(val: &str) -> Self {
        Self(val.to_owned())
    }
}

impl Display for Latin {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        f.write_str(&self.0)
    }
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

        if self.insektarium_pl != other.insektarium_pl {
            self.insektarium_pl = other.insektarium_pl.clone();
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

        if self.taxonomic_rank != other.taxonomic_rank {
            self.taxonomic_rank = other.taxonomic_rank;
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
        self.search_parts.add(self.taxonomic_rank.en_name());
        self.search_parts.add(self.taxonomic_rank.pl_name());
        for item in items {
            for part in item.split_whitespace() {
                self.search_parts.add(part);
            }
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

    pub fn id(&self) -> SpeciesId {
        SpeciesId {
            latin: self.latin.clone(),
            taxonomic_rank: self.taxonomic_rank,
        }
    }
}

impl From<SpeciesId> for String {
    fn from(s: SpeciesId) -> Self {
        if s.taxonomic_rank.is_species() {
            s.latin.to_string()
        } else {
            format!("{}#{}", s.latin, s.taxonomic_rank.latin_name())
        }
    }
}

impl From<&SpeciesId> for String {
    fn from(s: &SpeciesId) -> Self {
        if s.taxonomic_rank.is_species() {
            s.latin.to_string()
        } else {
            format!("{}#{}", s.latin, s.taxonomic_rank.latin_name())
        }
    }
}

impl Display for SpeciesId {
    fn fmt(&self, w: &mut Formatter<'_>) -> Result<(), Error> {
        write!(w, "{} {}", self.taxonomic_rank.latin_name(), self.latin)
    }
}

impl TryFrom<String> for SpeciesId {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some((latin, taxonomic_rank)) = value.split_once('#') {
            let taxonomic_rank = TaxonomicRank::try_from(taxonomic_rank)?;

            Ok(SpeciesId {
                latin: latin.into(),
                taxonomic_rank,
            })
        } else {
            Ok(SpeciesId {
                latin: value.into(),
                taxonomic_rank: TaxonomicRank::default(),
            })
        }
    }
}
