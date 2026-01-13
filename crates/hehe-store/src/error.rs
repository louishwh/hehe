use hehe_core::error::Error as CoreError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StoreError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Query error: {0}")]
    Query(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Backend not available: {0}")]
    BackendNotAvailable(String),

    #[error("Pool exhausted")]
    PoolExhausted,

    #[error("Timeout")]
    Timeout,

    #[error("Internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Core(#[from] CoreError),

    #[cfg(feature = "sqlite")]
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[cfg(feature = "duckdb")]
    #[error("DuckDB error: {0}")]
    DuckDb(#[from] duckdb::Error),
}

pub type Result<T> = std::result::Result<T, StoreError>;

impl StoreError {
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    pub fn query(msg: impl Into<String>) -> Self {
        Self::Query(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    pub fn transaction(msg: impl Into<String>) -> Self {
        Self::Transaction(msg.into())
    }

    pub fn migration(msg: impl Into<String>) -> Self {
        Self::Migration(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}
