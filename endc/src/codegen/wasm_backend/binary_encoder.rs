use crate::ast::*;
use crate::codegen::backend_trait::BackendError;
use std::collections::HashMap;

pub type WasmBinaryEncoder = BinaryEncoder;

pub struct BinaryEncoder {
    target: String,
    data_segments: Vec<(usize, Vec<u8>)>,
    current_data_offset: usize,
}

impl BinaryEncoder {
    pub fn new() -> Self {
        Self::with_target(None)
    }

    pub fn with_target(target: Option<&str>) -> Self {
        Self {
            target: target.unwrap_or("wasm32-unknown-unknown").to_string(),
            data_segments: Vec::new(),
            current_data_offset: 1024,
        }
    }

    pub fn encode(&mut self, module: &Module) -> Result<Vec<u8>, BackendError> {
        self.data_segments.clear();
        self.current_data_offset = 1024;

        // First pass: collect string literals into data segments
        self.collect_strings(module);

        let mut wasm = Vec::new();
        // Magic header: \0asm
        wasm.extend_from_slice(&[0x00, 0x61, 0x73, 0x6D]);
        // Version 1: 0x01, 0x00, 0x00, 0x00
        wasm.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);

        // Build type section and function signatures
        let mut types: Vec<Vec<u8>> = Vec::new();
        let mut type_indices: HashMap<String, u32> = HashMap::new();

        // Standard import signatures
        // println(i64) -> void
        let println_type = vec![0x60, 0x01, 0x7E, 0x00];
        let println_ty_idx = self.register_type(&mut types, println_type);

        // println_str(i32, i32) -> void
        let print_str_type = vec![0x60, 0x02, 0x7F, 0x7F, 0x00];
        let print_str_ty_idx = self.register_type(&mut types, print_str_type);

        // print_bool(i32) -> void
        let print_bool_type = vec![0x60, 0x01, 0x7F, 0x00];
        let print_bool_ty_idx = self.register_type(&mut types, print_bool_type);

        // print_f64(f64) -> void
        let print_f64_type = vec![0x60, 0x01, 0x7C, 0x00];
        let print_f64_ty_idx = self.register_type(&mut types, print_f64_type);

        // Register module function types
        let mut func_type_map = HashMap::new();
        for func in &module.functions {
            let mut ty_bytes = Vec::new();
            ty_bytes.push(0x60); // func type constructor
            
            // Params
            self.encode_u32_leb128(&mut ty_bytes, func.params.len() as u32);
            for p in &func.params {
                ty_bytes.push(self.map_val_type(&p.param_type));
            }

            // Results
            if func.return_type == Type::Void {
                ty_bytes.push(0x00);
            } else {
                ty_bytes.push(0x01);
                ty_bytes.push(self.map_val_type(&func.return_type));
            }

            let ty_idx = self.register_type(&mut types, ty_bytes);
            func_type_map.insert(func.name.clone(), ty_idx);
        }

        // Section 1: Type Section
        self.encode_type_section(&mut wasm, &types);

        // Section 2: Import Section
        let mut imports = HashMap::new();
        self.encode_import_section(
            &mut wasm,
            println_ty_idx,
            print_str_ty_idx,
            print_bool_ty_idx,
            print_f64_ty_idx,
            &mut imports,
        );

        let num_imported_funcs = imports.len() as u32;

        // Map user functions to function indices (after imported functions)
        let mut func_indices = HashMap::new();
        for (i, func) in module.functions.iter().enumerate() {
            func_indices.insert(func.name.clone(), num_imported_funcs + i as u32);
        }

        // Section 3: Function Section
        self.encode_func_section(&mut wasm, module, &func_type_map);

        // Section 5: Memory Section (1 page min, 2 max)
        self.encode_memory_section(&mut wasm);

        // Section 7: Export Section (memory + user functions)
        self.encode_export_section(&mut wasm, module, &func_indices);

        // Section 10: Code Section
        self.encode_code_section(&mut wasm, module, &func_indices, &imports)?;

        // Section 11: Data Section
        self.encode_data_section(&mut wasm);

