use super::state::CBackend;
use crate::ast::{Expression, FunctionDef, Statement};

impl CBackend {
    pub(crate) fn gen_ops_and_events_statement(&mut self, stmt: &Statement) -> bool {
        match stmt {
            Statement::OperationDecl(op) => {
                self.output.push_str(&format!("{}/* ⚡ [OPERATION DECLARATION '{}']: requires=[{}], guarantees=[{}], effects=[{}] */\n",
                    self.indent(), op.name, op.requires.join(", "), op.guarantees.join(", "), op.effects.join(", ")));
                if !op.name.is_empty() {
                    let fn_def = FunctionDef {
                        name: op.name.clone(),
                        generic_params: Vec::new(),
                        is_pub: op.is_pub,
                        params: op.params.clone(),
                        return_type: op.return_type.clone(),
                        body: op.body.clone(),
                        directives: Vec::new(),
                        morphic_param: None,
                        span: op.span.clone(),
                    };
                    self.gen_function(&fn_def);
                }
                true
            }
            Statement::EventDecl(ev) => {
                self.output.push_str(&format!("{}/* 🔔 [EVENT DECLARATION '{}'] */\n", self.indent(), ev.name));
                true
            }
            Statement::EventHubDecl(hub) => {
                self.output.push_str(&format!("{}/* 🌐 [EVENT HUB '{}']: owns=[{}] */\n", self.indent(), hub.name, hub.owns_events.join(", ")));
                true
            }
            Statement::EmitEvent { event_name, args, .. } => {
                let args_str = args.iter().map(|a| {
                    match a {
                        Expression::StructInit { name, fields, span } if name.is_empty() => {
                            let named_init = Expression::StructInit {
                                name: event_name.clone(),
                                fields: fields.clone(),
                                span: span.clone(),
                            };
                            self.gen_expression(&named_init)
                        }
                        _ => self.gen_expression(a),
                    }
                }).collect::<Vec<_>>().join(", ");
                let args_part = if args_str.is_empty() { "0" } else { &args_str };
                self.output.push_str(&format!("{}end_emit_event(\"{}\", {});\n", self.indent(), event_name, args_part));
                true
            }
            Statement::ObserveOp { op_expr, alias, .. } => {
                let expr_str = self.gen_expression(op_expr);
                self.output.push_str(&format!("{}EndTrace* {} = end_observe_operation({});\n", self.indent(), alias, expr_str));
                true
            }
            Statement::AnalyzeOp { op_expr, .. } => {
                let expr_str = self.gen_expression(op_expr);
                self.output.push_str(&format!("{}end_analyze_operation({});\n", self.indent(), expr_str));
                true
            }
            Statement::ExtractOpDecl { op_name, from_mod, condition, .. } => {
                self.output.push_str(&format!("{}/* ✂️ [EXTRACT OPERATION '{}' FROM '{}' WHERE '{}'] */\n", self.indent(), op_name, from_mod, condition));
                true
            }
            Statement::InlineOpDecl { op_name, .. } => {
                self.output.push_str(&format!("{}/* 📥 [INLINE OPERATION '{}'] */\n", self.indent(), op_name));
                true
            }
            Statement::SplitOpDecl { op_name, sub_ops, .. } => {
                self.output.push_str(&format!("{}/* 🪓 [SPLIT OPERATION '{}' INTO [{}]] */\n", self.indent(), op_name, sub_ops.join(", ")));
                true
            }
            Statement::MergeOpDecl { source_ops, as_name, .. } => {
                self.output.push_str(&format!("{}/* 🔗 [MERGE OPERATIONS [{}] AS '{}'] */\n", self.indent(), source_ops.join(", "), as_name));
                true
            }
            Statement::ExplainOpDecl { op_name, .. } => {
                self.output.push_str(&format!("{}/* 💡 [EXPLAIN OPERATION '{}'] */\n", self.indent(), op_name));
                true
            }
            Statement::EvolveOpDecl { op_name, preserve, optimize, allow, reject, .. } => {
                self.output.push_str(&format!("{}/* 🧬 [EVOLVE OPERATION '{}']: preserve=[{}], optimize=[{}], allow=[{}], reject=[{}] */\n",
                    self.indent(), op_name, preserve.join(", "), optimize.join(", "), allow.join(", "), reject.join(", ")));
                true
            }
            Statement::OnEventStmt(on_ev) => {
                self.output.push_str(&format!("{}/* 🔔 [ON EVENT '{}'] */\n", self.indent(), on_ev.event_pattern));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&on_ev.body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::OnceEventStmt(once_ev) => {
                self.output.push_str(&format!("{}/* 🔔 [ONCE EVENT '{}'] */\n", self.indent(), once_ev.event_pattern));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&once_ev.body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::EveryEventStmt(ev) => {
                self.output.push_str(&format!("{}/* ⏱️ [EVERY TICK: {}] */\n", self.indent(), ev.interval_str));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&ev.body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::AfterEventStmt(ev) => {
                self.output.push_str(&format!("{}/* ⏱️ [AFTER DELAY: {}] */\n", self.indent(), ev.delay_str));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&ev.body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::BeforeEventStmt(ev) => {
                self.output.push_str(&format!("{}/* 🛡️ [BEFORE HOOK '{}'] */\n", self.indent(), ev.event_pattern));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&ev.body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::ReactiveStateStmt(st) => {
                let init_str = self.gen_expression(&st.initial_val);
                self.output.push_str(&format!("{}int64_t {} = {}; /* ⚡ Reactive State */\n", self.indent(), st.name, init_str));
                true
            }
            Statement::DeriveStmt(d) => {
                let expr_str = self.gen_expression(&d.expr);
                self.output.push_str(&format!("{}int64_t {} = {}; /* 🔄 Derived State */\n", self.indent(), d.target_var, expr_str));
                true
            }
            Statement::TopologyStmt(top) => {
                self.output.push_str(&format!("{}/* 🌐 [TOPOLOGY GRAPH '{}']: nodes=[{}], edges=[{}] */\n",
                    self.indent(), top.name, top.nodes.join(", "), top.edges.iter().map(|(a,b)| format!("{}->{}", a, b)).collect::<Vec<_>>().join(", ")));
                true
            }
            Statement::EventStreamOpStmt(op) => {
                self.output.push_str(&format!("{}/* 🌊 [STREAM OP '{}' ON '{}']: params=[{}] */\n", self.indent(), op.op_kind, op.target, op.params.join(", ")));
                if let Some(b) = &op.body {
                    self.output.push_str(&format!("{}{{\n", self.indent()));
                    self.indent_level += 1;
                    self.gen_block_statements(&b.statements);
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                true
            }
            Statement::EventTransactionStmt(tx) => {
                self.output.push_str(&format!("{}/* 🔒 [EVENT TRANSACTION] */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&tx.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::EventControlStmt(ctl) => {
                self.output.push_str(&format!("{}/* 🎮 [EVENT CONTROL '{}' ON '{}']: args=[{}] */\n", self.indent(), ctl.action, ctl.target, ctl.args.join(", ")));
                true
            }
            _ => false,
        }
    }
}
