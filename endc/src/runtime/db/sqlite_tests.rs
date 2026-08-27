#[cfg(test)]
pub mod tests {
    use crate::runtime::db::engine::SqliteEngine;
    use crate::runtime::db::error::SqliteError;
    use rusqlite::ToSql;
    use std::fs;
    use std::io::Read;
    use std::process::Command;
    use std::time::Duration;

    fn get_temp_db_path(name: &str) -> String {
        let mut p = std::env::temp_dir();
        p.push(format!("end_sqlite_test_{}_{}.db", name, std::process::id()));
        p.to_string_lossy().to_string()
    }

    // -----------------------------------------------------------------------------
    // Gate 1 & 2: Connection Tests (New, Reopen, In-Memory, Invalid)
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_connection_lifecycle() {
        let db_path = get_temp_db_path("conn_lifecycle");
        let _ = fs::remove_file(&db_path);

        // 1. Create new database
        {
            let mut engine = SqliteEngine::open(&db_path).expect("Failed to create new SQLite database");
            assert_eq!(engine.path, db_path);
            let res = engine.execute("CREATE TABLE init_test (id INTEGER PRIMARY KEY, note TEXT);", &[]);
            assert!(res.is_ok());
            let insert_res = engine.execute("INSERT INTO init_test (note) VALUES (?1);", &[&"first_run"]);
            assert_eq!(insert_res.unwrap(), 1);
        }

        // 2. Re-open existing database and verify data persists
        {
            let mut engine = SqliteEngine::open(&db_path).expect("Failed to re-open SQLite database");
            let rows = engine.query_json("SELECT note FROM init_test;", &[]).expect("Query failed");
            assert_eq!(rows.as_array().unwrap().len(), 1);
            assert_eq!(rows.as_array().unwrap()[0]["note"], "first_run");
        }

        // 3. In-memory database
        {
            let mut mem_engine = SqliteEngine::open(":memory:").expect("Failed to open in-memory SQLite");
            let res = mem_engine.execute("CREATE TABLE mem (val INTEGER);", &[]);
            assert!(res.is_ok());
        }

        let _ = fs::remove_file(&db_path);
    }

    // -----------------------------------------------------------------------------
    // Gate 3: Real CRUD Operations (CREATE, INSERT, SELECT, UPDATE, DELETE, WHERE)
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_crud_and_where() {
        let mut engine = SqliteEngine::open(":memory:").expect("Failed to open in-memory database");

        // CREATE TABLE
        let create_res = engine.execute(
            "CREATE TABLE users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                email TEXT NOT NULL,
                age INTEGER NOT NULL
            );",
            &[],
        );
        assert!(create_res.is_ok());

        // INSERT (Multiple rows)
        let count1 = engine
            .execute(
                "INSERT INTO users (username, email, age) VALUES (?1, ?2, ?3);",
                &[&"alice", &"alice@end-lang.org", &30i64],
            )
            .expect("Insert alice failed");
        assert_eq!(count1, 1);

        let count2 = engine
            .execute(
                "INSERT INTO users (username, email, age) VALUES (?1, ?2, ?3);",
                &[&"bob", &"bob@end-lang.org", &24i64],
            )
            .expect("Insert bob failed");
        assert_eq!(count2, 1);

        let count3 = engine
            .execute(
                "INSERT INTO users (username, email, age) VALUES (?1, ?2, ?3);",
                &[&"carol", &"carol@end-lang.org", &35i64],
            )
            .expect("Insert carol failed");
        assert_eq!(count3, 1);

        // SELECT ALL
        let all_users = engine
            .query_json("SELECT id, username, email, age FROM users ORDER BY id ASC;", &[])
            .expect("Select all failed");
        let users_arr = all_users.as_array().expect("Expected JSON array");
        assert_eq!(users_arr.len(), 3);
        assert_eq!(users_arr[0]["username"], "alice");
        assert_eq!(users_arr[1]["username"], "bob");
        assert_eq!(users_arr[2]["username"], "carol");

