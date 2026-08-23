use super::state::LlvmBackend;
use crate::ast::*;
use crate::codegen::backend_trait::BackendError;
use std::fmt::Write;

impl LlvmBackend {
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

    pub(crate) fn generate_function(&mut self, func: &FunctionDef, out: &mut String) -> Result<(), BackendError> {
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
}
