use surrealdb::sql;

use crate::FoldResult;

use super::SurrealDb;

pub async fn upgrade_to_version_1(db: &SurrealDb) -> Result<(), surrealdb::error::Db> {
    let tables = "
        DEFINE TABLE Article SCHEMAFULL;
        DEFINE TABLE Element SCHEMAFULL;
        // DEFINE TABLE DataVersion SCHEMAFULL;
        DEFINE TABLE DataType SCHEMAFULL;
        // DEFINE TABLE DataTypeField SCHEMAFULL;
        ";

    db.query(tables).await;

    let article_rows = "
        DEFINE FIELD timestamp_created 
        ON TABLE Article
        TYPE int
        ASSERT $value != NONE;
        ";

    let element_rows = "
        DEFINE FIELD article_id
        ON TABLE Element
        TYPE record(Article)
        ASSERT $value != NONE;
    
        DEFINE FIELD data_type_id
        ON TABLE Element
        TYPE record()
        ASSERT $value != NONE;
        ";

    // let data_version_rows = "
    //     DEFINE FIELD element_id
    //     ON TABLE DataVersion
    //     TYPE record(Element)
    //     ASSERT $value != NONE;

    //     DEFINE FIELD timestamp_created
    //     ON TABLE DataVersion
    //     TYPE int
    //     ASSERT $value != NONE;

    //     DEFINE FIELD hidden
    //     ON TABLE DataVersion
    //     TYPE bool
    //     ASSERT $value != NONE;

    //     DEFINE FIELD timestamp_removed
    //     ON TABLE DataVersion
    //     TYPE int
    //     ASSERT $value != NONE;
    //     ";

    let data_type_rows = "
        DEFINE FIELD name
        ON TABLE DataType
        TYPE string
        ASSERT $value != NONE;

        DEFINE FIELD table_id
        ON TABLE DataType
        TYPE record(Element)
        ASSERT $value != NONE;

        DEFINE FIELD data_type
        ON TABLE DataType
        TYPE int
        ASSERT $value != NONE;
        ";

    let table_definitions = vec![
        article_rows,
        element_rows,
        // data_version_rows,
        data_type_rows,
    ];

    for rows in table_definitions {
        db.query(rows).await;
    }
    let mut table_info = db.query("INFO FOR TABLE DataType").await;
    db.create(resource)
    println!("{:#?}", table_info.unwrap());

    Ok(())
}