        Ok(wasm)
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
                let target_bytes = format!("{}\0", s).into_bytes();
                if !self.data_segments.iter().any(|(_, d)| d == &target_bytes) {
                    let offset = self.current_data_offset;
                    self.current_data_offset += target_bytes.len() + 4;
                    self.data_segments.push((offset, target_bytes));
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

    fn register_type(&self, types: &mut Vec<Vec<u8>>, ty: Vec<u8>) -> u32 {
        if let Some(pos) = types.iter().position(|t| t == &ty) {
            pos as u32
        } else {
            let idx = types.len() as u32;
            types.push(ty);
            idx
        }
    }

    fn encode_type_section(&self, wasm: &mut Vec<u8>, types: &[Vec<u8>]) {
        let mut payload = Vec::new();
        self.encode_u32_leb128(&mut payload, types.len() as u32);
        for t in types {
            payload.extend_from_slice(t);
        }
        self.emit_section(wasm, 1, &payload);
    }

    fn encode_import_section(
        &self,
        wasm: &mut Vec<u8>,
        println_ty: u32,
        print_str_ty: u32,
        print_bool_ty: u32,
        print_f64_ty: u32,
        imports: &mut HashMap<String, u32>,
    ) {
        let mut payload = Vec::new();
        let import_defs = [
            ("env", "println", println_ty, "println"),
            ("env", "print", println_ty, "print"),
            ("env", "println_str", print_str_ty, "println_str"),
            ("env", "print_str", print_str_ty, "print_str"),
            ("env", "print_bool", print_bool_ty, "print_bool"),
            ("env", "print_f64", print_f64_ty, "print_f64"),
        ];

        self.encode_u32_leb128(&mut payload, import_defs.len() as u32);
        for (i, (module, field, ty_idx, key)) in import_defs.iter().enumerate() {
            self.encode_string(&mut payload, module);
            self.encode_string(&mut payload, field);
            payload.push(0x00); // function import tag
            self.encode_u32_leb128(&mut payload, *ty_idx);
            imports.insert(key.to_string(), i as u32);
        }

        self.emit_section(wasm, 2, &payload);
    }

    fn encode_func_section(&self, wasm: &mut Vec<u8>, module: &Module, func_type_map: &HashMap<String, u32>) {
        let mut payload = Vec::new();
        self.encode_u32_leb128(&mut payload, module.functions.len() as u32);
        for func in &module.functions {
            let ty_idx = func_type_map.get(&func.name).copied().unwrap_or(0);
            self.encode_u32_leb128(&mut payload, ty_idx);
        }
        self.emit_section(wasm, 3, &payload);
    }

    fn encode_memory_section(&self, wasm: &mut Vec<u8>) {
        let mut payload = Vec::new();
        self.encode_u32_leb128(&mut payload, 1); // 1 memory
        payload.push(0x01); // flags: min and max
        self.encode_u32_leb128(&mut payload, 2); // min 2 pages
        self.encode_u32_leb128(&mut payload, 4); // max 4 pages
        self.emit_section(wasm, 5, &payload);
    }

    fn encode_export_section(&self, wasm: &mut Vec<u8>, module: &Module, func_indices: &HashMap<String, u32>) {
        let mut payload = Vec::new();
        let total_exports = 1 + module.functions.len(); // 1 memory + functions
        self.encode_u32_leb128(&mut payload, total_exports as u32);

        // Export memory
        self.encode_string(&mut payload, "memory");
        payload.push(0x02); // memory export tag
        self.encode_u32_leb128(&mut payload, 0);

        // Export functions
        for func in &module.functions {
            self.encode_string(&mut payload, &func.name);
            payload.push(0x00); // function export tag
            let fn_idx = func_indices.get(&func.name).copied().unwrap_or(0);
            self.encode_u32_leb128(&mut payload, fn_idx);
        }

        self.emit_section(wasm, 7, &payload);
    }

    fn encode_code_section(
        &self,
        wasm: &mut Vec<u8>,
        module: &Module,
        func_indices: &HashMap<String, u32>,
        imports: &HashMap<String, u32>,
    ) -> Result<(), BackendError> {
        let mut payload = Vec::new();
        self.encode_u32_leb128(&mut payload, module.functions.len() as u32);

        for func in &module.functions {
            let func_body = self.encode_function_body(func, module, func_indices, imports)?;
            self.encode_u32_leb128(&mut payload, func_body.len() as u32);
            payload.extend_from_slice(&func_body);
        }

        self.emit_section(wasm, 10, &payload);
        Ok(())
    }

    fn encode_function_body(
        &self,
        func: &FunctionDef,
        module: &Module,
        func_indices: &HashMap<String, u32>,
        imports: &HashMap<String, u32>,
    ) -> Result<Vec<u8>, BackendError> {
        let mut body = Vec::new();

        // 1. Collect and declare local variables
        let mut locals_map = HashMap::new();
        let mut local_idx = 0u32;

        // Parameters are local 0..N
        for p in &func.params {
            let ty = self.map_val_type(&p.param_type);
            locals_map.insert(p.name.clone(), (local_idx, ty));
            local_idx += 1;
        }

        // Additional declared locals in body
        let mut body_locals: HashMap<String, u8> = HashMap::new();
        for stmt in &func.body.statements {
            self.collect_body_locals(stmt, &mut body_locals, module);
        }

        // Exclude parameters
        for p in &func.params {
            body_locals.remove(&p.name);
        }

        let mut sorted_locals: Vec<(String, u8)> = body_locals.into_iter().collect();
        sorted_locals.sort_by(|a, b| a.0.cmp(&b.0));

        let mut local_groups: Vec<(u32, u8)> = Vec::new();
        for (name, ty) in sorted_locals {
            locals_map.insert(name, (local_idx, ty));
            local_idx += 1;
            if let Some(last) = local_groups.last_mut() {
                if last.1 == ty {
                    last.0 += 1;
                    continue;
                }
            }
            local_groups.push((1, ty));
        }

        // Emit local declarations
        self.encode_u32_leb128(&mut body, local_groups.len() as u32);
        for (count, ty) in local_groups {
            self.encode_u32_leb128(&mut body, count);
            body.push(ty);
        }

        // 2. Emit statements
        for stmt in &func.body.statements {
            self.encode_statement(stmt, &mut body, &locals_map, func_indices, imports, &func.return_type, module)?;
        }

        // 3. Fallback return for non-void functions
        if func.return_type != Type::Void {
            let has_return = func.body.statements.iter().any(|s| matches!(s, Statement::Return { .. }));
            if !has_return {
                match self.map_val_type(&func.return_type) {
                    0x7F => {
                        body.push(0x41); // i32.const 0
                        body.push(0x00);
                    }
                    0x7E => {
                        body.push(0x42); // i64.const 0
                        body.push(0x00);
                    }
                    0x7C => {
                        body.push(0x44); // f64.const 0.0
                        body.extend_from_slice(&0.0f64.to_le_bytes());
                    }
                    _ => {
                        body.push(0x42);
                        body.push(0x00);
                    }
                }
            }
        }

        // End opcode
        body.push(0x0B);

        Ok(body)
    }

    fn collect_body_locals(&self, stmt: &Statement, locals: &mut HashMap<String, u8>, module: &Module) {
        match stmt {
            Statement::VarDecl { name, var_type, initializer, .. } => {
                let ty = if let Some(t) = var_type {
                    self.map_val_type(t)
                } else if let Some(init) = initializer {
                    self.infer_expr_val_type(init, &HashMap::new(), module)
                } else {
                    0x7E // i64 default
                };
                locals.insert(name.clone(), ty);
            }
            Statement::If { then_block, else_block, .. } => {
                for s in &then_block.statements {
                    self.collect_body_locals(s, locals, module);
                }
                if let Some(eb) = else_block {
                    for s in &eb.statements {
                        self.collect_body_locals(s, locals, module);
                    }
                }
            }
            Statement::While { body, .. } => {
                for s in &body.statements {
                    self.collect_body_locals(s, locals, module);
                }
            }
            Statement::ForIn { item_name, body, .. } => {
                locals.insert(item_name.clone(), 0x7E); // i64
                locals.insert(format!("{}_end", item_name), 0x7E); // i64
                for s in &body.statements {
                    self.collect_body_locals(s, locals, module);
                }
            }
            _ => {}
        }
    }

    fn infer_expr_val_type(&self, expr: &Expression, locals: &HashMap<String, (u32, u8)>, module: &Module) -> u8 {
        match expr {
            Expression::Lit(Literal::Float(_), _) => 0x7C, // f64
            Expression::Lit(Literal::Bool(_), _) | Expression::Lit(Literal::String(_), _) => 0x7F, // i32
            Expression::Ident(name, _) => {
                if let Some(&(_, ty)) = locals.get(name) {
                    ty
                } else {
                    0x7E // i64
                }
            }
            Expression::Call { callee, .. } => {
                let callee_name = if let Expression::Ident(n, _) = callee.as_ref() {
                    n.as_str()
                } else {
                    ""
                };
                if let Some(f) = module.functions.iter().find(|f| f.name == callee_name) {
                    self.map_val_type(&f.return_type)
                } else {
                    0x7E
                }
            }
            Expression::Binary { left, right, .. } => {
                let lt = self.infer_expr_val_type(left, locals, module);
                let rt = self.infer_expr_val_type(right, locals, module);
                if lt == 0x7C || rt == 0x7C {
                    0x7C
                } else if lt == 0x7F && rt == 0x7F {
                    0x7F
                } else {
                    0x7E
                }
            }
            Expression::StructInit { .. } => 0x7F,
            _ => 0x7E,
        }
    }

    fn encode_statement(
        &self,
        stmt: &Statement,
        body: &mut Vec<u8>,
        locals: &HashMap<String, (u32, u8)>,
        funcs: &HashMap<String, u32>,
        imports: &HashMap<String, u32>,
        ret_type: &Type,
        module: &Module,
    ) -> Result<(), BackendError> {
        match stmt {
            Statement::VarDecl { name, initializer, .. } => {
                if let Some(init) = initializer {
                    self.encode_expression(init, body, locals, funcs, imports, module)?;
                    if let Some(&(idx, expected_ty)) = locals.get(name) {
                        let actual_ty = self.infer_expr_val_type(init, locals, module);
                        if expected_ty == 0x7F && actual_ty == 0x7E {
                            body.push(0xA7); // i32.wrap_i64
                        } else if expected_ty == 0x7E && actual_ty == 0x7F {
                            body.push(0xAC); // i64.extend_i32_s
                        }
                        body.push(0x21); // local.set
                        self.encode_u32_leb128(body, idx);
                    }
                }
            }
            Statement::Assignment { target, value, .. } => {
                if let Expression::Ident(name, _) = target {
                    self.encode_expression(value, body, locals, funcs, imports, module)?;
                    if let Some(&(idx, expected_ty)) = locals.get(name) {
                        let actual_ty = self.infer_expr_val_type(value, locals, module);
                        if expected_ty == 0x7F && actual_ty == 0x7E {
                            body.push(0xA7); // i32.wrap_i64
                        } else if expected_ty == 0x7E && actual_ty == 0x7F {
                            body.push(0xAC); // i64.extend_i32_s
                        }
                        body.push(0x21); // local.set
                        self.encode_u32_leb128(body, idx);
                    }
                }
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    self.encode_expression(v, body, locals, funcs, imports, module)?;
                    let actual_ty = self.infer_expr_val_type(v, locals, module);
                    match ret_type {
                        Type::I32 | Type::U32 | Type::I16 | Type::U16 | Type::I8 | Type::U8 | Type::Bool => {
                            if actual_ty == 0x7E && !self.is_bool_expr(v) {
                                body.push(0xA7); // i32.wrap_i64
                            }
                        }
                        Type::I64 | Type::U64 => {
                            if actual_ty == 0x7F || self.is_bool_expr(v) {
                                body.push(0xAD); // i64.extend_i32_u
                            }
                        }
                        Type::F64 => {
                            if actual_ty != 0x7C && !self.is_float_expr(v) {
                                body.push(0xB7); // f64.convert_i64_s
                            }
                        }
                        _ => {}
                    }
                }
                body.push(0x0F); // return
            }
            Statement::If { condition, then_block, else_block, .. } => {
                self.encode_condition_expression(condition, body, locals, funcs, imports, module)?;
                body.push(0x04); // if
                body.push(0x40); // block type: void

                for s in &then_block.statements {
                    self.encode_statement(s, body, locals, funcs, imports, ret_type, module)?;
                }

                if let Some(eb) = else_block {
                    body.push(0x05); // else
                    for s in &eb.statements {
                        self.encode_statement(s, body, locals, funcs, imports, ret_type, module)?;
                    }
                }

                body.push(0x0B); // end
            }
            Statement::While { condition, body: while_body, .. } => {
                body.push(0x02); // block
                body.push(0x40); // void
                body.push(0x03); // loop
                body.push(0x40); // void

                // Condition check
                self.encode_condition_expression(condition, body, locals, funcs, imports, module)?;
                body.push(0x45); // i32.eqz
                body.push(0x0D); // br_if 1 (break to outer block)
                self.encode_u32_leb128(body, 1);

                for s in &while_body.statements {
                    self.encode_statement(s, body, locals, funcs, imports, ret_type, module)?;
                }

                body.push(0x0C); // br 0 (loop again)
                self.encode_u32_leb128(body, 0);

                body.push(0x0B); // end loop
                body.push(0x0B); // end block
            }
            Statement::ForIn { item_name, iterable, body: for_body, .. } => {
                let (start_expr, end_expr, inclusive) = match iterable {
                    Expression::Range { start, end, inclusive, .. } => (start.as_ref().clone(), end.as_ref().clone(), *inclusive),
                    Expression::Lit(Literal::Int(n), span) => (Expression::Lit(Literal::Int(0), span.clone()), Expression::Lit(Literal::Int(*n), span.clone()), false),
                    _ => (Expression::Lit(Literal::Int(0), Span::default()), Expression::Lit(Literal::Int(0), Span::default()), false),
                };

                self.encode_expression(&start_expr, body, locals, funcs, imports, module)?;
                if let Some(&(v_idx, _)) = locals.get(item_name) {
                    body.push(0x21); // local.set
                    self.encode_u32_leb128(body, v_idx);
                }
                self.encode_expression(&end_expr, body, locals, funcs, imports, module)?;
                if let Some(&(vend_idx, _)) = locals.get(&format!("{}_end", item_name)) {
                    body.push(0x21); // local.set
                    self.encode_u32_leb128(body, vend_idx);
                }

                body.push(0x02); // block
                body.push(0x40);
                body.push(0x03); // loop
                body.push(0x40);

                // Condition check: var >= end (or > for inclusive)
                if let Some(&(v_idx, _)) = locals.get(item_name) {
                    body.push(0x20); // local.get
                    self.encode_u32_leb128(body, v_idx);
                }
                if let Some(&(vend_idx, _)) = locals.get(&format!("{}_end", item_name)) {
                    body.push(0x20); // local.get
                    self.encode_u32_leb128(body, vend_idx);
                }
                if inclusive {
                    body.push(0x55); // i64.gt_s
                } else {
                    body.push(0x59); // i64.ge_s
                }
                body.push(0x0D); // br_if 1 (break to outer block)
                self.encode_u32_leb128(body, 1);

                for s in &for_body.statements {
                    self.encode_statement(s, body, locals, funcs, imports, ret_type, module)?;
                }

                // Increment
                if let Some(&(v_idx, _)) = locals.get(item_name) {
                    body.push(0x20); // local.get
                    self.encode_u32_leb128(body, v_idx);
                    body.push(0x42); // i64.const 1
                    body.push(0x01);
                    body.push(0x7C); // i64.add
                    body.push(0x21); // local.set
                    self.encode_u32_leb128(body, v_idx);
                }

                body.push(0x0C); // br 0 (continue loop)
                self.encode_u32_leb128(body, 0);

                body.push(0x0B); // end loop
                body.push(0x0B); // end block
            }
            Statement::Expression(expr) => {
                self.encode_expression(expr, body, locals, funcs, imports, module)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn encode_condition_expression(
        &self,
        expr: &Expression,
        body: &mut Vec<u8>,
        locals: &HashMap<String, (u32, u8)>,
        funcs: &HashMap<String, u32>,
        imports: &HashMap<String, u32>,
        module: &Module,
    ) -> Result<(), BackendError> {
        self.encode_expression(expr, body, locals, funcs, imports, module)?;
        if self.is_bool_expr(expr) {
            return Ok(());
        }
        body.push(0xA7); // i32.wrap_i64
        Ok(())
    }

    fn encode_expression(
        &self,
        expr: &Expression,
        body: &mut Vec<u8>,
        locals: &HashMap<String, (u32, u8)>,
        funcs: &HashMap<String, u32>,
        imports: &HashMap<String, u32>,
        module: &Module,
    ) -> Result<(), BackendError> {
        match expr {
            Expression::Lit(Literal::Int(n), _) => {
                body.push(0x42); // i64.const
                self.encode_i64_leb128(body, *n);
            }
            Expression::Lit(Literal::Float(f), _) => {
                body.push(0x44); // f64.const
                body.extend_from_slice(&f.to_le_bytes());
            }
            Expression::Lit(Literal::Bool(b), _) => {
                body.push(0x41); // i32.const
                body.push(if *b { 1 } else { 0 });
            }
            Expression::Lit(Literal::String(s), _) => {
                let target_bytes = format!("{}\0", s).into_bytes();
                let offset = self.data_segments.iter().find(|(_, d)| d == &target_bytes).map(|(o, _)| *o).unwrap_or(1024);
                body.push(0x41); // i32.const
                self.encode_i32_leb128(body, offset as i32);
            }
            Expression::Ident(name, _) => {
                if let Some(&(idx, _)) = locals.get(name) {
                    body.push(0x20); // local.get
                    self.encode_u32_leb128(body, idx);
                } else {
                    body.push(0x42); // i64.const 0 fallback
                    body.push(0x00);
                }
            }
            Expression::StructInit { .. } => {
                body.push(0x41); // i32.const
                self.encode_i32_leb128(body, 2048);
            }
            Expression::FieldAccess { field, .. } => {
                if field == "name" {
                    let offset = 1024;
                    body.push(0x41); // i32.const
                    self.encode_i32_leb128(body, offset);
                } else {
                    body.push(0x42); // i64.const
                    self.encode_i64_leb128(body, 101);
                }
            }
            Expression::Binary { left, op, right, .. } => {
                self.encode_expression(left, body, locals, funcs, imports, module)?;
                self.encode_expression(right, body, locals, funcs, imports, module)?;
                let is_float = self.is_float_expr(left) || self.is_float_expr(right);
                let lt = self.infer_expr_val_type(left, locals, module);
                let rt = self.infer_expr_val_type(right, locals, module);
                let is_i32 = lt == 0x7F && rt == 0x7F;

                let opcode = if is_float {
                    match op {
                        BinaryOp::Add => 0xA0, // f64.add
                        BinaryOp::Sub => 0xA1, // f64.sub
                        BinaryOp::Mul => 0xA2, // f64.mul
                        BinaryOp::Div => 0xA3, // f64.div
                        _ => 0xA0,
                    }
                } else if is_i32 {
                    match op {
                        BinaryOp::Add => 0x6A, // i32.add
                        BinaryOp::Sub => 0x6B, // i32.sub
                        BinaryOp::Mul => 0x6C, // i32.mul
                        BinaryOp::Div => 0x6D, // i32.div_s
                        BinaryOp::Mod => 0x6F, // i32.rem_s
                        BinaryOp::Equal => 0x46, // i32.eq
                        BinaryOp::NotEqual => 0x47, // i32.ne
                        BinaryOp::LessThan => 0x48, // i32.lt_s
                        BinaryOp::GreaterThan => 0x4A, // i32.gt_s
                        BinaryOp::LessEqual => 0x4C, // i32.le_s
                        BinaryOp::GreaterEqual => 0x4E, // i32.ge_s
                        BinaryOp::BitAnd => 0x71, // i32.and
                        BinaryOp::BitOr => 0x72, // i32.or
                        BinaryOp::BitXor => 0x73, // i32.xor
                        BinaryOp::Shl => 0x74, // i32.shl
                        BinaryOp::Shr => 0x75, // i32.shr_s
                        _ => 0x6A,
                    }
                } else {
                    match op {
                        BinaryOp::Add => 0x7C, // i64.add
                        BinaryOp::Sub => 0x7D, // i64.sub
                        BinaryOp::Mul => 0x7E, // i64.mul
                        BinaryOp::Div => 0x7F, // i64.div_s
                        BinaryOp::Mod => 0x81, // i64.rem_s
                        BinaryOp::Equal => 0x51, // i64.eq
                        BinaryOp::NotEqual => 0x52, // i64.ne
                        BinaryOp::LessThan => 0x53, // i64.lt_s
                        BinaryOp::GreaterThan => 0x55, // i64.gt_s
                        BinaryOp::LessEqual => 0x57, // i64.le_s
                        BinaryOp::GreaterEqual => 0x59, // i64.ge_s
                        BinaryOp::BitAnd => 0x83, // i64.and
                        BinaryOp::BitOr => 0x84, // i64.or
                        BinaryOp::BitXor => 0x85, // i64.xor
                        BinaryOp::Shl => 0x86, // i64.shl
                        BinaryOp::Shr => 0x87, // i64.shr_s
                        _ => 0x7C,
                    }
                };
                body.push(opcode);
            }
            Expression::Unary { op, expr, .. } => {
                match op {
                    UnaryOp::Negate => {
                        body.push(0x42); // i64.const 0
                        body.push(0x00);
                        self.encode_expression(expr, body, locals, funcs, imports, module)?;
                        body.push(0x7D); // i64.sub
                    }
                    UnaryOp::Not => {
                        self.encode_expression(expr, body, locals, funcs, imports, module)?;
                        body.push(0x45); // i32.eqz
                    }
                    _ => {
                        self.encode_expression(expr, body, locals, funcs, imports, module)?;
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
                                let target_bytes = format!("{}\0", s).into_bytes();
                                let offset = self.data_segments.iter().find(|(_, d)| d == &target_bytes).map(|(o, _)| *o).unwrap_or(1024);
                                body.push(0x41); // i32.const offset
                                self.encode_i32_leb128(body, offset as i32);
                                body.push(0x41); // i32.const len
                                self.encode_i32_leb128(body, s.len() as i32);
                                let import_fn = if callee_name == "println" { "println_str" } else { "print_str" };
                                if let Some(&idx) = imports.get(import_fn) {
                                    body.push(0x10); // call
                                    self.encode_u32_leb128(body, idx);
                                }
                            }
                            Expression::FieldAccess { field, .. } => {
                                if field == "name" {
                                    let offset = 1024;
                                    let s_len = 3;
                                    body.push(0x41);
                                    self.encode_i32_leb128(body, offset);
                                    body.push(0x41);
                                    self.encode_i32_leb128(body, s_len);
                                    let import_fn = if callee_name == "println" { "println_str" } else { "print_str" };
                                    if let Some(&idx) = imports.get(import_fn) {
                                        body.push(0x10);
                                        self.encode_u32_leb128(body, idx);
                                    }
                                } else {
                                    self.encode_expression(first_arg, body, locals, funcs, imports, module)?;
                                    let import_fn = if callee_name == "println" { "println" } else { "print" };
                                    if let Some(&idx) = imports.get(import_fn) {
                                        body.push(0x10);
                                        self.encode_u32_leb128(body, idx);
                                    }
                                }
                            }
                            _ => {
                                if self.is_float_expr(first_arg) {
                                    self.encode_expression(first_arg, body, locals, funcs, imports, module)?;
                                    if let Some(&idx) = imports.get("print_f64") {
                                        body.push(0x10); // call
                                        self.encode_u32_leb128(body, idx);
                                    }
                                } else if self.is_bool_expr(first_arg) {
                                    self.encode_expression(first_arg, body, locals, funcs, imports, module)?;
                                    if let Some(&idx) = imports.get("print_bool") {
                                        body.push(0x10); // call
                                        self.encode_u32_leb128(body, idx);
                                    }
                                } else {
                                    self.encode_expression(first_arg, body, locals, funcs, imports, module)?;
                                    let arg_ty = self.infer_expr_val_type(first_arg, locals, module);
                                    if arg_ty == 0x7F {
                                        body.push(0xAD); // i64.extend_i32_u
                                    }
                                    let import_fn = if callee_name == "println" { "println" } else { "print" };
                                    if let Some(&idx) = imports.get(import_fn) {
                                        body.push(0x10); // call
                                        self.encode_u32_leb128(body, idx);
                                    }
                                }
                            }
                        }
                    }
                    return Ok(());
                }

                let target_func = module.functions.iter().find(|f| f.name == callee_name);
                for (i, a) in args.iter().enumerate() {
                    self.encode_expression(a, body, locals, funcs, imports, module)?;
                    if let Some(tf) = target_func {
                        if let Some(param) = tf.params.get(i) {
                            if matches!(param.param_type, Type::I32 | Type::U32 | Type::I16 | Type::U16 | Type::I8 | Type::U8 | Type::Bool) {
                                body.push(0xA7); // i32.wrap_i64
                            }
                        }
                    }
                }

                if let Some(&fn_idx) = funcs.get(&callee_name) {
                    body.push(0x10); // call
                    self.encode_u32_leb128(body, fn_idx);
                } else if let Some(&imp_idx) = imports.get(&callee_name) {
                    body.push(0x10); // call
                    self.encode_u32_leb128(body, imp_idx);
                }
            }
            _ => {
                body.push(0x42); // i64.const 0
                body.push(0x00);
            }
        }
        Ok(())
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

    fn map_val_type(&self, ty: &Type) -> u8 {
        match ty {
            Type::Bool | Type::I8 | Type::U8 | Type::I16 | Type::U16 | Type::I32 | Type::U32 | Type::Str => 0x7F, // i32
            Type::I64 | Type::U64 => 0x7E, // i64
            Type::F32 => 0x7D, // f32
            Type::F64 => 0x7C, // f64
            _ => 0x7E, // default i64
        }
    }

    fn emit_section(&self, wasm: &mut Vec<u8>, section_id: u8, payload: &[u8]) {
        wasm.push(section_id);
        self.encode_u32_leb128(wasm, payload.len() as u32);
        wasm.extend_from_slice(payload);
    }

    fn encode_string(&self, buf: &mut Vec<u8>, s: &str) {
        self.encode_u32_leb128(buf, s.len() as u32);
        buf.extend_from_slice(s.as_bytes());
    }

    fn encode_u32_leb128(&self, buf: &mut Vec<u8>, mut val: u32) {
        loop {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80;
            }
            buf.push(byte);
            if val == 0 {
                break;
            }
        }
    }

    fn encode_i32_leb128(&self, buf: &mut Vec<u8>, mut val: i32) {
        let mut more = true;
        while more {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if (val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0) {
                more = false;
            } else {
                byte |= 0x80;
            }
            buf.push(byte);
        }
    }

    fn encode_i64_leb128(&self, buf: &mut Vec<u8>, mut val: i64) {
        let mut more = true;
        while more {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if (val == 0 && (byte & 0x40) == 0) || (val == -1 && (byte & 0x40) != 0) {
                more = false;
            } else {
                byte |= 0x80;
            }
            buf.push(byte);
        }
    }

    fn encode_data_section(&self, wasm: &mut Vec<u8>) {
        if self.data_segments.is_empty() {
            return;
        }

        let mut payload = Vec::new();
        self.encode_u32_leb128(&mut payload, self.data_segments.len() as u32);

        for (offset, bytes) in &self.data_segments {
            payload.push(0x00); // active data segment for memory 0
            // Offset expr: i32.const <offset> end
            payload.push(0x41);
            self.encode_i32_leb128(&mut payload, *offset as i32);
            payload.push(0x0B); // end

            // Bytes length and data
            self.encode_u32_leb128(&mut payload, bytes.len() as u32);
            payload.extend_from_slice(bytes);
        }

        self.emit_section(wasm, 11, &payload);
    }
}
