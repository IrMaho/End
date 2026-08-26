#[cfg(test)]
pub mod tests {
    use crate::runtime::db::postgres::PgEngine;
    use serde_json::Value;
    use std::process::Command;

    const TEST_PG_URL: &str = "host=127.0.0.1 port=5432 user=postgres dbname=postgres";

    fn is_postgres_running() -> bool {
        PgEngine::connect(TEST_PG_URL).is_ok()
    }

    #[test]
    fn test_pg_dependency_and_wire_handshake() {
        if !is_postgres_running() {
            println!("PostgreSQL not running on port 5432, skipping live test");
            return;
        }
        let engine = PgEngine::connect(TEST_PG_URL);
        assert!(engine.is_ok(), "PostgreSQL connection and handshake must succeed");
        let eng = engine.unwrap();
        assert!(eng.is_connected);
    }

    #[test]
    fn test_pg_crud_operations() {
        if !is_postgres_running() {
            return;
        }
        let mut eng = PgEngine::connect(TEST_PG_URL).expect("connect");

        // 1. DDL: CREATE TABLE
        eng.execute("DROP TABLE IF EXISTS test_users_crud CASCADE;", &[]).unwrap();
        let create_sql = "CREATE TABLE test_users_crud (
            id SERIAL PRIMARY KEY,
            name VARCHAR(100) NOT NULL,
            email VARCHAR(100) UNIQUE,
            age INT
        );";
        eng.execute(create_sql, &[]).expect("create table");

        // 2. DML: INSERT
        let insert_sql = "INSERT INTO test_users_crud (name, email, age) VALUES ($1, $2, $3);";
        let rows_aff1 = eng.execute(insert_sql, &[&"Alice", &"alice@example.com", &30i32]).expect("insert 1");
        assert_eq!(rows_aff1, 1);

        let rows_aff2 = eng.execute(insert_sql, &[&"Bob", &"bob@example.com", &25i32]).expect("insert 2");
        assert_eq!(rows_aff2, 1);

