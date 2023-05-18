use std::marker::PhantomData;

mod upgrade;

use surrealdb::{
    dbs::Session,
    engine::local::{Db, Mem},
    kvs::Datastore,
    sql::Thing,
    Connection, Surreal,
};

use self::upgrade::upgrade_to_version_1;

pub type SurrealDb = Surreal<Db>;

pub struct Record<T> {
    id: Thing,
    data_type: PhantomData<T>,
}

pub async fn get_db() -> Result<SurrealDb, surrealdb::error::Db> {
    let mut surreal = Surreal::new::<Mem>(()).await.unwrap();
    surreal.use_ns("test").use_db("test").await.unwrap();

    upgrade_to_version_1(&surreal).await?;

    Ok(surreal)
}

pub mod prelude {
    pub use super::Record;
    pub use super::SurrealDb;
}
