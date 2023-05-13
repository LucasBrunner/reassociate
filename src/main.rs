use std::time::Duration;

use polodb_core::{bson::doc, Database, IndexModel};
use serde::{Deserialize, Serialize};

mod db;
mod ui;

#[derive(Debug, Serialize, Deserialize)]
struct number {
    number: usize,
}

fn main() {
    let mut db = match db::ReassociateDb::get("test.db") {
        Ok(db) => db,
        Err(err) => {
            println!("{}", err);
            return;
        }
    };

    let mut db = Database::open_file("test2.db").unwrap();

    println!(
        "{:?}",
        db.collection::<number>("numbers")
            .find(None)
            .unwrap()
            .collect::<Vec<_>>()
    );

    db.collection::<number>("numbers").create_index(IndexModel {
        keys: doc! {
            "number": 1,
        },
        options: None,
    });

    for i in 0..10 {
        let time = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        println!(
            "{:?}, time: {}",
            db.collection("numbers").insert_one(number { number: time }),
            time,
        );
    }

    println!(
        "{:?}",
        db.collection::<number>("numbers")
            .find(None)
            .unwrap()
            .collect::<Vec<_>>()
    );

    // ui::start(db, Duration::from_millis(250));
}
