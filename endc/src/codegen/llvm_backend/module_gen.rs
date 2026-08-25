use std::collections::HashMap;
use inkwell::context::Context;
use inkwell::module::Module as InkwellModule;
use inkwell::builder::Builder;
use inkwell::types::{BasicType, BasicTypeEnum, StructType, BasicMetadataTypeEnum};
use inkwell::values::{BasicValueEnum, PointerValue, FunctionValue};
use inkwell::AddressSpace;
use crate::ast::*;
use crate::codegen::backend_trait::BackendError;

pub struct LlvmLoweringContext<'a, 'ctx> {
    pub context: &'ctx Context,
    pub module: &'a InkwellModule<'ctx>,
    pub builder: &'a Builder<'ctx>,
    pub variables: HashMap<String, (PointerValue<'ctx>, BasicTypeEnum<'ctx>)>,
    pub struct_defs: HashMap<String, (StructType<'ctx>, Vec<(String, Type)>)>,
    pub function_defs: HashMap<String, FunctionValue<'ctx>>,
    pub string_literals: HashMap<String, PointerValue<'ctx>>,
    pub emit_debug_info: bool,
    pub current_func_return_type: Type,
}

impl<'a, 'ctx> LlvmLoweringContext<'a, 'ctx> {
    pub fn new(
        context: &'ctx Context,
        module: &'a InkwellModule<'ctx>,
        builder: &'a Builder<'ctx>,
        emit_debug_info: bool,
    ) -> Self {
        Self {
            context,
            module,
            builder,
            variables: HashMap::new(),
            struct_defs: HashMap::new(),
            function_defs: HashMap::new(),
            string_literals: HashMap::new(),
            emit_debug_info,
            current_func_return_type: Type::Void,
        }
    }

    pub fn map_basic_type(&self, ty: &Type) -> BasicTypeEnum<'ctx> {
        match ty {
            Type::Bool => self.context.bool_type().into(),
            Type::I8 | Type::U8 => self.context.i8_type().into(),
            Type::I16 | Type::U16 => self.context.i16_type().into(),
            Type::I32 | Type::U32 => self.context.i32_type().into(),
            Type::I64 | Type::U64 => self.context.i64_type().into(),
            Type::F32 => self.context.f32_type().into(),
            Type::F64 => self.context.f64_type().into(),
            Type::Str | Type::Pointer(_) | Type::Slice(_) | Type::Box(_) | Type::Rc(_) | Type::Arc(_) => {
                self.context.ptr_type(AddressSpace::default()).into()
            }
            Type::Custom(name) => {
                if let Some((st, _)) = self.struct_defs.get(name) {
                    (*st).into()
                } else {
                    self.context.ptr_type(AddressSpace::default()).into()
                }
            }
            _ => self.context.i64_type().into(),
        }
    }

    pub fn declare_builtins(&mut self) {
        let ptr_ty = self.context.ptr_type(AddressSpace::default());
        let i32_ty = self.context.i32_type();
        let i64_ty = self.context.i64_type();
        let void_ty = self.context.void_type();

        // declare i32 @printf(ptr, ...)
        let printf_type = i32_ty.fn_type(&[ptr_ty.into()], true);
        self.module.add_function("printf", printf_type, None);

        // declare ptr @malloc(i64)
        let malloc_type = ptr_ty.fn_type(&[i64_ty.into()], false);
        self.module.add_function("malloc", malloc_type, None);

        // declare void @free(ptr)
        let free_type = void_ty.fn_type(&[ptr_ty.into()], false);
        self.module.add_function("free", free_type, None);

        // declare i32 @strcmp(ptr, ptr)
        let strcmp_type = i32_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        self.module.add_function("strcmp", strcmp_type, None);

        // declare ptr @end_str_concat(ptr, ptr)
        let str_concat_type = ptr_ty.fn_type(&[ptr_ty.into(), ptr_ty.into()], false);
        self.module.add_function("end_str_concat", str_concat_type, None);

        // Runtime helpers
        let arena_create_type = ptr_ty.fn_type(&[i64_ty.into()], false);
        self.module.add_function("end_arena_create", arena_create_type, None);

        let arena_destroy_type = void_ty.fn_type(&[ptr_ty.into()], false);
        self.module.add_function("end_arena_destroy", arena_destroy_type, None);
    }

