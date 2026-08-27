use super::error::SqliteError;
use rusqlite::types::ValueRef;
use rusqlite::{params_from_iter, Connection, Statement, ToSql};
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::path::Path;

pub struct SqliteEngine {
    pub conn: Connection,
    pub path: String,
    statements: HashMap<usize, String>,
    next_stmt_id: usize,
}

impl SqliteEngine {
    /// Open a real SQLite database file or in-memory instance
    pub fn open(path: &str) -> Result<Self, SqliteError> {
        let conn = if path == ":memory:" || path.is_empty() {
            Connection::open_in_memory()
                .map_err(|e| SqliteError::ConnectionFailed(format!("Failed to open in-memory SQLite database: {}", e)))?
        } else {
            // Ensure parent directory exists
            if let Some(parent) = Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            Connection::open(path)
                .map_err(|e| SqliteError::ConnectionFailed(format!("Failed to open SQLite database at '{}': {}", path, e)))?
        };

        // Enable foreign keys by default
        let _ = conn.execute("PRAGMA foreign_keys = ON;", []);

        Ok(Self {
            conn,
            path: path.to_string(),
            statements: HashMap::new(),
            next_stmt_id: 1,
        })
    }

    /// Execute a SQL statement (CREATE TABLE, INSERT, UPDATE, DELETE) and return rows affected
    pub fn execute(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<usize, SqliteError> {
        self.conn
            .execute(sql, params)
            .map_err(|e| SqliteError::ExecutionFailed(format!("Failed to execute SQL '{}': {}", sql, e)))
    }

    /// Execute a batch of SQL statements
    pub fn execute_batch(&mut self, sql: &str) -> Result<(), SqliteError> {
        self.conn
            .execute_batch(sql)
            .map_err(|e| SqliteError::ExecutionFailed(format!("Failed to execute SQL batch: {}", e)))
    }

    /// Execute a SELECT query and return rows as a JSON Value (Array of Objects)
    pub fn query_json(&mut self, sql: &str, params: &[&dyn ToSql]) -> Result<Value, SqliteError> {
        let mut stmt = self
            .conn
            .prepare(sql)
            .map_err(|e| SqliteError::QueryFailed(format!("Failed to prepare query '{}': {}", sql, e)))?;

        let col_names: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mut rows = stmt
            .query(params)
            .map_err(|e| SqliteError::QueryFailed(format!("Query execution failed for '{}': {}", sql, e)))?;

        let mut results = Vec::new();
        while let Some(row) = rows
            .next()
            .map_err(|e| SqliteError::QueryFailed(format!("Error fetching row: {}", e)))?
        {
            let mut obj = Map::new();
            for (idx, col_name) in col_names.iter().enumerate() {
                let val_ref = row
                    .get_ref(idx)
                    .map_err(|e| SqliteError::QueryFailed(format!("Error reading column '{}': {}", col_name, e)))?;
                let val = match val_ref {
                    ValueRef::Null => Value::Null,
                    ValueRef::Integer(i) => json!(i),
                    ValueRef::Real(f) => json!(f),
                    ValueRef::Text(t) => {
                        let text = String::from_utf8_lossy(t);
                        json!(text)
                    }
                    ValueRef::Blob(b) => {
                        let hex_str = b.iter().map(|byte| format!("{:02x}", byte)).collect::<String>();
                        json!(hex_str)
                    }
                };
                obj.insert(col_name.clone(), val);
            }
            results.push(Value::Object(obj));
        }

        Ok(Value::Array(results))
    }

    /// Begin an explicit transaction
    pub fn transaction_begin(&mut self) -> Result<(), SqliteError> {
        self.execute_batch("BEGIN TRANSACTION;")
            .map_err(|e| SqliteError::TransactionFailed(format!("Failed to begin transaction: {}", e)))
    }

    /// Commit an explicit transaction
    pub fn transaction_commit(&mut self) -> Result<(), SqliteError> {
        self.execute_batch("COMMIT;")
            .map_err(|e| SqliteError::TransactionFailed(format!("Failed to commit transaction: {}", e)))
    }

    /// Rollback an explicit transaction
    pub fn transaction_rollback(&mut self) -> Result<(), SqliteError> {
        self.execute_batch("ROLLBACK;")
            .map_err(|e| SqliteError::TransactionFailed(format!("Failed to rollback transaction: {}", e)))
    }

    /// Register a prepared statement SQL string and return an ID handle
    pub fn prepare_statement(&mut self, sql: &str) -> Result<usize, SqliteError> {
        // Validate SQL syntax by preparing once
        let _ = self
            .conn
            .prepare(sql)
            .map_err(|e| SqliteError::PreparedStatementFailed(format!("Invalid prepared statement SQL '{}': {}", sql, e)))?;

        let id = self.next_stmt_id;
        self.next_stmt_id += 1;
        self.statements.insert(id, sql.to_string());
        Ok(id)
    }

    /// Execute a registered prepared statement with parameters
    pub fn execute_prepared(&mut self, stmt_id: usize, params: &[&dyn ToSql]) -> Result<usize, SqliteError> {
        let sql = self
            .statements
            .get(&stmt_id)
            .ok_or_else(|| SqliteError::PreparedStatementFailed(format!("Prepared statement handle {} not found", stmt_id)))?
            .clone();

        let mut stmt = self
            .conn
            .prepare(&sql)
            .map_err(|e| SqliteError::PreparedStatementFailed(format!("Failed to prepare SQL: {}", e)))?;

        stmt.execute(params)
            .map_err(|e| SqliteError::PreparedStatementFailed(format!("Failed to execute prepared statement: {}", e)))
    }

    /// Query a registered prepared statement with parameters
    pub fn query_prepared_json(&mut self, stmt_id: usize, params: &[&dyn ToSql]) -> Result<Value, SqliteError> {
        let sql = self
            .statements
            .get(&stmt_id)
            .ok_or_else(|| SqliteError::PreparedStatementFailed(format!("Prepared statement handle {} not found", stmt_id)))?
            .clone();

        self.query_json(&sql, params)
    }

    // --- Key-Value Backward Compatibility over Real SQLite ---

    fn ensure_kv_table(&mut self) -> Result<(), SqliteError> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS __end_kv_store (k TEXT PRIMARY KEY, v TEXT NOT NULL);",
                [],
            )
            .map_err(|e| SqliteError::ExecutionFailed(format!("Failed to initialize __end_kv_store table: {}", e)))?;
        Ok(())
    }

    /// Store key-value pair in SQLite
    pub fn kv_set(&mut self, key: &str, value: &str) -> Result<usize, SqliteError> {
        self.ensure_kv_table()?;
        self.conn
            .execute(
                "INSERT INTO __end_kv_store (k, v) VALUES (?1, ?2) ON CONFLICT(k) DO UPDATE SET v = excluded.v;",
                [key, value],
            )
            .map_err(|e| SqliteError::ExecutionFailed(format!("Failed to set key '{}' in SQLite: {}", key, e)))
    }

    /// Retrieve key-value pair from SQLite
    pub fn kv_get(&mut self, key: &str) -> Result<Option<String>, SqliteError> {
        self.ensure_kv_table()?;
        let mut stmt = self
            .conn
            .prepare("SELECT v FROM __end_kv_store WHERE k = ?1;")
            .map_err(|e| SqliteError::QueryFailed(format!("Failed to query key '{}': {}", key, e)))?;

        let mut rows = stmt
            .query([key])
            .map_err(|e| SqliteError::QueryFailed(format!("Query failed for key '{}': {}", key, e)))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| SqliteError::QueryFailed(format!("Error fetching value for key '{}': {}", key, e)))?
        {
            let val: String = row.get(0).map_err(|e| SqliteError::QueryFailed(format!("Type error: {}", e)))?;
            Ok(Some(val))
        } else {
            Ok(None)
        }
    }
}
