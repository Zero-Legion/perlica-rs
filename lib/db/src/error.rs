use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Failed to create saves dir {path}: {source}")]
    CreateDir {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to open SQLite database at {path}: {source}")]
    Open {
        path: PathBuf,
        #[source]
        source: sqlx::Error,
    },

    #[error("Failed to run embedded migrations: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("SQL error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Data corruption while loading {what} for uid={uid}: {reason}")]
    Corruption {
        uid: String,
        what: &'static str,
        reason: String,
    },
}

pub type Result<T> = std::result::Result<T, DbError>;
