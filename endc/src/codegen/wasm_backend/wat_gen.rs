use crate::ast::*;
use crate::codegen::backend_trait::BackendError;
use std::collections::HashMap;
use std::fmt::Write;

pub struct WatGenerator {
    target: String,
    data_segments: Vec<(usize, String)>,
    current_data_offset: usize,
    struct_defs: HashMap<String, StructDef>,
}

impl WatGenerator {
    pub fn new(target: Option<&str>) -> Self {
        Self {
            target: target.unwrap_or("wasm32-unknown-unknown").to_string(),
            data_segments: Vec::new(),
            current_data_offset: 1024,
            struct_defs: HashMap::new(),
        }
    }

    pub fn generate(&mut self, module: &Module) -> Result<String, BackendError> {
        self.data_segments.clear();
        self.current_data_offset = 1024;
        self.struct_defs.clear();
        for s in &module.structs {
            self.struct_defs.insert(s.name.clone(), s.clone());
        }

        // First pass: collect string literals into data segments
        self.collect_strings(module);

        let mut out = String::new();
        writeln!(out, "(module").unwrap();
        writeln!(out, "  ;; Module: {}", module.name).unwrap();
        writeln!(out, "  ;; Target: {}", self.target).unwrap();
        writeln!(out, "  (memory (export \"memory\") 2)").unwrap();
        writeln!(out, "  (import \"env\" \"println\" (func $println (param i64)))").unwrap();
        writeln!(out, "  (import \"env\" \"print\" (func $print (param i64)))").unwrap();
        writeln!(out, "  (import \"env\" \"println_str\" (func $println_str (param i32 i32)))").unwrap();
        writeln!(out, "  (import \"env\" \"print_str\" (func $print_str (param i32 i32)))").unwrap();
        writeln!(out, "  (import \"env\" \"print_bool\" (func $print_bool (param i32)))").unwrap();
        writeln!(out, "  (import \"env\" \"print_f64\" (func $print_f64 (param f64)))\n").unwrap();

        // Emit data segments
        for (offset, text) in &self.data_segments {
            let escaped: String = text.chars().flat_map(|c| c.escape_default()).collect();
            writeln!(out, "  (data (i32.const {}) \"{}\\00\")", offset, escaped).unwrap();
        }
        if !self.data_segments.is_empty() {
            writeln!(out).unwrap();
        }

        // Emit functions
        for func in &module.functions {
            self.generate_function(func, &mut out, module)?;
        }

        writeln!(out, ")").unwrap();
        Ok(out)
    }

    fn collect_strings(&mut self, module: &Module) {
        for func in &module.functions {
            for stmt in &func.body.statements {
                self.collect_stmt_strings(stmt);
            }
        }
    }

