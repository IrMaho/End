use serde_json::{json, Map, Value};
use std::fmt;
use tokio::runtime::Runtime;
use tokio_postgres::types::{ToSql, Type};
use tokio_postgres::{Client, NoTls};

#[derive(Debug, Clone, PartialEq)]
pub enum PgError {
    ConnectionFailed(String),
    ExecutionFailed(String),
    QueryFailed(String),
    TransactionFailed(String),
    ParameterBindingFailed(String),
    TypeConversionFailed(String),
}

impl fmt::Display for PgError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PgError::ConnectionFailed(msg) => write!(f, "PostgreSQL connection error: {}", msg),
            PgError::ExecutionFailed(msg) => write!(f, "PostgreSQL execution error: {}", msg),
            PgError::QueryFailed(msg) => write!(f, "PostgreSQL query error: {}", msg),
            PgError::TransactionFailed(msg) => write!(f, "PostgreSQL transaction error: {}", msg),
            PgError::ParameterBindingFailed(msg) => write!(f, "PostgreSQL parameter binding error: {}", msg),
            PgError::TypeConversionFailed(msg) => write!(f, "PostgreSQL type conversion error: {}", msg),
        }
    }
}

impl std::error::Error for PgError {}

pub struct PgEngine {
    client: Client,
    runtime: Runtime,
    pub conn_str: String,
    pub is_connected: bool,
}

impl PgEngine {
    /// Connect to a real PostgreSQL database server via wire protocol
    pub fn connect(conn_str: &str) -> Result<Self, PgError> {
        let rt = Runtime::new()
            .map_err(|e| PgError::ConnectionFailed(format!("Failed to initialize Tokio runtime: {}", e)))?;

        let (client, connection) = rt.block_on(async {
            tokio_postgres::connect(conn_str, NoTls).await
        }).map_err(|e| PgError::ConnectionFailed(format!("Failed to connect to PostgreSQL at '{}': {}", conn_str, e)))?;

        // Spawn connection task in background on runtime
        rt.spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("PostgreSQL connection error: {}", e);
            }
        });

        // Verify connection with `SELECT 1::INT4 as ping;`
        let rows = rt.block_on(async {
            client.query_one("SELECT 1::INT4 as ping;", &[]).await
        }).map_err(|e| PgError::ConnectionFailed(format!("PostgreSQL handshake verification failed: {}", e)))?;

        let ping_val: i32 = rows.get(0);
        if ping_val != 1 {
            return Err(PgError::ConnectionFailed("Unexpected ping response from PostgreSQL".to_string()));
        }

        Ok(Self {
            client,
            runtime: rt,
            conn_str: conn_str.to_string(),
            is_connected: true,
        })
    }

    /// Execute a SQL statement (CREATE, INSERT, UPDATE, DELETE) and return rows affected
    pub fn execute(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<u64, PgError> {
        self.runtime.block_on(async {
            if params.is_empty() && sql.contains(';') && sql.trim_end_matches(';').contains(';') {
                self.client.batch_execute(sql).await.map(|_| 1)
            } else {
                self.client.execute(sql, params).await
            }
        }).map_err(|e| PgError::ExecutionFailed(format!("Failed to execute SQL '{}': {}", sql, e)))
    }

    /// Execute a batch of SQL statements
    pub fn execute_batch(&mut self, sql: &str) -> Result<(), PgError> {
        self.runtime.block_on(async {
            self.client.batch_execute(sql).await
        }).map_err(|e| PgError::ExecutionFailed(format!("Failed to execute batch SQL: {}", e)))
    }

    /// Execute a SELECT query and return rows as JSON Value (Array of Objects), properly preserving NULL
    pub fn query_json(&mut self, sql: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Value, PgError> {
        let rows = self.runtime.block_on(async {
            self.client.query(sql, params).await
        }).map_err(|e| PgError::QueryFailed(format!("Failed to execute query '{}': {}", sql, e)))?;

        let mut results = Vec::new();
        for row in rows {
            let mut obj = Map::new();
            let columns = row.columns();
            for (idx, col) in columns.iter().enumerate() {
                let col_name = col.name().to_string();
                let col_type = col.type_();

                let val = if col_type == &Type::BOOL {
                    match row.get::<_, Option<bool>>(idx) {
                        Some(b) => json!(b),
                        None => Value::Null,
                    }
                } else if col_type == &Type::INT2 {
                    match row.get::<_, Option<i16>>(idx) {
                        Some(i) => json!(i),
                        None => Value::Null,
                    }
                } else if col_type == &Type::INT4 {
                    match row.get::<_, Option<i32>>(idx) {
                        Some(i) => json!(i),
                        None => Value::Null,
                    }
                } else if col_type == &Type::INT8 {
                    match row.get::<_, Option<i64>>(idx) {
                        Some(i) => json!(i),
                        None => Value::Null,
                    }
                } else if col_type == &Type::FLOAT4 {
                    match row.get::<_, Option<f32>>(idx) {
                        Some(f) => json!(f),
                        None => Value::Null,
                    }
                } else if col_type == &Type::FLOAT8 {
                    match row.get::<_, Option<f64>>(idx) {
                        Some(f) => json!(f),
                        None => Value::Null,
                    }
                } else if col_type == &Type::TEXT || col_type == &Type::VARCHAR || col_type == &Type::BPCHAR || col_type == &Type::NAME {
                    match row.get::<_, Option<String>>(idx) {
                        Some(s) => json!(s),
                        None => Value::Null,
                    }
                } else if col_type == &Type::JSON || col_type == &Type::JSONB {
                    match row.get::<_, Option<Value>>(idx) {
                        Some(v) => v,
                        None => Value::Null,
                    }
                } else if col_type == &Type::BYTEA {
                    match row.get::<_, Option<Vec<u8>>>(idx) {
                        Some(bytes) => {
                            let hex = bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>();
                            json!(hex)
                        }
                        None => Value::Null,
                    }
                } else {
                    match row.try_get::<_, Option<String>>(idx) {
                        Ok(Some(s)) => json!(s),
                        Ok(None) => Value::Null,
                        Err(_) => Value::Null,
                    }
                };

                obj.insert(col_name, val);
            }
            results.push(Value::Object(obj));
        }

        Ok(Value::Array(results))
    }

    /// Begin an explicit PostgreSQL transaction
    pub fn transaction_begin(&mut self) -> Result<(), PgError> {
        self.execute_batch("BEGIN;")
            .map_err(|e| PgError::TransactionFailed(format!("Failed to begin transaction: {}", e)))
    }

    /// Commit an explicit PostgreSQL transaction
    pub fn transaction_commit(&mut self) -> Result<(), PgError> {
        self.execute_batch("COMMIT;")
            .map_err(|e| PgError::TransactionFailed(format!("Failed to commit transaction: {}", e)))
    }

    /// Rollback an explicit PostgreSQL transaction
    pub fn transaction_rollback(&mut self) -> Result<(), PgError> {
        self.execute_batch("ROLLBACK;")
            .map_err(|e| PgError::TransactionFailed(format!("Failed to rollback transaction: {}", e)))
    }

    /// Close connection
    pub fn close(&mut self) {
        self.is_connected = false;
    }
}
