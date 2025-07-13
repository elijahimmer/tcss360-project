use bevy::prelude::*;
use std::cmp::Ordering;
use std::path::Path;

use const_format::formatcp;
use directories::ProjectDirs;
use serde::{Serialize, de::DeserializeOwned};
use sqlite::ConnectionThreadSafe;
use thiserror::Error;

type Version = i64;

const DB_VERSION: Version = 1;
const MIN_VERSION_MIGRATEABLE: Version = 1;

const GET_VERSION: &str = r#"
SELECT version FROM Version;
"#;

const ADD_SCHEMA: &str = formatcp!(
    r#"
BEGIN TRANSACTION;
CREATE TABLE Version(
  version INTEGER PRIMARY KEY
) WITHOUT ROWID;

INSERT INTO Version VALUES ({DB_VERSION});

CREATE TABLE KeyValue(
    key   VARCHAR(32) PRIMARY KEY,
    value TEXT
);

CREATE TABLE Keybinds(
    keybind VARCHAR(16) PRIMARY KEY,
    key1    MEDIUMINTEGER UNIQUE,
    key2    MEDIUMINTEGER UNIQUE
);

CREATE TABLE SaveGame(
    created DATETIME PRIMARY KEY,
    rand TEXT
);
COMMIT;
"#
);

const SCHEMA_VALIDATE: &str = r#"
PRAGMA integrity_check;
PRAGMA optimize;
"#;

pub struct DatabasePlugin;

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(
            Database::open()
                .inspect_err(|e| error!("Failed to open database with: {e}"))
                .unwrap(),
        );
    }
}

pub trait FromDatabase {
    /// Cannot fail, must resort to defaults.
    fn from_database(database: &Database) -> Self;
}

#[derive(Resource)]
pub struct Database {
    pub connection: ConnectionThreadSafe,
}

impl Database {
    pub fn open() -> Result<Self, OpenError> {
        let path = get_default_db_path();
        let exists = path.exists();
        let mut db = Self {
            // TODO: Open in memory when unable to open file otherwise.
            connection: sqlite::Connection::open_thread_safe(&path)?,
        };

        if exists {
            info!("Using existing database at '{}'!", path.display());
            match check_version(&mut db.connection)? {
                VersionCompatability::Future(v) => {
                    error!(
                        "Database is from a future version {v} compared to current version {DB_VERSION}, and is thus incompatable!"
                    );
                    return Err(OpenError::IncompatableVersion(v));
                }
                VersionCompatability::Same => {
                    info!("Database version is up to date! proceeding!");
                }
                VersionCompatability::Migratable(v) => {
                    warn!(
                        "Database version is out dated, but migrateable migrating from {v} to {DB_VERSION} . Backing up database then attempting migration..."
                    );
                    // This is out of the scope of the project, but good for the future.
                    todo!("Setup database migrations as needed/wanted!");
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
        db.connection.execute(SCHEMA_VALIDATE)?;
        info!("Passed database validation checks.");

        Ok(db)
    }

    pub fn get_kv_direct<T: sqlite::ReadableWithIndex>(
        &self,
        key: &str,
    ) -> Result<Option<T>, sqlite::Error> {
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

    pub fn set_kv_direct<T: sqlite::BindableWithIndex>(&self, key: &str, value: T) -> Result<(), sqlite::Error> {
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
    #[error("Version Incompatable found version `{0}`!")]
    IncompatableVersion(Version),
    #[error("Version Check Failed")]
    CheckVersionError(#[from] CheckVersionError),
    #[error("SQLite error occured: `{0}`")]
    SqliteError(#[from] sqlite::Error),
}

#[derive(Error, Debug)]
pub enum CheckVersionError {
    #[error("Version not found!")]
    VersionNotFound,
    #[error("Version table incompatable! Assuming data is invalid.")]
    IncompatableVersionTable,
    #[error("SQLite error occured: `{0}`")]
    SqliteError(#[from] sqlite::Error),
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
    SqliteError(#[from] sqlite::Error),
}

#[derive(Error, Debug)]
pub enum SetKvError {
    #[error("Failed to serialize value with error `{0}`")]
    SerializeError(#[from] ron::Error),
    #[error("SQLite error occured: `{0}`")]
    SqliteError(#[from] sqlite::Error),
}

fn check_version(
    connection: &ConnectionThreadSafe,
) -> Result<VersionCompatability, CheckVersionError> {
    let mut version_query = connection.prepare(GET_VERSION)?;
    let mut res = version_query.iter();

    let version = res.try_next()?;
    let version = match version.as_deref() {
        Some([sqlite::Value::Integer(version)]) => *version,
        Some(_) => {
            warn!("Version entry contains invalid values!");
            return Err(CheckVersionError::IncompatableVersionTable.into());
        }
        None => {
            warn!("Version entry not found in table!");
            return Err(CheckVersionError::VersionNotFound.into());
        }
    };
    if let Some(_) = res.try_next()? {
        warn!("Malformed version table! Expected only 1 entry, found multiple!");
        return Err(CheckVersionError::IncompatableVersionTable.into());
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

fn get_default_db_path() -> Box<Path> {
    let project_dir = ProjectDirs::from("com", "TeamCounterSpell", "TCSS360-Project");
    let config_dir = match project_dir.as_ref().map(|d| d.config_dir()) {
        Some(config_dir) if config_dir.is_dir() => config_dir,
        Some(config_dir) => {
            info!("Config directory not found! creating directory!");
            std::fs::DirBuilder::new()
                .recursive(true)
                .create(config_dir)
                .and(Ok(config_dir))
                .inspect_err(|e| warn!("Failed to create config directory with: {e}. Resorting to using local directory!"))
                .unwrap_or(Path::new(""))
        }
        None => Path::new(""),
    };

    config_dir.join("data.sqlite").into()
}
