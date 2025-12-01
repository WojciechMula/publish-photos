use crate::species_view::SpeciesList;
use db::Database;
use db::Date;
use db::Latin;
use db::PostId;
use db::SpeciesId;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct RecentSpecies {
    pub month: SpeciesList,
    pub day: SpeciesList,
    pub remaining: SpeciesList,
}

impl RecentSpecies {
    pub fn new(id: PostId, db: &Database) -> Self {
        let mut month = BTreeMap::<Latin, SpeciesId>::new();
        let mut day = BTreeMap::<Latin, SpeciesId>::new();
        let mut remaining = BTreeMap::<Latin, SpeciesId>::new();

        let this = db.post(&id);

        for species in &db.species {
            remaining.insert(species.latin.clone(), species.id);
        }

        for item in db.posts.iter() {
            let Some(latin) = &item.species else {
                continue;
            };

            if match_day(&this.date, &item.date) {
                let species = db.species_by_latin(latin).unwrap();
                day.insert(species.latin.clone(), species.id);
            }

            if match_month(&this.date, &item.date) {
                let species = db.species_by_latin(latin).unwrap();
                month.insert(species.latin.clone(), species.id);
            }
        }

        for latin in day.keys() {
            month.remove(latin);
            remaining.remove(latin);
        }

        for latin in month.keys() {
            remaining.remove(latin);
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

fn into_species_list(mut set: BTreeMap<Latin, SpeciesId>) -> SpeciesList {
    let mut ids = Vec::<SpeciesId>::with_capacity(set.len());
    while let Some((_, id)) = set.pop_first() {
        ids.push(id);
    }

    SpeciesList::default().with_custom_list(ids)
}
