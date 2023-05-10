use sqlx::{query, Pool, Sqlite};

#[derive(Default)]
struct DatabaseVersion {
    number: u32,
}

async fn db_version(db: &Pool<Sqlite>) -> DatabaseVersion {
    // let version_result = sqlx::query_as!(
    //     DatabaseVersion,
    //     r#"
    //         SELECT `number`
    //         FROM `DatabaseVersion`
    //         LIMIT 1;
    //     "#,
    // )
    // .fetch(&db)
    // .await;

    let version_result = Err(());
    version_result.unwrap_or(DatabaseVersion::default())
}

pub async fn upgrade_db(db: Pool<Sqlite>) -> Option<Pool<Sqlite>> {
    let version = db_version(&db).await;

    todo!()
}