        // 3. DML: SELECT
        let query_val = eng.query_json("SELECT name, email, age FROM test_users_crud ORDER BY id ASC;", &[]).expect("select");
        if let Value::Array(rows) = query_val {
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0]["name"], "Alice");
            assert_eq!(rows[0]["age"], 30);
            assert_eq!(rows[1]["name"], "Bob");
            assert_eq!(rows[1]["age"], 25);
        } else {
            panic!("Expected JSON array result");
        }

        // 4. DML: UPDATE
        let update_sql = "UPDATE test_users_crud SET age = $1 WHERE name = $2;";
        let rows_aff_up = eng.execute(update_sql, &[&31i32, &"Alice"]).expect("update");
        assert_eq!(rows_aff_up, 1);

        let updated_val = eng.query_json("SELECT age FROM test_users_crud WHERE name = 'Alice';", &[]).expect("select");
        assert_eq!(updated_val[0]["age"], 31);

        // 5. DML: DELETE
        let delete_sql = "DELETE FROM test_users_crud WHERE name = $1;";
        let rows_aff_del = eng.execute(delete_sql, &[&"Bob"]).expect("delete");
        assert_eq!(rows_aff_del, 1);

        let count_val = eng.query_json("SELECT COUNT(*)::INT4 as c FROM test_users_crud;", &[]).expect("count");
        assert_eq!(count_val[0]["c"], 1);

        eng.execute("DROP TABLE test_users_crud;", &[]).unwrap();
    }

    #[test]
    fn test_pg_parameter_binding() {
        if !is_postgres_running() {
            return;
        }
        let mut eng = PgEngine::connect(TEST_PG_URL).expect("connect");

        eng.execute("DROP TABLE IF EXISTS test_pg_params CASCADE;", &[]).unwrap();
        eng.execute("CREATE TABLE test_pg_params (id INT PRIMARY KEY, title TEXT, price FLOAT8);", &[]).unwrap();

        eng.execute("INSERT INTO test_pg_params (id, title, price) VALUES ($1, $2, $3);", &[&101i32, &"Laptop", &1299.99f64]).unwrap();
        eng.execute("INSERT INTO test_pg_params (id, title, price) VALUES ($1, $2, $3);", &[&102i32, &"Keyboard", &89.50f64]).unwrap();
        eng.execute("INSERT INTO test_pg_params (id, title, price) VALUES ($1, $2, $3);", &[&103i32, &"Mouse", &45.00f64]).unwrap();

        // Parameter query 1
        let res1 = eng.query_json("SELECT title, price FROM test_pg_params WHERE id = $1;", &[&101i32]).unwrap();
        assert_eq!(res1[0]["title"], "Laptop");

        // Parameter query 2
        let res2 = eng.query_json("SELECT title, price FROM test_pg_params WHERE id = $1;", &[&102i32]).unwrap();
        assert_eq!(res2[0]["title"], "Keyboard");

        // Parameter query 3 (no match)
        let res3 = eng.query_json("SELECT title FROM test_pg_params WHERE id = $1;", &[&999i32]).unwrap();
        assert_eq!(res3, Value::Array(vec![]));

        eng.execute("DROP TABLE test_pg_params;", &[]).unwrap();
    }

    #[test]
    fn test_pg_two_table_join() {
        if !is_postgres_running() {
            return;
        }
        let mut eng = PgEngine::connect(TEST_PG_URL).expect("connect");

        eng.execute("DROP TABLE IF EXISTS test_orders CASCADE;", &[]).unwrap();
        eng.execute("DROP TABLE IF EXISTS test_customers CASCADE;", &[]).unwrap();

        eng.execute("CREATE TABLE test_customers (id INT PRIMARY KEY, name TEXT);", &[]).unwrap();
        eng.execute("CREATE TABLE test_orders (id INT PRIMARY KEY, customer_id INT REFERENCES test_customers(id), amount FLOAT8);", &[]).unwrap();

        eng.execute("INSERT INTO test_customers (id, name) VALUES (1, 'Charlie'), (2, 'Diana');", &[]).unwrap();
        eng.execute("INSERT INTO test_orders (id, customer_id, amount) VALUES (10, 1, 150.0), (11, 1, 200.0), (12, 2, 50.0);", &[]).unwrap();

        let join_sql = "SELECT c.name, o.id as order_id, o.amount 
                        FROM test_customers c 
                        JOIN test_orders o ON c.id = o.customer_id 
                        ORDER BY o.id ASC;";
        let rows = eng.query_json(join_sql, &[]).unwrap();
        assert_eq!(rows.as_array().unwrap().len(), 3);
        assert_eq!(rows[0]["name"], "Charlie");
        assert_eq!(rows[0]["amount"], 150.0);
        assert_eq!(rows[1]["name"], "Charlie");
        assert_eq!(rows[1]["amount"], 200.0);
        assert_eq!(rows[2]["name"], "Diana");
        assert_eq!(rows[2]["amount"], 50.0);

        eng.execute("DROP TABLE test_orders; DROP TABLE test_customers;", &[]).unwrap();
    }

    #[test]
    fn test_pg_atomic_transaction_rollback() {
        if !is_postgres_running() {
            return;
        }
        let mut eng = PgEngine::connect(TEST_PG_URL).expect("connect");

        eng.execute("DROP TABLE IF EXISTS test_tx_rollback;", &[]).unwrap();
        eng.execute("CREATE TABLE test_tx_rollback (id INT PRIMARY KEY, val TEXT);", &[]).unwrap();

        // 1. Begin transaction
        eng.transaction_begin().expect("begin");

        // 2. Insert row inside transaction
        eng.execute("INSERT INTO test_tx_rollback (id, val) VALUES ($1, $2);", &[&1i32, &"Uncommitted"]).expect("insert");

        // Row is visible inside the transaction
        let in_tx = eng.query_json("SELECT val FROM test_tx_rollback WHERE id = 1;", &[]).unwrap();
        assert_eq!(in_tx[0]["val"], "Uncommitted");

        // 3. Rollback transaction
        eng.transaction_rollback().expect("rollback");

        // 4. Verify row is NOT present after rollback
        let after_rollback = eng.query_json("SELECT val FROM test_tx_rollback WHERE id = 1;", &[]).unwrap();
        assert_eq!(after_rollback, Value::Array(vec![]));

        eng.execute("DROP TABLE test_tx_rollback;", &[]).unwrap();
    }

    #[test]
    fn test_pg_atomic_transaction_commit() {
        if !is_postgres_running() {
            return;
        }
        let mut eng = PgEngine::connect(TEST_PG_URL).expect("connect");

        eng.execute("DROP TABLE IF EXISTS test_tx_commit;", &[]).unwrap();
        eng.execute("CREATE TABLE test_tx_commit (id INT PRIMARY KEY, val TEXT);", &[]).unwrap();

        // 1. Begin transaction
        eng.transaction_begin().expect("begin");

        // 2. Insert row
        eng.execute("INSERT INTO test_tx_commit (id, val) VALUES ($1, $2);", &[&42i32, &"Committed Value"]).expect("insert");

        // 3. Commit transaction
        eng.transaction_commit().expect("commit");

        // 4. Connect fresh client to verify persistence
        let mut fresh_eng = PgEngine::connect(TEST_PG_URL).expect("fresh connect");
        let rows = fresh_eng.query_json("SELECT val FROM test_tx_commit WHERE id = 42;", &[]).unwrap();
        assert_eq!(rows[0]["val"], "Committed Value");

        fresh_eng.execute("DROP TABLE test_tx_commit;", &[]).unwrap();
    }

    #[test]
    fn test_pg_null_semantics() {
        if !is_postgres_running() {
            return;
        }
        let mut eng = PgEngine::connect(TEST_PG_URL).expect("connect");

        eng.execute("DROP TABLE IF EXISTS test_pg_nulls;", &[]).unwrap();
        eng.execute("CREATE TABLE test_pg_nulls (id INT PRIMARY KEY, name TEXT, score INT, active BOOLEAN);", &[]).unwrap();

        // Insert non-null and null rows
        eng.execute("INSERT INTO test_pg_nulls (id, name, score, active) VALUES (1, 'Non-Null', 100, true);", &[]).unwrap();
        eng.execute("INSERT INTO test_pg_nulls (id, name, score, active) VALUES (2, NULL, NULL, NULL);", &[]).unwrap();

        let rows = eng.query_json("SELECT id, name, score, active FROM test_pg_nulls ORDER BY id ASC;", &[]).unwrap();
        assert_eq!(rows[0]["id"], 1);
        assert_eq!(rows[0]["name"], "Non-Null");
        assert_eq!(rows[0]["score"], 100);
        assert_eq!(rows[0]["active"], true);

        // Row 2 NULL validation
        assert_eq!(rows[1]["id"], 2);
        assert_eq!(rows[1]["name"], Value::Null);
        assert_eq!(rows[1]["score"], Value::Null);
        assert_eq!(rows[1]["active"], Value::Null);

        eng.execute("DROP TABLE test_pg_nulls;", &[]).unwrap();
    }

    #[test]
    fn test_pg_negative_error_handling() {
        if !is_postgres_running() {
            return;
        }
        let mut eng = PgEngine::connect(TEST_PG_URL).expect("connect");

        // 1. Invalid SQL syntax
        let err_syntax = eng.execute("NOT A VALID SQL STATEMENT;", &[]);
        assert!(err_syntax.is_err(), "Invalid SQL must produce an error");

        // 2. Non-existent table query
        let err_table = eng.query_json("SELECT * FROM non_existent_table_xyz_123;", &[]);
        assert!(err_table.is_err(), "Non-existent table query must produce an error");
    }

    #[test]
    fn test_pg_psql_interoperability() {
        if !is_postgres_running() {
            return;
        }
        let mut eng = PgEngine::connect(TEST_PG_URL).expect("connect");

        eng.execute("DROP TABLE IF EXISTS test_psql_interop;", &[]).unwrap();
        eng.execute("CREATE TABLE test_psql_interop (id INT PRIMARY KEY, hero_name TEXT, title TEXT);", &[]).unwrap();

        eng.execute("INSERT INTO test_psql_interop (id, hero_name, title) VALUES (1, 'Grace Hopper', 'Rear Admiral');", &[]).unwrap();
        eng.execute("INSERT INTO test_psql_interop (id, hero_name, title) VALUES (2, 'Claude Shannon', 'Father of Information Theory');", &[]).unwrap();

        // Check if psql is available in pgsql/bin/psql.exe or ../pgsql/bin/psql.exe
        let psql_candidates = ["pgsql\\bin\\psql.exe", "..\\pgsql\\bin\\psql.exe", "psql.exe", "psql"];
        let mut psql_found = None;
        for cand in &psql_candidates {
            if std::path::Path::new(cand).exists() || Command::new(cand).arg("--version").output().is_ok() {
                psql_found = Some(*cand);
                break;
            }
        }

        if let Some(psql_bin) = psql_found {
            let output = Command::new(psql_bin)
                .args(&["-h", "127.0.0.1", "-p", "5432", "-U", "postgres", "-d", "postgres", "-c", "SELECT hero_name, title FROM test_psql_interop ORDER BY id ASC;"])
                .output();

            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                println!("psql output:\n{}", stdout);
                assert!(stdout.contains("Grace Hopper"), "psql must observe data inserted by End PgEngine");
                assert!(stdout.contains("Claude Shannon"), "psql must observe data inserted by End PgEngine");
            }
        }

        eng.execute("DROP TABLE test_psql_interop;", &[]).unwrap();
    }
}
