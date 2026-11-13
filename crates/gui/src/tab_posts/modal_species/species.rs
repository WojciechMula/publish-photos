use crate::species_view::SpeciesList;
use db::Database;
use db::PostId;
use db::Selector;
use db::SelectorLevel;
use db::SpeciesId;
use db::SpeciesIndex;
use std::collections::BTreeSet;

#[derive(Default)]
pub struct RecentSpecies {
    pub history: SpeciesList,
    pub month: SpeciesList,
    pub day: SpeciesList,
    pub remaining: SpeciesList,
}

impl RecentSpecies {
    pub fn new(id: PostId, hist: &[SpeciesId], db: &Database) -> Self {
        let mut month = BTreeSet::<SpeciesIndex>::new();
        let mut day = BTreeSet::<SpeciesIndex>::new();
        let mut seen = BTreeSet::<SpeciesIndex>::new();
        let mut history = Vec::<SpeciesIndex>::with_capacity(hist.len());
        let mut remaining = BTreeSet::<SpeciesIndex>::new();

        let post = db.post(&id);

        let day_selector = Selector::ByDate(post.date);
        let month_selector = day_selector.trim_to(SelectorLevel::Month).unwrap();

        let species_index = |id: &Option<SpeciesId>| -> Option<SpeciesIndex> {
            match id {
                Some(id) => db.species_by_id(id).map(|species| species.index),
                None => None,
            }
        };

        for species_id in hist.iter().rev() {
            if let Some(species) = db.species_by_id(species_id) {
                let index = species.index;
                history.push(index);
                seen.insert(index);
            }
        }

        if let Some(view) = db.picture_views.get(day_selector) {
            for post in view.iter().map(|id| db.post(id)) {
                let Some(index) = species_index(&post.species_id) else {
                    continue;
                };

                if seen.contains(&index) {
                    continue;
                }

                seen.insert(index);
                day.insert(index);
            }
        }

        if let Some(view) = db.picture_views.get(month_selector) {
            for post in view.iter().map(|id| db.post(id)) {
                let Some(index) = species_index(&post.species_id) else {
                    continue;
                };

                if seen.contains(&index) {
                    continue;
                }

                seen.insert(index);
                month.insert(index);
            }
        }

        for species in db.species.iter() {
            let index = species.index;
            if !seen.contains(&index) {
                remaining.insert(index);
            }
        }

        Self {
            history: SpeciesList::default()
                .with_custom_list(history)
                .with_disabled_sorting(),
            day: into_species_list(day),
            month: into_species_list(month),
            remaining: into_species_list(remaining),
        }
    }

    pub fn for_each<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut SpeciesList),
    {
        callback(&mut self.history);
        callback(&mut self.day);
        callback(&mut self.month);
        callback(&mut self.remaining);
    }
}

fn into_species_list(mut set: BTreeSet<SpeciesIndex>) -> SpeciesList {
    let mut ids = Vec::<SpeciesIndex>::with_capacity(set.len());
    while let Some(id) = set.pop_first() {
        ids.push(id);
    }

    SpeciesList::default().with_custom_list(ids)
}
