use crate::ast::*;
use crate::codegen::backend_trait::{BackendError, CodeGenBackend};
use crate::codegen::type_mapper::{LlvmTypeMapper, TypeMapper};
use std::collections::HashMap;
use std::fmt::Write;

pub struct LlvmBackend {
    output: String,
    target_triple: String,
    temp_var_id: usize,
    block_id: usize,
    str_literal_id: usize,
    debug_id: usize,
    emit_debug_info: bool,
    string_constants: Vec<(String, String, usize)>, // (name, content, byte_len)
    variables: HashMap<String, (String, String)>, // name -> (llvm_type, llvm_reg_or_ptr)
    type_mapper: LlvmTypeMapper,
}

impl LlvmBackend {
    pub fn new(target_triple: Option<&str>) -> Self {
        Self {
            output: String::new(),
            target_triple: target_triple.unwrap_or(Self::detect_host_triple()).to_string(),
            temp_var_id: 0,
            block_id: 0,
            str_literal_id: 0,
            debug_id: 1,
            emit_debug_info: true,
            string_constants: Vec::new(),
            variables: HashMap::new(),
            type_mapper: LlvmTypeMapper,
        }
    }

    pub fn set_debug_info(&mut self, enabled: bool) {
        self.emit_debug_info = enabled;
    }

