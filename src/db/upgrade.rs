use async_std::println;
use futures_util::stream::StreamExt;
use sqlx::{query, sqlite::SqliteQueryResult, FromRow, Pool, Sqlite, SqliteConnection, SqlitePool};

#[derive(Default, FromRow)]
struct DatabaseVersion {
    version_number: i64,
}

pub enum DatabaseUpgradeError {
    CouldNotIncrementVersion,
    FailedUpdatingTables,
    FailedUpdatingValues,
}

async fn db_version(db: &SqlitePool) -> DatabaseVersion {
    let version_result = sqlx::query_as!(
        DatabaseVersion,
        r#"
            SELECT `version_number`
            FROM `Version`
            LIMIT 1;
        "#,
    )
    .fetch_one(db)
    .await;

    version_result.unwrap_or(DatabaseVersion::default())
}

async fn increment_version(db: &mut SqlitePool) -> Result<(), DatabaseUpgradeError> {
    let increment_result = query!(
        r#"
        INSERT OR ROLLBACK INTO `Version` (`version_number`)
        SELECT MAX(`version_number`) + 1 
        FROM `Version`;
        "#,
    )
    .execute(&*db)
    .await;

    match increment_result {
        Ok(query_result) => {
            if query_result.rows_affected() == 1 {
                Ok(())
            } else {
                Err(DatabaseUpgradeError::CouldNotIncrementVersion)
            }
        }
        Err(_) => Err(DatabaseUpgradeError::CouldNotIncrementVersion),
    }
}

pub async fn upgrade_db(db: &mut SqlitePool) -> Result<(), DatabaseUpgradeError> {
    let mut version = db_version(db).await.number;

    println!("Database version: {}", version).await;

    if version == 0 {
        println!("Upgrading database to version 1").await;
        let mut table_creation = sqlx::query!(
            r#"
            CREATE TABLE IF NOT EXISTS `Version` (
                `version_number` INTEGER PRIMARY KEY NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS `Article` (
                `article_id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                `timestamp_created` BIGINT NOT NULL,
                `timestamp_removed` BIGINT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS `Element` (
                `element_id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                `article_id` BIGINT UNSIGNED NOT NULL,
                `element_type_id` BIGINT UNSIGNED NOT NULL,

                FOREIGN KEY(`article_id`) REFERENCES `Article`(`article_id`),
                FOREIGN KEY(`element_type_id`) REFERENCES `ElementType`(`element_type_id`)
            );

            CREATE TABLE IF NOT EXISTS `DataVersion` (
                `data_version_id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                `element_id` BIGINT UNSIGNED NOT NULL,
                `data_id` BIGINT UNSIGNED NOT NULL,
                `timestamp_created` BIGINT UNSIGNED NOT NULL,
                `timestamp_removed` BIGINT UNSIGNED,
                `hidden` BOOLEAN NOT NULL DEFAULT FALSE,

                FOREIGN KEY(`element_id`) REFERENCES `Element`(`element_id`)
            );
            
            CREATE TABLE IF NOT EXISTS `ElementType` (
                `element_type_id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                `name` TEXT NOT NULL,
                `table_id` TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS `FieldType` (
                `field_type_id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                `name` TEXT NOT NULL
            );
            
            CREATE TABLE IF NOT EXISTS `ElementTypeField` (
                `element_type_field_id` INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                `element_type_id` BIGINT UNSIGNED NOT NULL,
                `field_type_id` BIGINT UNSIGNED NOT NULL,

                FOREIGN KEY(`element_type_id`) REFERENCES `ElementType`(`element_type_id`),
                FOREIGN KEY(`field_type_id`) REFERENCES `FeildType`(`field_type_id`)
            );
            "#
        )
        .execute_many(&*db)
        .await;

        while let Some(table_creation) = table_creation.next().await {
            match table_creation {
                Ok(_) => (),
                Err(err) => {
                    println!("{:?}", err).await;
                    return Err(DatabaseUpgradeError::FailedUpdatingTables);
                }
            }
        }

        let mut field_type_insert = query!(
            r#"
            INSERT INTO `FieldType` (
                `name`
            )
            VALUES (
                "PlainText"
            ), (
                "Timestamp"
            ), (
                "ReassociateMarkup"
            ), (
                "Integer"
            );

            INSERT INTO `Version` (`version_number`)
            VALUES (0);
            "#,
        )
        .execute_many(&*db)
        .await;

        while let Some(table_creation) = field_type_insert.next().await {
            match table_creation {
                Ok(_) => (),
                Err(err) => {
                    println!("{:?}", err).await;
                    return Err(DatabaseUpgradeError::FailedUpdatingTables);
                }
            }
        }

        increment_version(&mut *db).await?;
        version += 1;
        println!("Successfully upgraded database to version 1").await;
    }

    println!("Database upgraded to version {}", version).await;
    Ok(())
}
