#![allow(unused)]

mod db;

use async_std::task;
use surrealdb::{dbs::Session, kvs::Datastore};

use db::prelude::*;

struct Article {
    timestamp_created: i64,
}

async fn create_article(
    (ds, ses): &DB,
    title: &str,
    priority: i32,
) -> Result<Record<Article>, surrealdb::error::Db> {
    todo!()
}

trait FoldResult<T> {
    fn fold_result(self) -> Result<(), T>;
}

impl<T, U, V> FoldResult<T> for V
where
    V: Iterator<Item = Result<U, T>>,
{
    fn fold_result(mut self) -> Result<(), T> {
        let mut acc = Ok(());
        for item in self {
            acc = match (item, acc) {
                (_, Err(err)) => Err(err),
                (Err(err), _) => Err(err),
                (Ok(_), Ok(_)) => Ok(()),
            };
        }
        acc
    }
}

async fn async_main() -> Result<(), surrealdb::error::Db> {
    let db = db::get_db().await?;
    Ok(())
}

fn main() {
    let result = task::block_on(async_main());
    println!("{:?}", result);
}
