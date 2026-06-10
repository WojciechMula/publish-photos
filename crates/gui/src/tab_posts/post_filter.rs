use db::Post;
use db::Species;

#[derive(Default)]
pub struct PostFilter {
    pub pred: Vec<PostPredicate>,
}

pub enum PostPredicate {
    Fragment(String),
    NoTags,
    NoPlDescription,
    NoEnDescription,
}

impl PostPredicate {
    pub fn new(s: &str) -> crate::Result<Self> {
        if let Some(op) = s.strip_prefix(":") {
            match op {
                "no-tags" | "notags" => Ok(Self::NoTags),
                "no-pl" | "nopl" => Ok(Self::NoPlDescription),
                "no-en" | "noen" => Ok(Self::NoEnDescription),
                _ => Err(format!("{s} is not a valid predicate").into()),
            }
        } else {
            Ok(Self::Fragment(s.to_string()))
        }
    }

    pub fn matches_post(&self, post: &Post) -> bool {
        match self {
            Self::Fragment(frag) => post.search_parts.matches(frag),
            Self::NoTags => post.tags.is_empty(),
            Self::NoPlDescription => post.pl.is_empty(),
            Self::NoEnDescription => post.en.is_empty(),
        }
    }

    pub fn matches_species(&self, species: &Species) -> bool {
        match self {
            Self::Fragment(frag) => species.search_parts.matches(frag),
            Self::NoTags | Self::NoPlDescription | Self::NoEnDescription => false,
        }
    }
}

impl PostFilter {
    pub fn new(s: &str) -> crate::Result<Self> {
        let mut result = Self { pred: Vec::new() };

        for frag in s.split_whitespace() {
            result.pred.push(PostPredicate::new(frag)?);
        }

        Ok(result)
    }

    pub fn matches_post(&self, post: &Post) -> bool {
        if self.pred.is_empty() {
            return true;
        }

        self.pred.iter().all(|pred| pred.matches_post(post))
    }

    pub fn matches_species(&self, species: &Species) -> bool {
        if self.pred.is_empty() {
            return true;
        }

        self.pred.iter().all(|pred| pred.matches_species(species))
    }
}
