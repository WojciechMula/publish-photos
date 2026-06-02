use crate::Month;
use crate::Post;
use crate::PostId;
use crate::Selector;
use crate::Year;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default)]
pub struct PictureViews {
    pub selectors: Vec<Selector>,
    pub views: HashMap<Selector, Vec<PostId>>,
}

impl PictureViews {
    pub fn is_empty(&self) -> bool {
        self.selectors.is_empty() || self.views.is_empty()
    }

    pub fn get(&self, selector: Selector) -> Option<&Vec<PostId>> {
        self.views.get(&selector)
    }
}

#[derive(Default)]
pub struct PictureViewsBuilder {
    lookup: HashMap<Selector, Vec<PostId>>,
    years: BTreeSet<Year>,
    months: HashSet<(Year, Month)>,
}

impl PictureViewsBuilder {
    pub fn add(&mut self, post: &Post) {
        let selector = Selector::ByDate(post.date);
        let from_date = self.lookup.entry(selector).or_default();
        from_date.push(post.id);

        self.months.insert((post.date.year, post.date.month));
        self.years.insert(post.date.year);
    }

    pub fn capture(mut self) -> PictureViews {
        for (year, month) in &self.months {
            let mut all = Vec::<PostId>::new();

            for (selector, list) in &self.lookup {
                if let Selector::ByDate(d) = selector
                    && d.year == *year
                    && d.month == *month
                {
                    all.extend(list);
                }
            }

            let selector = Selector::ByMonth(*year, *month);
            self.lookup.insert(selector, all);
        }

        for year in &self.years {
            let mut all = Vec::<PostId>::new();

            for (selector, list) in &self.lookup {
                if let Selector::ByMonth(y, _) = selector
                    && y == year
                {
                    all.extend(list);
                }
            }

            let selector = Selector::ByYear(*year);
            self.lookup.insert(selector, all);
        }

        let mut res = PictureViews {
            selectors: self.lookup.keys().cloned().collect(),
            views: self.lookup,
        };

        res.selectors.sort();

        res
    }
}
