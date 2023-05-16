use std::marker::PhantomData;

pub mod executable;
mod upgrade;

use surrealdb::{dbs::Session, kvs::Datastore, sql::Thing};

use self::upgrade::upgrade_to_version_1;

pub struct Db {
    ds: Datastore,
    ses: Session,
}

impl Db {
    pub fn add_article<'a>(&self, name: &'a str) -> Result<(), ()> {
        todo!()
    }
}

pub struct Record<T> {
    id: Thing,
    data_type: PhantomData<T>,
}

pub async fn get_db() -> Result<Db, surrealdb::error::Db> {
    let db = Db {
        ds: Datastore::new("memory").await?,
        ses: Session::for_db("test", "test"),
    };

    upgrade_to_version_1(&db).await?;

    Ok(db)
}

pub mod prelude {
    pub use super::executable::prelude::*;
    pub use super::Db;
    pub use super::Record;
}
