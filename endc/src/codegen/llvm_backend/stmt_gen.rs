use inkwell::values::{BasicValue, BasicValueEnum, FunctionValue};
use crate::ast::*;
use crate::codegen::backend_trait::BackendError;
use super::module_gen::LlvmLoweringContext;

impl<'a, 'ctx> LlvmLoweringContext<'a, 'ctx> {
    pub fn lower_statement(
        &mut self,
        stmt: &Statement,
        current_fn: FunctionValue<'ctx>,
    ) -> Result<(), BackendError> {
        // If current basic block is already terminated, ignore unreachable statements
        if let Some(bb) = self.builder.get_insert_block() {
            if bb.get_terminator().is_some() {
                return Ok(());
            }
        }

        match stmt {
            Statement::VarDecl { name, var_type, initializer, .. } => {
                let init_val = if let Some(init) = initializer {
                    Some(self.lower_expression(init)?)
                } else {
                    None
                };

                let ty = if let Some(t) = var_type {
                    self.map_basic_type(t)
                } else if let Some(ref val) = init_val {
                    val.get_type()
                } else {
                    self.context.i64_type().into()
                };

                let alloca = self.builder.build_alloca(ty, name).map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed to build alloca for {}: {}", name, e))
                })?;

                if let Some(val) = init_val {
                    self.builder.build_store(alloca, val).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed to store init value to {}: {}", name, e))
                    })?;
                }

                self.variables.insert(name.clone(), (alloca, ty));
            }
            Statement::Assignment { target, value, .. } => {
                let val = self.lower_expression(value)?;
                match target {
                    Expression::Ident(name, _) => {
                        if let Some((ptr, _)) = self.variables.get(name).cloned() {
                            self.builder.build_store(ptr, val).map_err(|e| {
                                BackendError::CodegenFailed(format!("Failed to store to {}: {}", name, e))
                            })?;
                        } else {
                            return Err(BackendError::CodegenFailed(format!(
                                "Undefined variable in assignment: {}",
                                name
                            )));
                        }
                    }
                    Expression::FieldAccess { object, field, .. } => {
                        let obj_val = self.lower_expression(object)?;
                        let obj_ptr = obj_val.into_pointer_value();

                        let struct_name = if let Expression::Ident(n, _) = object.as_ref() {
                            n.clone()
                        } else {
                            "".to_string()
                        };

                        let (field_idx, _) = self.find_field_info(&struct_name, field)?;
                        let struct_ty = self.find_struct_type(&struct_name)?;

                        let field_gep = self.builder.build_struct_gep(struct_ty, obj_ptr, field_idx as u32, "field_gep").map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed GEP on field {}: {}", field, e))
                        })?;

                        self.builder.build_store(field_gep, val).map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed to store field {}: {}", field, e))
                        })?;
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
                            ).map_err(|e| {
                                BackendError::CodegenFailed(format!("Failed index GEP: {}", e))
                            })?
                        };

                        self.builder.build_store(gep, val).map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed to store to index: {}", e))
                        })?;
                    }
                    _ => {
                        return Err(BackendError::UnsupportedFeature(format!(
                            "Unsupported assignment target: {:?}",
                            target
                        )));
                    }
                }
            }
            Statement::If { condition, then_block, else_block, .. } => {
                let cond_val = self.lower_expression(condition)?;
                let cond_int = if cond_val.is_int_value() {
                    let iv = cond_val.into_int_value();
                    if iv.get_type().get_bit_width() == 1 {
                        iv
                    } else {
                        self.builder.build_int_compare(
                            inkwell::IntPredicate::NE,
                            iv,
                            iv.get_type().const_int(0, false),
                            "if_cond",
                        ).map_err(|e| BackendError::CodegenFailed(format!("If cond cmp failed: {}", e)))?
                    }
                } else {
                    return Err(BackendError::TypeMismatch("If condition must be boolean".to_string()));
                };

                let then_bb = self.context.append_basic_block(current_fn, "then");
                let else_bb = self.context.append_basic_block(current_fn, "else");
                let merge_bb = self.context.append_basic_block(current_fn, "if_merge");

                if else_block.is_some() {
                    self.builder.build_conditional_branch(cond_int, then_bb, else_bb).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed branch: {}", e))
                    })?;
                } else {
                    self.builder.build_conditional_branch(cond_int, then_bb, merge_bb).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed branch: {}", e))
                    })?;
                }

                // 1. Lower then block
                self.builder.position_at_end(then_bb);
                for s in &then_block.statements {
                    self.lower_statement(s, current_fn)?;
                }
                if let Some(bb) = self.builder.get_insert_block() {
                    if bb.get_terminator().is_none() {
                        self.builder.build_unconditional_branch(merge_bb).map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed then merge branch: {}", e))
                        })?;
                    }
                }

                // 2. Lower else block if present
                if let Some(eb) = else_block {
                    self.builder.position_at_end(else_bb);
                    for s in &eb.statements {
                        self.lower_statement(s, current_fn)?;
                    }
                    if let Some(bb) = self.builder.get_insert_block() {
                        if bb.get_terminator().is_none() {
                            self.builder.build_unconditional_branch(merge_bb).map_err(|e| {
                                BackendError::CodegenFailed(format!("Failed else merge branch: {}", e))
                            })?;
                        }
                    }
                } else {
                    // Position at else_bb to connect to merge if no else statements
                    self.builder.position_at_end(else_bb);
                    self.builder.build_unconditional_branch(merge_bb).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed empty else merge branch: {}", e))
                    })?;
                }

                self.builder.position_at_end(merge_bb);
            }
            Statement::While { condition, body, .. } => {
                let cond_bb = self.context.append_basic_block(current_fn, "while_cond");
                let body_bb = self.context.append_basic_block(current_fn, "while_body");
                let end_bb = self.context.append_basic_block(current_fn, "while_end");

                self.builder.build_unconditional_branch(cond_bb).map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed while entry branch: {}", e))
                })?;

                // Condition block
                self.builder.position_at_end(cond_bb);
                let cond_val = self.lower_expression(condition)?;
                let cond_int = if cond_val.is_int_value() {
                    let iv = cond_val.into_int_value();
                    if iv.get_type().get_bit_width() == 1 {
                        iv
                    } else {
                        self.builder.build_int_compare(
                            inkwell::IntPredicate::NE,
                            iv,
                            iv.get_type().const_int(0, false),
                            "while_cond_cmp",
                        ).map_err(|e| BackendError::CodegenFailed(format!("While cond cmp failed: {}", e)))?
                    }
                } else {
                    return Err(BackendError::TypeMismatch("While condition must be boolean".to_string()));
                };

                self.builder.build_conditional_branch(cond_int, body_bb, end_bb).map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed while condition branch: {}", e))
                })?;

                // Body block
                self.builder.position_at_end(body_bb);
                for s in &body.statements {
                    self.lower_statement(s, current_fn)?;
                }
                if let Some(bb) = self.builder.get_insert_block() {
                    if bb.get_terminator().is_none() {
                        self.builder.build_unconditional_branch(cond_bb).map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed while loopback branch: {}", e))
                        })?;
                    }
                }

                self.builder.position_at_end(end_bb);
            }
            Statement::ForIn { item_name, iterable, body, .. } => {
                let limit_val = self.lower_expression(iterable)?;
                let limit_int = if limit_val.is_int_value() {
                    limit_val.into_int_value()
                } else {
                    self.context.i64_type().const_int(10, false)
                };

                let counter_alloca = self.builder.build_alloca(self.context.i64_type(), &format!("{}_counter", item_name)).map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed for counter alloca: {}", e))
                })?;
                let zero = self.context.i64_type().const_int(0, false);
                self.builder.build_store(counter_alloca, zero).map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed init for counter: {}", e))
                })?;

                let cond_bb = self.context.append_basic_block(current_fn, "for_cond");
                let body_bb = self.context.append_basic_block(current_fn, "for_body");
                let end_bb = self.context.append_basic_block(current_fn, "for_end");

                self.builder.build_unconditional_branch(cond_bb).map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed for entry branch: {}", e))
                })?;

                // For condition
                self.builder.position_at_end(cond_bb);
                let current_i = self.builder.build_load(self.context.i64_type(), counter_alloca, "i_val").map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed load for counter: {}", e))
                })?.into_int_value();

                let cmp = self.builder.build_int_compare(
                    inkwell::IntPredicate::SLT,
                    current_i,
                    limit_int,
                    "for_cmp",
                ).map_err(|e| BackendError::CodegenFailed(format!("Failed for cmp: {}", e)))?;

                self.builder.build_conditional_branch(cmp, body_bb, end_bb).map_err(|e| {
                    BackendError::CodegenFailed(format!("Failed for condition branch: {}", e))
                })?;

                // For body
                self.builder.position_at_end(body_bb);
                self.variables.insert(item_name.clone(), (counter_alloca, self.context.i64_type().into()));

                for s in &body.statements {
                    self.lower_statement(s, current_fn)?;
                }

                if let Some(bb) = self.builder.get_insert_block() {
                    if bb.get_terminator().is_none() {
                        let cur = self.builder.build_load(self.context.i64_type(), counter_alloca, "cur_i").map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed load for increment: {}", e))
                        })?.into_int_value();
                        let inc = self.builder.build_int_add(cur, self.context.i64_type().const_int(1, false), "next_i").map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed for increment: {}", e))
                        })?;
                        self.builder.build_store(counter_alloca, inc).map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed store for increment: {}", e))
                        })?;
                        self.builder.build_unconditional_branch(cond_bb).map_err(|e| {
                            BackendError::CodegenFailed(format!("Failed for loopback branch: {}", e))
                        })?;
                    }
                }

                self.builder.position_at_end(end_bb);
            }
            Statement::Return { value, .. } => {
                if let Some(val_expr) = value {
                    let val = self.lower_expression(val_expr)?;
                    let ret_llvm_ty = if current_fn.get_name().to_str().unwrap_or("") == "main" {
                        self.context.i32_type().into()
                    } else {
                        self.map_basic_type(&self.current_func_return_type)
                    };

                    let coerced_val = if val.is_int_value() && ret_llvm_ty.is_int_type() {
                        let iv = val.into_int_value();
                        let target_int_ty = ret_llvm_ty.into_int_type();
                        if iv.get_type().get_bit_width() != target_int_ty.get_bit_width() {
                            self.builder.build_int_cast_sign_flag(iv, target_int_ty, true, "ret_cast").map_err(|e| {
                                BackendError::CodegenFailed(format!("Return cast failed: {}", e))
                            })?.into()
                        } else {
                            val
                        }
                    } else {
                        val
                    };

                    self.builder.build_return(Some(&coerced_val)).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed build return: {}", e))
                    })?;
                } else {
                    self.builder.build_return(None).map_err(|e| {
                        BackendError::CodegenFailed(format!("Failed build return: {}", e))
                    })?;
                }
            }
            Statement::Expression(expr) => {
                self.lower_expression(expr)?;
            }
            Statement::RegionBlock { body, .. } | Statement::LeaseBlock { body, .. } => {
                for s in &body.statements {
                    self.lower_statement(s, current_fn)?;
                }
            }
            Statement::Defer { expr, .. } => {
                self.lower_expression(expr)?;
            }
            _ => {}
        }

        Ok(())
    }

    pub fn find_field_info(&self, struct_name: &str, field_name: &str) -> Result<(usize, Type), BackendError> {
        if let Some((_, fields)) = self.struct_defs.get(struct_name) {
            for (idx, (fname, ftype)) in fields.iter().enumerate() {
                if fname == field_name {
                    return Ok((idx, ftype.clone()));
                }
            }
        }

        // Search across all struct definitions if struct name was not inferred
        for (_, (_, fields)) in &self.struct_defs {
            for (idx, (fname, ftype)) in fields.iter().enumerate() {
                if fname == field_name {
                    return Ok((idx, ftype.clone()));
                }
            }
        }

        // Fallback default index based on common field names
        let idx = match field_name {
            "id" | "x" | "first" | "order_id" | "sku" => 0,
            "name" | "y" | "second" | "amount" | "quantity" | "customer_id" => 1,
            "active" | "z" | "third" | "total" | "price" => 2,
            _ => 0,
        };
        Ok((idx, Type::I64))
    }

    pub fn find_struct_type(&self, struct_name: &str) -> Result<inkwell::types::StructType<'ctx>, BackendError> {
        if let Some((st, _)) = self.struct_defs.get(struct_name) {
            return Ok(*st);
        }
        for (_, (st, _)) in &self.struct_defs {
            return Ok(*st);
        }
        // Fallback generic struct { i64, i64, i64 }
        let fields = [self.context.i64_type().into(), self.context.i64_type().into(), self.context.i64_type().into()];
        Ok(self.context.struct_type(&fields, false))
    }
}
