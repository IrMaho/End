use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum SqliteError {
    ConnectionFailed(String),
    ExecutionFailed(String),
    QueryFailed(String),
    TransactionFailed(String),
    PreparedStatementFailed(String),
    InvalidParameter(String),
    LockingConflict(String),
    DatabaseNotFound(String),
    InvalidDatabaseFile(String),
}

impl fmt::Display for SqliteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SqliteError::ConnectionFailed(msg) => write!(f, "SQLite connection error: {}", msg),
            SqliteError::ExecutionFailed(msg) => write!(f, "SQLite execution error: {}", msg),
            SqliteError::QueryFailed(msg) => write!(f, "SQLite query error: {}", msg),
            SqliteError::TransactionFailed(msg) => write!(f, "SQLite transaction error: {}", msg),
            SqliteError::PreparedStatementFailed(msg) => write!(f, "SQLite prepared statement error: {}", msg),
            SqliteError::InvalidParameter(msg) => write!(f, "SQLite parameter error: {}", msg),
            SqliteError::LockingConflict(msg) => write!(f, "SQLite locking conflict: {}", msg),
            SqliteError::DatabaseNotFound(msg) => write!(f, "SQLite database not found: {}", msg),
            SqliteError::InvalidDatabaseFile(msg) => write!(f, "Invalid SQLite database file: {}", msg),
        }
    }
}

impl std::error::Error for SqliteError {}
