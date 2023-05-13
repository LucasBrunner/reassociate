#![allow(unused)]

use async_std::task;
use surrealdb::kvs::Datastore;

async fn async_main() -> Result<(), surrealdb::error::Db> {
    let db = Datastore::new("file://temp.db").await?;
    println!("DB created!");
    todo!()
}

fn main() {
    let resutl = task::block_on(async_main());
    println!("{:?}", resutl);
}
