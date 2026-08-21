use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Void,
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Struct(String, HashMap<String, Value>),
    Enum(Option<String>, String, Option<Box<Value>>),
    Pointer(usize),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Struct(name, fields) => {
                write!(f, "{} {{ ", name)?;
                for (k, v) in fields {
                    write!(f, "{}: {}, ", k, v)?;
                }
                write!(f, "}}")
            }
            Value::Enum(ename, vname, payload) => {
                if let Some(en) = ename {
                    write!(f, "{}.{}", en, vname)?;
                } else {
                    write!(f, ".{}", vname)?;
                }
                if let Some(p) = payload {
                    write!(f, "({})", p)?;
                }
                Ok(())
            }
            Value::Pointer(p) => write!(f, "*0x{:x}", p),
        }
    }
}

pub struct Interpreter {
    variables: Vec<HashMap<String, Value>>,
    functions: HashMap<String, FunctionDef>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: vec![HashMap::new()],
            functions: HashMap::new(),
        }
    }

    pub fn run(&mut self, module: &Module) -> Result<Value, String> {
        for f in &module.functions {
            self.functions.insert(f.name.clone(), f.clone());
        }

        if let Some(main_fn) = self.functions.get("main").cloned() {
            self.eval_function(&main_fn, vec![])
        } else {
            Err("No 'main' function found in module".to_string())
        }
    }

    fn push_scope(&mut self) {
        self.variables.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.variables.pop();
    }

    fn set_var(&mut self, name: &str, val: Value) {
        if let Some(scope) = self.variables.last_mut() {
            scope.insert(name.to_string(), val);
        }
    }

    fn update_var(&mut self, name: &str, val: Value) -> Result<(), String> {
        for scope in self.variables.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), val);
                return Ok(());
            }
        }
        self.set_var(name, val);
        Ok(())
    }

    fn get_var(&self, name: &str) -> Option<Value> {
        for scope in self.variables.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    fn eval_function(&mut self, func: &FunctionDef, args: Vec<Value>) -> Result<Value, String> {
        self.push_scope();

        for (param, arg) in func.params.iter().zip(args.into_iter()) {
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
        Ok(ret_val)
    }

    fn eval_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, String> {
        match stmt {
            Statement::VarDecl {
                name, initializer, ..
            } => {
                let val = if let Some(init) = initializer {
                    self.eval_expression(init)?
                } else {
                    Value::Void
                };
                self.set_var(name, val);
                Ok(None)
            }
            Statement::Assignment { target, value, .. } => {
                let val = self.eval_expression(value)?;
                match target {
                    Expression::Ident(name, _) => {
                        self.update_var(name, val)?;
                    }
                    _ => {}
                }
                Ok(None)
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    let val = self.eval_expression(v)?;
                    Ok(Some(val))
                } else {
                    Ok(Some(Value::Void))
                }
            }
            Statement::Expression(expr) => {
                self.eval_expression(expr)?;
                Ok(None)
            }
            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond = self.eval_expression(condition)?;
                let is_true = match cond {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    _ => false,
                };

                if is_true {
                    for s in &then_block.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                } else if let Some(eb) = else_block {
                    for s in &eb.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::While { condition, body, .. } => {
                loop {
                    let cond = self.eval_expression(condition)?;
                    let is_true = match cond {
                        Value::Bool(b) => b,
                        Value::Int(n) => n != 0,
                        _ => false,
                    };
                    if !is_true {
                        break;
                    }
                    for s in &body.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(ret));
                        }
                    }
                }
                Ok(None)
            }
            Statement::ForIn {
                item_name,
                iterable,
                body,
                ..
            } | Statement::ParallelFor {
                item_name,
                iterable,
                body,
                ..
            } => {
                let count = match self.eval_expression(iterable)? {
                    Value::Int(n) => n,
                    _ => 0,
                };
                self.push_scope();
                for i in 0..count {
                    self.set_var(item_name, Value::Int(i));
                    for s in &body.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(ret));
                        }
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::Match { expr, arms, .. } => {
                let target_val = self.eval_expression(expr)?;
                for arm in arms {
                    if let Some(bindings) = self.matches_pattern(&target_val, &arm.pattern) {
                        self.push_scope();
                        for (k, v) in bindings {
                            self.set_var(&k, v);
                        }

                        let guard_ok = if let Some(g) = &arm.guard {
                            match self.eval_expression(g)? {
                                Value::Bool(b) => b,
                                _ => false,
                            }
                        } else {
                            true
                        };

                        if guard_ok {
                            for s in &arm.body.statements {
                                if let Some(ret) = self.eval_statement(s)? {
                                    self.pop_scope();
                                    return Ok(Some(ret));
                                }
                            }
                            self.pop_scope();
                            break;
                        }
                        self.pop_scope();
                    }
                }
                Ok(None)
            }
            Statement::RegionBlock { name, body, .. } => {
                self.push_scope();
                self.set_var(&format!("region_{}", name), Value::String(format!("Region<{}>", name)));
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(ret));
                    }
                }
                self.pop_scope();
                Ok(None)
            }
            Statement::AsmBlock { arch, .. } => {
                // In interpreter mode, simulate asm execution
                let _ = arch;
                Ok(None)
            }
            Statement::TargetBlock { body, .. } => {
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        return Ok(Some(ret));
                    }
                }
                Ok(None)
            }
            Statement::Defer { expr, .. } => {
                let _ = expr;
                Ok(None)
            }
            Statement::Spawn { call, .. } => {
                let _ = self.eval_expression(call)?;
                Ok(None)
            }
        }
    }

    fn matches_pattern(&self, target: &Value, pattern: &Pattern) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard => Some(Vec::new()),
            Pattern::Ident(id) => Some(vec![(id.clone(), target.clone())]),
            Pattern::Literal(lit) => {
                let lit_val = match lit {
                    Literal::Int(n) => Value::Int(*n),
                    Literal::Float(f) => Value::Float(*f),
                    Literal::String(s) => Value::String(s.clone()),
                    Literal::Bool(b) => Value::Bool(*b),
                    Literal::Null => Value::Pointer(0),
                };
                if *target == lit_val {
                    Some(Vec::new())
                } else {
                    None
                }
            }
            Pattern::Variant { variant_name, binding, .. } => {
                if let Value::Enum(_, vname, payload) = target {
                    if vname == variant_name {
                        let mut out = Vec::new();
                        if let (Some(b), Some(p)) = (binding, payload) {
                            out.push((b.clone(), *p.clone()));
                        }
                        return Some(out);
                    }
                }
                None
            }
        }
    }

    fn eval_expression(&mut self, expr: &Expression) -> Result<Value, String> {
        match expr {
            Expression::Lit(lit, _) => match lit {
                Literal::Int(n) => Ok(Value::Int(*n)),
                Literal::Float(f) => Ok(Value::Float(*f)),
                Literal::String(s) => Ok(Value::String(s.clone())),
                Literal::Bool(b) => Ok(Value::Bool(*b)),
                Literal::Null => Ok(Value::Pointer(0)),
            },
            Expression::Ident(name, _) => {
                if let Some(v) = self.get_var(name) {
                    Ok(v)
                } else {
                    Ok(Value::String(name.clone()))
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let l = self.eval_expression(left)?;
                let r = self.eval_expression(right)?;

                match (l, op, r) {
                    (Value::Int(a), BinaryOp::Add, Value::Int(b)) => Ok(Value::Int(a + b)),
                    (Value::Int(a), BinaryOp::Sub, Value::Int(b)) => Ok(Value::Int(a - b)),
                    (Value::Int(a), BinaryOp::Mul, Value::Int(b)) => Ok(Value::Int(a * b)),
                    (Value::Int(a), BinaryOp::Div, Value::Int(b)) => {
                        if b == 0 {
                            Err("Division by zero in End runtime".to_string())
                        } else {
                            Ok(Value::Int(a / b))
                        }
                    }
                    (Value::Int(a), BinaryOp::Equal, Value::Int(b)) => Ok(Value::Bool(a == b)),
                    (Value::Int(a), BinaryOp::LessThan, Value::Int(b)) => Ok(Value::Bool(a < b)),
                    (Value::Int(a), BinaryOp::GreaterThan, Value::Int(b)) => Ok(Value::Bool(a > b)),
                    (Value::String(a), BinaryOp::Add, Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                    _ => Ok(Value::Int(0)),
                }
            }
            Expression::Unary { op, expr, .. } => {
                let v = self.eval_expression(expr)?;
                match (op, v) {
                    (UnaryOp::Negate, Value::Int(n)) => Ok(Value::Int(-n)),
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
                            println!("{}", msg);
                        } else {
                            print!("{}", msg);
                        }
                        return Ok(Value::Void);
                    }

                    if let Some(func) = self.functions.get(name).cloned() {
                        return self.eval_function(&func, eval_args);
                    }
                }

                Ok(Value::Void)
            }
            Expression::FieldAccess { object, field, .. } => {
                let obj = self.eval_expression(object)?;
                if let Value::Struct(_, fields) = obj {
                    if let Some(v) = fields.get(field) {
                        return Ok(v.clone());
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
            Expression::Index { .. } => Ok(Value::Int(0)),
            Expression::Promote { expr, .. } => self.eval_expression(expr),
            Expression::Block(_) => Ok(Value::Void),
        }
    }
}
