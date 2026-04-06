use db::Database;
use db::Latin;
use db::Species;
use std::collections::BTreeMap;

pub fn update_species(databases: &mut [Database]) {
    println!("merging species in {} databases", databases.len());
    let all_species = collect_all_species(databases);

    for db in databases.iter_mut() {
        println!("updating {}", db.rootpath.display());
        for (latin, species) in &all_species {
            match db.species_mut_by_latin(latin) {
                Some(existing) => {
                    let modified = update_species_entry(existing, species);
                    if modified {
                        println!("\tupdated {}", latin.as_str());
                    }
                }
                None => {
                    db.add_species(species);
                    db.refresh_caches();
                }
            }
        }
    }
}

fn collect_all_species(dbs: &[Database]) -> BTreeMap<Latin, Species> {
    let mut res = BTreeMap::<Latin, Species>::new();
    for db in dbs {
        for species in &db.species {
            res.entry(species.latin.clone())
                .and_modify(|prev| {
                    update_species_entry(prev, species);
                })
                .or_insert(species.clone());
        }
    }

    res
}

fn update_species_entry(existing: &mut Species, new: &Species) -> bool {
    let m1 = update_string(&mut existing.pl, &new.pl);
    let m2 = update_string(&mut existing.en, &new.en);
    let m3 = update_string(&mut existing.wikipedia_pl, &new.wikipedia_pl);
    let m4 = update_string(&mut existing.wikipedia_en, &new.wikipedia_en);
    let m5 = update_opt_string(&mut existing.category, &new.category);

    m1 | m2 | m3 | m4 | m5
}

fn update_string(existing: &mut String, new: &String) -> bool {
    if existing.is_empty() {
        if !new.is_empty() {
            println!("updating field '{existing}' => '{new}'");
            *existing = new.clone();
            return true;
        }

        return false;
    }

    if new.is_empty() {
        return false;
    }

    assert_eq!(existing, new);

    false
}

fn update_opt_string(existing: &mut Option<String>, new: &Option<String>) -> bool {
    if existing.is_none() {
        if new.is_some() {
            println!("updating field '{existing:?}' => '{new:?}'");
            *existing = new.clone();
            return true;
        }
        return false;
    }

    if new.is_none() {
        return false;
    }

    let a = existing.as_ref().unwrap();
    let b = new.as_ref().unwrap();
    assert_eq!(a, b);

    false
}
