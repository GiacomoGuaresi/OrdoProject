//! Error type of the core library.

use std::path::PathBuf;

/// Errors returned by `ordo-core`.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error on {path}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("invalid JSON in {path}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("unsupported file version {found} in {path} (max supported: {supported})")]
    UnsupportedVersion {
        path: PathBuf,
        found: u32,
        supported: u32,
    },

    #[error("invalid pattern in rule \"{rule}\": {message}")]
    InvalidPattern { rule: String, message: String },

    #[error("invalid rule: {0}")]
    InvalidRule(String),

    #[error("rule not found: {0}")]
    RuleNotFound(String),

    #[error("history database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("batch not found: {0}")]
    BatchNotFound(i64),

    #[error("move not found: {0}")]
    MoveNotFound(i64),

    #[error("cannot determine the user's home directory")]
    NoHomeDir,
}

impl Error {
    pub(crate) fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Error::Io {
            path: path.into(),
            source,
        }
    }
}

/// Result type of the core library.
pub type Result<T> = std::result::Result<T, Error>;
