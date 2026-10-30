use crate::species_view::SpeciesList;
use db::Database;
use db::Date;
use db::PostId;
use db::SpeciesId;
use db::SpeciesIndex;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct RecentSpecies {
    pub month: SpeciesList,
    pub day: SpeciesList,
    pub remaining: SpeciesList,
}

impl RecentSpecies {
    pub fn new(id: PostId, db: &Database) -> Self {
        let mut month = BTreeMap::<SpeciesId, SpeciesIndex>::new();
        let mut day = BTreeMap::<SpeciesId, SpeciesIndex>::new();
        let mut remaining = BTreeMap::<SpeciesId, SpeciesIndex>::new();

        let this = db.post(&id);

        for species in &db.species {
            remaining.insert(species.id(), species.index);
        }

        for item in db.posts.iter() {
            let Some(species_id) = &item.species_id else {
                continue;
            };

            if match_day(&this.date, &item.date) {
                let species = db.species_by_id(species_id).unwrap();
                day.insert(species_id.clone(), species.index);
            }

            if match_month(&this.date, &item.date) {
                let species = db.species_by_id(species_id).unwrap();
                month.insert(species_id.clone(), species.index);
            }
        }

        for species_id in day.keys() {
            month.remove(species_id);
            remaining.remove(species_id);
        }

        for species_id in month.keys() {
            remaining.remove(species_id);
        }

        Self {
            day: into_species_list(day),
            month: into_species_list(month),
            remaining: into_species_list(remaining),
        }
    }
}

fn match_day(a: &Date, b: &Date) -> bool {
    a == b
}

fn match_month(a: &Date, b: &Date) -> bool {
    a.year == b.year && a.month == b.month
}

fn into_species_list(mut set: BTreeMap<SpeciesId, SpeciesIndex>) -> SpeciesList {
    let mut ids = Vec::<SpeciesIndex>::with_capacity(set.len());
    while let Some((_, id)) = set.pop_first() {
        ids.push(id);
    }

    SpeciesList::default().with_custom_list(ids)
}
