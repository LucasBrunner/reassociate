use std::{fmt::Display, time::SystemTime};

use chrono::naive::NaiveDateTime;
use polodb_core::{ClientSession, Config, ConfigBuilder, Database, TransactionType};
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use super::ReassociateDb;
}

pub const CURRENT_VERSION: u64 = 1;

pub enum DatabaseOpenError {
    Upgrade {
        current_version_number: u64,
        db_version: DatabaseVersion,
    },
    Open(polodb_core::Error),
    FindVersion,
}

impl Display for DatabaseOpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display_string = match self {
            DatabaseOpenError::Upgrade {
                current_version_number,
                db_version,
            } => format!(
                "Failed to upgrade to version {}, current version: {}",
                current_version_number, db_version.version
            ),
            DatabaseOpenError::Open(err) => format!("{}", err),
            DatabaseOpenError::FindVersion => "Could not find database version".to_owned(),
        };
        f.write_str(&display_string)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseVersion {
    version: u64,
    timestamp: i64,
}

impl Display for DatabaseVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "version: {}, date: {}",
            self.version,
            NaiveDateTime::from_timestamp(self.timestamp, 0)
        ))
    }
}

impl DatabaseVersion {
    fn new(version: u64, timestamp: i64) -> Self {
        Self { version, timestamp }
    }
}

pub struct UpgradeActions(pub Vec<String>);

impl UpgradeActions {
    fn new() -> Self {
        Self(Vec::new())
    }
}

#[derive(strum::Display)]
pub enum ConstantTables {
    DatabaseVersion,
    Article,
    Element,
    ArticleHistory,
}

impl ConstantTables {
    fn name(&self) -> String {
        format!("{}", self)
    }
}

pub struct ReassociateDb(Database);

fn upgrade_to_version_1(
    db: &mut ReassociateDb,
    session: &mut ClientSession,
) -> Result<(), polodb_core::Error> {
    db.0.create_collection_with_session(&ConstantTables::DatabaseVersion.name(), session)?;
    db.0.create_collection_with_session(&ConstantTables::Article.name(), session)?;
    db.0.create_collection_with_session(&ConstantTables::Element.name(), session)?;
    db.0.create_collection_with_session(&ConstantTables::ArticleHistory.name(), session)?;

    db.set_version(0, session);
    Ok(())
}

impl ReassociateDb {
    pub fn inner(&mut self) -> &Database {
        &self.0
    }

    pub fn version(&self) -> Result<DatabaseVersion, ()> {
        let mut version_history = match self
            .0
            .collection::<DatabaseVersion>(&ConstantTables::DatabaseVersion.name())
            .find(None)
        {
            Ok(vh) => vh,
            Err(_) => return Err(()),
        };

        let version_history = version_history.collect::<Vec<_>>();
        println!("{:?}", version_history);
        Ok(version_history
            .into_iter()
            .flat_map(|item| item.ok())
            .max_by(|first, second| first.version.cmp(&second.version))
            .unwrap_or(DatabaseVersion {
                version: 0,
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            }))
    }

    pub fn start_transaction(
        &mut self,
        transaction_type: Option<TransactionType>,
    ) -> Result<ClientSession, ()> {
        let Ok(mut session) = self.0.start_session() else {
            return Err(());
        };
        if session.start_transaction(transaction_type).is_err() {
            return Err(());
        }
        Ok(session)
    }

    fn set_version(&mut self, new_version: u64, session: &mut ClientSession) -> Result<(), ()> {
        let version_history = self
            .0
            .collection::<DatabaseVersion>(&ConstantTables::DatabaseVersion.name());
        match version_history.insert_one_with_session(
            DatabaseVersion {
                version: new_version,
                timestamp: SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            },
            session,
        ) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn upgrade(mut self) -> Result<(Self, UpgradeActions), DatabaseOpenError> {
        let version = match self.version() {
            Ok(v) => v,
            Err(_) => return Err(DatabaseOpenError::FindVersion),
        };

        let mut upgrade_actions = UpgradeActions::new();

        if version.version == 0 {
            let mut session = match self.start_transaction(Some(TransactionType::Write)) {
                Ok(s) => s,
                Err(_) => {
                    return Err(DatabaseOpenError::Upgrade {
                        current_version_number: CURRENT_VERSION,
                        db_version: version,
                    });
                }
            };

            match upgrade_to_version_1(&mut self, &mut session) {
                Ok(_) => (),
                Err(_) => {
                    _ = session.abort_transaction();
                    return Err(DatabaseOpenError::Upgrade {
                        current_version_number: CURRENT_VERSION,
                        db_version: version,
                    });
                }
            };

            match self.set_version(version.version + 1, &mut session) {
                Ok(_) => (),
                Err(_) => {
                    _ = session.abort_transaction();
                    return Err(DatabaseOpenError::Upgrade {
                        current_version_number: CURRENT_VERSION,
                        db_version: version,
                    });
                }
            };

            upgrade_actions
                .0
                .push("Upgraded database to version 1".to_owned());
            println!("{:?}", session.commit_transaction());
        }

        upgrade_actions
            .0
            .push("Database at current version".to_owned());

        println!("{:?}", self.version());

        Ok((self, upgrade_actions))
    }

    pub fn get(location: &str) -> Result<ReassociateDb, DatabaseOpenError> {
        let mut config = ConfigBuilder::new();
        config
            .set_sync_log_count(0)
            .set_journal_full_size(0)
            .set_init_block_count(0);
        let db = match Database::open_file_with_config(location, config.take()) {
            Ok(db) => db,
            Err(err) => {
                println!("{:?}", err);
                return Err(DatabaseOpenError::Open(err));
            }
        };
        let db = ReassociateDb(db);

        let (db, upgrade_actions) = db.upgrade()?;
        upgrade_actions
            .0
            .iter()
            .for_each(|action| println!("{}", action));
        Ok(db)
    }
}
