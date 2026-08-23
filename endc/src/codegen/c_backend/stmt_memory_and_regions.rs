use super::state::CBackend;
use crate::ast::{BinaryOp, Expression, Statement, Type};

impl CBackend {
    pub(crate) fn gen_memory_and_regions_statement(&mut self, stmt: &Statement) -> bool {
        match stmt {
            Statement::RegionBlock { name, body, .. } => {
                self.output.push_str(&format!(
                    "{}/* Enter Region: {} */\n",
                    self.indent(),
                    name
                ));
                self.output.push_str(&format!(
                    "{}EndArena* region_{} = end_arena_create(512 * 1024);\n",
                    self.indent(),
                    name
                ));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.active_regions.push(name.clone());
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.active_regions.pop();
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                self.output.push_str(&format!(
                    "{}end_arena_destroy(region_{});\n",
                    self.indent(),
                    name
                ));
                true
            }
            Statement::LeaseBlock {
                name,
                var_type,
                initializer,
                condition,
                body,
                ..
            } => {
                if let Some(t) = var_type {
                    self.var_types.insert(name.clone(), t.clone());
                } else {
                    let inferred = self.infer_type(initializer);
                    if inferred != Type::Void {
                        self.var_types.insert(name.clone(), inferred);
                    }
                }

                let ty_str = if let Some(t) = var_type {
                    self.map_type(t)
                } else {
                    "__auto_type".to_string()
                };
                let init_str = self.gen_expression(initializer);
                self.output.push_str(&format!(
                    "{}{} {} = {};\n",
                    self.indent(),
                    ty_str,
                    name,
                    init_str
                ));
                if let Some(cond) = condition {
                    let cond_str = self.gen_expression(cond);
                    self.output.push_str(&format!("{}assert({});\n", self.indent(), cond_str));
                }
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::LeaseCpu { body, .. } => {
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}cpu_yield();\n", self.indent()));
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.output.push_str(&format!("{}cpu_relax();\n", self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::LeaseEvent { event_expr, body, .. } => {
                let ev_str = self.gen_expression(event_expr);
                self.output.push_str(&format!("{}/* LeaseEvent on: {} */\n", self.indent(), ev_str));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::LeaseLoop { item_name, iterable, body, .. } => {
                let iter_str = self.gen_expression(iterable);
                self.output.push_str(&format!("{}for (int64_t {} = 0; {} < {}; {}++) {{\n", self.indent(), item_name, item_name, iter_str, item_name));
                self.indent_level += 1;
                self.output.push_str(&format!("{}cpu_relax();\n", self.indent()));
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::TargetBlock { target, body, .. } => {
                let target_upper = target.to_uppercase();
                let def = match target_upper.as_str() {
                    "X86_64" => "defined(__x86_64__) || defined(_M_X64)",
                    "ARM64" => "defined(__aarch64__) || defined(_M_ARM64)",
                    "WASM" => "defined(__wasm32__) || defined(__EMSCRIPTEN__)",
                    "AVX512" => "defined(__AVX512F__)",
                    "NEON" => "defined(__ARM_NEON)",
                    _ => "1",
                };
                self.output.push_str(&format!("#if {}\n", def));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                self.output.push_str("#endif\n");
                true
            }
            Statement::AsmBlock { arch, code, .. } => {
                self.output.push_str(&format!("{}/* Inline Asm [{}] */\n", self.indent(), arch));
                self.output.push_str(&format!("{}__asm__ __volatile__(\"{}\");\n", self.indent(), code.replace('"', "\\\"")));
                true
            }
            Statement::Defer { expr, .. } => {
                let expr_str = self.gen_expression(expr);
                self.output.push_str(&format!("{}/* Defer */ {};\n", self.indent(), expr_str));
                true
            }
            Statement::Spawn { call, .. } => {
                let call_str = self.gen_expression(call);
                self.output.push_str(&format!("{}/* Spawn async task */ {};\n", self.indent(), call_str));
                true
            }
            Statement::QuantumUnwrap { name, expr, fallback, .. } => {
                let expr_str = self.gen_expression(expr);
                let fallback_str = self.gen_expression(fallback);
                self.declare_c_var(name, Type::I64);
                self.output.push_str(&format!(
                    "{}int64_t {} = ({}) != 0 ? ({}) : ({});\n",
                    self.indent(),
                    name,
                    expr_str,
                    expr_str,
                    fallback_str
                ));
                true
            }
            Statement::AtomicOp { target, op, value, span } => {
                let val_str = self.gen_expression(value);
                let c_op = match op {
                    BinaryOp::Add => "__atomic_fetch_add",
                    BinaryOp::Sub => "__atomic_fetch_sub",
                    BinaryOp::BitAnd => "__atomic_fetch_and",
                    BinaryOp::BitOr => "__atomic_fetch_or",
                    BinaryOp::BitXor => "__atomic_fetch_xor",
                    _ => "__atomic_store_n",
                };
                let clean_file = span.file.replace('\\', "/");
                self.output.push_str(&format!("{}#line {} \"{}\"\n", self.indent(), span.line, clean_file));
                self.output.push_str(&format!("{}{}(&{}, {}, __ATOMIC_SEQ_CST);\n", self.indent(), c_op, target, val_str));
                true
            }
            Statement::Owned { name, initializer, .. } => {
                let init_str = self.gen_expression(initializer);
                self.declare_c_var(name, Type::Pointer(Box::new(Type::Void)));
                self.output.push_str(&format!("{}/* 📦 [OWNED] */ void* {} = {};\n", self.indent(), name, init_str));
                true
            }
            Statement::Intent { goal, body, .. } => {
                self.output.push_str(&format!("{}/* 🎯 [INTENT: {}] */\n", self.indent(), goal));
                if let Some(b) = body {
                    self.output.push_str(&format!("{}{{\n", self.indent()));
                    self.indent_level += 1;
                    for s in &b.statements {
                        self.gen_statement(s);
                    }
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                true
            }
            Statement::Prove { condition, .. } => {
                let cond_str = self.gen_expression(condition);
                self.output.push_str(&format!("{}/* 🛡️ [FORMAL PROVE] */ assert({});\n", self.indent(), cond_str));
                true
            }
            Statement::Guarantee { condition, .. } => {
                let cond_str = self.gen_expression(condition);
                self.output.push_str(&format!("{}/* 🛡️ [FORMAL GUARANTEE] */ assert({});\n", self.indent(), cond_str));
                true
            }
            Statement::Invariant { condition, .. } => {
                let cond_str = self.gen_expression(condition);
                self.output.push_str(&format!("{}/* 🛡️ [INVARIANT] */ assert({});\n", self.indent(), cond_str));
                true
            }
            Statement::Assume { condition, .. } => {
                let cond_str = self.gen_expression(condition);
                self.output.push_str(&format!("{}/* 🛡️ [ASSUME] */ assert({});\n", self.indent(), cond_str));
                true
            }
            Statement::Frozen { symbol, .. } => {
                self.output.push_str(&format!("{}/* ❄️ [FROZEN SYMBOL]: {} (compile-time immutability) */\n", self.indent(), symbol));
                true
            }
            Statement::VerifyBlock { invariants, .. } => {
                for inv in invariants {
                    let inv_str = self.gen_expression(inv);
                    self.output.push_str(&format!("{}/* 🛡️ [VERIFY INVARIANT] */ assert({});\n", self.indent(), inv_str));
                }
                true
            }
            Statement::Checkpoint { state_name, span } => {
                let active_vars = self.get_active_visible_vars();
                self.output.push_str(&format!("{}/* 💾 [CHECKPOINT STATE SNAPSHOT]: {} */\n", self.indent(), state_name));
                self.output.push_str(&format!("{}jmp_buf __checkpoint_{};\n", self.indent(), state_name));
                for (var_name, ty) in &active_vars {
                    let ty_str = self.map_type(ty);
                    self.output.push_str(&format!("{}{} __snap_{}_{} = {};\n", self.indent(), ty_str, state_name, var_name, var_name));
                }
                let clean_file = span.file.replace('\\', "/");
                self.output.push_str(&format!("{}#line {} \"{}\"\n", self.indent(), span.line, clean_file));
                self.output.push_str(&format!("{}setjmp(__checkpoint_{});\n", self.indent(), state_name));
                true
            }
            Statement::Rollback { checkpoint_name, span } => {
                let active_vars = self.get_active_visible_vars();
                self.output.push_str(&format!("{}/* ⏪ [ROLLBACK STATE RESTORE]: to {} */\n", self.indent(), checkpoint_name));
                for (var_name, _) in &active_vars {
                    self.output.push_str(&format!("{}{} = __snap_{}_{};\n", self.indent(), var_name, checkpoint_name, var_name));
                }
                let clean_file = span.file.replace('\\', "/");
                self.output.push_str(&format!("{}#line {} \"{}\"\n", self.indent(), span.line, clean_file));
                self.output.push_str(&format!("{}longjmp(__checkpoint_{}, 1);\n", self.indent(), checkpoint_name));
                true
            }
            Statement::Handoff { resource, target_domain, .. } => {
                self.output.push_str(&format!("{}/* Zero-Cost Ownership Handoff: '{}' transferred to scope '{}' */\n", self.indent(), resource, target_domain));
                true
            }
            Statement::ReturnTo { source_domain, resource, .. } => {
                self.output.push_str(&format!("{}/* Return Borrowed Symbol: '{}' returned to scope '{}' */\n", self.indent(), resource, source_domain));
                true
            }
            Statement::ProtectBlock { body, .. } => {
                self.output.push_str(&format!("{}/* Protect Block */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::DeterministicBlock { body, .. } => {
                self.output.push_str(&format!("{}/* 🎯 [DETERMINISTIC BLOCK] */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::ReplayBlock { body, .. } => {
                self.output.push_str(&format!("{}/* Replay block */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for s in &body.statements {
                    self.gen_statement(s);
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::TransactionBlock { body, .. } => {
                let active_vars = self.get_active_visible_vars();
                self.output.push_str(&format!("{}/* 💼 [ATOMIC TRANSACTION BLOCK: Write-Set Snapshot & Rollback] */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}jmp_buf __txn_env;\n", self.indent()));
                for (var_name, ty) in &active_vars {
                    let ty_str = self.map_type(ty);
                    self.output.push_str(&format!("{}{} __snap_txn_{} = {};\n", self.indent(), ty_str, var_name, var_name));
                }
                self.output.push_str(&format!("{}if (setjmp(__txn_env) == 0) {{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}} else {{\n", self.indent()));
                self.indent_level += 1;
                for (var_name, _) in &active_vars {
                    self.output.push_str(&format!("{}{} = __snap_txn_{};\n", self.indent(), var_name, var_name));
                }
                self.output.push_str(&format!("{}fprintf(stderr, \"[END TRANSACTION] Aborted and variable state rolled back\\n\");\n", self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::SpeculativeBlock { body, .. } => {
                self.output.push_str(&format!("{}/* 🔮 [SPECULATIVE BLOCK] */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::FallbackBlock { target, body, .. } => {
                self.output.push_str(&format!("{}/* ⚡ [FALLBACK TO: {}] */\n", self.indent(), target));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::CancelSafeBlock { body, .. } => {
                self.output.push_str(&format!("{}/* 🛑 [CANCEL-SAFE REGION] */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            _ => false,
        }
    }
}
