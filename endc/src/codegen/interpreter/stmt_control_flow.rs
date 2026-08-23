use super::state::Interpreter;
use super::value::Value;
use crate::ast::{Expression, Statement};

impl Interpreter {
    pub(crate) fn eval_control_flow_statement(&mut self, stmt: &Statement) -> Result<Option<Option<Value>>, String> {
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
                Ok(Some(None))
            }
            Statement::Assignment { target, value, .. } => {
                let val = self.eval_expression(value)?;
                match target {
                    Expression::Ident(name, _) => {
                        self.update_var(name, val)?;
                    }
                    Expression::FieldAccess { object, field, .. } => {
                        if let Expression::Ident(obj_name, _) = object.as_ref() {
                            if let Some(Value::Struct(sname, mut fields)) = self.get_var(obj_name) {
                                fields.insert(field.clone(), val);
                                self.update_var(obj_name, Value::Struct(sname, fields))?;
                            }
                        }
                    }
                    _ => {}
                }
                Ok(Some(None))
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    let val = self.eval_expression(v)?;
                    Ok(Some(Some(val)))
                } else {
                    Ok(Some(Some(Value::Void)))
                }
            }
            Statement::Expression(expr) => {
                let v = self.eval_expression(expr)?;
                if matches!(expr, Expression::Match { .. }) && !matches!(v, Value::Void) {
                    return Ok(Some(Some(v)));
                }
                Ok(Some(None))
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
                            return Ok(Some(Some(ret)));
                        }
                    }
                } else if let Some(eb) = else_block {
                    for s in &eb.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(Some(ret)));
                        }
                    }
                }
                Ok(Some(None))
            }
            Statement::ClassDecl(class_def) => {
                for m in &class_def.methods {
                    self.functions.insert(m.name.clone(), m.clone());
                    self.functions.insert(format!("{}_{}", class_def.name, m.name), m.clone());
                }
                Ok(Some(None))
            }
            Statement::Guard {
                condition,
                else_block,
                ..
            } => {
                let cond = self.eval_expression(condition)?;
                let is_true = match cond {
                    Value::Bool(b) => b,
                    Value::Int(n) => n != 0,
                    _ => false,
                };
                if !is_true {
                    for s in &else_block.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(Some(ret)));
                        }
                    }
                }
                Ok(Some(None))
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
                            return Ok(Some(Some(ret)));
                        }
                    }
                }
                Ok(Some(None))
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
                            return Ok(Some(Some(ret)));
                        }
                    }
                }
                self.pop_scope();
                Ok(Some(None))
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
                                    return Ok(Some(Some(ret)));
                                }
                            }
                            self.pop_scope();
                            break;
                        }
                        self.pop_scope();
                    }
                }
                Ok(Some(None))
            }
            _ => Ok(None),
        }
    }
}
