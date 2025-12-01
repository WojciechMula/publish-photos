use crate::TagList;
use std::collections::HashMap;

#[derive(Default)]
pub struct TagHints {
    entries: HashMap<String, TagList>,
}

impl TagHints {
    pub fn lookup(&self, tag: &String) -> Option<&TagList> {
        self.entries.get(tag)
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

#[derive(Default)]
pub struct Builder {
    entries: HashMap<String, Entry>,
}

#[derive(Default)]
struct Entry {
    pub hist: HashMap<String, usize>,
}

impl Builder {
    pub fn record_occurances(&mut self, tags: &TagList) {
        for tag in tags.iter() {
            let entry = self.entries.entry(tag.clone()).or_default();

            for other_tag in tags.iter() {
                entry
                    .hist
                    .entry(other_tag.clone())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
    }

    pub fn capture(self) -> TagHints {
        let mut result = TagHints::default();
        for (main_tag, entry) in self.entries {
            let mut tmp1 = Vec::<(String, usize)>::with_capacity(entry.hist.len());
            for (tag, count) in entry.hist {
                if tag != main_tag {
                    tmp1.push((tag, count));
                }
            }

            tmp1.sort_by_key(|(_tag, count)| *count);

            let mut tmp2: Vec<String> = tmp1.drain(..).map(|(tag, _count)| tag).collect();
            tmp2.reverse();

            result.entries.insert(main_tag, TagList(tmp2));
        }

        result
    }
}
