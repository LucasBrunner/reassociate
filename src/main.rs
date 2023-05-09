use sqlx::{migrate::MigrateDatabase, Sqlite, SqlitePool};

use async_std::{prelude::*, task};

const DB_URL: &str = "sqlite://sqlite.db";

#[derive(strum_macros::Display)]
enum DatabaseExistsResult {
    Exists,
    Created,
    CouldNotBeCreated,
}

async fn db_exists() -> DatabaseExistsResult {
    if !Sqlite::database_exists(DB_URL).await.unwrap_or(false) {
        println!("Creating database {}", DB_URL);
        match Sqlite::create_database(DB_URL).await {
            Ok(_) => DatabaseExistsResult::Created,
            Err(_) => DatabaseExistsResult::CouldNotBeCreated,
        }
    } else {
        DatabaseExistsResult::Exists
    }
}

fn main() {
    match task::block_on(db_exists()) {
        DatabaseExistsResult::CouldNotBeCreated => {
            println!("Database could not be found or created!");
            return;
        }
        DatabaseExistsResult::Created => println!("Database created!"),
        DatabaseExistsResult::Exists => println!("Database found!"),
    };

    let db = match task::block_on(SqlitePool::connect(DB_URL)) {
        Ok(db) => db,
        Err(_) => {
            println!("Could not connect to database!");
            return;
        }
    };

    _ = task::block_on(sqlx::query!("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY NOT NULL, name VARCHAR(250) NOT NULL);").execute(&db));

    let result = sqlx::query!(
        r#"
            SELECT name
            FROM sqlite_schema
            WHERE type = "table";
        "#
    )
    .fetch_all(&db);
    let result = task::block_on(result);
    println!("{:?}", result);
}