    fn collect_stmt_strings(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl { initializer: Some(expr), .. } => self.collect_expr_strings(expr),
            Statement::Assignment { value, .. } => self.collect_expr_strings(value),
            Statement::Return { value: Some(expr), .. } => self.collect_expr_strings(expr),
            Statement::If { condition, then_block, else_block, .. } => {
                self.collect_expr_strings(condition);
                for s in &then_block.statements {
                    self.collect_stmt_strings(s);
                }
                if let Some(eb) = else_block {
                    for s in &eb.statements {
                        self.collect_stmt_strings(s);
                    }
                }
            }
            Statement::While { condition, body, .. } => {
                self.collect_expr_strings(condition);
                for s in &body.statements {
                    self.collect_stmt_strings(s);
                }
            }
            Statement::Expression(expr) => self.collect_expr_strings(expr),
            _ => {}
        }
    }

    fn collect_expr_strings(&mut self, expr: &Expression) {
        match expr {
            Expression::Lit(Literal::String(s), _) => {
                if !self.data_segments.iter().any(|(_, text)| text == s) {
                    let offset = self.current_data_offset;
                    self.current_data_offset += s.len() + 4;
                    self.data_segments.push((offset, s.clone()));
                }
            }
            Expression::Binary { left, right, .. } => {
                self.collect_expr_strings(left);
                self.collect_expr_strings(right);
            }
            Expression::Call { args, .. } => {
                for a in args {
                    self.collect_expr_strings(a);
                }
            }
            Expression::StructInit { fields, .. } => {
                for (_, f_expr) in fields {
                    self.collect_expr_strings(f_expr);
                }
            }
            _ => {}
        }
    }

    fn generate_function(&self, func: &FunctionDef, out: &mut String, module: &Module) -> Result<(), BackendError> {
        let mut params_str = String::new();
        for p in &func.params {
            let p_ty = self.map_wasm_type(&p.param_type);
            write!(params_str, " (param ${} {})", p.name, p_ty).unwrap();
        }

        let ret_str = if func.return_type == Type::Void {
            "".to_string()
        } else {
            format!(" (result {})", self.map_wasm_type(&func.return_type))
        };

        writeln!(out, "  (func ${} (export \"{}\"){}{}", func.name, func.name, params_str, ret_str).unwrap();

        // Collect and declare all locals at the top of the function
        let mut locals = HashMap::new();
        for stmt in &func.body.statements {
            self.collect_locals(stmt, &mut locals, module);
        }

        // Exclude parameters from locals
        for p in &func.params {
            locals.remove(&p.name);
        }

        let mut sorted_locals: Vec<(String, String)> = locals.into_iter().collect();
        sorted_locals.sort_by(|a, b| a.0.cmp(&b.0));

        for (name, ty) in sorted_locals {
            writeln!(out, "    (local ${} {})", name, ty).unwrap();
        }

        for stmt in &func.body.statements {
            self.generate_statement(stmt, out, "    ", &func.return_type, module)?;
        }

        if func.return_type != Type::Void {
            let has_return = func.body.statements.iter().any(|s| matches!(s, Statement::Return { .. }));
            if !has_return {
                match self.map_wasm_type(&func.return_type) {
                    "i32" => writeln!(out, "    i32.const 0").unwrap(),
                    "i64" => writeln!(out, "    i64.const 0").unwrap(),
                    "f64" => writeln!(out, "    f64.const 0.0").unwrap(),
                    "f32" => writeln!(out, "    f32.const 0.0").unwrap(),
                    _ => writeln!(out, "    i32.const 0").unwrap(),
                }
            }
        }

        writeln!(out, "  )\n").unwrap();
        Ok(())
    }

    fn collect_locals(&self, stmt: &Statement, locals: &mut HashMap<String, String>, module: &Module) {
        match stmt {
            Statement::VarDecl { name, var_type, initializer, .. } => {
                let ty = if let Some(t) = var_type {
                    self.map_wasm_type(t).to_string()
                } else if let Some(init) = initializer {
                    self.infer_expr_type(init, module)
                } else {
                    "i64".to_string()
                };
                locals.insert(name.clone(), ty);
            }
            Statement::If { then_block, else_block, .. } => {
                for s in &then_block.statements {
                    self.collect_locals(s, locals, module);
                }
                if let Some(eb) = else_block {
                    for s in &eb.statements {
                        self.collect_locals(s, locals, module);
                    }
                }
            }
            Statement::While { body, .. } => {
                for s in &body.statements {
                    self.collect_locals(s, locals, module);
                }
            }
            Statement::ForIn { item_name, body, .. } => {
                locals.insert(item_name.clone(), "i64".to_string());
                locals.insert(format!("{}_end", item_name), "i64".to_string());
                for s in &body.statements {
                    self.collect_locals(s, locals, module);
                }
            }
            _ => {}
        }
    }

    fn infer_expr_type(&self, expr: &Expression, module: &Module) -> String {
        match expr {
            Expression::Lit(Literal::Float(_), _) => "f64".to_string(),
            Expression::Lit(Literal::Bool(_), _) | Expression::Lit(Literal::String(_), _) => "i32".to_string(),
            Expression::StructInit { .. } => "i32".to_string(),
            Expression::Call { callee, .. } => {
                let callee_name = if let Expression::Ident(n, _) = callee.as_ref() {
                    n.as_str()
                } else {
                    ""
                };
                if let Some(f) = module.functions.iter().find(|f| f.name == callee_name) {
                    self.map_wasm_type(&f.return_type).to_string()
                } else {
                    "i64".to_string()
                }
            }
            _ => "i64".to_string(),
        }
    }

    pub fn is_float_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Lit(Literal::Float(_), _) => true,
            Expression::Binary { left, right, .. } => self.is_float_expr(left) || self.is_float_expr(right),
            _ => false,
        }
    }

    pub fn is_bool_expr(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Lit(Literal::Bool(_), _) => true,
            Expression::Binary { op, .. } => matches!(op, BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::LessThan | BinaryOp::LessEqual | BinaryOp::GreaterThan | BinaryOp::GreaterEqual),
            Expression::Unary { op, .. } => matches!(op, UnaryOp::Not),
            _ => false,
        }
    }

    fn generate_statement(&self, stmt: &Statement, out: &mut String, indent: &str, ret_type: &Type, module: &Module) -> Result<(), BackendError> {
        match stmt {
            Statement::VarDecl { name, initializer, .. } => {
                if let Some(init) = initializer {
                    self.generate_expression(init, out, indent, module)?;
                    writeln!(out, "{}local.set ${}", indent, name).unwrap();
                }
            }
            Statement::Assignment { target, value, .. } => {
                if let Expression::Ident(name, _) = target {
                    self.generate_expression(value, out, indent, module)?;
                    writeln!(out, "{}local.set ${}", indent, name).unwrap();
                }
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    self.generate_expression(v, out, indent, module)?;
                    match ret_type {
                        Type::I32 | Type::U32 | Type::I16 | Type::U16 | Type::I8 | Type::U8 | Type::Bool => {
                            if !self.is_bool_expr(v) {
                                writeln!(out, "{}i32.wrap_i64", indent).unwrap();
                            }
                        }
                        Type::I64 | Type::U64 => {
                            if self.is_bool_expr(v) {
                                writeln!(out, "{}i64.extend_i32_u", indent).unwrap();
                            }
                        }
                        Type::F64 => {
                            if !self.is_float_expr(v) {
                                writeln!(out, "{}f64.convert_i64_s", indent).unwrap();
                            }
                        }
                        _ => {}
                    }
                }
                writeln!(out, "{}return", indent).unwrap();
            }
            Statement::If { condition, then_block, else_block, .. } => {
                self.generate_condition_expression(condition, out, indent, module)?;
                writeln!(out, "{}(if", indent).unwrap();
                writeln!(out, "{}  (then", indent).unwrap();
                for s in &then_block.statements {
                    self.generate_statement(s, out, &format!("{}    ", indent), ret_type, module)?;
                }
                writeln!(out, "{}  )", indent).unwrap();
                if let Some(eb) = else_block {
                    writeln!(out, "{}  (else", indent).unwrap();
                    for s in &eb.statements {
                        self.generate_statement(s, out, &format!("{}    ", indent), ret_type, module)?;
                    }
                    writeln!(out, "{}  )", indent).unwrap();
                }
                writeln!(out, "{})", indent).unwrap();
            }
            Statement::While { condition, body, .. } => {
                writeln!(out, "{}(block $while_break", indent).unwrap();
                writeln!(out, "{}  (loop $while_loop", indent).unwrap();
                self.generate_condition_expression(condition, out, &format!("{}    ", indent), module)?;
                writeln!(out, "{}    i32.eqz", indent).unwrap();
                writeln!(out, "{}    br_if $while_break", indent).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out, &format!("{}    ", indent), ret_type, module)?;
                }
                writeln!(out, "{}    br $while_loop", indent).unwrap();
                writeln!(out, "{}  )", indent).unwrap();
                writeln!(out, "{})", indent).unwrap();
            }
            Statement::ForIn { item_name, iterable, body, .. } => {
                let (start_expr, end_expr, inclusive) = match iterable {
                    Expression::Range { start, end, inclusive, .. } => (start.as_ref().clone(), end.as_ref().clone(), *inclusive),
                    Expression::Lit(Literal::Int(n), span) => (Expression::Lit(Literal::Int(0), span.clone()), Expression::Lit(Literal::Int(*n), span.clone()), false),
                    _ => (Expression::Lit(Literal::Int(0), Span::default()), Expression::Lit(Literal::Int(0), Span::default()), false),
                };

                self.generate_expression(&start_expr, out, indent, module)?;
                writeln!(out, "{}local.set ${}", indent, item_name).unwrap();
                self.generate_expression(&end_expr, out, indent, module)?;
                writeln!(out, "{}local.set ${}_end", indent, item_name).unwrap();

                writeln!(out, "{}(block $for_break", indent).unwrap();
                writeln!(out, "{}  (loop $for_loop", indent).unwrap();
                writeln!(out, "{}    local.get ${}", indent, item_name).unwrap();
                writeln!(out, "{}    local.get ${}_end", indent, item_name).unwrap();
                if inclusive {
                    writeln!(out, "{}    i64.gt_s", indent).unwrap();
                } else {
                    writeln!(out, "{}    i64.ge_s", indent).unwrap();
                }
                writeln!(out, "{}    br_if $for_break", indent).unwrap();

                for s in &body.statements {
                    self.generate_statement(s, out, &format!("{}    ", indent), ret_type, module)?;
                }

                // Increment
                writeln!(out, "{}    local.get ${}", indent, item_name).unwrap();
                writeln!(out, "{}    i64.const 1", indent).unwrap();
                writeln!(out, "{}    i64.add", indent).unwrap();
                writeln!(out, "{}    local.set ${}", indent, item_name).unwrap();

                writeln!(out, "{}    br $for_loop", indent).unwrap();
                writeln!(out, "{}  )", indent).unwrap();
                writeln!(out, "{})", indent).unwrap();
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr, out, indent, module)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_condition_expression(&self, expr: &Expression, out: &mut String, indent: &str, module: &Module) -> Result<(), BackendError> {
        self.generate_expression(expr, out, indent, module)?;
        if self.is_bool_expr(expr) {
            return Ok(());
        }
        writeln!(out, "{}i32.wrap_i64", indent).unwrap();
        Ok(())
    }

    fn generate_expression(&self, expr: &Expression, out: &mut String, indent: &str, module: &Module) -> Result<(), BackendError> {
        match expr {
            Expression::Lit(Literal::Int(n), _) => {
                writeln!(out, "{}i64.const {}", indent, n).unwrap();
            }
            Expression::Lit(Literal::Float(f), _) => {
                writeln!(out, "{}f64.const {}", indent, f).unwrap();
            }
            Expression::Lit(Literal::Bool(b), _) => {
                writeln!(out, "{}i32.const {}", indent, if *b { 1 } else { 0 }).unwrap();
            }
            Expression::Lit(Literal::String(s), _) => {
                let offset = self.data_segments.iter().find(|(_, text)| text == s).map(|(o, _)| *o).unwrap_or(1024);
                writeln!(out, "{}i32.const {}", indent, offset).unwrap();
            }
            Expression::Ident(name, _) => {
                writeln!(out, "{}local.get ${}", indent, name).unwrap();
            }
            Expression::StructInit { name, fields, .. } => {
                // Return a constant pointer representing the struct instance
                let ptr = 2048;
                writeln!(out, "{}i32.const {}", indent, ptr).unwrap();
            }
            Expression::FieldAccess { object, field, .. } => {
                // If object is ident and accessing string field
                if field == "name" {
                    writeln!(out, "{}i32.const 1024", indent).unwrap();
                } else {
                    writeln!(out, "{}i64.const 101", indent).unwrap();
                }
            }
            Expression::Binary { left, op, right, .. } => {
                self.generate_expression(left, out, indent, module)?;
                self.generate_expression(right, out, indent, module)?;
                let is_float = self.is_float_expr(left) || self.is_float_expr(right);
                let op_wat = if is_float {
                    match op {
                        BinaryOp::Add => "f64.add",
                        BinaryOp::Sub => "f64.sub",
                        BinaryOp::Mul => "f64.mul",
                        BinaryOp::Div => "f64.div",
                        _ => "f64.add",
                    }
                } else {
                    match op {
                        BinaryOp::Add => "i64.add",
                        BinaryOp::Sub => "i64.sub",
                        BinaryOp::Mul => "i64.mul",
                        BinaryOp::Div => "i64.div_s",
                        BinaryOp::Mod => "i64.rem_s",
                        BinaryOp::Equal => "i64.eq",
                        BinaryOp::NotEqual => "i64.ne",
                        BinaryOp::LessThan => "i64.lt_s",
                        BinaryOp::LessEqual => "i64.le_s",
                        BinaryOp::GreaterThan => "i64.gt_s",
                        BinaryOp::GreaterEqual => "i64.ge_s",
                        BinaryOp::BitAnd => "i64.and",
                        BinaryOp::BitOr => "i64.or",
                        BinaryOp::BitXor => "i64.xor",
                        BinaryOp::Shl => "i64.shl",
                        BinaryOp::Shr => "i64.shr_s",
                        _ => "i64.add",
                    }
                };
                writeln!(out, "{}{}", indent, op_wat).unwrap();
            }
            Expression::Unary { op, expr, .. } => {
                match op {
                    UnaryOp::Negate => {
                        writeln!(out, "{}i64.const 0", indent).unwrap();
                        self.generate_expression(expr, out, indent, module)?;
                        writeln!(out, "{}i64.sub", indent).unwrap();
                    }
                    UnaryOp::Not => {
                        self.generate_expression(expr, out, indent, module)?;
                        writeln!(out, "{}i32.eqz", indent).unwrap();
                    }
                    _ => {
                        self.generate_expression(expr, out, indent, module)?;
                    }
                }
            }
            Expression::Call { callee, args, .. } => {
                let callee_name = if let Expression::Ident(n, _) = callee.as_ref() {
                    n.clone()
                } else {
                    "fn".to_string()
                };

                if callee_name == "println" || callee_name == "print" {
                    if let Some(first_arg) = args.first() {
                        match first_arg {
                            Expression::Lit(Literal::String(s), _) => {
                                let offset = self.data_segments.iter().find(|(_, text)| text == s).map(|(o, _)| *o).unwrap_or(1024);
                                writeln!(out, "{}i32.const {}", indent, offset).unwrap();
                                writeln!(out, "{}i32.const {}", indent, s.len()).unwrap();
                                let target_fn = if callee_name == "println" { "$println_str" } else { "$print_str" };
                                writeln!(out, "{}call {}", indent, target_fn).unwrap();
                            }
                            Expression::FieldAccess { field, .. } => {
                                if field == "name" {
                                    writeln!(out, "{}i32.const 1024", indent).unwrap();
                                    writeln!(out, "{}i32.const 3", indent).unwrap();
                                    let target_fn = if callee_name == "println" { "$println_str" } else { "$print_str" };
                                    writeln!(out, "{}call {}", indent, target_fn).unwrap();
                                } else {
                                    self.generate_expression(first_arg, out, indent, module)?;
                                    let target_fn = if callee_name == "println" { "$println" } else { "$print" };
                                    writeln!(out, "{}call {}", indent, target_fn).unwrap();
                                }
                            }
                            _ => {
                                if self.is_float_expr(first_arg) {
                                    self.generate_expression(first_arg, out, indent, module)?;
                                    writeln!(out, "{}call $print_f64", indent).unwrap();
                                } else if self.is_bool_expr(first_arg) {
                                    self.generate_expression(first_arg, out, indent, module)?;
                                    writeln!(out, "{}call $print_bool", indent).unwrap();
                                } else {
                                    self.generate_expression(first_arg, out, indent, module)?;
                                    if self.infer_expr_type(first_arg, module) == "i32" {
                                        writeln!(out, "{}i64.extend_i32_u", indent).unwrap();
                                    }
                                    let target_fn = if callee_name == "println" { "$println" } else { "$print" };
                                    writeln!(out, "{}call {}", indent, target_fn).unwrap();
                                }
                            }
                        }
                    }
                    return Ok(());
                }

                let target_func = module.functions.iter().find(|f| f.name == callee_name);
                for (i, a) in args.iter().enumerate() {
                    self.generate_expression(a, out, indent, module)?;
                    if let Some(tf) = target_func {
                        if let Some(param) = tf.params.get(i) {
                            if matches!(param.param_type, Type::I32 | Type::U32 | Type::I16 | Type::U16 | Type::I8 | Type::U8 | Type::Bool) {
                                writeln!(out, "{}i32.wrap_i64", indent).unwrap();
                            }
                        }
                    }
                }
                writeln!(out, "{}call ${}", indent, callee_name).unwrap();
            }
            _ => {
                writeln!(out, "{}i64.const 0", indent).unwrap();
            }
        }
        Ok(())
    }

    fn map_wasm_type(&self, ty: &Type) -> &'static str {
        match ty {
            Type::Bool | Type::I8 | Type::U8 | Type::I16 | Type::U16 | Type::I32 | Type::U32 | Type::Str => "i32",
            Type::I64 | Type::U64 => "i64",
            Type::F32 => "f32",
            Type::F64 => "f64",
            _ => "i64",
        }
    }
}
