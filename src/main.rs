mod db;

use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

use async_std::{prelude::*, task};

use db::*;

const DB_URL: &str = "sqlite://sqlite.db";

async fn async_main() {
    let Some(db) = db::db(DB_URL).await else {
        return;
    };
}

fn main() {
    task::block_on(async_main());
}