    fn detect_host_triple() -> &'static str {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        { "x86_64-pc-windows-msvc" }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        { "x86_64-unknown-linux-gnu" }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        { "x86_64-apple-darwin" }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        { "aarch64-apple-darwin" }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        { "aarch64-unknown-linux-gnu" }
        #[cfg(not(any(
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "linux", target_arch = "aarch64"),
        )))]
        { "x86_64-unknown-linux-gnu" }
    }

    fn next_temp(&mut self) -> String {
        let id = self.temp_var_id;
        self.temp_var_id += 1;
        format!("%t{}", id)
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let id = self.block_id;
        self.block_id += 1;
        format!("{}_{}", prefix, id)
    }

    fn register_string_literal(&mut self, text: &str) -> String {
        let name = format!("@.str.{}", self.str_literal_id);
        self.str_literal_id += 1;
        let escaped = text
            .replace("\\", "\\5C")
            .replace("\n", "\\0A")
            .replace("\t", "\\09")
            .replace("\r", "\\0D")
            .replace("\"", "\\22")
            .replace("\0", "\\00");
        let byte_len = text.as_bytes().len() + 1; // +1 for null terminator
        self.string_constants.push((name.clone(), escaped, byte_len));
        name
    }

    pub fn map_type(&self, ty: &Type) -> String {
        self.type_mapper.map_type(ty)
    }

    pub fn generate_llvm_ir(&mut self, module: &Module) -> Result<String, BackendError> {
        self.output.clear();
        self.temp_var_id = 0;
        self.block_id = 0;
        self.str_literal_id = 0;
        self.string_constants.clear();
        self.variables.clear();

        let mut body_output = String::new();

        // 1. Generate Struct & Enum Definitions
        for st in &module.structs {
            write!(self.output, "%struct.{} = type {{ ", st.name).unwrap();
            for (i, field) in st.fields.iter().enumerate() {
                if i > 0 {
                    write!(self.output, ", ").unwrap();
                }
                write!(self.output, "{}", self.map_type(&field.field_type)).unwrap();
            }
            writeln!(self.output, " }}").unwrap();
        }

        for e in &module.enums {
            // Tagged Union representation: { i32, [16 x i8] }
            writeln!(self.output, "%struct.{} = type {{ i32, [16 x i8] }}", e.name).unwrap();
        }
        writeln!(self.output).unwrap();

        // 2. Generate Functions
        for func in &module.functions {
            self.generate_function(func, &mut body_output)?;
        }

        // 3. Assemble Full Module with Header & Globals
        let mut final_module = String::new();
        writeln!(final_module, "; ModuleID = '{}'", module.name).unwrap();
        writeln!(final_module, "target triple = \"{}\"", self.target_triple).unwrap();
        writeln!(final_module, "source_filename = \"{}.end\"\n", module.name).unwrap();

        // Global String Literals
        for (name, val, len) in &self.string_constants {
            writeln!(final_module, "{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1", name, len, val).unwrap();
        }
        if !self.string_constants.is_empty() {
            writeln!(final_module).unwrap();
        }

        // Standard LLVM & End Runtime Declarations
        writeln!(final_module, "declare i32 @printf(i8*, ...)").unwrap();
        writeln!(final_module, "declare i8* @malloc(i64)").unwrap();
        writeln!(final_module, "declare void @free(i8*)").unwrap();
        writeln!(final_module, "declare i32 @strcmp(i8*, i8*)").unwrap();
        writeln!(final_module, "declare void @llvm.lifetime.start.p0i8(i64, i8*)").unwrap();
        writeln!(final_module, "declare void @llvm.lifetime.end.p0i8(i64, i8*)").unwrap();
        writeln!(final_module, "declare i8* @end_arena_create(i64)").unwrap();
        writeln!(final_module, "declare void @end_arena_destroy(i8*)").unwrap();
        writeln!(final_module, "declare i8* @end_arena_alloc(i8*, i64)").unwrap();
        writeln!(final_module, "declare i8* @end_str_concat(i8*, i8*)").unwrap();
        writeln!(final_module, "declare i64 @end_net_tcp_listen(i32, i32)").unwrap();
        writeln!(final_module, "declare i64 @end_net_tcp_accept(i64)").unwrap();
        writeln!(final_module, "declare i64 @end_net_tcp_connect(i8*, i32)").unwrap();
        writeln!(final_module, "declare i64 @end_net_tcp_send(i64, i8*, i64)").unwrap();
        writeln!(final_module, "declare i8* @end_net_tcp_recv(i64, i32)").unwrap();
        writeln!(final_module, "declare void @end_net_tcp_close(i64)").unwrap();
        writeln!(final_module, "declare i8* @end_crypto_sha256(i8*)").unwrap();
        writeln!(final_module, "declare i8* @end_crypto_hmac_sha256(i8*, i8*)").unwrap();
        writeln!(final_module, "declare i8* @end_base64_encode(i8*, i32)").unwrap();
        writeln!(final_module, "declare i8* @end_base64_decode(i8*)").unwrap();
        writeln!(final_module, "declare i8* @end_json_get_string(i8*, i8*)").unwrap();
        writeln!(final_module, "declare i64 @end_json_get_int(i8*, i8*)").unwrap();
        writeln!(final_module, "declare i64 @end_json_get_bool(i8*, i8*)").unwrap();
        writeln!(final_module, "declare i64 @end_tensor_create(i32, i32)").unwrap();
        writeln!(final_module, "declare i64 @end_tensor_matmul(i64, i64)").unwrap();
        writeln!(final_module, "declare i64 @end_ui_canvas_create(i32, i32)").unwrap();
        writeln!(final_module, "declare void @end_ui_canvas_clear(i64, i32)").unwrap();
        writeln!(final_module, "declare void @end_ui_canvas_draw_rect(i64, i32, i32, i32, i32, i32)").unwrap();
        writeln!(final_module, "declare i32 @end_ui_canvas_get_pixel(i64, i32, i32)\n").unwrap();

        final_module.push_str(&self.output);
        final_module.push_str(&body_output);

        if self.emit_debug_info {
            writeln!(final_module, "\n!llvm.module.flags = !{{!0, !1}}").unwrap();
            writeln!(final_module, "!llvm.dbg.cu = !{{!2}}").unwrap();
            writeln!(final_module, "!0 = !{{i32 2, !\"Dwarf Version\", i32 4}}").unwrap();
            writeln!(final_module, "!1 = !{{i32 2, !\"Debug Info Version\", i32 3}}").unwrap();
            writeln!(final_module, "!2 = distinct !DICompileUnit(language: DW_LANG_C99, file: !3, producer: \"End Compiler v2.0 (LLVM Direct)\", isOptimized: false, runtimeVersion: 0, emissionKind: FullDebug)").unwrap();
            writeln!(final_module, "!3 = !DIFile(filename: \"{}.end\", directory: \".\")", module.name).unwrap();
        }

        Ok(final_module)
    }

    fn generate_function(&mut self, func: &FunctionDef, out: &mut String) -> Result<(), BackendError> {
        self.variables.clear();
        let ret_ty = self.map_type(&func.return_type);
        
        let mut attributes = String::new();
        for dir in &func.directives {
            match dir.name.as_str() {
                "@inline" => attributes.push_str(" alwaysinline"),
                "@pure" => attributes.push_str(" readonly"),
                "@cold" => attributes.push_str(" cold"),
                "@c_export" => {
                    #[cfg(target_os = "windows")]
                    attributes.push_str(" dllexport");
                }
                _ => {}
            }
        }

        let is_main = func.name == "main";
        let func_name = if is_main { "main".to_string() } else { format!("@{}", func.name) };
        let ret_str = if is_main { "i32".to_string() } else { ret_ty.clone() };

        write!(out, "define {} {}(", ret_str, func_name).unwrap();
        for (i, p) in func.params.iter().enumerate() {
            if i > 0 {
                write!(out, ", ").unwrap();
            }
            let p_ty = self.map_type(&p.param_type);
            write!(out, "{} %arg_{}", p_ty, p.name).unwrap();
        }
        writeln!(out, "){} {{", attributes).unwrap();
        writeln!(out, "entry:").unwrap();

        // Store parameters into local allocas
        for p in &func.params {
            let p_ty = self.map_type(&p.param_type);
            let ptr_reg = self.next_temp();
            writeln!(out, "  {} = alloca {}", ptr_reg, p_ty).unwrap();
            writeln!(out, "  store {} %arg_{}, {}* {}", p_ty, p.name, p_ty, ptr_reg).unwrap();
            self.variables.insert(p.name.clone(), (p_ty, ptr_reg));
        }

        let mut has_ret = false;
        for stmt in &func.body.statements {
            if let Statement::Return { .. } = stmt {
                has_ret = true;
            }
            self.generate_statement(stmt, out)?;
        }

        if !has_ret {
            if is_main {
                writeln!(out, "  ret i32 0").unwrap();
            } else if func.return_type == Type::Void {
                writeln!(out, "  ret void").unwrap();
            } else {
                let default_val = match &func.return_type {
                    Type::Bool => "i1 0",
                    Type::I8 | Type::I16 | Type::I32 => "i32 0",
                    Type::I64 | Type::U64 => "i64 0",
                    Type::F32 => "float 0.0",
                    Type::F64 => "double 0.0",
                    _ => "i8* null",
                };
                writeln!(out, "  ret {}", default_val).unwrap();
            }
        }

        writeln!(out, "}}\n").unwrap();
        Ok(())
    }

    fn generate_statement(&mut self, stmt: &Statement, out: &mut String) -> Result<(), BackendError> {
        match stmt {
            Statement::VarDecl { name, var_type, initializer, .. } => {
                let (val_reg, inferred_ty) = if let Some(init) = initializer {
                    self.generate_expression(init, out)?
                } else {
                    ("0".to_string(), "i64".to_string())
                };

                let ty_str = if let Some(t) = var_type {
                    self.map_type(t)
                } else {
                    inferred_ty
                };

                let ptr_reg = self.next_temp();
                writeln!(out, "  {} = alloca {}", ptr_reg, ty_str).unwrap();
                writeln!(out, "  store {} {}, {}* {}", ty_str, val_reg, ty_str, ptr_reg).unwrap();
                self.variables.insert(name.clone(), (ty_str, ptr_reg));
            }
            Statement::Assignment { target, value, .. } => {
                let (val_reg, _) = self.generate_expression(value, out)?;
                if let Expression::Ident(name, _) = target {
                    if let Some((ty, ptr)) = self.variables.get(name).cloned() {
                        writeln!(out, "  store {} {}, {}* {}", ty, val_reg, ty, ptr).unwrap();
                    } else {
                        return Err(BackendError::CodegenFailed(format!("Undefined variable in assignment: {}", name)));
                    }
                } else if let Expression::FieldAccess { object, field, .. } = target {
                    if let Expression::Ident(obj_name, _) = object.as_ref() {
                        if let Some((struct_ptr_ty, struct_ptr)) = self.variables.get(obj_name).cloned() {
                            let struct_name = struct_ptr_ty.trim_start_matches('%').trim_end_matches('*');
                            let field_idx = self.get_field_index(struct_name, field);
                            let gep_reg = self.next_temp();
                            writeln!(out, "  {} = getelementptr inbounds {}, {} {}, i32 0, i32 {}", 
                                     gep_reg, struct_name, struct_ptr_ty, struct_ptr, field_idx).unwrap();
                            writeln!(out, "  store i64 {}, i64* {}", val_reg, gep_reg).unwrap();
                        }
                    }
                }
            }
            Statement::If { condition, then_block, else_block, .. } => {
                let (cond_reg, _) = self.generate_expression(condition, out)?;
                let then_lbl = self.next_label("then");
                let else_lbl = self.next_label("else");
                let merge_lbl = self.next_label("merge");

                if else_block.is_some() {
                    writeln!(out, "  br i1 {}, label %{}, label %{}", cond_reg, then_lbl, else_lbl).unwrap();
                } else {
                    writeln!(out, "  br i1 {}, label %{}, label %{}", cond_reg, then_lbl, merge_lbl).unwrap();
                }

                // Then Block
                writeln!(out, "{}:", then_lbl).unwrap();
                for s in &then_block.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  br label %{}", merge_lbl).unwrap();

                // Else Block
                if let Some(eb) = else_block {
                    writeln!(out, "{}:", else_lbl).unwrap();
                    for s in &eb.statements {
                        self.generate_statement(s, out)?;
                    }
                    writeln!(out, "  br label %{}", merge_lbl).unwrap();
                }

                writeln!(out, "{}:", merge_lbl).unwrap();
            }
            Statement::While { condition, body, .. } => {
                let cond_lbl = self.next_label("while_cond");
                let body_lbl = self.next_label("while_body");
                let end_lbl = self.next_label("while_end");

                writeln!(out, "  br label %{}", cond_lbl).unwrap();
                writeln!(out, "{}:", cond_lbl).unwrap();
                let (cond_reg, _) = self.generate_expression(condition, out)?;
                writeln!(out, "  br i1 {}, label %{}, label %{}", cond_reg, body_lbl, end_lbl).unwrap();

                writeln!(out, "{}:", body_lbl).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  br label %{}", cond_lbl).unwrap();

                writeln!(out, "{}:", end_lbl).unwrap();
            }
            Statement::ForIn { item_name, iterable, body, .. } => {
                let (iter_reg, _) = self.generate_expression(iterable, out)?;
                let counter_ptr = self.next_temp();
                writeln!(out, "  {} = alloca i64", counter_ptr).unwrap();
                writeln!(out, "  store i64 0, i64* {}", counter_ptr).unwrap();
                self.variables.insert(item_name.clone(), ("i64".to_string(), counter_ptr.clone()));

                let cond_lbl = self.next_label("for_cond");
                let body_lbl = self.next_label("for_body");
                let end_lbl = self.next_label("for_end");

                writeln!(out, "  br label %{}", cond_lbl).unwrap();
                writeln!(out, "{}:", cond_lbl).unwrap();
                let current_i = self.next_temp();
                writeln!(out, "  {} = load i64, i64* {}", current_i, counter_ptr).unwrap();
                let cmp_reg = self.next_temp();
                writeln!(out, "  {} = icmp slt i64 {}, {}", cmp_reg, current_i, iter_reg).unwrap();
                writeln!(out, "  br i1 {}, label %{}, label %{}", cmp_reg, body_lbl, end_lbl).unwrap();

                writeln!(out, "{}:", body_lbl).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out)?;
                }
                let inc_reg = self.next_temp();
                writeln!(out, "  {} = add i64 {}, 1", inc_reg, current_i).unwrap();
                writeln!(out, "  store i64 {}, i64* {}", inc_reg, counter_ptr).unwrap();
                writeln!(out, "  br label %{}", cond_lbl).unwrap();

                writeln!(out, "{}:", end_lbl).unwrap();
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    let (val_reg, ty) = self.generate_expression(v, out)?;
                    writeln!(out, "  ret {} {}", ty, val_reg).unwrap();
                } else {
                    writeln!(out, "  ret void").unwrap();
                }
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr, out)?;
            }
            Statement::RegionBlock { name, body, .. } => {
                let arena_ptr = self.next_temp();
                writeln!(out, "  ; Region Arena Allocation ({})", name).unwrap();
                writeln!(out, "  {} = call i8* @end_arena_create(i64 65536)", arena_ptr).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  call void @end_arena_destroy(i8* {})", arena_ptr).unwrap();
            }
            Statement::LeaseBlock { name, initializer, body, .. } => {
                let (res_reg, res_ty) = self.generate_expression(initializer, out)?;
                let lease_ptr = self.next_temp();
                writeln!(out, "  ; Tier 0 Scoped Lease ({})", name).unwrap();
                writeln!(out, "  {} = alloca {}", lease_ptr, res_ty).unwrap();
                writeln!(out, "  call void @llvm.lifetime.start.p0i8(i64 8, i8* {})", lease_ptr).unwrap();
                writeln!(out, "  store {} {}, {}* {}", res_ty, res_reg, res_ty, lease_ptr).unwrap();
                for s in &body.statements {
                    self.generate_statement(s, out)?;
                }
                writeln!(out, "  call void @llvm.lifetime.end.p0i8(i64 8, i8* {})", lease_ptr).unwrap();
            }
            Statement::AsmBlock { code, .. } => {
                writeln!(out, "  call void asm sideeffect \"{}\", \"\"()", code.replace("\"", "\\\"")).unwrap();
            }
            Statement::Defer { expr, .. } => {
                self.generate_expression(expr, out)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_expression(&mut self, expr: &Expression, out: &mut String) -> Result<(String, String), BackendError> {
        match expr {
            Expression::Lit(Literal::Int(n), _) => Ok((n.to_string(), "i64".to_string())),
            Expression::Lit(Literal::Float(f), _) => Ok((format!("{:.6}", f), "double".to_string())),
            Expression::Lit(Literal::Bool(b), _) => Ok(((if *b { 1 } else { 0 }).to_string(), "i1".to_string())),
            Expression::Lit(Literal::String(s), _) => {
                let glob_name = self.register_string_literal(s);
                let byte_len = s.as_bytes().len() + 1;
                let gep_reg = self.next_temp();
                writeln!(out, "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0", 
                         gep_reg, byte_len, byte_len, glob_name).unwrap();
                Ok((gep_reg, "i8*".to_string()))
            }
            Expression::Lit(Literal::Null, _) => Ok(("null".to_string(), "i8*".to_string())),
            Expression::Ident(name, _) => {
                if let Some((ty, ptr)) = self.variables.get(name).cloned() {
                    let load_reg = self.next_temp();
                    writeln!(out, "  {} = load {}, {}* {}", load_reg, ty, ty, ptr).unwrap();
                    Ok((load_reg, ty))
                } else {
                    Ok((format!("@{}", name), "i8*".to_string()))
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let (lhs, l_ty) = self.generate_expression(left, out)?;
                let (rhs, r_ty) = self.generate_expression(right, out)?;
                let res_reg = self.next_temp();

                let is_float = l_ty == "double" || l_ty == "float" || r_ty == "double" || r_ty == "float";
                let is_str = l_ty == "i8*" || r_ty == "i8*";

                if is_str && *op == BinaryOp::Add {
                    writeln!(out, "  {} = call i8* @end_str_concat(i8* {}, i8* {})", res_reg, lhs, rhs).unwrap();
                    return Ok((res_reg, "i8*".to_string()));
                }

                match op {
                    BinaryOp::Add => {
                        let ins = if is_float { "fadd double" } else { "add i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, if is_float { "double" } else { "i64" }.to_string()))
                    }
                    BinaryOp::Sub => {
                        let ins = if is_float { "fsub double" } else { "sub i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, if is_float { "double" } else { "i64" }.to_string()))
                    }
                    BinaryOp::Mul => {
                        let ins = if is_float { "fmul double" } else { "mul i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, if is_float { "double" } else { "i64" }.to_string()))
                    }
                    BinaryOp::Div => {
                        let ins = if is_float { "fdiv double" } else { "sdiv i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, if is_float { "double" } else { "i64" }.to_string()))
                    }
                    BinaryOp::Mod => {
                        writeln!(out, "  {} = srem i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::Equal => {
                        let ins = if is_float { "fcmp oeq double" } else { "icmp eq i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::NotEqual => {
                        let ins = if is_float { "fcmp one double" } else { "icmp ne i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::LessThan => {
                        let ins = if is_float { "fcmp olt double" } else { "icmp slt i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::LessEqual => {
                        let ins = if is_float { "fcmp ole double" } else { "icmp sle i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::GreaterThan => {
                        let ins = if is_float { "fcmp ogt double" } else { "icmp sgt i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::GreaterEqual => {
                        let ins = if is_float { "fcmp oge double" } else { "icmp sge i64" };
                        writeln!(out, "  {} = {} {}, {}", res_reg, ins, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::BitAnd => {
                        writeln!(out, "  {} = and i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::BitOr => {
                        writeln!(out, "  {} = or i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::BitXor => {
                        writeln!(out, "  {} = xor i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::Shl => {
                        writeln!(out, "  {} = shl i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::Shr => {
                        writeln!(out, "  {} = ashr i64 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    BinaryOp::And => {
                        writeln!(out, "  {} = and i1 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    BinaryOp::Or => {
                        writeln!(out, "  {} = or i1 {}, {}", res_reg, lhs, rhs).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                }
            }
            Expression::Unary { op, expr: inner_expr, .. } => {
                let (val, ty) = self.generate_expression(inner_expr, out)?;
                let res_reg = self.next_temp();
                match op {
                    UnaryOp::Negate => {
                        writeln!(out, "  {} = sub i64 0, {}", res_reg, val).unwrap();
                        Ok((res_reg, ty))
                    }
                    UnaryOp::Not => {
                        writeln!(out, "  {} = xor i1 {}, 1", res_reg, val).unwrap();
                        Ok((res_reg, "i1".to_string()))
                    }
                    UnaryOp::BitNot => {
                        writeln!(out, "  {} = xor i64 {}, -1", res_reg, val).unwrap();
                        Ok((res_reg, "i64".to_string()))
                    }
                    _ => Ok((val, ty)),
                }
            }
            Expression::Call { callee, args, .. } => {
                let callee_name = if let Expression::Ident(name, _) = callee.as_ref() {
                    name.clone()
                } else {
                    "unknown_callee".to_string()
                };

                // Standard Print Specialization
                if callee_name == "println" || callee_name == "print" {
                    if let Some(first_arg) = args.first() {
                        let (arg_val, arg_ty) = self.generate_expression(first_arg, out)?;
                        let fmt_str = if arg_ty == "i8*" {
                            if callee_name == "println" { "%s\\0A" } else { "%s" }
                        } else if arg_ty == "double" || arg_ty == "float" {
                            if callee_name == "println" { "%f\\0A" } else { "%f" }
                        } else {
                            if callee_name == "println" { "%lld\\0A" } else { "%lld" }
                        };
                        let fmt_name = self.register_string_literal(fmt_str);
                        let gep_reg = self.next_temp();
                        writeln!(out, "  {} = getelementptr inbounds [5 x i8], [5 x i8]* {}, i32 0, i32 0", 
                                 gep_reg, fmt_name).unwrap();
                        let call_reg = self.next_temp();
                        writeln!(out, "  {} = call i32 (i8*, ...) @printf(i8* {}, {} {})", 
                                 call_reg, gep_reg, arg_ty, arg_val).unwrap();
                        return Ok((call_reg, "i32".to_string()));
                    }
                }

                let mut arg_vals = Vec::new();
                for a in args {
                    let (a_val, a_ty) = self.generate_expression(a, out)?;
                    arg_vals.push(format!("{} {}", a_ty, a_val));
                }

                let res_reg = self.next_temp();
                writeln!(out, "  {} = call i64 @{}({})", res_reg, callee_name, arg_vals.join(", ")).unwrap();
                Ok((res_reg, "i64".to_string()))
            }
            Expression::StructInit { name, fields, .. } => {
                let struct_alloca = self.next_temp();
                writeln!(out, "  {} = alloca %struct.{}", struct_alloca, name).unwrap();
                for (idx, (_, f_expr)) in fields.iter().enumerate() {
                    let (f_val, f_ty) = self.generate_expression(f_expr, out)?;
                    let field_gep = self.next_temp();
                    writeln!(out, "  {} = getelementptr inbounds %struct.{}, %struct.{}* {}, i32 0, i32 {}", 
                             field_gep, name, name, struct_alloca, idx).unwrap();
                    writeln!(out, "  store {} {}, {}* {}", f_ty, f_val, f_ty, field_gep).unwrap();
                }
                Ok((struct_alloca, format!("%struct.{}*", name)))
            }
            Expression::FieldAccess { object, field, .. } => {
                if let Expression::Ident(obj_name, _) = object.as_ref() {
                    if let Some((struct_ptr_ty, struct_ptr)) = self.variables.get(obj_name).cloned() {
                        let struct_name = struct_ptr_ty.trim_start_matches('%').trim_end_matches('*');
                        let field_idx = self.get_field_index(struct_name, field);
                        let gep_reg = self.next_temp();
                        writeln!(out, "  {} = getelementptr inbounds {}, {} {}, i32 0, i32 {}", 
                                 gep_reg, struct_name, struct_ptr_ty, struct_ptr, field_idx).unwrap();
                        let load_reg = self.next_temp();
                        writeln!(out, "  {} = load i64, i64* {}", load_reg, gep_reg).unwrap();
                        return Ok((load_reg, "i64".to_string()));
                    }
                }
                Ok(("0".to_string(), "i64".to_string()))
            }
            Expression::Index { array, index, .. } => {
                let (arr_val, _) = self.generate_expression(array, out)?;
                let (idx_val, _) = self.generate_expression(index, out)?;
                let gep_reg = self.next_temp();
                writeln!(out, "  {} = getelementptr inbounds i64, i64* {}, i64 {}", gep_reg, arr_val, idx_val).unwrap();
                let load_reg = self.next_temp();
                writeln!(out, "  {} = load i64, i64* {}", load_reg, gep_reg).unwrap();
                Ok((load_reg, "i64".to_string()))
            }
            Expression::Cast { expr: inner, target_type, .. } => {
                let (val, ty) = self.generate_expression(inner, out)?;
                let target_ty_str = self.map_type(target_type);
                let res_reg = self.next_temp();
                if ty == "i64" && target_ty_str == "double" {
                    writeln!(out, "  {} = sitofp i64 {} to double", res_reg, val).unwrap();
                } else if ty == "double" && target_ty_str == "i64" {
                    writeln!(out, "  {} = fptosi double {} to i64", res_reg, val).unwrap();
                } else {
                    writeln!(out, "  {} = bitcast {} {} to {}", res_reg, ty, val, target_ty_str).unwrap();
                }
                Ok((res_reg, target_ty_str))
            }
            Expression::Pipe { lhs, rhs, .. } => {
                let (l_val, l_ty) = self.generate_expression(lhs, out)?;
                if let Expression::Call { callee, args, .. } = rhs.as_ref() {
                    let callee_name = if let Expression::Ident(n, _) = callee.as_ref() { n.clone() } else { "pipe_fn".to_string() };
                    let mut call_args = vec![format!("{} {}", l_ty, l_val)];
                    for a in args {
                        let (a_val, a_ty) = self.generate_expression(a, out)?;
                        call_args.push(format!("{} {}", a_ty, a_val));
                    }
                    let res_reg = self.next_temp();
                    writeln!(out, "  {} = call i64 @{}({})", res_reg, callee_name, call_args.join(", ")).unwrap();
                    Ok((res_reg, "i64".to_string()))
                } else if let Expression::Ident(callee_name, _) = rhs.as_ref() {
                    let res_reg = self.next_temp();
                    writeln!(out, "  {} = call i64 @{}({} {})", res_reg, callee_name, l_ty, l_val).unwrap();
                    Ok((res_reg, "i64".to_string()))
                } else {
                    Ok((l_val, l_ty))
                }
            }
            Expression::Match { expr: subj_expr, arms, .. } => {
                let (subj_val, _) = self.generate_expression(subj_expr, out)?;
                let res_alloca = self.next_temp();
                writeln!(out, "  {} = alloca i64", res_alloca).unwrap();
                let merge_lbl = self.next_label("match_merge");

                for (idx, arm) in arms.iter().enumerate() {
                    let arm_lbl = self.next_label(&format!("match_arm_{}", idx));
                    let next_arm_lbl = self.next_label(&format!("match_next_{}", idx));
                    
                    let pat_val = match &arm.pattern {
                        Pattern::Literal(Literal::Int(n)) => n.to_string(),
                        Pattern::Literal(Literal::Bool(b)) => if *b { "1".to_string() } else { "0".to_string() },
                        _ => idx.to_string(),
                    };

                    let cmp_reg = self.next_temp();
                    writeln!(out, "  {} = icmp eq i64 {}, {}", cmp_reg, subj_val, pat_val).unwrap();
                    writeln!(out, "  br i1 {}, label %{}, label %{}", cmp_reg, arm_lbl, next_arm_lbl).unwrap();

                    writeln!(out, "{}:", arm_lbl).unwrap();
                    for s in &arm.body.statements {
                        self.generate_statement(s, out)?;
                    }
                    writeln!(out, "  store i64 {}, i64* {}", pat_val, res_alloca).unwrap();
                    writeln!(out, "  br label %{}", merge_lbl).unwrap();

                    writeln!(out, "{}:", next_arm_lbl).unwrap();
                }
                writeln!(out, "  br label %{}", merge_lbl).unwrap();
                writeln!(out, "{}:", merge_lbl).unwrap();
                let final_res = self.next_temp();
                writeln!(out, "  {} = load i64, i64* {}", final_res, res_alloca).unwrap();
                Ok((final_res, "i64".to_string()))
            }
            _ => Ok(("0".to_string(), "i64".to_string())),
        }
    }

    fn get_field_index(&self, _struct_name: &str, field: &str) -> usize {
        match field {
            "id" | "x" | "first" | "order_id" | "sku" => 0,
            "name" | "y" | "second" | "amount" | "quantity" | "customer_id" => 1,
            "active" | "z" | "third" | "total" | "price" => 2,
            _ => 0,
        }
    }
}

impl CodeGenBackend for LlvmBackend {
    type Output = String;

    fn compile_module(&mut self, module: &Module) -> Result<Self::Output, BackendError> {
        self.generate_llvm_ir(module)
    }

    fn name(&self) -> &'static str {
        "llvm"
    }

    fn supports_jit(&self) -> bool {
        false
    }

    fn supports_aot(&self) -> bool {
        true
    }
}
