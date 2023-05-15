#![allow(unused)]

use async_std::task;
use async_trait::async_trait;
use std::marker::PhantomData;
use surrealdb::{dbs::Session, kvs::Datastore, sql::Thing};

type DB = (Datastore, Session);

struct Article {
    timestamp_created: i64,
}

struct Record<T> {
    id: Thing,
    data_type: PhantomData<T>,
}

async fn create_article(
    (ds, ses): &DB,
    title: &str,
    priority: i32,
) -> Result<Record<Article>, surrealdb::error::Db> {
    todo!()
}

#[async_trait]
trait Executable {
    type Result;
    async fn execute(self, db: &DB, variables: Option<()>, strict: bool) -> Self::Result;
}

#[async_trait]
impl Executable for &str {
    type Result = Result<Vec<surrealdb::dbs::Response>, surrealdb::error::Db>;

    async fn execute(self, (ds, ses): &DB, variables: Option<()>, strict: bool) -> Self::Result {
        ds.execute(self, ses, None, strict).await
    }
}

#[async_trait(?Send)]
trait ExecuteMany<T>
where
    T: Executable,
{
    async fn execute(self, db: &DB, variables: Option<()>, strict: bool) -> Vec<T::Result>;
}

#[async_trait(?Send)]
impl<T, U> ExecuteMany<T> for U
where
    T: Executable,
    U: IntoIterator<Item = T>,
{
    async fn execute(
        self,
        db: &DB,
        variables: Option<()>,
        strict: bool,
    ) -> Vec<<T as Executable>::Result> {
        let mut results = Vec::new();
        for executable in self.into_iter() {
            results.push(executable.execute(db, variables, strict).await);
        }
        results
    }
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

async fn upgrade_to_version_1(db: &DB) -> Result<(), surrealdb::error::Db> {
    let (ds, ses) = db;
    let create_article_table = r#"
        DEFINE TABLE Article SCHEMAFULL;
        "#;
    ds.execute(create_article_table, ses, None, false).await?;

    let define_article_row = r#"
        DEFINE FIELD timestamp_created 
        ON TABLE Article
        TYPE int
        ASSERT 
            $value != NONE;
    "#;
    ds.execute(define_article_row, ses, None, false).await?;

    let create_element_table = "DEFINE TABLE Element SCHEMAFULL;";
    println!(
        "{:?}",
        ds.execute(create_element_table, ses, None, false).await
    );

    let define_element_table = vec![
        "
        DEFINE FIELD article_id
        ON TABLE Element
        TYPE record(Article)
        ASSERT 
            $value != NONE;
        ",
        //         "
        // DEFINE FIELD data_type_id
        // ON TABLE Element
        // TYPE record
        // ASSERT
        //     $value != NONE;
        //         ",
    ];
    define_element_table
        .execute(db, None, true)
        .await
        .into_iter()
        .fold_result()?;

    Ok(())
}

async fn async_main() -> Result<(), surrealdb::error::Db> {
    let db: &mut DB = &mut (
        Datastore::new("memory").await?,
        Session::for_db("test", "test"),
    );

    // let (ds, ses) = db;
    // ds.execute("DEFINE NAMESPACE test;", ses, None, true);
    // ds.execute("DEFINE DATABASE test;", ses, None, true);

    // db.1 = Session::for_kv().with_ns("test").with_db("test"); // Session::for_db("test_ns", "test_db");

    upgrade_to_version_1(db).await?;

    Ok(())
}

fn main() {
    let resutl = task::block_on(async_main());
    println!("{:?}", resutl);
}
