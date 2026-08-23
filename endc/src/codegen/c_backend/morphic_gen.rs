use super::state::CBackend;
use crate::ast::*;
use std::collections::HashSet;

impl CBackend {
    pub(crate) fn gen_morphic_specializations(&mut self, module: &Module) {
        let mut morphic_templates: Vec<FunctionDef> = Vec::new();
        for f in &module.functions {
            if f.morphic_param.is_some() {
                morphic_templates.push(f.clone());
            }
        }

        let mut all_call_names: Vec<String> = Vec::new();
        for f in &module.functions {
            collect_call_names(&f.body.statements, &mut all_call_names);
        }

        // For each morphic template, find matching calls and generate concrete functions
        for mt in &morphic_templates {
            if let Some(ref morphic_var) = mt.morphic_param {
                let template_name = &mt.name;
                let brace_open = template_name.find('{');
                let brace_close = template_name.find('}');
                if let (Some(bo), Some(bc)) = (brace_open, brace_close) {
                    let prefix = &template_name[..bo];
                    let suffix = &template_name[bc+1..];

                    let mut seen = HashSet::new();
                    for call_name in &all_call_names {
                        if call_name.ends_with(suffix) && call_name.len() > suffix.len() + prefix.len() {
                            let concrete_value = &call_name[prefix.len()..call_name.len()-suffix.len()];
                            if !concrete_value.is_empty() && !seen.contains(concrete_value) {
                                seen.insert(concrete_value.to_string());
                                let mut concrete_fn = mt.clone();
                                concrete_fn.name = call_name.clone();
                                concrete_fn.morphic_param = None;

                                let morphic_decl = Statement::VarDecl {
                                    name: morphic_var.clone(),
                                    var_type: Some(Type::Str),
                                    is_mut: false,
                                    is_lease: false,
                                    initializer: Some(Expression::Lit(
                                        Literal::String(concrete_value.to_string()),
                                        mt.span.clone(),
                                    )),
                                    span: mt.span.clone(),
                                };
                                let mut new_stmts = vec![morphic_decl];
                                new_stmts.extend(concrete_fn.body.statements.clone());
                                concrete_fn.body.statements = new_stmts;

                                self.gen_function(&concrete_fn);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn collect_call_names(stmts: &[Statement], names: &mut Vec<String>) {
    for stmt in stmts {
        match stmt {
            Statement::Expression(expr) | Statement::Return { value: Some(expr), .. } |
            Statement::Defer { expr, .. } | Statement::Spawn { call: expr, .. } => {
                collect_expr_calls(expr, names);
            }
            Statement::VarDecl { initializer: Some(init), .. } => {
                collect_expr_calls(init, names);
            }
            Statement::Assignment { value, .. } => {
                collect_expr_calls(value, names);
            }
            Statement::If { condition, then_block, else_block, .. } => {
                collect_expr_calls(condition, names);
                collect_call_names(&then_block.statements, names);
                if let Some(eb) = else_block {
                    collect_call_names(&eb.statements, names);
                }
            }
            Statement::Guard { condition, else_block, .. } => {
                collect_expr_calls(condition, names);
                collect_call_names(&else_block.statements, names);
            }
            Statement::While { condition, body, .. } => {
                collect_expr_calls(condition, names);
                collect_call_names(&body.statements, names);
            }
            Statement::ForIn { body, .. } | Statement::ParallelFor { body, .. } |
            Statement::RegionBlock { body, .. } | Statement::TargetBlock { body, .. } => {
                collect_call_names(&body.statements, names);
            }
            Statement::Match { expr, arms, .. } => {
                collect_expr_calls(expr, names);
                for arm in arms {
                    collect_call_names(&arm.body.statements, names);
                }
            }
            Statement::QuantumUnwrap { expr, fallback, .. } => {
                collect_expr_calls(expr, names);
                collect_expr_calls(fallback, names);
            }
            Statement::AtomicOp { value, .. } => {
                collect_expr_calls(value, names);
            }
            Statement::Owned { initializer, .. } => {
                collect_expr_calls(initializer, names);
            }
            Statement::Prove { condition, .. }
            | Statement::Assume { condition, .. }
            | Statement::Guarantee { condition, .. }
            | Statement::Invariant { condition, .. } => {
                collect_expr_calls(condition, names);
            }
            Statement::VerifyBlock { invariants, .. } => {
                for inv in invariants {
                    collect_expr_calls(inv, names);
                }
            }
            Statement::Intent { body: Some(body), .. }
            | Statement::ProtectBlock { body, .. }
            | Statement::DeterministicBlock { body, .. }
            | Statement::ReplayBlock { body, .. }
            | Statement::TransactionBlock { body, .. }
            | Statement::SpeculativeBlock { body, .. }
            | Statement::FallbackBlock { body, .. }
            | Statement::CancelSafeBlock { body, .. }
            | Statement::TaskDecl { body, .. }
            | Statement::PatchDecl { body, .. }
            | Statement::RaceFreeBlock { body, .. }
            | Statement::DeadlineBlock { body, .. }
            | Statement::PriorityBlock { body, .. }
            | Statement::QualityBlock { body, .. }
            | Statement::TradeoffBlock { body, .. }
            | Statement::WatchBlock { handler: body, .. } => {
                collect_call_names(&body.statements, names);
            }
            Statement::ReactBlock { event, handler, .. } => {
                collect_expr_calls(event, names);
                collect_call_names(&handler.statements, names);
            }
            Statement::StreamBlock { source, operations, .. } => {
                collect_expr_calls(source, names);
                for op in operations {
                    collect_expr_calls(op, names);
                }
            }
            Statement::FlowBlock { steps, .. } => {
                for step in steps {
                    collect_expr_calls(step, names);
                }
            }
            Statement::ComputeBlock { body, fallback, .. } => {
                collect_call_names(&body.statements, names);
                if let Some(fb) = fallback {
                    collect_call_names(&fb.statements, names);
                }
            }
            Statement::BudgetBlock { body: Some(body), .. }
            | Statement::ContextBlock { body: Some(body), .. }
            | Statement::AgentContract { body: Some(body), .. }
            | Statement::EvolveBlock { body: Some(body), .. } => {
                collect_call_names(&body.statements, names);
            }
            Statement::AdaptBlock { branches, .. } => {
                for (cond, blk) in branches {
                    collect_expr_calls(cond, names);
                    collect_call_names(&blk.statements, names);
                }
            }
            Statement::ParallelChoose { branches, .. } => {
                for (_, blk) in branches {
                    collect_call_names(&blk.statements, names);
                }
            }
            Statement::RaceBlock { branches, .. } => {
                for blk in branches {
                    collect_call_names(&blk.statements, names);
                }
            }
            Statement::HedgeBlock { delay_ms, primary, fallback, .. } => {
                collect_expr_calls(delay_ms, names);
                collect_call_names(&primary.statements, names);
                collect_call_names(&fallback.statements, names);
            }
            _ => {}
        }
    }
}

fn collect_expr_calls(expr: &Expression, names: &mut Vec<String>) {
    match expr {
        Expression::Call { callee, args, .. } => {
            if let Expression::Ident(name, _) = callee.as_ref() {
                names.push(name.clone());
            }
            for a in args {
                collect_expr_calls(a, names);
            }
        }
        Expression::Binary { left, right, .. } => {
            collect_expr_calls(left, names);
            collect_expr_calls(right, names);
        }
        Expression::Unary { expr, .. } | Expression::Await { expr, .. } => {
            collect_expr_calls(expr, names);
        }
        Expression::Pipe { lhs, rhs, .. } | Expression::NullCollapse { left: lhs, right: rhs, .. } => {
            collect_expr_calls(lhs, names);
            collect_expr_calls(rhs, names);
        }
        _ => {}
    }
}
