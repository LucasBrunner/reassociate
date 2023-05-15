use std::marker::PhantomData;

pub mod executable;
mod upgrade;

use surrealdb::{dbs::Session, kvs::Datastore, sql::Thing};

use self::upgrade::upgrade_to_version_1;

pub type DB = (Datastore, Session);

pub struct Record<T> {
    id: Thing,
    data_type: PhantomData<T>,
}

pub async fn get_db() -> Result<DB, surrealdb::error::Db> {
    let db: DB = (
        Datastore::new("memory").await?,
        Session::for_db("test", "test"),
    );

    upgrade_to_version_1(&db).await?;

    Ok(db)
}

pub mod prelude {
    pub use super::executable::prelude::*;
    pub use super::Record;
    pub use super::DB;
}
