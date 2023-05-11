use sqlx::{migrate::MigrateDatabase, Pool, Sqlite, SqlitePool};

pub mod upgrade;

#[derive(strum_macros::Display)]
pub enum DatabaseExistsResult {
    Exists,
    Created,
    CouldNotBeCreated,
}

async fn db_exists(location: &str) -> DatabaseExistsResult {
    if !Sqlite::database_exists(location).await.unwrap_or(false) {
        println!("Creating database {}", location);
        match Sqlite::create_database(location).await {
            Ok(_) => DatabaseExistsResult::Created,
            Err(_) => DatabaseExistsResult::CouldNotBeCreated,
        }
    } else {
        DatabaseExistsResult::Exists
    }
}

pub enum GetDatabaseError {
    Exists(DatabaseExistsResult),
    Access,
    Upgrade(),
}

pub async fn get_db(location: &str) -> Option<Pool<Sqlite>> {
    match db_exists(location).await {
        DatabaseExistsResult::CouldNotBeCreated => {
            println!("Database could not be found or created!");
            return None;
        }
        DatabaseExistsResult::Created => println!("Database created!"),
        DatabaseExistsResult::Exists => println!("Database found!"),
    };

    let mut db = match SqlitePool::connect(location).await {
        Ok(db) => db,
        Err(_) => {
            println!("Could not connect to database!");
            return None;
        }
    };

    match upgrade::upgrade_db(&mut db).await {
        Ok(_) => Some(db),
        Err(_) => None,
    }
}
