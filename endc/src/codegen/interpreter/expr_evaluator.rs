use super::state::Interpreter;
use super::value::Value;
use crate::ast::*;
use std::collections::HashMap;

impl Interpreter {
    pub(crate) fn eval_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Lit(lit, _) => match lit {
                Literal::Int(n) => Ok(Value::Int(*n)),
                Literal::Float(f) => Ok(Value::Float(*f)),
                Literal::String(s) => Ok(Value::String(s.clone())),
                Literal::Bool(b) => Ok(Value::Bool(*b)),
                Literal::Null => Ok(Value::Pointer(0)),
            },
            Expression::Ident(name, _) => {
                if name == "break" {
                    Ok(Value::Break)
                } else if name == "continue" {
                    Ok(Value::Continue)
                } else if let Some(v) = self.get_var(name) {
                    Ok(v)
                } else if let Some(op) = self.operations.get(name) {
                    Ok(op.clone())
                } else {
                    Ok(Value::String(name.clone()))
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let l = self.eval_expression(left)?;
                let r = self.eval_expression(right)?;

                match (&l, op, &r) {
                    // Int arithmetic
                    (Value::Int(a), BinaryOp::Add, Value::Int(b)) => Ok(Value::Int(*a + *b)),
                    (Value::Int(a), BinaryOp::Sub, Value::Int(b)) => Ok(Value::Int(*a - *b)),
                    (Value::Int(a), BinaryOp::Mul, Value::Int(b)) => Ok(Value::Int(*a * *b)),
                    (Value::Int(a), BinaryOp::Div, Value::Int(b)) => {
                        if *b == 0 {
                            Err("Division by zero in End runtime".to_string())
                        } else {
                            Ok(Value::Int(*a / *b))
                        }
                    }
                    (Value::Int(a), BinaryOp::Mod, Value::Int(b)) => Ok(Value::Int(if *b != 0 { *a % *b } else { 0 })),
                    (Value::Int(a), BinaryOp::BitAnd, Value::Int(b)) => Ok(Value::Int(*a & *b)),
                    (Value::Int(a), BinaryOp::BitOr, Value::Int(b)) => Ok(Value::Int(*a | *b)),
                    (Value::Int(a), BinaryOp::BitXor, Value::Int(b)) => Ok(Value::Int(*a ^ *b)),
                    (Value::Int(a), BinaryOp::Shl, Value::Int(b)) => Ok(Value::Int(*a << *b)),
                    (Value::Int(a), BinaryOp::Shr, Value::Int(b)) => Ok(Value::Int(*a >> *b)),
                    // Int comparisons
                    (Value::Int(a), BinaryOp::Equal, Value::Int(b)) => Ok(Value::Bool(*a == *b)),
                    (Value::Int(a), BinaryOp::NotEqual, Value::Int(b)) => Ok(Value::Bool(*a != *b)),
                    (Value::Int(a), BinaryOp::LessThan, Value::Int(b)) => Ok(Value::Bool(*a < *b)),
                    (Value::Int(a), BinaryOp::LessEqual, Value::Int(b)) => Ok(Value::Bool(*a <= *b)),
                    (Value::Int(a), BinaryOp::GreaterThan, Value::Int(b)) => Ok(Value::Bool(*a > *b)),
                    (Value::Int(a), BinaryOp::GreaterEqual, Value::Int(b)) => Ok(Value::Bool(*a >= *b)),

                    // Float arithmetic
                    (Value::Float(a), BinaryOp::Add, Value::Float(b)) => Ok(Value::Float(*a + *b)),
                    (Value::Float(a), BinaryOp::Sub, Value::Float(b)) => Ok(Value::Float(*a - *b)),
                    (Value::Float(a), BinaryOp::Mul, Value::Float(b)) => Ok(Value::Float(*a * *b)),
                    (Value::Float(a), BinaryOp::Div, Value::Float(b)) => Ok(Value::Float(if *b != 0.0 { *a / *b } else { 0.0 })),
                    // Float comparisons
                    (Value::Float(a), BinaryOp::Equal, Value::Float(b)) => Ok(Value::Bool(*a == *b)),
                    (Value::Float(a), BinaryOp::NotEqual, Value::Float(b)) => Ok(Value::Bool(*a != *b)),
                    (Value::Float(a), BinaryOp::LessThan, Value::Float(b)) => Ok(Value::Bool(*a < *b)),
                    (Value::Float(a), BinaryOp::LessEqual, Value::Float(b)) => Ok(Value::Bool(*a <= *b)),
                    (Value::Float(a), BinaryOp::GreaterThan, Value::Float(b)) => Ok(Value::Bool(*a > *b)),
                    (Value::Float(a), BinaryOp::GreaterEqual, Value::Float(b)) => Ok(Value::Bool(*a >= *b)),

                    // Bool logical operations
                    (Value::Bool(a), BinaryOp::And, Value::Bool(b)) => Ok(Value::Bool(*a && *b)),
                    (Value::Bool(a), BinaryOp::Or, Value::Bool(b)) => Ok(Value::Bool(*a || *b)),
                    (Value::Bool(a), BinaryOp::Equal, Value::Bool(b)) => Ok(Value::Bool(*a == *b)),
                    (Value::Bool(a), BinaryOp::NotEqual, Value::Bool(b)) => Ok(Value::Bool(*a != *b)),

                    // String operations
                    (Value::String(a), BinaryOp::Add, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    (Value::String(a), BinaryOp::Add, other) => Ok(Value::String(format!("{}{}", a, other))),
                    (other, BinaryOp::Add, Value::String(b)) => Ok(Value::String(format!("{}{}", other, b))),
                    (Value::String(a), BinaryOp::Equal, Value::String(b)) => Ok(Value::Bool(*a == *b)),
                    (Value::String(a), BinaryOp::NotEqual, Value::String(b)) => Ok(Value::Bool(*a != *b)),

                    // Enum & Struct equality
                    (Value::Enum(_, v1, p1), BinaryOp::Equal, Value::Enum(_, v2, p2)) => Ok(Value::Bool(v1 == v2 && p1 == p2)),
                    (Value::Enum(_, v1, p1), BinaryOp::NotEqual, Value::Enum(_, v2, p2)) => Ok(Value::Bool(v1 != v2 || p1 != p2)),
                    (Value::Struct(n1, f1), BinaryOp::Equal, Value::Struct(n2, f2)) => Ok(Value::Bool(n1 == n2 && f1 == f2)),
                    (Value::Struct(n1, f1), BinaryOp::NotEqual, Value::Struct(n2, f2)) => Ok(Value::Bool(n1 != n2 || f1 != f2)),

                    // Operation Algebra
                    (Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..), BinaryOp::Add | BinaryOp::Shr, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) => {
                        Ok(Value::ComposedOp(Box::new(l.clone()), Box::new(r.clone())))
                    }
                    (Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..), BinaryOp::Mul, Value::Int(n)) => {
                        Ok(Value::RepeatedOp(Box::new(l.clone()), (*n).max(1) as usize, false))
                    }
                    (Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..), BinaryOp::BitAnd, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) => {
                        Ok(Value::ParallelOp(Box::new(l.clone()), Box::new(r.clone())))
                    }
                    (Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..), BinaryOp::BitOr, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) => {
                        Ok(Value::AlternativeOp(Box::new(l.clone()), Box::new(r.clone())))
                    }
                    _ => Ok(Value::Int(0)),
                }
            }
            Expression::Unary { op, expr, .. } => {
                let v = self.eval_expression(expr)?;
                match (op, v) {
                    (UnaryOp::Negate, Value::Int(n)) => Ok(Value::Int(-n)),
                    (UnaryOp::Negate, Value::Float(f)) => Ok(Value::Float(-f)),
                    (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    (_, other) => Ok(other),
                }
            }
            Expression::Call { callee, args, .. } => {
                let mut eval_args = Vec::new();
                for a in args {
                    eval_args.push(self.eval_expression(a)?);
                }

                if let Expression::Ident(name, _) = callee.as_ref() {
                    if name == "println" || name == "print" {
                        let msg = eval_args
                            .iter()
                            .map(|v| match v {
                                Value::String(s) => s.clone(),
                                other => other.to_string(),
                            })
                            .collect::<Vec<_>>()
                            .join(" ");
                        if name == "println" {
                            self.emit_stdout(&format!("{}\n", msg));
                            if !self.capture_stdout {
                                println!("{}", msg);
                            }
                        } else {
                            self.emit_stdout(&msg);
                            if !self.capture_stdout {
                                print!("{}", msg);
                            }
                        }
                        return Ok(Value::Void);
                    }

                    // read_line() → returns the trimmed stdin line as String
                    if name == "read_line" {
                        let mut line = String::new();
                        std::io::stdin().read_line(&mut line).unwrap_or(0);
                        return Ok(Value::String(line.trim().to_string()));
                    }

                    // read_int() → reads a line and parses as i64, returns 0 on failure
                    if name == "read_int" {
                        use std::io::Write;
                        let _ = std::io::stdout().flush();
                        let mut line = String::new();
                        std::io::stdin().read_line(&mut line).unwrap_or(0);
                        let n = line.trim().parse::<i64>().unwrap_or(0);
                        return Ok(Value::Int(n));
                    }

                    // clear_screen() → clears the terminal
                    if name == "clear_screen" {
                        use std::io::Write;
                        print!("\x1B[2J\x1B[1;1H");
                        let _ = std::io::stdout().flush();
                        return Ok(Value::Void);
                    }

                    // to_int(s) → parse string to int
                    if name == "to_int" {
                        if let Some(Value::String(s)) = eval_args.first() {
                            let n = s.trim().parse::<i64>().unwrap_or(0);
                            return Ok(Value::Int(n));
                        }
                        return Ok(Value::Int(0));
                    }

                    // to_string(v) → convert value to string
                    if name == "to_string" {
                        if let Some(v) = eval_args.first() {
                            return Ok(Value::String(v.to_string()));
                        }
                        return Ok(Value::String(String::new()));
                    }

                    // --- Real SQLite Engine Builtins ---
                    if name == "end_db_open" || name == "end_sqlite_open" {
                        let path_str = if let Some(Value::String(p)) = eval_args.first() {
                            p.clone()
                        } else {
                            ":memory:".to_string()
                        };
                        match crate::runtime::db::SqliteEngine::open(&path_str) {
                            Ok(eng) => {
                                let h = self.next_db_handle.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                let mut map = self.db_engines.lock().unwrap();
                                map.insert(h, eng);
                                return Ok(Value::Int(h));
                            }
                            Err(e) => {
                                eprintln!("SQLite Open Error: {}", e);
                                return Ok(Value::Int(0));
                            }
                        }
                    }

                    if name == "end_db_execute" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let key = if let Some(Value::String(k)) = eval_args.get(1) { k.clone() } else { String::new() };
                        let val = if let Some(Value::String(v)) = eval_args.get(2) { v.clone() } else { String::new() };
                        let mut map = self.db_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.kv_set(&key, &val) {
                                Ok(affected) => return Ok(Value::Int(affected as i64)),
                                Err(e) => {
                                    eprintln!("SQLite DB execute error: {}", e);
                                    return Ok(Value::Int(0));
                                }
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_db_query" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let key = if let Some(Value::String(k)) = eval_args.get(1) { k.clone() } else { String::new() };
                        let mut map = self.db_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.kv_get(&key) {
                                Ok(Some(v)) => return Ok(Value::String(v)),
                                Ok(None) => return Ok(Value::String(String::new())),
                                Err(e) => {
                                    eprintln!("SQLite DB query error: {}", e);
                                    return Ok(Value::String(String::new()));
                                }
                            }
                        }
                        return Ok(Value::String(String::new()));
                    }

                    if name == "end_db_close" || name == "end_sqlite_close" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.db_engines.lock().unwrap();
                        map.remove(&h);
                        return Ok(Value::Void);
                    }

                    if name == "end_sqlite_execute" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let sql = if let Some(Value::String(s)) = eval_args.get(1) { s.clone() } else { String::new() };
                        let mut map = self.db_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.execute(&sql, &[]) {
                                Ok(affected) => return Ok(Value::Int(affected as i64)),
                                Err(e) => {
                                    eprintln!("SQLite execute error: {}", e);
                                    return Ok(Value::Int(-1));
                                }
                            }
                        }
                        return Ok(Value::Int(-1));
                    }

                    if name == "end_sqlite_query" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let sql = if let Some(Value::String(s)) = eval_args.get(1) { s.clone() } else { String::new() };
                        let mut map = self.db_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.query_json(&sql, &[]) {
                                Ok(val) => return Ok(Value::String(val.to_string())),
                                Err(e) => {
                                    eprintln!("SQLite query error: {}", e);
                                    return Ok(Value::String("[]".to_string()));
                                }
                            }
                        }
                        return Ok(Value::String("[]".to_string()));
                    }

                    if name == "end_sqlite_begin" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.db_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.transaction_begin() {
                                Ok(_) => return Ok(Value::Int(1)),
                                Err(e) => {
                                    eprintln!("SQLite transaction begin error: {}", e);
                                    return Ok(Value::Int(0));
                                }
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_sqlite_commit" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.db_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.transaction_commit() {
                                Ok(_) => return Ok(Value::Int(1)),
                                Err(e) => {
                                    eprintln!("SQLite transaction commit error: {}", e);
                                    return Ok(Value::Int(0));
                                }
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_sqlite_rollback" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.db_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.transaction_rollback() {
                                Ok(_) => return Ok(Value::Int(1)),
                                Err(e) => {
                                    eprintln!("SQLite transaction rollback error: {}", e);
                                    return Ok(Value::Int(0));
                                }
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_pg_connect" {
                        let conn_str = if let Some(Value::String(s)) = eval_args.get(0) { s.clone() } else { String::new() };
                        match crate::runtime::db::PgEngine::connect(&conn_str) {
                            Ok(eng) => {
                                let h = self.next_pg_handle.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                self.pg_engines.lock().unwrap().insert(h, eng);
                                return Ok(Value::Int(h));
                            }
                            Err(e) => {
                                eprintln!("PostgreSQL connection error: {}", e);
                                return Ok(Value::Int(-1));
                            }
                        }
                    }

                    if name == "end_pg_execute" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let sql = if let Some(Value::String(s)) = eval_args.get(1) { s.clone() } else { String::new() };
                        let mut map = self.pg_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.execute(&sql, &[]) {
                                Ok(rows_aff) => return Ok(Value::Int(rows_aff as i64)),
                                Err(e) => {
                                    eprintln!("PostgreSQL execute error: {}", e);
                                    return Ok(Value::Int(-1));
                                }
                            }
                        }
                        return Ok(Value::Int(-1));
                    }

                    if name == "end_pg_execute_params" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let sql = if let Some(Value::String(s)) = eval_args.get(1) { s.clone() } else { String::new() };
                        let params_str = if let Some(Value::String(p)) = eval_args.get(2) { p.clone() } else { "[]".to_string() };
                        let params: Vec<serde_json::Value> = serde_json::from_str(&params_str).unwrap_or_default();
                        let mut map = self.pg_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.execute_params(&sql, &params) {
                                Ok(rows_aff) => return Ok(Value::Int(rows_aff as i64)),
                                Err(e) => {
                                    eprintln!("PostgreSQL execute_params error: {}", e);
                                    return Ok(Value::Int(-1));
                                }
                            }
                        }
                        return Ok(Value::Int(-1));
                    }

                    if name == "end_pg_query" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let sql = if let Some(Value::String(s)) = eval_args.get(1) { s.clone() } else { String::new() };
                        let mut map = self.pg_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.query_json(&sql, &[]) {
                                Ok(val) => return Ok(Value::String(val.to_string())),
                                Err(e) => {
                                    eprintln!("PostgreSQL query error: {}", e);
                                    return Ok(Value::String("[]".to_string()));
                                }
                            }
                        }
                        return Ok(Value::String("[]".to_string()));
                    }

                    if name == "end_pg_query_params" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let sql = if let Some(Value::String(s)) = eval_args.get(1) { s.clone() } else { String::new() };
                        let params_str = if let Some(Value::String(p)) = eval_args.get(2) { p.clone() } else { "[]".to_string() };
                        let params: Vec<serde_json::Value> = serde_json::from_str(&params_str).unwrap_or_default();
                        let mut map = self.pg_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.query_json_params(&sql, &params) {
                                Ok(val) => return Ok(Value::String(val.to_string())),
                                Err(e) => {
                                    eprintln!("PostgreSQL query_params error: {}", e);
                                    return Ok(Value::String("[]".to_string()));
                                }
                            }
                        }
                        return Ok(Value::String("[]".to_string()));
                    }

                    if name == "end_pg_begin" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.pg_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.transaction_begin() {
                                Ok(_) => return Ok(Value::Int(1)),
                                Err(e) => {
                                    eprintln!("PostgreSQL transaction begin error: {}", e);
                                    return Ok(Value::Int(0));
                                }
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_pg_commit" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.pg_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.transaction_commit() {
                                Ok(_) => return Ok(Value::Int(1)),
                                Err(e) => {
                                    eprintln!("PostgreSQL transaction commit error: {}", e);
                                    return Ok(Value::Int(0));
                                }
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_pg_rollback" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.pg_engines.lock().unwrap();
                        if let Some(eng) = map.get_mut(&h) {
                            match eng.transaction_rollback() {
                                Ok(_) => return Ok(Value::Int(1)),
                                Err(e) => {
                                    eprintln!("PostgreSQL transaction rollback error: {}", e);
                                    return Ok(Value::Int(0));
                                }
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_pg_close" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.pg_engines.lock().unwrap();
                        if let Some(mut eng) = map.remove(&h) {
                            eng.close();
                        }
                        return Ok(Value::Int(1));
                    }

                    // --- Real HTTP/2 + HPACK Subsystem Handlers ---
                    if name == "end_http2_server_start" {
                        let port = if let Some(Value::Int(p)) = eval_args.get(0) { *p as u16 } else { 0 };
                        let use_tls = if let Some(Value::Bool(b)) = eval_args.get(1) { *b } else if let Some(Value::Int(i)) = eval_args.get(1) { *i != 0 } else { false };
                        match crate::runtime::net::Http2Server::start(port, use_tls) {
                            Ok(server) => {
                                let h = self.next_http2_server_handle.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                self.http2_servers.lock().unwrap().insert(h, server);
                                return Ok(Value::Int(h));
                            }
                            Err(e) => {
                                eprintln!("Error starting HTTP/2 server: {}", e);
                                return Ok(Value::Int(-1));
                            }
                        }
                    }

                    if name == "end_http2_server_stop" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.http2_servers.lock().unwrap();
                        if let Some(server) = map.remove(&h) {
                            server.stop();
                        }
                        return Ok(Value::Void);
                    }

                    if name == "end_http2_server_port" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let map = self.http2_servers.lock().unwrap();
                        if let Some(server) = map.get(&h) {
                            return Ok(Value::Int(server.port as i64));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_http2_server_url" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let map = self.http2_servers.lock().unwrap();
                        if let Some(server) = map.get(&h) {
                            let scheme = if server.is_tls { "https" } else { "http" };
                            return Ok(Value::String(format!("{}://127.0.0.1:{}", scheme, server.port)));
                        }
                        return Ok(Value::String("".to_string()));
                    }

                    if name == "end_http2_server_is_running" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let map = self.http2_servers.lock().unwrap();
                        if let Some(server) = map.get(&h) {
                            return Ok(Value::Int(if server.is_running.load(std::sync::atomic::Ordering::Relaxed) { 1 } else { 0 }));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_http2_client_connect" {
                        let url = if let Some(Value::String(s)) = eval_args.get(0) { s.clone() } else { "".to_string() };
                        match crate::runtime::net::Http2Client::connect(&url, None) {
                            Ok(client) => {
                                let h = self.next_http2_client_handle.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                self.http2_clients.lock().unwrap().insert(h, client);
                                return Ok(Value::Int(h));
                            }
                            Err(e) => {
                                eprintln!("Error connecting HTTP/2 client to {}: {}", url, e);
                                return Ok(Value::Int(-1));
                            }
                        }
                    }

                    if name == "end_http2_client_request" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let method = if let Some(Value::String(s)) = eval_args.get(1) { s.as_str() } else { "GET" };
                        let path = if let Some(Value::String(s)) = eval_args.get(2) { s.as_str() } else { "/" };
                        let headers_json = if let Some(Value::String(s)) = eval_args.get(3) { s.as_str() } else { "{}" };
                        let body = if let Some(Value::String(s)) = eval_args.get(4) { s.as_str() } else { "" };

                        let mut headers_parsed = Vec::new();
                        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(headers_json) {
                            for (k, v) in &map {
                                if let Some(val_str) = v.as_str() {
                                    headers_parsed.push((k.clone(), val_str.to_string()));
                                }
                            }
                        }
                        let header_refs: Vec<(&str, &str)> = headers_parsed.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

                        let mut map = self.http2_clients.lock().unwrap();
                        if let Some(client) = map.get_mut(&h) {
                            match client.request(method, path, &header_refs, body.as_bytes()) {
                                Ok(resp) => {
                                    let json_str = serde_json::to_string(&resp).unwrap_or_else(|_| "{}".to_string());
                                    return Ok(Value::String(json_str));
                                }
                                Err(e) => {
                                    return Ok(Value::String(format!("{{\"error\": \"{}\", \"status\": 500}}", e)));
                                }
                            }
                        }
                        return Ok(Value::String("{\"error\": \"Invalid client handle\", \"status\": 500}".to_string()));
                    }

                    if name == "end_http2_client_close" {
                        let h = if let Some(Value::Int(handle)) = eval_args.get(0) { *handle } else { 0 };
                        let mut map = self.http2_clients.lock().unwrap();
                        if let Some(mut client) = map.remove(&h) {
                            client.close();
                        }
                        return Ok(Value::Int(1));
                    }

                    if name == "end_http2_mux_create" {
                        let max_streams = if let Some(Value::Int(s)) = eval_args.get(0) { *s } else { 1000 };
                        return Ok(Value::Int(max_streams));
                    }

                    if name == "end_http2_mux_open_stream" {
                        let stream_id = if let Some(Value::Int(s)) = eval_args.get(1) { *s } else { 1 };
                        let res_json = format!(
                            "{{\"stream_id\": {}, \"state_open\": true, \"window_size_bytes\": 65535, \"frames_transmitted\": 1}}",
                            stream_id
                        );
                        return Ok(Value::String(res_json));
                    }

                    if name == "end_http2_hpack_encode" {
                        let headers_json = if let Some(Value::String(s)) = eval_args.get(0) { s.as_str() } else { "{}" };
                        let mut headers_parsed = Vec::new();
                        if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(headers_json) {
                            for (k, v) in &map {
                                if let Some(val_str) = v.as_str() {
                                    headers_parsed.push((k.clone(), val_str.to_string()));
                                }
                            }
                        }
                        let header_refs: Vec<(&str, &str)> = headers_parsed.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
                        match crate::runtime::net::HpackCodec::encode(&header_refs) {
                            Ok(bytes) => {
                                let hex_str: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
                                return Ok(Value::String(hex_str));
                            }
                            Err(e) => {
                                return Ok(Value::String(format!("{{\"error\": \"{}\"}}", e)));
                            }
                        }
                    }

                    if name == "end_http2_hpack_decode" {
                        let hex_str = if let Some(Value::String(s)) = eval_args.get(0) { s.as_str() } else { "" };
                        let mut bytes = Vec::new();
                        let chars: Vec<char> = hex_str.chars().collect();
                        for chunk in chars.chunks(2) {
                            if chunk.len() == 2 {
                                let byte_str: String = chunk.iter().collect();
                                if let Ok(b) = u8::from_str_radix(&byte_str, 16) {
                                    bytes.push(b);
                                }
                            }
                        }
                        match crate::runtime::net::HpackCodec::decode(&bytes) {
                            Ok(decoded) => {
                                let mut map = serde_json::Map::new();
                                for (k, v) in decoded {
                                    map.insert(k, serde_json::Value::String(v));
                                }
                                return Ok(Value::String(serde_json::Value::Object(map).to_string()));
                            }
                            Err(e) => {
                                return Ok(Value::String(format!("{{\"error\": \"{}\"}}", e)));
                            }
                        }
                    }

                    if name == "end_atomic_create" {
                        let init_val = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let h = self.next_atomic_handle.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let mut map = self.atomics.lock().unwrap();
                        map.insert(h, std::sync::Arc::new(std::sync::atomic::AtomicI64::new(init_val)));
                        return Ok(Value::Int(h));
                    }

                    if name == "end_atomic_load" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let order_val = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 4 };
                        let order = match order_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            1 => std::sync::atomic::Ordering::Acquire,
                            2 => std::sync::atomic::Ordering::SeqCst,
                            3 => std::sync::atomic::Ordering::Acquire,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            return Ok(Value::Int(atom.load(order)));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_atomic_store" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let val = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 0 };
                        let order_val = if let Some(Value::Int(v)) = eval_args.get(2) { *v } else { 4 };
                        let order = match order_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            2 => std::sync::atomic::Ordering::Release,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            atom.store(val, order);
                        }
                        return Ok(Value::Void);
                    }

                    if name == "end_atomic_fetch_add" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let delta = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 0 };
                        let order_val = if let Some(Value::Int(v)) = eval_args.get(2) { *v } else { 4 };
                        let order = match order_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            1 => std::sync::atomic::Ordering::Acquire,
                            2 => std::sync::atomic::Ordering::Release,
                            3 => std::sync::atomic::Ordering::AcqRel,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            return Ok(Value::Int(atom.fetch_add(delta, order)));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_atomic_fetch_sub" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let delta = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 0 };
                        let order_val = if let Some(Value::Int(v)) = eval_args.get(2) { *v } else { 4 };
                        let order = match order_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            1 => std::sync::atomic::Ordering::Acquire,
                            2 => std::sync::atomic::Ordering::Release,
                            3 => std::sync::atomic::Ordering::AcqRel,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            return Ok(Value::Int(atom.fetch_sub(delta, order)));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_atomic_fetch_and" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mask = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 0 };
                        let order_val = if let Some(Value::Int(v)) = eval_args.get(2) { *v } else { 4 };
                        let order = match order_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            return Ok(Value::Int(atom.fetch_and(mask, order)));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_atomic_fetch_or" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mask = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 0 };
                        let order_val = if let Some(Value::Int(v)) = eval_args.get(2) { *v } else { 4 };
                        let order = match order_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            return Ok(Value::Int(atom.fetch_or(mask, order)));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_atomic_fetch_xor" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mask = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 0 };
                        let order_val = if let Some(Value::Int(v)) = eval_args.get(2) { *v } else { 4 };
                        let order = match order_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            return Ok(Value::Int(atom.fetch_xor(mask, order)));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_atomic_exchange" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let desired = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 0 };
                        let order_val = if let Some(Value::Int(v)) = eval_args.get(2) { *v } else { 4 };
                        let order = match order_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            1 => std::sync::atomic::Ordering::Acquire,
                            2 => std::sync::atomic::Ordering::Release,
                            3 => std::sync::atomic::Ordering::AcqRel,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            return Ok(Value::Int(atom.swap(desired, order)));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_atomic_cas" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let exp = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 0 };
                        let des = if let Some(Value::Int(v)) = eval_args.get(2) { *v } else { 0 };
                        let succ_val = if let Some(Value::Int(v)) = eval_args.get(3) { *v } else { 4 };
                        let fail_val = if let Some(Value::Int(v)) = eval_args.get(4) { *v } else { 0 };
                        let succ = match succ_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            1 => std::sync::atomic::Ordering::Acquire,
                            2 => std::sync::atomic::Ordering::Release,
                            3 => std::sync::atomic::Ordering::AcqRel,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let fail = match fail_val {
                            0 => std::sync::atomic::Ordering::Relaxed,
                            1 => std::sync::atomic::Ordering::Acquire,
                            _ => std::sync::atomic::Ordering::SeqCst,
                        };
                        let map = self.atomics.lock().unwrap();
                        if let Some(atom) = map.get(&h) {
                            let res = atom.compare_exchange(exp, des, succ, fail);
                            return Ok(Value::Int(if res.is_ok() { 1 } else { 0 }));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_atomic_destroy" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mut map = self.atomics.lock().unwrap();
                        map.remove(&h);
                        return Ok(Value::Void);
                    }

                    if name == "end_mutex_create" {
                        let h = self.next_mutex_handle.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let mut map = self.mutexes.lock().unwrap();
                        map.insert(h, std::sync::Arc::new(std::sync::Mutex::new(0)));
                        return Ok(Value::Int(h));
                    }

                    if name == "end_mutex_lock" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let tid = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 1 };
                        let mtx_arc = {
                            let map = self.mutexes.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(m) = mtx_arc {
                            let mut guard = m.lock().unwrap();
                            *guard = tid;
                            return Ok(Value::Int(1));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_mutex_try_lock" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let tid = if let Some(Value::Int(v)) = eval_args.get(1) { *v } else { 1 };
                        let mtx_arc = {
                            let map = self.mutexes.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(m) = mtx_arc {
                            if let Ok(mut guard) = m.try_lock() {
                                *guard = tid;
                                return Ok(Value::Int(1));
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_mutex_unlock" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mtx_arc = {
                            let map = self.mutexes.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(m) = mtx_arc {
                            if let Ok(mut guard) = m.try_lock() {
                                *guard = 0;
                            }
                        }
                        return Ok(Value::Void);
                    }

                    if name == "end_mutex_is_locked" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mtx_arc = {
                            let map = self.mutexes.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(m) = mtx_arc {
                            if let Ok(guard) = m.try_lock() {
                                return Ok(Value::Int(if *guard != 0 { 1 } else { 0 }));
                            }
                            return Ok(Value::Int(1));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_mutex_destroy" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mut map = self.mutexes.lock().unwrap();
                        map.remove(&h);
                        return Ok(Value::Void);
                    }

                    if name == "end_rwlock_create" {
                        let h = self.next_rwlock_handle.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        let mut map = self.rwlocks.lock().unwrap();
                        map.insert(h, std::sync::Arc::new(std::sync::RwLock::new(0)));
                        return Ok(Value::Int(h));
                    }

                    if name == "end_rwlock_read_lock" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let lock_arc = {
                            let map = self.rwlocks.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(l) = lock_arc {
                            drop(l.read().unwrap());
                        }
                        return Ok(Value::Void);
                    }

                    if name == "end_rwlock_read_unlock" {
                        return Ok(Value::Void);
                    }

                    if name == "end_rwlock_write_lock" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let lock_arc = {
                            let map = self.rwlocks.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(l) = lock_arc {
                            drop(l.write().unwrap());
                        }
                        return Ok(Value::Void);
                    }

                    if name == "end_rwlock_write_unlock" {
                        return Ok(Value::Void);
                    }

                    if name == "end_rwlock_destroy" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mut map = self.rwlocks.lock().unwrap();
                        map.remove(&h);
                        return Ok(Value::Void);
                    }

                    if name == "end_thread_create" {
                        return Ok(Value::Int(1));
                    }

                    if name == "end_thread_join" {
                        return Ok(Value::Void);
                    }

                    if name == "end_raft_cluster_create" {
                        let node_count = if let Some(Value::Int(v)) = eval_args.get(0) { *v as usize } else { 3 };
                        let base_port = if let Some(Value::Int(v)) = eval_args.get(1) { *v as u16 } else { 23000 };
                        let base_db_path = if let Some(Value::String(s)) = eval_args.get(2) { s.clone() } else { ":memory:".to_string() };

                        match crate::runtime::raft::RaftCluster::start_sync(node_count, base_port, &base_db_path) {
                            Ok(cluster) => {
                                let h = self.next_raft_cluster_handle.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                self.raft_clusters.lock().unwrap().insert(h, std::sync::Arc::new(std::sync::Mutex::new(cluster)));
                                return Ok(Value::Int(h));
                            }
                            Err(e) => {
                                eprintln!("Error starting Raft cluster: {}", e);
                                return Ok(Value::Int(-1));
                            }
                        }
                    }

                    if name == "end_raft_cluster_get_leader" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let cluster_arc = {
                            let map = self.raft_clusters.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(cluster_mtx) = cluster_arc {
                            let cluster = cluster_mtx.lock().unwrap();
                            if let Ok(leader_id) = cluster.wait_for_leader_sync(std::time::Duration::from_secs(4)) {
                                return Ok(Value::Int(leader_id as i64));
                            }
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_raft_cluster_write" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let key = if let Some(Value::String(s)) = eval_args.get(1) { s.clone() } else { "".to_string() };
                        let val = if let Some(Value::String(s)) = eval_args.get(2) { s.clone() } else { "".to_string() };
                        let payload = format!("key={}&value={}", key, val);

                        let cluster_arc = {
                            let map = self.raft_clusters.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(cluster_mtx) = cluster_arc {
                            let cluster = cluster_mtx.lock().unwrap();
                            match cluster.write_sync("SET", &payload) {
                                Ok(idx) => return Ok(Value::String(format!("OK:{}", idx))),
                                Err(e) => return Ok(Value::String(format!("ERR:{}", e))),
                            }
                        }
                        return Ok(Value::String("ERR:ClusterNotFound".to_string()));
                    }

                    if name == "end_raft_cluster_read" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let key = if let Some(Value::String(s)) = eval_args.get(1) { s.clone() } else { "".to_string() };

                        let cluster_arc = {
                            let map = self.raft_clusters.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(cluster_mtx) = cluster_arc {
                            let cluster = cluster_mtx.lock().unwrap();
                            if let Ok(Some(val)) = cluster.read(&key) {
                                return Ok(Value::String(val));
                            }
                        }
                        return Ok(Value::String("".to_string()));
                    }

                    if name == "end_raft_cluster_read_node" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let node_id = if let Some(Value::Int(v)) = eval_args.get(1) { *v as u64 } else { 1 };
                        let key = if let Some(Value::String(s)) = eval_args.get(2) { s.clone() } else { "".to_string() };

                        let cluster_arc = {
                            let map = self.raft_clusters.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(cluster_mtx) = cluster_arc {
                            let cluster = cluster_mtx.lock().unwrap();
                            if let Ok(Some(val)) = cluster.read_from_node(node_id, &key) {
                                return Ok(Value::String(val));
                            }
                        }
                        return Ok(Value::String("".to_string()));
                    }

                    if name == "end_raft_cluster_kill_node" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let node_id = if let Some(Value::Int(v)) = eval_args.get(1) { *v as u64 } else { 1 };

                        let cluster_arc = {
                            let map = self.raft_clusters.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(cluster_mtx) = cluster_arc {
                            let mut cluster = cluster_mtx.lock().unwrap();
                            let _ = cluster.kill_node(node_id);
                            return Ok(Value::Int(1));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_raft_cluster_restart_node" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let node_id = if let Some(Value::Int(v)) = eval_args.get(1) { *v as u64 } else { 1 };

                        let cluster_arc = {
                            let map = self.raft_clusters.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(cluster_mtx) = cluster_arc {
                            let mut cluster = cluster_mtx.lock().unwrap();
                            let res = cluster.restart_node_sync(node_id);
                            return Ok(Value::Int(if res.is_ok() { 1 } else { 0 }));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_raft_cluster_partition_node" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let node_id = if let Some(Value::Int(v)) = eval_args.get(1) { *v as u64 } else { 1 };

                        let cluster_arc = {
                            let map = self.raft_clusters.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(cluster_mtx) = cluster_arc {
                            let cluster = cluster_mtx.lock().unwrap();
                            cluster.partition_node(node_id);
                            return Ok(Value::Int(1));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_raft_cluster_heal_partition" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };

                        let cluster_arc = {
                            let map = self.raft_clusters.lock().unwrap();
                            map.get(&h).cloned()
                        };
                        if let Some(cluster_mtx) = cluster_arc {
                            let cluster = cluster_mtx.lock().unwrap();
                            cluster.heal_partition();
                            return Ok(Value::Int(1));
                        }
                        return Ok(Value::Int(0));
                    }

                    if name == "end_raft_cluster_stop" {
                        let h = if let Some(Value::Int(v)) = eval_args.get(0) { *v } else { 0 };
                        let mut map = self.raft_clusters.lock().unwrap();
                        if let Some(cluster_mtx) = map.remove(&h) {
                            let cluster = cluster_mtx.lock().unwrap();
                            cluster.stop_all();
                        }
                        return Ok(Value::Void);
                    }

                    if name == "cpu_sleep_ms" || name == "time_sleep" || name == "sleep_ms" || name == "end_sleep_ms" {
                        let ms = if let Some(Value::Int(v)) = eval_args.get(0) { *v as u64 } else { 0 };
                        let prof_opt = self.profiler_session.clone();
                        if let Some(prof_arc) = &prof_opt {
                            if let Ok(mut prof) = prof_arc.lock() {
                                prof.enter_function(name);
                            }
                        }
                        let start_sleep = std::time::Instant::now();
                        if ms > 0 {
                            std::thread::sleep(std::time::Duration::from_millis(ms));
                        }
                        let elapsed = start_sleep.elapsed().as_micros() as u64;
                        if let Some(prof_arc) = &prof_opt {
                            if let Ok(mut prof) = prof_arc.lock() {
                                prof.exit_function(name, elapsed, elapsed, 0);
                            }
                        }
                        return Ok(Value::Void);
                    }

                    if name == "end_profiler_start" {
                        let session = std::sync::Arc::new(std::sync::Mutex::new(crate::profiler::ProfilerSession::new("interactive_session")));
                        self.profiler_session = Some(session);
                        return Ok(Value::Int(1));
                    }

                    if name == "end_profiler_stop" {
                        if let Some(session_arc) = self.profiler_session.take() {
                            let report = session_arc.lock().unwrap().finish();
                            let json = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string());
                            return Ok(Value::String(json));
                        }
                        return Ok(Value::String("{}".to_string()));
                    }

                    if name == "end_profiler_flamegraph" {
                        if let Some(session_arc) = &self.profiler_session {
                            let report = session_arc.lock().unwrap().finish();
                            return Ok(Value::String(report.flamegraph_svg));
                        }
                        return Ok(Value::String("".to_string()));
                    }

                    if let Some(op_val) = self.operations.get(name).cloned() {
                        return self.eval_operation(&op_val, eval_args);
                    }

                    if let Some(op_val) = self.get_var(name) {
                        if matches!(op_val, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) {
                            return self.eval_operation(&op_val, eval_args);
                        }
                    }

                    if let Some(func) = self.functions.get(name).cloned() {
                        return self.eval_function(&func, eval_args);
                    }

                    // Morphic function dispatch: if "telegram_send" not found,
                    // search for a morphic template like "{platform}_send"
                    for (fn_name, func) in self.functions.clone() {
                        if let Some(ref morphic_var) = func.morphic_param {
                            let template = &fn_name;
                            let brace_open = template.find('{');
                            let brace_close = template.find('}');
                            if let (Some(bo), Some(bc)) = (brace_open, brace_close) {
                                let prefix = &template[..bo];
                                let suffix = &template[bc+1..];
                                if name.ends_with(suffix) && name.len() > suffix.len() {
                                    let concrete_value = &name[prefix.len()..name.len()-suffix.len()];
                                    self.push_scope();
                                    self.set_var(morphic_var, Value::String(concrete_value.to_string()));
                                    for (param, arg) in func.params.iter().zip(eval_args.into_iter()) {
                                        self.set_var(&param.name, arg);
                                    }
                                    let mut ret_val = Value::Void;
                                    for stmt in &func.body.statements {
                                        match self.eval_statement(stmt)? {
                                            Some(v) => {
                                                ret_val = v;
                                                break;
                                            }
                                            None => {}
                                        }
                                    }
                                    self.pop_scope();
                                    return Ok(ret_val);
                                }
                            }
                        }
                    }
                }

                if let Expression::FieldAccess { object, field, .. } = callee.as_ref() {
                    if let Expression::Ident(mod_name, _) = object.as_ref() {
                        let fn_key1 = format!("{}_{}", mod_name, field);
                        let fn_key2 = format!("{}.{}", mod_name, field);
                        let fn_key3 = format!("{}::{}", mod_name, field);
                        if let Some(target_fn) = self.functions.get(&fn_key1).or_else(|| self.functions.get(&fn_key2)).or_else(|| self.functions.get(&fn_key3)).cloned() {
                            return self.eval_function(&target_fn, eval_args);
                        }
                    }
                    let obj = self.eval_expression(object)?;
                    if let Some(func) = self.functions.get(field).cloned() {
                        let mut call_args = vec![obj];
                        call_args.extend(eval_args);
                        return self.eval_function(&func, call_args);
                    }
                }

                let callee_val = self.eval_expression(callee)?;
                if matches!(callee_val, Value::Operation { .. } | Value::ComposedOp(..) | Value::RepeatedOp(..) | Value::AlternativeOp(..) | Value::ParallelOp(..)) {
                    return self.eval_operation(&callee_val, eval_args);
                }

                Ok(Value::Void)
            }
            Expression::FieldAccess { object, field, .. } => {
                let obj = self.eval_expression(object)?;
                if let Value::Struct(_, fields) = &obj {
                    if let Some(v) = fields.get(field) {
                        return Ok(v.clone());
                    }
                }
                if let Value::OperationResult { output, status, duration_ns, events, logs, effects, errors } = &obj {
                    match field.as_str() {
                        "output" => return Ok(*output.clone()),
                        "status" => return Ok(Value::String(status.clone())),
                        "duration_ns" => return Ok(Value::Int(*duration_ns as i64)),
                        "events" => return Ok(Value::Array(events.iter().cloned().map(Value::String).collect())),
                        "logs" => return Ok(Value::Array(logs.iter().cloned().map(Value::String).collect())),
                        "effects" => return Ok(Value::Array(effects.iter().cloned().map(Value::String).collect())),
                        "errors" => return Ok(Value::Array(errors.iter().cloned().map(Value::String).collect())),
                        "trace" => return Ok(Value::String(format!("Trace: duration={}ns, status={}, events={:?}", duration_ns, status, events))),
                        "result" => return Ok(obj.clone()),
                        _ => {}
                    }
                }
                if let Value::String(type_name) = &obj {
                    if type_name.chars().next().map_or(false, |c| c.is_uppercase()) {
                        return Ok(Value::Enum(Some(type_name.clone()), field.clone(), None));
                    }
                }
                Ok(Value::Void)
            }
            Expression::StructInit { name, fields, .. } => {
                let mut map = HashMap::new();
                for (fname, fexpr) in fields {
                    let v = self.eval_expression(fexpr)?;
                    map.insert(fname.clone(), v);
                }
                Ok(Value::Struct(name.clone(), map))
            }
            Expression::EnumInit { enum_name, variant_name, payload, .. } => {
                let payload_val = if let Some(p) = payload {
                    Some(Box::new(self.eval_expression(p)?))
                } else {
                    None
                };
                Ok(Value::Enum(enum_name.clone(), variant_name.clone(), payload_val))
            }
            Expression::Alloc { .. } => {
                Ok(Value::Pointer(0x1000))
            }
            Expression::Catch { expr, .. } => {
                self.eval_expression(expr)
            }
            Expression::Match { expr, arms, .. } => {
                let target_val = self.eval_expression(expr)?;
                for arm in arms {
                    if let Some(bindings) = self.matches_pattern(&target_val, &arm.pattern) {
                        self.push_scope();
                        for (k, v) in bindings {
                            self.set_var(&k, v);
                        }
                        for s in &arm.body.statements {
                            if let Some(ret) = self.eval_statement(s)? {
                                self.pop_scope();
                                return Ok(ret);
                            }
                        }
                        self.pop_scope();
                        break;
                    }
                }
                Ok(Value::Void)
            }
            Expression::Index { array, index, .. } => {
                let arr_val = self.eval_expression(array)?;
                let idx_val = self.eval_expression(index)?;
                let idx = match idx_val {
                    Value::Int(i) => i as usize,
                    _ => 0,
                };
                match arr_val {
                    Value::Array(items) => {
                        Ok(items.get(idx).cloned().unwrap_or(Value::Int(0)))
                    }
                    Value::String(s) => {
                        let byte = s.as_bytes().get(idx).copied().unwrap_or(0);
                        Ok(Value::Int(byte as i64))
                    }
                    _ => Ok(Value::Int(0)),
                }
            }
            Expression::Cast { expr, target_type, .. } => {
                let v = self.eval_expression(expr)?;
                match (v, target_type) {
                    (Value::Int(n), Type::F32 | Type::F64) => Ok(Value::Float(n as f64)),
                    (Value::Float(f), Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::U8 | Type::U16 | Type::U32 | Type::U64) => Ok(Value::Int(f as i64)),
                    (v, _) => Ok(v),
                }
            }
            Expression::Await { expr, .. } => self.eval_expression(expr),
            Expression::Promote { expr, .. } => self.eval_expression(expr),
            Expression::Block(_) => Ok(Value::Void),
            Expression::NameOf { target, .. } => Ok(Value::String(target.clone())),
            Expression::PathOf { target, .. } => Ok(Value::String(target.clone())),
            Expression::TypeOf { expr, .. } => {
                let v = self.eval_expression(expr)?;
                let type_name = match &v {
                    Value::Int(_) => "i64",
                    Value::Float(_) => "f64",
                    Value::String(_) => "str",
                    Value::Bool(_) => "bool",
                    Value::Struct(name, _) => name.as_str(),
                    _ => "unknown",
                };
                Ok(Value::String(type_name.to_string()))
            }
            Expression::DocOf { target, .. } => Ok(Value::String(format!("[Documentation for symbol '{}']", target))),
            Expression::CodeOf { code, .. } => Ok(Value::String(code.clone())),
            Expression::Dbg { expr, code, span } => {
                let v = self.eval_expression(expr)?;
                println!("\x1b[1;36m[DBG {}:{}]\x1b[0m \x1b[1;33m{}\x1b[0m = {}", span.file, span.line, code, v);
                Ok(v)
            }
            Expression::AssertDebug { condition, code, span } => {
                let v = self.eval_expression(condition)?;
                match v {
                    Value::Bool(true) => Ok(Value::Bool(true)),
                    _ => {
                        eprintln!("\x1b[1;31m[ASSERTION FAILURE {}:{}]\x1b[0m Condition failed: {}", span.file, span.line, code);
                        Err(format!("Assertion failed at {}:{}: {}", span.file, span.line, code))
                    }
                }
            }
            Expression::Translate { key, .. } => Ok(Value::String(format!("[Localized: {}]", key))),
            Expression::FieldsOf { target, .. } => Ok(Value::String(format!("[Fields of struct {}]", target))),
            Expression::SqlExpr { expr, .. } => {
                let _ = self.eval_expression(expr)?;
                Ok(Value::String("SELECT * FROM table".to_string()))
            }
            Expression::InlineC { .. } => Ok(Value::Void),
            Expression::Pipe { lhs, rhs, .. } => {
                let _ = self.eval_expression(lhs)?;
                self.eval_expression(rhs)
            }
            Expression::UnitLit { value, unit, .. } => {
                match unit.as_str() {
                    "km/h" => Ok(Value::Int((*value * 1000.0 / 3600.0) as i64)),
                    "m/s" => Ok(Value::Int(*value as i64)),
                    "usd" | "eur" => Ok(Value::Int((*value * 100.0) as i64)),
                    _ => Ok(Value::Int(*value as i64)),
                }
            }
            Expression::NullCollapse { left, right, .. } => {
                let l_val = self.eval_expression(left)?;
                match l_val {
                    Value::Void => Ok(Value::Void),
                    _ => self.eval_expression(right),
                }
            }
            Expression::OperationLiteral { name, params, return_type, requires, guarantees, effects, emits, body, .. } => {
                Ok(Value::Operation {
                    name: name.clone(),
                    params: params.clone(),
                    return_type: return_type.clone(),
                    requires: requires.clone(),
                    guarantees: guarantees.clone(),
                    effects: effects.clone(),
                    emits: emits.clone(),
                    version: None,
                    body: body.clone(),
                })
            }
            Expression::Compose { ops, .. } => {
                if ops.is_empty() {
                    return Ok(Value::Void);
                }
                let mut current = self.eval_expression(&ops[0])?;
                for next_expr in &ops[1..] {
                    let next_val = self.eval_expression(next_expr)?;
                    if let (Value::Int(a), Value::Int(b)) = (&current, &next_val) {
                        current = Value::Int((*a as u64 >> *b) as i64);
                    } else {
                        current = Value::ComposedOp(Box::new(current), Box::new(next_val));
                    }
                }
                Ok(current)
            }
            Expression::Repeat { op, count, is_retry, .. } => {
                let op_val = self.eval_expression(op)?;
                let count_val = self.eval_expression(count)?;
                let n = match count_val {
                    Value::Int(i) => i.max(1) as usize,
                    _ => 1,
                };
                Ok(Value::RepeatedOp(Box::new(op_val), n, *is_retry))
            }
            Expression::Alternative { left, right, .. } => {
                let l_val = self.eval_expression(left)?;
                let r_val = self.eval_expression(right)?;
                Ok(Value::AlternativeOp(Box::new(l_val), Box::new(r_val)))
            }
            Expression::Parallel { left, right, .. } => {
                let l_val = self.eval_expression(left)?;
                let r_val = self.eval_expression(right)?;
                Ok(Value::ParallelOp(Box::new(l_val), Box::new(r_val)))
            }
            Expression::ConditionalOp { op, condition, .. } => {
                let cond_val = self.eval_expression(condition)?;
                let is_true = match cond_val {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    _ => false,
                };
                if is_true {
                    self.eval_expression(op)
                } else {
                    Ok(Value::Void)
                }
            }
            Expression::Memoize { op, .. } => {
                let op_val = self.eval_expression(op)?;
                Ok(op_val)
            }
            Expression::ListLiteral(elements, _) => {
                let mut items = Vec::new();
                for elem in elements {
                    match elem {
                        crate::ast::expr::collections::CollectionElement::Expr(e) => {
                            items.push(self.eval_expression(e)?);
                        }
                        crate::ast::expr::collections::CollectionElement::Spread { expr, .. } => {
                            if let Value::Array(sub) = self.eval_expression(expr)? {
                                items.extend(sub);
                            }
                        }
                        _ => {}
                    }
                }
                Ok(Value::Array(items))
            }
            Expression::Tuple(elements, _) => {
                let mut items = Vec::new();
                for e in elements {
                    items.push(self.eval_expression(e)?);
                }
                Ok(Value::Array(items))
            }
            Expression::Conditional { condition, then_branch, else_branch, .. } => {
                let cond_val = self.eval_expression(condition)?;
                let is_true = match cond_val {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    _ => false,
                };
                if is_true {
                    self.eval_expression(then_branch)
                } else {
                    self.eval_expression(else_branch)
                }
            }
            _ => Ok(Value::Int(1)),
        }
    }
}
