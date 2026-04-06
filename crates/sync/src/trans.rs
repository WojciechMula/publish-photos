use db::Database;
use db::Translation;
use std::collections::BTreeSet;

pub fn merge(databases: &mut [Database]) {
    println!("merging tag translations in {} databases", databases.len());

    let mut all = BTreeSet::<Translation>::new();
    for db in databases.iter() {
        for trans in &db.tag_translations.0 {
            all.insert(trans.clone());
        }
    }

    for db in databases.iter_mut() {
        let mut header = false;
        let mut tmp = BTreeSet::<Translation>::new();
        for trans in &db.tag_translations.0 {
            tmp.insert(trans.clone());
        }

        for trans in &all {
            if tmp.remove(trans) {
                continue;
            }

            if !header {
                println!("updating {}", db.rootpath.display());
                header = true;
            }

            println!("+ adding {} => {}", trans.en, trans.pl);
            db.tag_translations.0.push(trans.clone());
            db.current_version.tag_translations += 1;
        }
    }
}
