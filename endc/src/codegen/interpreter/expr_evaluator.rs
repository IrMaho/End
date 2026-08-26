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
