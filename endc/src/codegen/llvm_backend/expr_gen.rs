use inkwell::values::{BasicValue, BasicValueEnum, BasicMetadataValueEnum, CallSiteValue, ValueKind};
use inkwell::{IntPredicate, FloatPredicate, AddressSpace};
use crate::ast::*;
use crate::codegen::backend_trait::BackendError;
use super::module_gen::LlvmLoweringContext;

impl<'a, 'ctx> LlvmLoweringContext<'a, 'ctx> {
    pub fn extract_call_val(&self, call: CallSiteValue<'ctx>) -> Option<BasicValueEnum<'ctx>> {
        match call.try_as_basic_value() {
            ValueKind::Basic(val) => Some(val),
            _ => None,
        }
    }

    pub fn lower_expression(&mut self, expr: &Expression) -> Result<BasicValueEnum<'ctx>, BackendError> {
        match expr {
            Expression::Lit(Literal::Int(n), _) => {
                Ok(self.context.i64_type().const_int(*n as u64, true).into())
            }
            Expression::Lit(Literal::Float(f), _) => {
                Ok(self.context.f64_type().const_float(*f).into())
            }
            Expression::Lit(Literal::Bool(b), _) => {
                Ok(self.context.bool_type().const_int(if *b { 1 } else { 0 }, false).into())
            }
            Expression::Lit(Literal::String(s), _) => {
                let glob_str = self.builder.build_global_string_ptr(s, "str_lit").map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed to build global string ptr: {}", e))
                })?;
                Ok(glob_str.as_basic_value_enum())
            }
            Expression::Lit(Literal::Null, _) => {
                Ok(self.context.ptr_type(AddressSpace::default()).const_null().into())
            }
            Expression::Ident(name, _) => {
                if let Some((ptr, ty)) = self.variables.get(name).cloned() {
                    let loaded = self.builder.build_load(ty, ptr, name).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed load variable {}: {}", name, e))
                    })?;
                    Ok(loaded)
                } else if let Some(func_val) = self.function_defs.get(name).cloned() {
                    Ok(func_val.as_global_value().as_pointer_value().into())
                } else {
                    // Fallback literal 0 if undefined variable
                    Ok(self.context.i64_type().const_int(0, false).into())
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let lhs = self.lower_expression(left)?;
                let rhs = self.lower_expression(right)?;

                let is_float = lhs.is_float_value() || rhs.is_float_value();
                let is_str = (lhs.is_pointer_value() || rhs.is_pointer_value()) && *op == BinaryOp::Add;

                if is_str {
                    let concat_fn = self.module.get_function("end_str_concat").ok_or_else(|| {
                        BackendError::Internal("Runtime function end_str_concat not found".to_string())
                    })?;
                    let lhs_ptr = lhs.into_pointer_value();
                    let rhs_ptr = rhs.into_pointer_value();
                    let call = self.builder.build_call(
                        concat_fn,
                        &[lhs_ptr.into(), rhs_ptr.into()],
                        "str_concat",
                    ).map_err(|e| BackendError::CodegenFailed(format!("Str concat call failed: {}", e)))?;
                    return Ok(self.extract_call_val(call).unwrap_or(lhs));
                }

                if is_float {
                    let l_flt = if lhs.is_float_value() {
                        lhs.into_float_value()
                    } else {
                        self.builder.build_signed_int_to_float(
                            lhs.into_int_value(),
                            self.context.f64_type(),
                            "to_flt",
                        ).map_err(|e| BackendError::CodegenFailed(format!("Int to float failed: {}", e)))?
                    };

                    let r_flt = if rhs.is_float_value() {
                        rhs.into_float_value()
                    } else {
                        self.builder.build_signed_int_to_float(
                            rhs.into_int_value(),
                            self.context.f64_type(),
                            "to_flt",
                        ).map_err(|e| BackendError::CodegenFailed(format!("Int to float failed: {}", e)))?
                    };

                    match op {
                        BinaryOp::Add => Ok(self.builder.build_float_add(l_flt, r_flt, "fadd").map_err(|e| BackendError::CodegenFailed(format!("fadd failed: {}", e)))?.into()),
                        BinaryOp::Sub => Ok(self.builder.build_float_sub(l_flt, r_flt, "fsub").map_err(|e| BackendError::CodegenFailed(format!("fsub failed: {}", e)))?.into()),
                        BinaryOp::Mul => Ok(self.builder.build_float_mul(l_flt, r_flt, "fmul").map_err(|e| BackendError::CodegenFailed(format!("fmul failed: {}", e)))?.into()),
                        BinaryOp::Div => Ok(self.builder.build_float_div(l_flt, r_flt, "fdiv").map_err(|e| BackendError::CodegenFailed(format!("fdiv failed: {}", e)))?.into()),
                        BinaryOp::Equal => Ok(self.builder.build_float_compare(FloatPredicate::OEQ, l_flt, r_flt, "feq").map_err(|e| BackendError::CodegenFailed(format!("feq failed: {}", e)))?.into()),
                        BinaryOp::NotEqual => Ok(self.builder.build_float_compare(FloatPredicate::ONE, l_flt, r_flt, "fne").map_err(|e| BackendError::CodegenFailed(format!("fne failed: {}", e)))?.into()),
                        BinaryOp::LessThan => Ok(self.builder.build_float_compare(FloatPredicate::OLT, l_flt, r_flt, "flt").map_err(|e| BackendError::CodegenFailed(format!("flt failed: {}", e)))?.into()),
                        BinaryOp::LessEqual => Ok(self.builder.build_float_compare(FloatPredicate::OLE, l_flt, r_flt, "fle").map_err(|e| BackendError::CodegenFailed(format!("fle failed: {}", e)))?.into()),
                        BinaryOp::GreaterThan => Ok(self.builder.build_float_compare(FloatPredicate::OGT, l_flt, r_flt, "fgt").map_err(|e| BackendError::CodegenFailed(format!("fgt failed: {}", e)))?.into()),
                        BinaryOp::GreaterEqual => Ok(self.builder.build_float_compare(FloatPredicate::OGE, l_flt, r_flt, "fge").map_err(|e| BackendError::CodegenFailed(format!("fge failed: {}", e)))?.into()),
                        _ => Ok(l_flt.into()),
                    }
                } else {
                    let l_int = lhs.into_int_value();
                    let r_int = rhs.into_int_value();

                    // Coerce bit widths if needed
                    let (l_int, r_int) = if l_int.get_type().get_bit_width() != r_int.get_type().get_bit_width() {
                        let max_width = l_int.get_type().get_bit_width().max(r_int.get_type().get_bit_width());
                        let target_ty = if max_width <= 8 {
                            self.context.i8_type()
                        } else if max_width <= 16 {
                            self.context.i16_type()
                        } else if max_width <= 32 {
                            self.context.i32_type()
                        } else {
                            self.context.i64_type()
                        };
                        let l_ext = self.builder.build_int_cast_sign_flag(l_int, target_ty, true, "l_ext").map_err(|e| BackendError::CodegenFailed(format!("Cast failed: {}", e)))?;
                        let r_ext = self.builder.build_int_cast_sign_flag(r_int, target_ty, true, "r_ext").map_err(|e| BackendError::CodegenFailed(format!("Cast failed: {}", e)))?;
                        (l_ext, r_ext)
                    } else {
                        (l_int, r_int)
                    };

                    match op {
                        BinaryOp::Add => Ok(self.builder.build_int_add(l_int, r_int, "add").map_err(|e| BackendError::CodegenFailed(format!("add failed: {}", e)))?.into()),
                        BinaryOp::Sub => Ok(self.builder.build_int_sub(l_int, r_int, "sub").map_err(|e| BackendError::CodegenFailed(format!("sub failed: {}", e)))?.into()),
                        BinaryOp::Mul => Ok(self.builder.build_int_mul(l_int, r_int, "mul").map_err(|e| BackendError::CodegenFailed(format!("mul failed: {}", e)))?.into()),
                        BinaryOp::Div => Ok(self.builder.build_int_signed_div(l_int, r_int, "sdiv").map_err(|e| BackendError::CodegenFailed(format!("sdiv failed: {}", e)))?.into()),
                        BinaryOp::Mod => Ok(self.builder.build_int_signed_rem(l_int, r_int, "srem").map_err(|e| BackendError::CodegenFailed(format!("srem failed: {}", e)))?.into()),
                        BinaryOp::BitAnd | BinaryOp::And => Ok(self.builder.build_and(l_int, r_int, "and").map_err(|e| BackendError::CodegenFailed(format!("and failed: {}", e)))?.into()),
                        BinaryOp::BitOr | BinaryOp::Or => Ok(self.builder.build_or(l_int, r_int, "or").map_err(|e| BackendError::CodegenFailed(format!("or failed: {}", e)))?.into()),
                        BinaryOp::BitXor => Ok(self.builder.build_xor(l_int, r_int, "xor").map_err(|e| BackendError::CodegenFailed(format!("xor failed: {}", e)))?.into()),
                        BinaryOp::Shl => Ok(self.builder.build_left_shift(l_int, r_int, "shl").map_err(|e| BackendError::CodegenFailed(format!("shl failed: {}", e)))?.into()),
                        BinaryOp::Shr => Ok(self.builder.build_right_shift(l_int, r_int, true, "shr").map_err(|e| BackendError::CodegenFailed(format!("shr failed: {}", e)))?.into()),
                        BinaryOp::Equal => Ok(self.builder.build_int_compare(IntPredicate::EQ, l_int, r_int, "eq").map_err(|e| BackendError::CodegenFailed(format!("eq failed: {}", e)))?.into()),
                        BinaryOp::NotEqual => Ok(self.builder.build_int_compare(IntPredicate::NE, l_int, r_int, "ne").map_err(|e| BackendError::CodegenFailed(format!("ne failed: {}", e)))?.into()),
                        BinaryOp::LessThan => Ok(self.builder.build_int_compare(IntPredicate::SLT, l_int, r_int, "slt").map_err(|e| BackendError::CodegenFailed(format!("slt failed: {}", e)))?.into()),
                        BinaryOp::LessEqual => Ok(self.builder.build_int_compare(IntPredicate::SLE, l_int, r_int, "sle").map_err(|e| BackendError::CodegenFailed(format!("sle failed: {}", e)))?.into()),
                        BinaryOp::GreaterThan => Ok(self.builder.build_int_compare(IntPredicate::SGT, l_int, r_int, "sgt").map_err(|e| BackendError::CodegenFailed(format!("sgt failed: {}", e)))?.into()),
                        BinaryOp::GreaterEqual => Ok(self.builder.build_int_compare(IntPredicate::SGE, l_int, r_int, "sge").map_err(|e| BackendError::CodegenFailed(format!("sge failed: {}", e)))?.into()),
                    }
                }
            }
            Expression::Unary { op, expr: inner_expr, .. } => {
                let val = self.lower_expression(inner_expr)?;
                match op {
                    UnaryOp::Negate => {
                        if val.is_float_value() {
                            Ok(self.builder.build_float_neg(val.into_float_value(), "fneg").map_err(|e| BackendError::CodegenFailed(format!("fneg failed: {}", e)))?.into())
                        } else {
                            Ok(self.builder.build_int_neg(val.into_int_value(), "ineg").map_err(|e| BackendError::CodegenFailed(format!("ineg failed: {}", e)))?.into())
                        }
                    }
                    UnaryOp::Not => {
                        let iv = val.into_int_value();
                        let one = iv.get_type().const_int(1, false);
                        Ok(self.builder.build_xor(iv, one, "not").map_err(|e| BackendError::CodegenFailed(format!("not failed: {}", e)))?.into())
                    }
                    UnaryOp::BitNot => {
                        let iv = val.into_int_value();
                        let all_ones = iv.get_type().const_all_ones();
                        Ok(self.builder.build_xor(iv, all_ones, "bitnot").map_err(|e| BackendError::CodegenFailed(format!("bitnot failed: {}", e)))?.into())
                    }
                    _ => Ok(val),
                }
            }
            Expression::Call { callee, args, .. } => {
                let callee_name = if let Expression::Ident(name, _) = callee.as_ref() {
                    name.clone()
                } else {
                    "unknown_callee".to_string()
                };

                // Specialized standard IO: println / print
                if callee_name == "println" || callee_name == "print" {
                    let printf_fn = self.module.get_function("printf").ok_or_else(|| {
                        BackendError::Internal("printf function not declared".to_string())
                    })?;

                    if let Some(first_arg) = args.first() {
                        let arg_val = self.lower_expression(first_arg)?;
                        let is_newline = callee_name == "println";

                        let (fmt_str, val_to_print) = if arg_val.is_pointer_value() {
                            (if is_newline { "%s\n" } else { "%s" }, arg_val)
                        } else if arg_val.is_float_value() {
                            (if is_newline { "%f\n" } else { "%f" }, arg_val)
                        } else if arg_val.is_int_value() && arg_val.into_int_value().get_type().get_bit_width() == 1 {
                            let iv = arg_val.into_int_value();
                            let str_true = self.builder.build_global_string_ptr("true", "str_true").map_err(|e| {
                                BackendError::CodegenFailed(format!("Failed str_true: {}", e))
                            })?.as_basic_value_enum();
                            let str_false = self.builder.build_global_string_ptr("false", "str_false").map_err(|e| {
                                BackendError::CodegenFailed(format!("Failed str_false: {}", e))
                            })?.as_basic_value_enum();
                            let sel = self.builder.build_select(iv, str_true, str_false, "bool_str").map_err(|e| {
                                BackendError::CodegenFailed(format!("Failed select bool: {}", e))
                            })?;
                            (if is_newline { "%s\n" } else { "%s" }, sel)
                        } else if arg_val.is_int_value() {
                            let iv = arg_val.into_int_value();
                            let i64_val = if iv.get_type().get_bit_width() != 64 {
                                self.builder.build_int_cast_sign_flag(iv, self.context.i64_type(), true, "cast_i64").map_err(|e| {
                                    BackendError::CodegenFailed(format!("Failed cast int: {}", e))
                                })?.into()
                            } else {
                                iv.into()
                            };
                            (if is_newline { "%lld\n" } else { "%lld" }, i64_val)
                        } else {
                            (if is_newline { "%lld\n" } else { "%lld" }, self.context.i64_type().const_int(0, false).into())
                        };

                        let fmt_ptr = self.builder.build_global_string_ptr(fmt_str, "printf_fmt").map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed printf fmt: {}", e))
                        })?;

                        let call = self.builder.build_call(
                            printf_fn,
                            &[fmt_ptr.as_basic_value_enum().into(), val_to_print.into()],
                            "printf_call",
                        ).map_err(|e| BackendError::CodegenFailed(format!("Failed printf call: {}", e)))?;

                        return Ok(self.extract_call_val(call).unwrap_or(self.context.i32_type().const_int(0, false).into()));
                    } else if callee_name == "println" {
                        let fmt_ptr = self.builder.build_global_string_ptr("\n", "nl_fmt").map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed nl fmt: {}", e))
                        })?;
                        let call = self.builder.build_call(
                            printf_fn,
                            &[fmt_ptr.as_basic_value_enum().into()],
                            "printf_call",
                        ).map_err(|e| BackendError::CodegenFailed(format!("Failed printf call: {}", e)))?;
                        return Ok(self.extract_call_val(call).unwrap_or(self.context.i32_type().const_int(0, false).into()));
                    }
                }

                // General function call (including recursive function calls)
                let func_val = self.function_defs.get(&callee_name).cloned().or_else(|| {
                    self.module.get_function(&callee_name)
                }).ok_or_else(|| {
                    BackendError::CodegenFailed(format!("Function not found: {}", callee_name))
                })?;

                let param_types = func_val.get_type().get_param_types();
                let mut lowered_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
                for (i, a) in args.iter().enumerate() {
                    let mut v = self.lower_expression(a)?;
                    if let Some(target_meta_ty) = param_types.get(i) {
                        let target_ty = *target_meta_ty;
                        if v.is_int_value() && target_ty.is_int_type() {
                            let arg_iv = v.into_int_value();
                            let tgt_ity = target_ty.into_int_type();
                            if arg_iv.get_type().get_bit_width() != tgt_ity.get_bit_width() {
                                v = self.builder.build_int_cast_sign_flag(arg_iv, tgt_ity, true, "arg_cast").map_err(|e| {
                                    BackendError::CodegenFailed(format!("Arg cast failed: {}", e))
                                })?.into();
                            }
                        } else if v.is_int_value() && target_ty.is_float_type() {
                            let arg_iv = v.into_int_value();
                            let tgt_fty = target_ty.into_float_type();
                            v = self.builder.build_signed_int_to_float(arg_iv, tgt_fty, "arg_flt_cast").map_err(|e| {
                                BackendError::CodegenFailed(format!("Arg float cast failed: {}", e))
                            })?.into();
                        } else if v.is_float_value() && target_ty.is_float_type() {
                            let arg_fv = v.into_float_value();
                            let tgt_fty = target_ty.into_float_type();
                            if arg_fv.get_type() != tgt_fty {
                                v = self.builder.build_float_cast(arg_fv, tgt_fty, "arg_flt_cast").map_err(|e| {
                                    BackendError::CodegenFailed(format!("Arg float cast failed: {}", e))
                                })?.into();
                            }
                        }
                    }
                    lowered_args.push(v.into());
                }

                let call = self.builder.build_call(func_val, &lowered_args, &format!("{}_ret", callee_name)).map_err(|e| {
                    BackendError::CodegenFailed(format!("Function call to {} failed: {}", callee_name, e))
                })?;

                if let Some(basic_val) = self.extract_call_val(call) {
                    Ok(basic_val)
                } else {
                    Ok(self.context.i64_type().const_int(0, false).into())
                }
            }
            Expression::StructInit { name, fields, .. } => {
                let (struct_ty, fields_meta) = self.struct_defs.get(name).cloned().ok_or_else(|| {
                    BackendError::CodegenFailed(format!("Struct definition not found: {}", name))
                })?;

                let alloca = self.builder.build_alloca(struct_ty, name).map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed alloca for struct {}: {}", name, e))
                })?;

                for (f_name, f_expr) in fields {
                    let field_idx = fields_meta.iter().position(|(n, _)| n == f_name).unwrap_or(0);
                    let field_gep = self.builder.build_struct_gep(
                        struct_ty,
                        alloca,
                        field_idx as u32,
                        &format!("{}_{}", name, f_name),
                    ).map_err(|e| BackendError::CodegenFailed(format!("Failed struct init GEP: {}", e)))?;

                    let f_val = self.lower_expression(f_expr)?;
                    self.builder.build_store(field_gep, f_val).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed struct init store: {}", e))
                    })?;
                }

                Ok(alloca.into())
            }
            Expression::FieldAccess { object, field, .. } => {
                let struct_name = if let Expression::Ident(n, _) = object.as_ref() {
                    n.clone()
                } else {
                    "".to_string()
                };

                let (field_idx, field_type) = self.find_field_info(&struct_name, field)?;
                let struct_ty = self.find_struct_type(&struct_name)?;
                let llvm_field_type = self.map_basic_type(&field_type);

                // If object is an ident, check if variable alloca pointer is available
                if let Expression::Ident(var_name, _) = object.as_ref() {
                    if let Some((ptr, _)) = self.variables.get(var_name).cloned() {
                        let field_gep = self.builder.build_struct_gep(
                            struct_ty,
                            ptr,
                            field_idx as u32,
                            &format!("gep_{}", field),
                        ).map_err(|e| BackendError::CodegenFailed(format!("Failed field access GEP: {}", e)))?;

                        let loaded = self.builder.build_load(
                            llvm_field_type,
                            field_gep,
                            &format!("val_{}", field),
                        ).map_err(|e| BackendError::CodegenFailed(format!("Failed field load: {}", e)))?;

                        return Ok(loaded);
                    }
                }

                let obj_val = self.lower_expression(object)?;
                if obj_val.is_pointer_value() {
                    let obj_ptr = obj_val.into_pointer_value();
                    let field_gep = self.builder.build_struct_gep(
                        struct_ty,
                        obj_ptr,
                        field_idx as u32,
                        &format!("gep_{}", field),
                    ).map_err(|e| BackendError::CodegenFailed(format!("Failed field access GEP: {}", e)))?;

                    let loaded = self.builder.build_load(
                        llvm_field_type,
                        field_gep,
                        &format!("val_{}", field),
                    ).map_err(|e| BackendError::CodegenFailed(format!("Failed field load: {}", e)))?;

                    Ok(loaded)
                } else if obj_val.is_struct_value() {
                    let st_val = obj_val.into_struct_value();
                    let extracted = self.builder.build_extract_value(
                        st_val,
                        field_idx as u32,
                        &format!("val_{}", field),
                    ).map_err(|e| BackendError::CodegenFailed(format!("Failed field extract: {}", e)))?;
                    Ok(extracted)
                } else {
                    Ok(obj_val)
                }
            }
            Expression::Index { array, index, .. } => {
                let arr_val = self.lower_expression(array)?;
                let idx_val = self.lower_expression(index)?;
                let arr_ptr = arr_val.into_pointer_value();
                let idx_int = idx_val.into_int_value();

                let gep = unsafe {
                    self.builder.build_gep(
                        self.context.i64_type(),
                        arr_ptr,
                        &[idx_int],
                        "idx_gep",
                    ).map_err(|e| BackendError::CodegenFailed(format!("Failed index GEP: {}", e)))?
                };

                let loaded = self.builder.build_load(
                    self.context.i64_type(),
                    gep,
                    "idx_load",
                ).map_err(|e| BackendError::CodegenFailed(format!("Failed index load: {}", e)))?;

                Ok(loaded)
            }
            Expression::Cast { expr: inner, target_type, .. } => {
                let val = self.lower_expression(inner)?;
                let target_llvm_ty = self.map_basic_type(target_type);

                if val.is_int_value() && target_llvm_ty.is_float_type() {
                    let flt = self.builder.build_signed_int_to_float(
                        val.into_int_value(),
                        target_llvm_ty.into_float_type(),
                        "cast_f",
                    ).map_err(|e| BackendError::CodegenFailed(format!("Cast failed: {}", e)))?;
                    Ok(flt.into())
                } else if val.is_float_value() && target_llvm_ty.is_int_type() {
                    let iv = self.builder.build_float_to_signed_int(
                        val.into_float_value(),
                        target_llvm_ty.into_int_type(),
                        "cast_i",
                    ).map_err(|e| BackendError::CodegenFailed(format!("Cast failed: {}", e)))?;
                    Ok(iv.into())
                } else if val.is_int_value() && target_llvm_ty.is_int_type() {
                    let iv = self.builder.build_int_cast_sign_flag(
                        val.into_int_value(),
                        target_llvm_ty.into_int_type(),
                        true,
                        "cast_int",
                    ).map_err(|e| BackendError::CodegenFailed(format!("Cast failed: {}", e)))?;
                    Ok(iv.into())
                } else {
                    Ok(val)
                }
            }
            Expression::Pipe { lhs, rhs, .. } => {
                let l_val = self.lower_expression(lhs)?;
                if let Expression::Call { callee, args, .. } = rhs.as_ref() {
                    let mut pipe_args = vec![l_val.into()];
                    for a in args {
                        pipe_args.push(self.lower_expression(a)?.into());
                    }

                    let callee_name = if let Expression::Ident(n, _) = callee.as_ref() {
                        n.clone()
                    } else {
                        "pipe_fn".to_string()
                    };

                    let func_val = self.function_defs.get(&callee_name).cloned().or_else(|| {
                        self.module.get_function(&callee_name)
                    }).ok_or_else(|| {
                        BackendError::CodegenFailed(format!("Pipe function {} not found", callee_name))
                    })?;

                    let call = self.builder.build_call(func_val, &pipe_args, "pipe_ret").map_err(|e| {
                        BackendError::CodegenFailed(format!("Pipe call failed: {}", e))
                    })?;

                    Ok(self.extract_call_val(call).unwrap_or(l_val))
                } else if let Expression::Ident(callee_name, _) = rhs.as_ref() {
                    let func_val = self.function_defs.get(callee_name).cloned().or_else(|| {
                        self.module.get_function(callee_name)
                    }).ok_or_else(|| {
                        BackendError::CodegenFailed(format!("Pipe function {} not found", callee_name))
                    })?;

                    let call = self.builder.build_call(func_val, &[l_val.into()], "pipe_ret").map_err(|e| {
                        BackendError::CodegenFailed(format!("Pipe call failed: {}", e))
                    })?;

                    Ok(self.extract_call_val(call).unwrap_or(l_val))
                } else {
                    Ok(l_val)
                }
            }
            _ => Ok(self.context.i64_type().const_int(0, false).into()),
        }
    }
}
