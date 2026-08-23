use super::state::Interpreter;
use super::value::Value;
use crate::ast::{FunctionDef, Statement};

impl Interpreter {
    pub(crate) fn eval_operations_and_events_statement(&mut self, stmt: &Statement) -> Result<Option<Option<Value>>, String> {
        match stmt {
            Statement::AdapterDecl { body, .. }
            | Statement::PreserveRefactorDecl { body, .. }
            | Statement::CompatDecl { body, .. } => {
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                Ok(Some(None))
            }
            Statement::SplitDecl { entity, parts, .. } => {
                let parts_str = parts.join(", ");
                self.set_var(&format!("__split_{}", entity), Value::String(parts_str));
                Ok(Some(None))
            }
            Statement::PartitionDecl { entity, by, parts, .. } => {
                let parts_str = parts.join(", ");
                self.set_var(&format!("__partition_{}_{}", entity, by), Value::String(parts_str));
                Ok(Some(None))
            }
            Statement::ExtractDecl { symbols, into_module, .. } => {
                self.set_var(&format!("__extract_{}", into_module), Value::String(symbols.join(", ")));
                Ok(Some(None))
            }
            Statement::ClusterDecl { by, predicate, .. } => {
                self.set_var(&format!("__cluster_{}", by), Value::String(predicate.clone()));
                Ok(Some(None))
            }
            Statement::SeparateDecl { left, right, .. } => {
                self.set_var(&format!("__separate_{}_{}", left, right), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::MoveDecl { symbol, from_mod, to_mod, .. } => {
                self.set_var(&format!("__move_{}_{}_{}", symbol, from_mod, to_mod), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::MigrateDecl { entity, from_mod, to_mod, .. } => {
                self.set_var(&format!("__migrate_{}_{}_{}", entity, from_mod, to_mod), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::RedirectDecl { from_api, to_api, .. } => {
                self.set_var(&format!("__redirect_{}_{}", from_api, to_api), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::DecomposeDecl { target, target_modules, .. } => {
                let count = target_modules.unwrap_or(25);
                self.set_var(&format!("__decompose_{}_target", target), Value::Int(count as i64));
                Ok(Some(None))
            }
            Statement::ModularizeDecl { target, target_files_min, target_files_max, .. } => {
                self.set_var(&format!("__modularize_{}", target), Value::Int(*target_files_max as i64));
                self.set_var(&format!("__modularize_{}_min", target), Value::Int(*target_files_min as i64));
                Ok(Some(None))
            }
            Statement::EvolveArchDecl { from, toward, target_modules, .. } => {
                self.set_var(&format!("__evolve_{}_{}", from, toward), Value::Int(*target_modules as i64));
                Ok(Some(None))
            }
            Statement::GravityDecl { weights, .. } => {
                for (k, w) in weights {
                    self.set_var(&format!("__gravity_{}", k), Value::Float(*w));
                }
                Ok(Some(None))
            }
            Statement::BudgetContextDecl { name, token_budget, .. } => {
                self.set_var(&format!("__budget_context_{}", name), Value::Int(*token_budget as i64));
                Ok(Some(None))
            }
            Statement::RepairDecl { target, .. } => {
                self.set_var(&format!("__repair_{}", target), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::OperationDecl(op_def) => {
                let op_val = Value::Operation {
                    name: if op_def.name.is_empty() { None } else { Some(op_def.name.clone()) },
                    params: op_def.params.clone(),
                    return_type: op_def.return_type.clone(),
                    requires: op_def.requires.clone(),
                    guarantees: op_def.guarantees.clone(),
                    effects: op_def.effects.clone(),
                    emits: op_def.emits.clone(),
                    version: op_def.version,
                    body: op_def.body.clone(),
                };
                if !op_def.name.is_empty() {
                    self.operations.insert(op_def.name.clone(), op_val.clone());
                    self.set_var(&op_def.name, op_val);
                    let fn_def = FunctionDef {
                        name: op_def.name.clone(),
                        generic_params: Vec::new(),
                        is_pub: op_def.is_pub,
                        params: op_def.params.clone(),
                        return_type: op_def.return_type.clone(),
                        body: op_def.body.clone(),
                        directives: Vec::new(),
                        morphic_param: None,
                        span: op_def.span.clone(),
                    };
                    self.functions.insert(op_def.name.clone(), fn_def);
                }
                Ok(Some(None))
            }
            Statement::EventDecl(ev_def) => {
                self.set_var(&format!("__event_{}", ev_def.name), Value::String(ev_def.name.clone()));
                Ok(Some(None))
            }
            Statement::EventHubDecl(hub_def) => {
                self.event_hubs.insert(hub_def.name.clone(), hub_def.clone());
                for handler in &hub_def.handlers {
                    self.event_handlers.entry(handler.event_name.clone()).or_insert_with(Vec::new).push(handler.clone());
                }
                Ok(Some(None))
            }
            Statement::EmitEvent { event_name, args, .. } => {
                self.emitted_events.push(event_name.clone());
                let mut eval_args = Vec::new();
                for a in args {
                    eval_args.push(self.eval_expression(a)?);
                }
                if let Some(handlers) = self.event_handlers.get(event_name).cloned() {
                    for h in handlers {
                        if let Some(ref op_expr) = h.handler_op {
                            let op_val = self.eval_expression(op_expr)?;
                            if matches!(op_val, Value::Operation { .. }) {
                                let _ = self.eval_operation(&op_val, eval_args.clone())?;
                            }
                        } else if let Some(ref blk) = h.body {
                            self.push_scope();
                            for s in &blk.statements {
                                if let Some(ret) = self.eval_statement(s)? {
                                    self.pop_scope();
                                    return Ok(Some(Some(ret)));
                                }
                            }
                            self.pop_scope();
                        }
                    }
                }
                Ok(Some(None))
            }
            Statement::ObserveOp { op_expr, alias, .. } => {
                let op_val = if let crate::ast::Expression::Ident(id, _) = op_expr {
                    self.operations.get(id).cloned().or_else(|| self.get_var(id))
                } else {
                    self.eval_expression(op_expr).ok()
                }.unwrap_or_else(|| Value::String(format!("{:?}", op_expr)));

                self.set_var(alias, op_val);
                self.traces.push(alias.clone());
                Ok(Some(None))
            }
            Statement::AnalyzeOp { op_expr, .. } => {
                let (op_name, op_val) = if let crate::ast::Expression::Ident(id, _) = op_expr {
                    (id.clone(), self.operations.get(id).cloned())
                } else {
                    ("anon".to_string(), self.eval_expression(op_expr).ok())
                };
                let analysis_summary = if let Some(Value::Operation { name, requires, guarantees, effects, emits, .. }) = &op_val {
                    format!("Operation: {:?}, requires: {:?}, guarantees: {:?}, effects: {:?}, emits: {:?}", name, requires, guarantees, effects, emits)
                } else {
                    format!("Operation: {}, static analysis complete", op_name)
                };
                self.set_var(&format!("__analysis_{}", op_name), Value::String(analysis_summary));
                Ok(Some(None))
            }
            Statement::ExtractOpDecl { op_name, from_mod, condition, .. } => {
                self.set_var(&format!("__extract_op_{}", op_name), Value::String(format!("{}: {}", from_mod, condition)));
                Ok(Some(None))
            }
            Statement::InlineOpDecl { op_name, .. } => {
                self.set_var(&format!("__inline_op_{}", op_name), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::SplitOpDecl { op_name, sub_ops, .. } => {
                self.set_var(&format!("__split_op_{}", op_name), Value::String(sub_ops.join(", ")));
                Ok(Some(None))
            }
            Statement::MergeOpDecl { source_ops, as_name, .. } => {
                self.set_var(&format!("__merge_op_{}", as_name), Value::String(source_ops.join(" + ")));
                Ok(Some(None))
            }
            Statement::ExplainOpDecl { op_name, .. } => {
                self.set_var(&format!("__explain_op_{}", op_name), Value::String(format!("Contract explanation for operation {}", op_name)));
                Ok(Some(None))
            }
            Statement::EvolveOpDecl { op_name, preserve, optimize, allow, reject, .. } => {
                self.set_var(&format!("__evolve_op_{}", op_name), Value::String(format!("preserve: {:?}, optimize: {:?}, allow: {:?}, reject: {:?}", preserve, optimize, allow, reject)));
                Ok(Some(None))
            }
            Statement::OnEventStmt(on_ev) => {
                let should_run = if let Some(g) = &on_ev.guard {
                    match self.eval_expression(g)? {
                        Value::Bool(b) => b,
                        Value::Int(n) => n != 0,
                        _ => true,
                    }
                } else {
                    true
                };
                if should_run {
                    self.push_scope();
                    for s in &on_ev.body.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(Some(ret)));
                        }
                    }
                    self.pop_scope();
                }
                self.set_var(&format!("__on_event_{}", on_ev.event_pattern.replace('.', "_")), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::OnceEventStmt(once_ev) => {
                self.push_scope();
                for s in &once_ev.body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                self.set_var(&format!("__once_event_{}", once_ev.event_pattern.replace('.', "_")), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::EveryEventStmt(ev) => {
                self.push_scope();
                for s in &ev.body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                self.set_var("__every_event", Value::String(ev.interval_str.clone()));
                Ok(Some(None))
            }
            Statement::AfterEventStmt(ev) => {
                self.push_scope();
                for s in &ev.body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                self.set_var("__after_event", Value::String(ev.delay_str.clone()));
                Ok(Some(None))
            }
            Statement::BeforeEventStmt(ev) => {
                self.push_scope();
                for s in &ev.body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                self.set_var(&format!("__before_event_{}", ev.event_pattern.replace('.', "_")), Value::Bool(true));
                Ok(Some(None))
            }
            Statement::ReactiveStateStmt(st) => {
                let init_val = self.eval_expression(&st.initial_val)?;
                self.set_var(&st.name, init_val.clone());
                self.set_var(&format!("__state_{}", st.name), init_val);
                Ok(Some(None))
            }
            Statement::DeriveStmt(d) => {
                let derived_val = self.eval_expression(&d.expr)?;
                self.set_var(&d.target_var, derived_val.clone());
                self.set_var(&format!("__derived_{}", d.target_var), derived_val);
                Ok(Some(None))
            }
            Statement::TopologyStmt(top) => {
                self.set_var(&format!("__topology_{}", top.name), Value::String(format!("nodes={:?}, edges={:?}", top.nodes, top.edges)));
                Ok(Some(None))
            }
            Statement::EventStreamOpStmt(op) => {
                if let Some(b) = &op.body {
                    self.push_scope();
                    for s in &b.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(Some(ret)));
                        }
                    }
                    self.pop_scope();
                }
                self.set_var(&format!("__{}_{}", op.op_kind, op.target), Value::String(op.params.join(", ")));
                Ok(Some(None))
            }
            Statement::EventTransactionStmt(tx) => {
                let checkpoint = self.variables.clone();
                let mut failed = false;
                self.push_scope();
                for s in &tx.statements {
                    if self.eval_statement(s).is_err() {
                        failed = true;
                        break;
                    }
                }
                self.pop_scope();
                if failed {
                    self.variables = checkpoint;
                    if let Some(r) = &tx.on_rollback {
                        self.push_scope();
                        for s in &r.statements {
                            let _ = self.eval_statement(s);
                        }
                        self.pop_scope();
                    }
                }
                Ok(Some(None))
            }
            Statement::EventControlStmt(ctl) => {
                self.set_var(&format!("__{}_{}", ctl.action, ctl.target), Value::String(ctl.args.join(", ")));
                Ok(Some(None))
            }
            _ => Ok(None),
        }
    }
}
