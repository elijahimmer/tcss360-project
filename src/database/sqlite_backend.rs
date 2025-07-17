//! The SQLite Database backend!
//!
//! TODO: Alert the user in the game when there is a database issue.
//!       Be it at startup or at runtime.
use super::*;

use bevy::prelude::*;
use sqlite::ConnectionThreadSafe;

use std::cmp::Ordering;

use const_format::formatcp;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

pub type DatabaseError = sqlite::Error;

type Version = i64;

const DB_VERSION: Version = 3;

const ADD_SCHEMA: &str = formatcp!(
    r#"
BEGIN TRANSACTION;

CREATE TABLE Version(
  version INTEGER PRIMARY KEY
) STRICT;

INSERT INTO Version VALUES ({DB_VERSION});

CREATE TABLE KeyValue(
    key   TEXT PRIMARY KEY,
    value ANY
);

CREATE TABLE Keybinds(
    keybind TEXT PRIMARY KEY,
    key1    TEXT,
    key2    TEXT
) STRICT;

CREATE TABLE Colors(
    name  TEXT PRIMARY KEY,
    color INTEGER
) STRICT;

COMMIT;
"#
);

#[derive(Resource)]
pub struct Database {
    pub connection: ConnectionThreadSafe,
}

impl Database {
    pub fn open() -> Result<Self, OpenError> {
        let mut path = get_default_db_directory();
        path.push("database.sqlite");

        let exists = path.exists();
        let db = {
            let connection = match sqlite::Connection::open_thread_safe(&path) {
                Ok(conn) => conn,
                Err(err) => {
                    warn!(
                        "Failed to open database at '{}' with error: {err}",
                        path.display()
                    );
                    sqlite::Connection::open_thread_safe(":memory:")?
                }
            };
            Self {
                connection: connection,
            }
        };

        if exists {
            info!("Using existing database at '{}'!", path.display());
            match check_version(&db)? {
                VersionCompatability::Future(v) => {
                    error!(
                        "Database is from a future version {v} compared to current version {DB_VERSION}! You may be running an outdated version of the game"
                    );
                    return Err(OpenError::IncompatableVersion(v));
                }
                VersionCompatability::Same => {
                    info!("Database version is up to date!");
                }
                VersionCompatability::Migratable(v) => {
                    warn!(
                        "Database version is out dated, but migrateable. Backing up database then attempting migration..."
                    );

                    if let Err(err) = backup_database() {
                        error!("Failed to back up database before migration! {err}");
                        return Err(err.into());
                    }

                    info!("Backup successful! Migrating from database version {v} to {DB_VERSION}");

                    if let Err(err) = migrate_database(&db, v) {
                        error!("Failed to migrate database with error {err}");
                        return Err(err.into());
                    }

                    info!("Database migration successful!");
                }
                VersionCompatability::Incompatable(v) => {
                    error!(
                        "Database version is out dated, and not migrateable. Version is {v} when expected in the range of versions {MIN_VERSION_MIGRATEABLE} to {DB_VERSION}"
                    );
                    error!(
                        "Ask the developers to help get your data back, or on how to delete it to proceed!"
                    );
                    return Err(OpenError::IncompatableVersion(v));
                }
            }
        } else {
            info!("Database not found! Creating it at '{}'!", path.display());
            db.connection.execute(ADD_SCHEMA)?;
        }

        info!("Running database validation checks.");
        match validate_schema(&db) {
            Ok(()) => {}
            Err(err) => {
                error!("Failed to validate SQLite Table with error {err}.");
                error!(
                    "Ask the developers to help get your data back, or on how to delete it to proceed!"
                );
                return Err(OpenError::ValidationFailed(err));
            }
        };
        info!("Passed database validation checks.");

        Ok(db)
    }

    pub fn get_kv_direct<T: sqlite::ReadableWithIndex>(
        &self,
        key: &str,
    ) -> Result<Option<T>, DatabaseError> {
        let query = "SELECT value FROM KeyValue WHERE key = :key";
        let mut statement = self.connection.prepare(query)?;

        statement.bind((":key", key))?;

        if let sqlite::State::Done = statement.next()? {
            return Ok(None);
        }
        assert_eq!(
            statement.column_count(),
            2,
            "There should only be 2 columns if it is a single table like this."
        );

        // read the value column index.
        let value = statement.read::<Option<T>, usize>(2)?;

        assert!(matches!(statement.next()?, sqlite::State::Done));

        Ok(value)
    }

