use crate::{db::prelude::ExecuteMany, FoldResult};

use super::DB;

pub async fn upgrade_to_version_1(db: &DB) -> Result<(), surrealdb::error::Db> {
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