    pub fn lower_module(&mut self, ast_module: &crate::ast::Module) -> Result<(), BackendError> {
        self.declare_builtins();

        // 1. Declare and define all struct types
        for st in &ast_module.structs {
            let struct_type = self.context.opaque_struct_type(&st.name);
            let fields_meta: Vec<(String, Type)> = st.fields.iter().map(|f| (f.name.clone(), f.field_type.clone())).collect();
            self.struct_defs.insert(st.name.clone(), (struct_type, fields_meta));
        }

        for st in &ast_module.structs {
            let mut field_llvm_types: Vec<BasicTypeEnum<'ctx>> = Vec::new();
            for field in &st.fields {
                field_llvm_types.push(self.map_basic_type(&field.field_type));
            }
            if let Some((st_type, _)) = self.struct_defs.get(&st.name) {
                st_type.set_body(&field_llvm_types, false);
            }
        }

        // 2. Declare all functions
        for func in &ast_module.functions {
            let is_main = func.name == "main";
            let ret_type = &func.return_type;

            let mut param_types: Vec<BasicMetadataTypeEnum<'ctx>> = Vec::new();
            for p in &func.params {
                param_types.push(self.map_basic_type(&p.param_type).into());
            }

            let fn_val = if is_main {
                let fn_type = self.context.i32_type().fn_type(&[], false);
                self.module.add_function("main", fn_type, None)
            } else if *ret_type == Type::Void {
                let fn_type = self.context.void_type().fn_type(&param_types, false);
                self.module.add_function(&func.name, fn_type, None)
            } else {
                let basic_ret = self.map_basic_type(ret_type);
                let fn_type = basic_ret.fn_type(&param_types, false);
                self.module.add_function(&func.name, fn_type, None)
            };

            self.function_defs.insert(func.name.clone(), fn_val);
        }

        // 3. Lower function bodies
        for func in &ast_module.functions {
            self.lower_function(func)?;
        }

        // 4. Verify the entire generated LLVM module
        if let Err(err) = self.module.verify() {
            return Err(BackendError::CodegenFailed(format!(
                "LLVM Module verification failed: {}",
                err.to_string()
            )));
        }

        Ok(())
    }

    pub fn lower_function(&mut self, func: &FunctionDef) -> Result<(), BackendError> {
        let func_val = *self.function_defs.get(&func.name).ok_or_else(|| {
            BackendError::Internal(format!("Function {} not found in declarations", func.name))
        })?;

        let entry_block = self.context.append_basic_block(func_val, "entry");
        self.builder.position_at_end(entry_block);
        self.variables.clear();
        self.current_func_return_type = func.return_type.clone();

        // Allocate parameters and store initial arguments
        for (i, p) in func.params.iter().enumerate() {
            let p_ty = self.map_basic_type(&p.param_type);
            let alloca = self.builder.build_alloca(p_ty, &p.name).map_err(|e| {
                BackendError::CodegenFailed(format!("Failed to build alloca for param {}: {}", p.name, e))
            })?;
            let param_val = func_val.get_nth_param(i as u32).ok_or_else(|| {
                BackendError::CodegenFailed(format!("Missing parameter {} for function {}", i, func.name))
            })?;
            self.builder.build_store(alloca, param_val).map_err(|e| {
                BackendError::CodegenFailed(format!("Failed to store param {}: {}", p.name, e))
            })?;
            self.variables.insert(p.name.clone(), (alloca, p_ty));
        }

        // Lower body statements
        for stmt in &func.body.statements {
            self.lower_statement(stmt, func_val)?;
        }

        // Ensure current basic block has a valid terminator
        let current_bb = self.builder.get_insert_block();
        if let Some(bb) = current_bb {
            if bb.get_terminator().is_none() {
                if func.name == "main" {
                    let zero = self.context.i32_type().const_int(0, false);
                    self.builder.build_return(Some(&zero)).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed to build return for main: {}", e))
                    })?;
                } else if func.return_type == Type::Void {
                    self.builder.build_return(None).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed to build void return: {}", e))
                    })?;
                } else {
                    let default_val: BasicValueEnum<'ctx> = match &func.return_type {
                        Type::Bool => self.context.bool_type().const_int(0, false).into(),
                        Type::I8 | Type::U8 => self.context.i8_type().const_int(0, false).into(),
                        Type::I16 | Type::U16 => self.context.i16_type().const_int(0, false).into(),
                        Type::I32 | Type::U32 => self.context.i32_type().const_int(0, false).into(),
                        Type::I64 | Type::U64 => self.context.i64_type().const_int(0, false).into(),
                        Type::F32 => self.context.f32_type().const_float(0.0).into(),
                        Type::F64 => self.context.f64_type().const_float(0.0).into(),
                        _ => self.context.ptr_type(AddressSpace::default()).const_null().into(),
                    };
                    self.builder.build_return(Some(&default_val)).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed to build fallback return: {}", e))
                    })?;
                }
            }
        }

        Ok(())
    }
}
