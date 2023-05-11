use std::time::SystemTime;

use polodb_core::{ClientSession, Database, TransactionType};
use serde::{Deserialize, Serialize};

pub const CURRENT_VERSION: u64 = 1;

pub enum DatabaseOpenError {
    Upgrade {
        current_version_number: u64,
        db_version: DatabaseVersion,
    },
    Open(polodb_core::Error),
    FindVersion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseVersion {
    version: u64,
    timestamp: i64,
}

pub struct ReassociateDb(Database);

fn upgrade_to_version_1(
    db: &mut ReassociateDb,
    session: &mut ClientSession,
) -> Result<(), polodb_core::Error> {
    db.0.create_collection_with_session("database version", session)?;
    db.0.create_collection_with_session("article", session)?;
    db.0.create_collection_with_session("element", session)?;
    db.0.create_collection_with_session("article history", session)?;

    todo!()
}

impl ReassociateDb {
    pub fn version(&self) -> Result<DatabaseVersion, ()> {
        let version_history = match self
            .0
            .collection::<DatabaseVersion>("database version")
            .find(None)
        {
            Ok(vh) => vh,
            Err(_) => return Err(()),
        };

        Ok(version_history
            .filter_map(|item| item.ok())
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
        let version_history = self.0.collection::<DatabaseVersion>("database version");
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

    fn upgrade(mut self) -> Result<Self, DatabaseOpenError> {
        let version = match self.version() {
            Ok(v) => v,
            Err(_) => return Err(DatabaseOpenError::FindVersion),
        };

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
        }

        Ok(self)
    }

    pub fn get(location: &str) -> Result<ReassociateDb, DatabaseOpenError> {
        let db = match Database::open_file(location) {
            Ok(db) => db,
            Err(err) => return Err(DatabaseOpenError::Open(err)),
        };
        let db = ReassociateDb(db);

        db.upgrade()
    }
}