        // SELECT ... WHERE (Equality and parameter filtering)
        let filtered = engine
            .query_json("SELECT username, age FROM users WHERE age >= ?1 ORDER BY age ASC;", &[&30i64])
            .expect("Select WHERE failed");
        let filtered_arr = filtered.as_array().unwrap();
        assert_eq!(filtered_arr.len(), 2);
        assert_eq!(filtered_arr[0]["username"], "alice");
        assert_eq!(filtered_arr[1]["username"], "carol");

        // SELECT ... WHERE (Zero rows match)
        let zero_rows = engine
            .query_json("SELECT username FROM users WHERE age > ?1;", &[&100i64])
            .expect("Select zero rows failed");
        assert_eq!(zero_rows.as_array().unwrap().len(), 0);

        // UPDATE
        let update_count = engine
            .execute(
                "UPDATE users SET email = ?1 WHERE username = ?2;",
                &[&"alice_updated@end-lang.org", &"alice"],
            )
            .expect("Update failed");
        assert_eq!(update_count, 1);

        let updated_row = engine
            .query_json("SELECT email FROM users WHERE username = ?1;", &[&"alice"])
            .expect("Select updated failed");
        assert_eq!(updated_row.as_array().unwrap()[0]["email"], "alice_updated@end-lang.org");

        // DELETE
        let delete_count = engine
            .execute("DELETE FROM users WHERE username = ?1;", &[&"bob"])
            .expect("Delete failed");
        assert_eq!(delete_count, 1);

