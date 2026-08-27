use super::error::RaftError;
use super::types::LogEntry;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;

pub struct SqliteRaftStorage {
    conn: Connection,
    pub path: String,
}

impl SqliteRaftStorage {
    pub fn open(path: &str) -> Result<Self, RaftError> {
        let conn = if path == ":memory:" || path.is_empty() {
            Connection::open_in_memory()
                .map_err(|e| RaftError::StorageError(format!("Failed to open in-memory SQLite: {}", e)))?
        } else {
            if let Some(parent) = Path::new(path).parent() {
                if !parent.as_os_str().is_empty() {
                    let _ = std::fs::create_dir_all(parent);
                }
            }
            Connection::open(path)
                .map_err(|e| RaftError::StorageError(format!("Failed to open SQLite at '{}': {}", path, e)))?
        };

        // Enable WAL mode and synchronous normal for high performance and durability
        let _ = conn.execute("PRAGMA journal_mode = WAL;", []);
        let _ = conn.execute("PRAGMA synchronous = NORMAL;", []);

        let mut storage = Self {
            conn,
            path: path.to_string(),
        };
        storage.init_tables()?;
        Ok(storage)
    }

    fn init_tables(&mut self) -> Result<(), RaftError> {
        self.conn
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS raft_meta (
                    key TEXT PRIMARY KEY,
                    value INTEGER
                );
                CREATE TABLE IF NOT EXISTS raft_log (
                    log_index INTEGER PRIMARY KEY,
                    term INTEGER NOT NULL,
                    command TEXT NOT NULL,
                    payload TEXT NOT NULL
                );
                CREATE TABLE IF NOT EXISTS raft_state_machine (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                );",
            )
            .map_err(|e| RaftError::StorageError(format!("Failed to initialize Raft tables: {}", e)))?;
        Ok(())
    }

    pub fn save_term_and_vote(&mut self, term: u64, voted_for: Option<u64>) -> Result<(), RaftError> {
        let tx = self
            .conn
            .transaction()
            .map_err(|e| RaftError::StorageError(format!("Failed to begin transaction: {}", e)))?;

        tx.execute(
            "INSERT OR REPLACE INTO raft_meta (key, value) VALUES ('current_term', ?1)",
            params![term as i64],
        )
        .map_err(|e| RaftError::StorageError(format!("Failed to save current_term: {}", e)))?;

        let vote_val = voted_for.map(|v| v as i64).unwrap_or(-1);
        tx.execute(
            "INSERT OR REPLACE INTO raft_meta (key, value) VALUES ('voted_for', ?1)",
            params![vote_val],
        )
        .map_err(|e| RaftError::StorageError(format!("Failed to save voted_for: {}", e)))?;

        tx.commit()
            .map_err(|e| RaftError::StorageError(format!("Failed to commit term and vote: {}", e)))?;
        Ok(())
    }

    pub fn load_term_and_vote(&self) -> Result<(u64, Option<u64>), RaftError> {
        let term: u64 = self
            .conn
            .query_row(
                "SELECT value FROM raft_meta WHERE key = 'current_term'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map(|v| v as u64)
            .unwrap_or(0);

        let voted_for: Option<u64> = self
            .conn
            .query_row(
                "SELECT value FROM raft_meta WHERE key = 'voted_for'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .ok()
            .and_then(|v| if v >= 0 { Some(v as u64) } else { None });

        Ok((term, voted_for))
    }

    pub fn append_entries(&mut self, entries: &[LogEntry]) -> Result<(), RaftError> {
        if entries.is_empty() {
            return Ok(());
        }

        let tx = self
            .conn
            .transaction()
            .map_err(|e| RaftError::StorageError(format!("Failed to begin transaction: {}", e)))?;

        for entry in entries {
            tx.execute(
                "INSERT OR REPLACE INTO raft_log (log_index, term, command, payload) VALUES (?1, ?2, ?3, ?4)",
                params![entry.index as i64, entry.term as i64, &entry.command, &entry.payload],
            )
            .map_err(|e| RaftError::StorageError(format!("Failed to insert log entry {}: {}", entry.index, e)))?;
        }

        tx.commit()
            .map_err(|e| RaftError::StorageError(format!("Failed to commit log entries: {}", e)))?;
        Ok(())
    }

    pub fn truncate_after(&mut self, index: u64) -> Result<(), RaftError> {
        self.conn
            .execute(
                "DELETE FROM raft_log WHERE log_index > ?1",
                params![index as i64],
            )
            .map_err(|e| RaftError::StorageError(format!("Failed to truncate log after {}: {}", index, e)))?;
        Ok(())
    }

    pub fn get_entry(&self, index: u64) -> Result<Option<LogEntry>, RaftError> {
        let mut stmt = self
            .conn
            .prepare("SELECT log_index, term, command, payload FROM raft_log WHERE log_index = ?1")
            .map_err(|e| RaftError::StorageError(format!("Failed to prepare get_entry statement: {}", e)))?;

        let entry = stmt
            .query_row(params![index as i64], |row| {
                Ok(LogEntry {
                    index: row.get::<_, i64>(0)? as u64,
                    term: row.get::<_, i64>(1)? as u64,
                    command: row.get(2)?,
                    payload: row.get(3)?,
                })
            })
            .ok();

        Ok(entry)
    }

    pub fn get_entries_from(&self, start_index: u64) -> Result<Vec<LogEntry>, RaftError> {
        let mut stmt = self
            .conn
            .prepare("SELECT log_index, term, command, payload FROM raft_log WHERE log_index >= ?1 ORDER BY log_index ASC")
            .map_err(|e| RaftError::StorageError(format!("Failed to prepare get_entries_from statement: {}", e)))?;

        let rows = stmt
            .query_map(params![start_index as i64], |row| {
                Ok(LogEntry {
                    index: row.get::<_, i64>(0)? as u64,
                    term: row.get::<_, i64>(1)? as u64,
                    command: row.get(2)?,
                    payload: row.get(3)?,
                })
            })
            .map_err(|e| RaftError::StorageError(format!("Failed to query log entries: {}", e)))?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(entry) = r {
                list.push(entry);
            }
        }
        Ok(list)
    }

    pub fn last_log_info(&self) -> Result<(u64, u64), RaftError> {
        let mut stmt = self
            .conn
            .prepare("SELECT log_index, term FROM raft_log ORDER BY log_index DESC LIMIT 1")
            .map_err(|e| RaftError::StorageError(format!("Failed to prepare last_log_info statement: {}", e)))?;

        let res = stmt
            .query_row([], |row| {
                Ok((row.get::<_, i64>(0)? as u64, row.get::<_, i64>(1)? as u64))
            })
            .unwrap_or((0, 0));

        Ok(res)
    }

    pub fn apply_to_state_machine(&mut self, command: &str, payload: &str) -> Result<(), RaftError> {
        match command {
            "SET" | "WRITE" | "PUT" => {
                let mut k = payload.to_string();
                let mut v = String::new();

                if let Ok(val) = serde_json::from_str::<serde_json::Value>(payload) {
                    if let (Some(key_val), Some(val_val)) = (
                        val.get("key").and_then(|x| x.as_str()),
                        val.get("value").and_then(|x| x.as_str()),
                    ) {
                        k = key_val.to_string();
                        v = val_val.to_string();
                    }
                } else if payload.contains("key=") && payload.contains("&value=") {
                    for part in payload.split('&') {
                        if let Some(rest) = part.strip_prefix("key=") {
                            k = rest.to_string();
                        } else if let Some(rest) = part.strip_prefix("value=") {
                            v = rest.to_string();
                        }
                    }
                } else if let Some(idx) = payload.find('=') {
                    k = payload[..idx].to_string();
                    v = payload[idx + 1..].to_string();
                }

                self.conn
                    .execute(
                        "INSERT OR REPLACE INTO raft_state_machine (key, value) VALUES (?1, ?2)",
                        params![&k, &v],
                    )
                    .map_err(|e| RaftError::StorageError(format!("Failed to apply state machine write: {}", e)))?;
            }
            "DEL" | "DELETE" => {
                self.conn
                    .execute("DELETE FROM raft_state_machine WHERE key = ?1", params![payload])
                    .map_err(|e| RaftError::StorageError(format!("Failed to apply state machine delete: {}", e)))?;
            }
            _ => {
                // Generic command recorded as key/value
                self.conn
                    .execute(
                        "INSERT OR REPLACE INTO raft_state_machine (key, value) VALUES (?1, ?2)",
                        params![command, payload],
                    )
                    .map_err(|e| RaftError::StorageError(format!("Failed to apply generic command: {}", e)))?;
            }
        }
        Ok(())
    }

    pub fn read_state_machine(&self, key: &str) -> Result<Option<String>, RaftError> {
        let mut stmt = self
            .conn
            .prepare("SELECT value FROM raft_state_machine WHERE key = ?1")
            .map_err(|e| RaftError::StorageError(format!("Failed to prepare read_state_machine: {}", e)))?;

        let res = stmt
            .query_row(params![key], |row| row.get::<_, String>(0))
            .ok();

        Ok(res)
    }

    pub fn dump_state_machine(&self) -> Result<HashMap<String, String>, RaftError> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM raft_state_machine")
            .map_err(|e| RaftError::StorageError(format!("Failed to prepare dump_state_machine: {}", e)))?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
            .map_err(|e| RaftError::StorageError(format!("Failed to query dump_state_machine: {}", e)))?;

        let mut map = HashMap::new();
        for r in rows {
            if let Ok((k, v)) = r {
                map.insert(k, v);
            }
        }
        Ok(map)
    }
}
