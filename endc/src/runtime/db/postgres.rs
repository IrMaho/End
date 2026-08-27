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

    /// Helper to decode a column value from a row into a serde_json::Value preserving NULLs
    fn decode_column_value(row: &tokio_postgres::Row, idx: usize, col_type: &Type) -> Value {
        if col_type == &Type::BOOL {
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
        }
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
                let val = Self::decode_column_value(&row, idx, col_type);
                obj.insert(col_name, val);
            }
            results.push(Value::Object(obj));
        }

        Ok(Value::Array(results))
    }

    /// Execute a query using dynamic JSON values as bound parameters
    pub fn query_json_params(&mut self, sql: &str, params: &[Value]) -> Result<Value, PgError> {
        let dyn_params: Vec<DynamicPgParam> = params.iter().map(DynamicPgParam::from_json).collect();
        let param_refs: Vec<&(dyn ToSql + Sync)> = dyn_params.iter().map(|p| p as &(dyn ToSql + Sync)).collect();
        self.query_json(sql, &param_refs)
    }

    /// Execute a statement using dynamic JSON values as bound parameters
    pub fn execute_params(&mut self, sql: &str, params: &[Value]) -> Result<u64, PgError> {
        let dyn_params: Vec<DynamicPgParam> = params.iter().map(DynamicPgParam::from_json).collect();
        let param_refs: Vec<&(dyn ToSql + Sync)> = dyn_params.iter().map(|p| p as &(dyn ToSql + Sync)).collect();
        self.execute(sql, &param_refs)
    }

    /// Prepare a statement and execute multiple times with different parameter sets
    pub fn prepare_and_query(&mut self, sql: &str, params_list: &[Vec<Value>]) -> Result<Vec<Value>, PgError> {
        self.runtime.block_on(async {
            let stmt = self.client.prepare(sql).await
                .map_err(|e| PgError::ExecutionFailed(format!("Failed to prepare statement '{}': {}", sql, e)))?;

            let mut results = Vec::new();
            for param_set in params_list {
                let dyn_params: Vec<DynamicPgParam> = param_set.iter().map(DynamicPgParam::from_json).collect();
                let param_refs: Vec<&(dyn ToSql + Sync)> = dyn_params.iter().map(|p| p as &(dyn ToSql + Sync)).collect();
                let rows = self.client.query(&stmt, &param_refs).await
                    .map_err(|e| PgError::QueryFailed(format!("Failed to execute prepared statement: {}", e)))?;

                let mut set_rows = Vec::new();
                for row in rows {
                    let mut obj = Map::new();
                    for (idx, col) in row.columns().iter().enumerate() {
                        let col_name = col.name().to_string();
                        let col_type = col.type_();
                        let val = Self::decode_column_value(&row, idx, col_type);
                        obj.insert(col_name, val);
                    }
                    set_rows.push(Value::Object(obj));
                }
                results.push(Value::Array(set_rows));
            }
            Ok(results)
        })
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

/// Dynamic parameter wrapper for ToSql trait
#[derive(Debug)]
pub enum DynamicPgParam {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
}

impl DynamicPgParam {
    pub fn from_json(val: &Value) -> Self {
        match val {
            Value::Null => DynamicPgParam::Null,
            Value::Bool(b) => DynamicPgParam::Bool(*b),
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    DynamicPgParam::Int(i)
                } else if let Some(f) = n.as_f64() {
                    DynamicPgParam::Float(f)
                } else {
                    DynamicPgParam::String(n.to_string())
                }
            }
            Value::String(s) => DynamicPgParam::String(s.clone()),
            _ => DynamicPgParam::String(val.to_string()),
        }
    }
}

impl ToSql for DynamicPgParam {
    fn to_sql(&self, ty: &Type, out: &mut bytes::BytesMut) -> Result<tokio_postgres::types::IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            DynamicPgParam::Null => Ok(tokio_postgres::types::IsNull::Yes),
            DynamicPgParam::Bool(b) => b.to_sql(ty, out),
            DynamicPgParam::Int(i) => {
                if ty == &Type::INT2 {
                    (*i as i16).to_sql(ty, out)
                } else if ty == &Type::INT4 {
                    (*i as i32).to_sql(ty, out)
                } else {
                    i.to_sql(ty, out)
                }
            }
            DynamicPgParam::Float(f) => {
                if ty == &Type::FLOAT4 {
                    (*f as f32).to_sql(ty, out)
                } else {
                    f.to_sql(ty, out)
                }
            }
            DynamicPgParam::String(s) => s.to_sql(ty, out),
        }
    }

    fn accepts(_ty: &Type) -> bool {
        true
    }

    tokio_postgres::types::to_sql_checked!();
}