    pub fn get_kv<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, GetKvError> {
        Ok(self
            .get_kv_direct::<String>(key)?
            .as_deref()
            .map(|str| ron::from_str(str))
            .transpose()?)
    }

    pub fn set_kv_direct<T: sqlite::BindableWithIndex>(
        &self,
        key: &str,
        value: T,
    ) -> Result<(), DatabaseError> {
        let query = "INSERT INTO KeyValue VALUES (:key, :value)";
        let mut statement = self.connection.prepare(query)?;
        statement.bind((":key", key))?;
        statement.bind((":value", value))?;

        assert!(matches!(statement.next()?, sqlite::State::Done));

        Ok(())
    }

    pub fn set_kv<T: Serialize>(&self, key: &str) -> Result<(), SetKvError> {
        Ok(self.set_kv_direct(key, ron::to_string(key)?.as_str())?)
    }
}

#[derive(Error, Debug)]
pub enum OpenError {
    #[error("Failed to backup database with {0}!")]
    BackupFailed(#[from] BackupError),
    #[error("Migration failed with {0}!")]
    MigrationFailed(#[from] MigrationError),
    #[error("Version Incompatable found version `{0}`!")]
    IncompatableVersion(Version),
    #[error("Version check failed with `{0}`")]
    CheckVersionError(#[from] CheckVersionError),
    #[error("Schema valdation failed with `{0}`")]
    ValidationFailed(#[from] ValidateSchemaError),
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

#[derive(Error, Debug)]
pub enum CheckVersionError {
    #[error("No version found in database!")]
    VersionNotFound,
    #[error("Version table incompatable! Assuming data is invalid.")]
    IncompatableVersionTable,
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

pub enum VersionCompatability {
    Same,
    Future(Version),
    Migratable(Version),
    Incompatable(Version),
}

#[derive(Error, Debug)]
pub enum GetKvError {
    #[error("Failed to deserialize value with error `{0}`")]
    DeserializerError(#[from] ron::error::SpannedError),
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

#[derive(Error, Debug)]
pub enum SetKvError {
    #[error("Failed to serialize value with error `{0}`")]
    SerializeError(#[from] ron::Error),
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

fn check_version(db: &Database) -> Result<VersionCompatability, CheckVersionError> {
    let mut statement = db.connection.prepare("SELECT version FROM Version;")?;

    if !matches!(statement.next()?, sqlite::State::Row) {
        error!("No version found in database!");
        return Err(CheckVersionError::VersionNotFound);
    }

    if statement.column_count() != 1 {
        warn!("Version entry contains invalid values!");
        return Err(CheckVersionError::IncompatableVersionTable);
    }

    let version = match statement.read::<i64, usize>(0) {
        Ok(v) => v,
        Err(err) => {
            warn!("Version entry not found in table with error: {err}");
            return Err(CheckVersionError::VersionNotFound);
        }
    };

    if let sqlite::State::Row = statement.next()? {
        warn!("Malformed version table! Expected only 1 entry, found multiple!");
        return Err(CheckVersionError::IncompatableVersionTable);
    }

    Ok(match version.cmp(&DB_VERSION) {
        Ordering::Equal => VersionCompatability::Same,
        Ordering::Less if version >= MIN_VERSION_MIGRATEABLE => {
            VersionCompatability::Migratable(version)
        }
        Ordering::Less => VersionCompatability::Incompatable(version),
        Ordering::Greater => VersionCompatability::Future(version),
    })
}

#[derive(Error, Debug)]
pub enum ValidateSchemaError {
    #[error("SQLite table '{0}' failed validation!")]
    Invalid(Box<str>),
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

fn validate_schema(db: &Database) -> Result<(), ValidateSchemaError> {
    let mut statement = db
        .connection
        .prepare("PRAGMA integrity_check; PRAGMA optimize;")?;
    assert!(matches!(statement.next()?, sqlite::State::Row));
    assert!(matches!(statement.next()?, sqlite::State::Done));

    validate_table(db, "Version", &[("version", "INTEGER")])?;
    validate_table(db, "KeyValue", &[("key", "TEXT"), ("value", "ANY")])?;
    validate_table(
        db,
        "Keybinds",
        &[("keybind", "TEXT"), ("key1", "TEXT"), ("key2", "TEXT")],
    )?;

    //validate_table(db, "SaveGame", &[("created", "DATETIME"), ("rand", "TEXT")])?;

    Ok(())
}

fn validate_table(
    db: &Database,
    table_name: &str,
    contents: &[(&str, &str)],
) -> Result<(), ValidateSchemaError> {
    let query = format!("PRAGMA table_info({table_name});");
    let mut statement = db.connection.prepare(query)?;

    for (expected_name, expected_ctype) in contents.iter() {
        if let sqlite::State::Done = statement.next()? {
            error!("SQLite table `{table_name}` missing column 'expected_name'!");
            return Err(ValidateSchemaError::Invalid(table_name.into()));
        }

        let name = statement.read::<String, usize>(1).unwrap();
        let ctype = statement.read::<String, usize>(2).unwrap();

        if &name != expected_name {
            error!(
                "SQLite table `{table_name}` found column `{name}` yet expected column `{expected_name}`"
            );
            return Err(ValidateSchemaError::Invalid(table_name.into()));
        }
        if &ctype != expected_ctype {
            error!(
                "SQLite table `{table_name}` found column `{name}` of type `{ctype}` yet expected the type `{expected_ctype}`"
            );
            return Err(ValidateSchemaError::Invalid(table_name.into()));
        }
    }

    if !matches!(statement.next()?, sqlite::State::Done) {
        let next_column = statement.read::<String, usize>(1)?;
        error!("SQLite table `{table_name}` has unexpected column '{next_column}'");
    };

    Ok(())
}

#[derive(Error, Debug)]
pub enum BackupError {
    #[error("Failed to find migration script!")]
    NoMigrationScript,
    #[error("Failed to save backup with error: {0}")]
    FileError(#[from] std::io::Error),
}

///
fn backup_database() -> Result<(), BackupError> {
    let mut db_path = get_default_db_directory();
    db_path.push("database.sqlite");

    let mut backup_path = get_default_db_directory();
    backup_path.push(format!("{}_database.sqlite.backup", chrono::offset::Utc::now().format("%+")));

    // While theoretically now bounded, this should be bounded in practice.
    while backup_path.exists() {
        backup_path.set_file_name(format!("{}-database.sqlite.backup", chrono::offset::Utc::now().format("%+")));
    }

    std::fs::copy(db_path, backup_path)?;

    Ok(())
}

#[derive(Error, Debug)]
pub enum MigrationError {
    #[error("Failed to find migration script!")]
    NoMigrationScript,
    #[error("SQLite error occured: `{0}`")]
    DatabaseError(#[from] DatabaseError),
}

const MIN_VERSION_MIGRATEABLE: Version = 3;
/// Make sure the migrations are set up properly
const _: () = assert!(DB_VERSION == 3, "UPDATE THE MIGRATION SCRIPT");

/// MAINTENANCE: UPDATE EVERY DATABASE UPDGRADE
fn migrate_database(db: &Database, from: Version) -> Result<(), MigrationError> {
    assert!((MIN_VERSION_MIGRATEABLE..DB_VERSION).contains(&from));

    let mut from = from;
    _ = &mut from;
    _ = db;

    //if (from == 1) {
    //    migrate_from_1_to_2(db)?;
    //    from = 2;
    //}

    assert_eq!(
        from, DB_VERSION,
        "Failed to find migration script to migrate fully."
    );
    Ok(())
}

//fn migrate_from_1_to_2(db: &Database) -> Result<(), DatabaseError> {
//    let query = r#"
//        BEGIN TRANSACTION;
//        CREATE TABLE Colors(
//            name  VARCHAR(16) PRIMARY KEY,
//            color MEDIUMINTEGER
//        );
//        UPDATE Version SET version = 2;
//        COMMIT;
//    "#;
//
//    db.connection.execute(query)?;
//
//    Ok(())
//}