        let after_delete = engine
            .query_json("SELECT username FROM users WHERE username = ?1;", &[&"bob"])
            .expect("Select after delete failed");
        assert_eq!(after_delete.as_array().unwrap().len(), 0);
    }

    // -----------------------------------------------------------------------------
    // Gate 4: Real Multi-Table JOIN
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_two_table_join() {
        let mut engine = SqliteEngine::open(":memory:").expect("Failed to open in-memory database");

        engine
            .execute(
                "CREATE TABLE authors (
                    id INTEGER PRIMARY KEY,
                    name TEXT NOT NULL
                );",
                &[],
            )
            .expect("Create authors failed");

        engine
            .execute(
                "CREATE TABLE books (
                    id INTEGER PRIMARY KEY,
                    author_id INTEGER NOT NULL,
                    title TEXT NOT NULL,
                    price REAL NOT NULL,
                    FOREIGN KEY(author_id) REFERENCES authors(id)
                );",
                &[],
            )
            .expect("Create books failed");

        // Insert authors
        engine.execute("INSERT INTO authors (id, name) VALUES (1, 'Ada Lovelace');", &[]).unwrap();
        engine.execute("INSERT INTO authors (id, name) VALUES (2, 'Alan Turing');", &[]).unwrap();

        // Insert books
        engine.execute("INSERT INTO books (id, author_id, title, price) VALUES (101, 1, 'Analytical Engine Notes', 49.99);", &[]).unwrap();
        engine.execute("INSERT INTO books (id, author_id, title, price) VALUES (102, 1, 'Sketch of the Analytical Engine', 35.50);", &[]).unwrap();
        engine.execute("INSERT INTO books (id, author_id, title, price) VALUES (103, 2, 'Computable Numbers', 59.95);", &[]).unwrap();

        // Perform INNER JOIN
        let join_results = engine
            .query_json(
                "SELECT a.name AS author_name, b.title AS book_title, b.price
                 FROM authors a
                 INNER JOIN books b ON a.id = b.author_id
                 WHERE a.id = ?1
                 ORDER BY b.id ASC;",
                &[&1i64],
            )
            .expect("Join query failed");

        let rows = join_results.as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0]["author_name"], "Ada Lovelace");
        assert_eq!(rows[0]["book_title"], "Analytical Engine Notes");
        assert_eq!(rows[0]["price"], 49.99);

        assert_eq!(rows[1]["author_name"], "Ada Lovelace");
        assert_eq!(rows[1]["book_title"], "Sketch of the Analytical Engine");
        assert_eq!(rows[1]["price"], 35.50);
    }

    // -----------------------------------------------------------------------------
    // Gate 5: Real Transactions (Commit Persistence & Rollback Reversion)
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_transaction_commit_and_rollback() {
        let db_path = get_temp_db_path("tx_test");
        let _ = fs::remove_file(&db_path);

        // 1. Transaction COMMIT Test
        {
            let mut engine = SqliteEngine::open(&db_path).unwrap();
            engine.execute("CREATE TABLE tx_data (id INTEGER PRIMARY KEY, status TEXT);", &[]).unwrap();

            engine.transaction_begin().expect("Begin tx failed");
            engine.execute("INSERT INTO tx_data (id, status) VALUES (1, 'committed_record');", &[]).unwrap();
            engine.transaction_commit().expect("Commit tx failed");
        }

        // Verify committed data through a new connection
        {
            let mut engine2 = SqliteEngine::open(&db_path).unwrap();
            let rows = engine2.query_json("SELECT status FROM tx_data WHERE id = 1;", &[]).unwrap();
            assert_eq!(rows.as_array().unwrap().len(), 1);
            assert_eq!(rows.as_array().unwrap()[0]["status"], "committed_record");

            // 2. Transaction ROLLBACK Test
            engine2.transaction_begin().expect("Begin tx 2 failed");
            engine2.execute("INSERT INTO tx_data (id, status) VALUES (2, 'uncommitted_record');", &[]).unwrap();

            // Row exists inside active uncommitted transaction
            let in_tx_rows = engine2.query_json("SELECT status FROM tx_data WHERE id = 2;", &[]).unwrap();
            assert_eq!(in_tx_rows.as_array().unwrap().len(), 1);

            // Rollback transaction
            engine2.transaction_rollback().expect("Rollback failed");

            // Row MUST NOT exist after rollback
            let after_rollback = engine2.query_json("SELECT status FROM tx_data WHERE id = 2;", &[]).unwrap();
            assert_eq!(after_rollback.as_array().unwrap().len(), 0, "Rollback failed to undo uncommitted insert!");
        }

        let _ = fs::remove_file(&db_path);
    }

    // -----------------------------------------------------------------------------
    // Gate 6: Prepared Statement Parameter Binding and Multi-Set Reuse
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_prepared_statements_reuse() {
        let mut engine = SqliteEngine::open(":memory:").unwrap();
        engine.execute("CREATE TABLE products (id INTEGER PRIMARY KEY, sku TEXT, price REAL);", &[]).unwrap();

        let stmt_id = engine
            .prepare_statement("INSERT INTO products (sku, price) VALUES (?1, ?2);")
            .expect("Prepare failed");

        // Execute same prepared statement with multiple distinct parameter sets
        let dataset = vec![
            ("SKU-NEO-001", 19.99f64),
            ("SKU-NEO-002", 49.50f64),
            ("SKU-NEO-003", 99.00f64),
            ("SKU-NEO-004", 149.25f64),
        ];

        for (sku, price) in &dataset {
            let p_sku: &dyn ToSql = sku;
            let p_price: &dyn ToSql = price;
            let affected = engine.execute_prepared(stmt_id, &[p_sku, p_price]).expect("Execute prepared failed");
            assert_eq!(affected, 1);
        }

        let rows = engine.query_json("SELECT sku, price FROM products ORDER BY id ASC;", &[]).unwrap();
        let arr = rows.as_array().unwrap();
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0]["sku"], "SKU-NEO-001");
        assert_eq!(arr[1]["sku"], "SKU-NEO-002");
        assert_eq!(arr[2]["sku"], "SKU-NEO-003");
        assert_eq!(arr[3]["sku"], "SKU-NEO-004");
    }

    // -----------------------------------------------------------------------------
    // Gate 7 & 8: Real Binary File Format and Official sqlite3 CLI Interoperability
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_official_cli_interoperability_and_binary_format() {
        let db_path = get_temp_db_path("cli_interop");
        let _ = fs::remove_file(&db_path);

        // 1. Create database and populate with End SQLite Engine
        {
            let mut engine = SqliteEngine::open(&db_path).unwrap();
            engine
                .execute(
                    "CREATE TABLE platform_users (
                        id INTEGER PRIMARY KEY,
                        handle TEXT NOT NULL,
                        clearance INTEGER NOT NULL
                    );",
                    &[],
                )
                .unwrap();

            engine.execute("INSERT INTO platform_users (id, handle, clearance) VALUES (1, 'maho_lead', 5);", &[]).unwrap();
            engine.execute("INSERT INTO platform_users (id, handle, clearance) VALUES (2, 'antigravity_agent', 4);", &[]).unwrap();
            engine.execute("INSERT INTO platform_users (id, handle, clearance) VALUES (3, 'kernel_operator', 3);", &[]).unwrap();
        }

        // 2. Validate official SQLite 3 binary file header: "SQLite format 3\0" (16 bytes)
        {
            let mut file = fs::File::open(&db_path).expect("Database file not found");
            let mut header = [0u8; 16];
            file.read_exact(&mut header).expect("Failed to read header");
            let magic = b"SQLite format 3\0";
            assert_eq!(&header, magic, "File is not a valid official SQLite database!");
        }

        // 3. Test Interoperability with official `sqlite3` CLI: `.tables`
        let tables_output = Command::new("sqlite3")
            .arg(&db_path)
            .arg(".tables")
            .output();

        if let Ok(out) = tables_output {
            assert!(out.status.success(), "sqlite3 .tables command failed");
            let stdout = String::from_utf8_lossy(&out.stdout);
            assert!(stdout.contains("platform_users"), "sqlite3 CLI did not list created table: {}", stdout);
        }

        // 4. Test Interoperability with official `sqlite3` CLI: `SELECT * FROM platform_users;`
        let query_output = Command::new("sqlite3")
            .arg(&db_path)
            .arg("SELECT id, handle, clearance FROM platform_users ORDER BY id ASC;")
            .output();

        if let Ok(out) = query_output {
            assert!(out.status.success(), "sqlite3 SELECT query failed");
            let stdout = String::from_utf8_lossy(&out.stdout);
            assert!(stdout.contains("1|maho_lead|5"), "sqlite3 CLI missing row 1: {}", stdout);
            assert!(stdout.contains("2|antigravity_agent|4"), "sqlite3 CLI missing row 2: {}", stdout);
            assert!(stdout.contains("3|kernel_operator|3"), "sqlite3 CLI missing row 3: {}", stdout);
        }

        let _ = fs::remove_file(&db_path);
    }

    // -----------------------------------------------------------------------------
    // Gate 9: Real SQLite Concurrency & Locking Semantics
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_real_concurrency_locking() {
        let db_path = get_temp_db_path("concurrency_lock");
        let _ = fs::remove_file(&db_path);

        // Setup table
        {
            let mut setup_engine = SqliteEngine::open(&db_path).unwrap();
            setup_engine.execute("CREATE TABLE concurrent_ledger (id INTEGER PRIMARY KEY, writer TEXT);", &[]).unwrap();
        }

        // Open two independent connections to the same database file
        let mut conn_a = SqliteEngine::open(&db_path).unwrap();
        let mut conn_b = SqliteEngine::open(&db_path).unwrap();

        // Connection A acquires an EXCLUSIVE lock via transaction
        conn_a.execute("BEGIN EXCLUSIVE TRANSACTION;", &[]).expect("Conn A failed to begin exclusive transaction");
        conn_a.execute("INSERT INTO concurrent_ledger (id, writer) VALUES (1, 'writer_a');", &[]).unwrap();

        // Connection B attempts an immediate write while Connection A holds EXCLUSIVE lock -> Must receive BUSY/LOCKED error from SQLite
        let b_res = conn_b.execute("INSERT INTO concurrent_ledger (id, writer) VALUES (2, 'writer_b');", &[]);
        assert!(b_res.is_err(), "Expected Connection B write to fail due to SQLite database locking!");

        // Connection A commits and releases lock
        conn_a.execute("COMMIT;", &[]).expect("Conn A commit failed");

        // Now Connection B can write successfully
        let b_res_after = conn_b.execute("INSERT INTO concurrent_ledger (id, writer) VALUES (2, 'writer_b');", &[]);
        assert!(b_res_after.is_ok(), "Connection B failed to write after Connection A released lock: {:?}", b_res_after);

        // Connection C verifies both writes persisted
        let mut conn_c = SqliteEngine::open(&db_path).unwrap();
        let all_rows = conn_c.query_json("SELECT id, writer FROM concurrent_ledger ORDER BY id ASC;", &[]).unwrap();
        let arr = all_rows.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0]["writer"], "writer_a");
        assert_eq!(arr[1]["writer"], "writer_b");

        let _ = fs::remove_file(&db_path);
    }

    // -----------------------------------------------------------------------------
    // Gate 10: Truthful Negative Tests & Error Surfacing
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_negative_error_handling() {
        let mut engine = SqliteEngine::open(":memory:").unwrap();

        // 1. Invalid SQL syntax
        let err1 = engine.execute("INVALID SQL SYNTAX HERE;", &[]);
        assert!(matches!(err1, Err(SqliteError::ExecutionFailed(_))));

        // 2. Query nonexistent table
        let err2 = engine.query_json("SELECT * FROM nonexistent_table;", &[]);
        assert!(matches!(err2, Err(SqliteError::QueryFailed(_))));

        // 3. Unique constraint violation
        engine.execute("CREATE TABLE unique_test (id INTEGER PRIMARY KEY, code TEXT UNIQUE);", &[]).unwrap();
        engine.execute("INSERT INTO unique_test (code) VALUES ('UNIQ_001');", &[]).unwrap();
        let err3 = engine.execute("INSERT INTO unique_test (code) VALUES ('UNIQ_001');", &[]);
        assert!(matches!(err3, Err(SqliteError::ExecutionFailed(_))));

        // 4. Invalid transaction rollback when no transaction is active
        let err4 = engine.transaction_rollback();
        assert!(matches!(err4, Err(SqliteError::TransactionFailed(_))));
    }

    // -----------------------------------------------------------------------------
    // Gate 12: Key-Value Store on Real SQLite Engine Compatibility
    // -----------------------------------------------------------------------------
    #[test]
    fn test_sqlite_kv_backward_compatibility() {
        let db_path = get_temp_db_path("kv_compat");
        let _ = fs::remove_file(&db_path);

        // First connection writes KV entries
        {
            let mut engine = SqliteEngine::open(&db_path).unwrap();
            engine.kv_set("config:theme", "cyberpunk_neon").unwrap();
            engine.kv_set("user:1001", "name=Maho,role=LeadArchitect").unwrap();

            let v1 = engine.kv_get("config:theme").unwrap();
            assert_eq!(v1, Some("cyberpunk_neon".to_string()));
        }

        // Second connection verifies persistence
        {
            let mut engine2 = SqliteEngine::open(&db_path).unwrap();
            let v1 = engine2.kv_get("config:theme").unwrap();
            assert_eq!(v1, Some("cyberpunk_neon".to_string()));

            let v2 = engine2.kv_get("user:1001").unwrap();
            assert_eq!(v2, Some("name=Maho,role=LeadArchitect".to_string()));

            let v_none = engine2.kv_get("nonexistent_key").unwrap();
            assert_eq!(v_none, None);
        }

        // Validate via official sqlite3 CLI that __end_kv_store exists and contains data
        let cli_out = Command::new("sqlite3")
            .arg(&db_path)
            .arg("SELECT k, v FROM __end_kv_store ORDER BY k ASC;")
            .output();

        if let Ok(out) = cli_out {
            assert!(out.status.success());
            let stdout = String::from_utf8_lossy(&out.stdout);
            assert!(stdout.contains("config:theme|cyberpunk_neon"));
            assert!(stdout.contains("user:1001|name=Maho,role=LeadArchitect"));
        }

        let _ = fs::remove_file(&db_path);
    }
}
