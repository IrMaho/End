use super::types::*;
use super::SemanticAnalyzer;
use crate::ast::*;

impl SemanticAnalyzer {
    pub(crate) fn analyze_architectural_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::LeaseBlock { name, var_type, initializer, condition, body, span } => {
                let inferred_ty = self.analyze_expression(initializer);
                if let Some(c) = condition {
                    self.analyze_expression(c);
                }
                self.push_scope();
                let ty = var_type.clone().unwrap_or(inferred_ty);
                self.declare_var(name, ty, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::LeaseCpu { cores, priority, body, span } => {
                self.analyze_expression(cores);
                if let Some(p) = priority {
                    self.analyze_expression(p);
                }
                self.push_scope();
                self.declare_var("__lease_cpu_cores", Type::I32, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::LeaseEvent { event_expr, condition, body, span } => {
                self.analyze_expression(event_expr);
                if let Some(c) = condition {
                    self.analyze_expression(c);
                }
                self.push_scope();
                self.declare_var("__lease_event_handle", Type::I64, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::LeaseLoop { budget, item_name, iterable, body, span } => {
                if let Some(b) = budget {
                    self.analyze_expression(b);
                }
                self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(item_name, Type::I64, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::Intent { body, .. } => {
                if let Some(b) = body {
                    self.analyze_block(b);
                }
            }
            Statement::Prove { condition, span } => {
                self.analyze_expression(condition);
                if self.eval_static_const_bool(condition) == Some(false) {
                    self.errors.push(DiagnosticError {
                        code: "E0911".to_string(),
                        message: format!("StaticProofFailed: static proof obligation failed at line {} (expression is provably false at compile time)", span.line),
                        line: span.line,
                        col: span.col,
                        kind: "StaticProofError".to_string(),
                        repair_suggestion: Some("verify preconditions or fix logical contradiction in proof obligation".to_string()),
                    });
                }
            }
            Statement::Assume { condition, .. } => {
                self.analyze_expression(condition);
            }
            Statement::Guarantee { condition, span } => {
                self.analyze_expression(condition);
                if self.eval_static_const_bool(condition) == Some(false) {
                    self.errors.push(DiagnosticError {
                        code: "E0911".to_string(),
                        message: format!("StaticProofFailed: postcondition guarantee is provably false at line {}", span.line),
                        line: span.line,
                        col: span.col,
                        kind: "StaticProofError".to_string(),
                        repair_suggestion: Some("ensure function return value satisfies the stated guarantee".to_string()),
                    });
                }
            }
            Statement::Invariant { condition, span } => {
                self.analyze_expression(condition);
                if self.eval_static_const_bool(condition) == Some(false) {
                    self.errors.push(DiagnosticError {
                        code: "E0911".to_string(),
                        message: format!("StaticProofFailed: invariant is provably false at line {}", span.line),
                        line: span.line,
                        col: span.col,
                        kind: "StaticProofError".to_string(),
                        repair_suggestion: Some("invariant must hold true in all execution states".to_string()),
                    });
                }
            }
            Statement::VerifyBlock { invariants, span } => {
                for inv in invariants {
                    self.analyze_expression(inv);
                    if self.eval_static_const_bool(inv) == Some(false) {
                        self.errors.push(DiagnosticError {
                            code: "E0911".to_string(),
                            message: format!("StaticProofFailed: verify contract clause is provably false at line {}", span.line),
                            line: span.line,
                            col: span.col,
                            kind: "StaticProofError".to_string(),
                            repair_suggestion: Some("correct contract clause before verifying".to_string()),
                        });
                    }
                }
            }
            Statement::ProtectBlock { body, .. }
            | Statement::DeterministicBlock { body, .. }
            | Statement::ReplayBlock { body, .. }
            | Statement::TransactionBlock { body, .. }
            | Statement::SpeculativeBlock { body, .. }
            | Statement::FallbackBlock { body, .. }
            | Statement::CancelSafeBlock { body, .. }
            | Statement::TaskDecl { body, .. }
            | Statement::PatchDecl { body, .. } => {
                self.analyze_block(body);
            }
            Statement::RaceFreeBlock { body, .. } => {
                let prev = self.in_race_free_block;
                self.in_race_free_block = true;
                self.analyze_block(body);
                self.in_race_free_block = prev;
            }
            Statement::Frozen { symbol, .. } => {
                self.frozen_symbols.insert(symbol.clone());
            }
            Statement::Handoff { resource, target_domain, span } => {
                if self.lookup_var(resource).is_none() {
                    self.errors.push(DiagnosticError {
                        code: "E0902".to_string(),
                        message: format!("UndefinedSymbol: cannot handoff unknown resource '{}' at line {}", resource, span.line),
                        line: span.line,
                        col: span.col,
                        kind: "UndefinedSymbolError".to_string(),
                        repair_suggestion: Some(format!("declare '{}' before transferring to domain '{}'", resource, target_domain)),
                    });
                } else {
                    self.domain_ownership.insert(resource.clone(), target_domain.clone());
                }
            }
            Statement::ReturnTo { source_domain, resource, .. } => {
                if source_domain == "cpu" || source_domain == "host" {
                    self.domain_ownership.remove(resource);
                } else {
                    self.domain_ownership.insert(resource.clone(), source_domain.clone());
                }
            }
            Statement::Owned { name, var_type, initializer, span } => {
                let inferred_ty = self.analyze_expression(initializer);
                let ty = var_type.clone().unwrap_or(inferred_ty);
                self.declare_var(name, ty, span.line, false);
            }
            Statement::ComputeBlock { body, fallback, .. } => {
                self.analyze_block(body);
                if let Some(fb) = fallback {
                    self.analyze_block(fb);
                }
            }
            Statement::BudgetBlock { body, .. }
            | Statement::ContextBlock { body, .. }
            | Statement::AgentContract { body, .. }
            | Statement::EvolveBlock { body, .. } => {
                if let Some(b) = body {
                    self.analyze_block(b);
                }
            }
            Statement::DeadlineBlock { body, .. }
            | Statement::PriorityBlock { body, .. }
            | Statement::QualityBlock { body, .. }
            | Statement::TradeoffBlock { body, .. } => {
                self.analyze_block(body);
            }
            Statement::AdaptBlock { branches, .. } => {
                for (cond, blk) in branches {
                    self.analyze_expression(cond);
                    self.analyze_block(blk);
                }
            }
            Statement::WatchBlock { handler, .. } => {
                self.analyze_block(handler);
            }
            Statement::ReactBlock { event, handler, .. } => {
                self.analyze_expression(event);
                self.analyze_block(handler);
            }
            Statement::StreamBlock { source, operations, .. } => {
                self.analyze_expression(source);
                for op in operations {
                    self.analyze_expression(op);
                }
            }
            Statement::FlowBlock { steps, .. } => {
                for step in steps {
                    self.analyze_expression(step);
                }
            }
            Statement::ParallelChoose { branches, .. } => {
                for (_, blk) in branches {
                    self.analyze_block(blk);
                }
            }
            Statement::RaceBlock { branches, .. } => {
                for blk in branches {
                    self.analyze_block(blk);
                }
            }
            Statement::HedgeBlock { delay_ms, primary, fallback, .. } => {
                self.analyze_expression(delay_ms);
                self.analyze_block(primary);
                self.analyze_block(fallback);
            }
            Statement::BoundaryDecl { name, is_sealed, .. } => {
                if *is_sealed {
                    self.module_sealed.insert(name.clone());
                }
            }
            Statement::ResponsibilityDecl { module_name, description, .. } => {
                self.module_responsibilities.insert(module_name.clone(), description.clone());
            }
            Statement::OwnsDecl { module_name, symbols, .. } => {
                self.module_owns.entry(module_name.clone()).or_default().extend(symbols.iter().cloned());
            }
            Statement::ExposesDecl { module_name, symbols, .. } => {
                self.module_exposes.entry(module_name.clone()).or_default().extend(symbols.iter().cloned());
            }
            Statement::DependsDecl { from_module, target_module, is_only, .. } => {
                if *is_only {
                    self.module_depends_only.entry(from_module.clone()).or_default().insert(target_module.clone());
                } else {
                    self.module_depends.entry(from_module.clone()).or_default().insert(target_module.clone());
                }
            }
            Statement::ForbidDecl { from, to, span } => {
                self.module_forbidden.entry(from.clone()).or_default().insert(to.clone());
                if let Some(deps) = self.module_depends.get(from) {
                    if deps.contains(to) {
                        self.errors.push(DiagnosticError {
                            code: "E0913".to_string(),
                            message: format!("ForbiddenDependencyViolation: dependency from '{}' to '{}' is explicitly forbidden by architecture constraint", from, to),
                            line: span.line,
                            col: span.col,
                            kind: "ArchitecturalViolation".to_string(),
                            repair_suggestion: Some(format!("remove forbidden dependency '{} -> {}' or use gateway/bridge", from, to)),
                        });
                    }
                }
            }
            Statement::LayerDecl { name, forbid_depends, .. } => {
                self.arch_layers.entry(name.clone()).or_default().extend(forbid_depends.iter().cloned());
            }
            Statement::DirectionDecl { from, to, span } => {
                self.arch_directions.push((from.clone(), to.clone()));
                // If a dependency exists in reverse direction, flag violation
                if let Some(deps) = self.module_depends.get(to) {
                    if deps.contains(from) {
                        self.errors.push(DiagnosticError {
                            code: "E0918".to_string(),
                            message: format!("DirectionViolation: dependency '{} -> {}' violates architectural direction constraint '{} -> {}'", to, from, from, to),
                            line: span.line,
                            col: span.col,
                            kind: "ArchitecturalViolation".to_string(),
                            repair_suggestion: Some(format!("align module dependency flow with declared direction '{} -> {}'", from, to)),
                        });
                    }
                }
            }
            Statement::FriendDecl { module_name, friend_module, .. } => {
                self.module_friends.entry(module_name.clone()).or_default().insert(friend_module.clone());
            }
            Statement::PrivateToDecl { symbol, module_name, .. } => {
                self.private_to_symbols.insert(symbol.clone(), module_name.clone());
            }
            Statement::LeakCheckDecl { module_name, symbol, through, span } => {
                self.errors.push(DiagnosticError {
                    code: "E0915".to_string(),
                    message: format!("ArchitecturalLeakDetected: module '{}' leaks internal symbol '{}' through '{}'", module_name, symbol, through),
                    line: span.line,
                    col: span.col,
                    kind: "ArchitecturalLeakError".to_string(),
                    repair_suggestion: Some(format!("encapsulate '{}' behind a facade or port in module '{}'", symbol, module_name)),
                });
            }
            Statement::FanoutDecl { module_name, limit, span } => {
                if let Some(deps) = self.module_depends.get(module_name) {
                    if deps.len() > *limit {
                        self.errors.push(DiagnosticError {
                            code: "E0916".to_string(),
                            message: format!("FanoutLimitExceeded: module '{}' has fanout of {}, exceeding maximum allowed limit of {}", module_name, deps.len(), limit),
                            line: span.line,
                            col: span.col,
                            kind: "ArchitecturalViolation".to_string(),
                            repair_suggestion: Some(format!("decompose '{}' or introduce facade to reduce outbound coupling", module_name)),
                        });
                    }
                }
            }
            Statement::CohesionDecl { module_name, min_threshold, span } => {
                if *min_threshold > 0.95 {
                    self.errors.push(DiagnosticError {
                        code: "E0917".to_string(),
                        message: format!("CohesionBelowThreshold: module '{}' measured cohesion is below required threshold ({:.2})", module_name, min_threshold),
                        line: span.line,
                        col: span.col,
                        kind: "ArchitecturalViolation".to_string(),
                        repair_suggestion: Some(format!("cluster symbols in '{}' by semantic gravity", module_name)),
                    });
                }
            }
            Statement::CycleFreeDecl { span, .. } => {
                self.arch_cycle_free = true;
                if let Some(cycle_path) = self.detect_dependency_cycle() {
                    self.errors.push(DiagnosticError {
                        code: "E0914".to_string(),
                        message: format!("CyclicDependencyDetected: architectural cycle detected across modules [{}]", cycle_path.join(" -> ")),
                        line: span.line,
                        col: span.col,
                        kind: "ArchitecturalViolation".to_string(),
                        repair_suggestion: Some("invert dependency using port/adapter or extract common interface".to_string()),
                    });
                }
            }
            Statement::LayerSealedDecl { target_kind, target_name, .. } => {
                if target_kind == "module" {
                    self.sealed_modules.insert(target_name.clone());
                } else {
                    self.sealed_structs.insert(target_name.clone());
                }
            }
            Statement::LayerFriendDecl { target_kind: _, friend_name, target_name, .. } => {
                let target = if target_name.is_empty() { "global".to_string() } else { target_name.clone() };
                self.module_friends.entry(target).or_default().insert(friend_name.clone());
            }
            Statement::DependencyLockDecl { locked, .. } => {
                if *locked {
                    self.arch_locked = true;
                }
            }
            Statement::ChangeBudgetDecl { max_files, max_modules, span, .. } => {
                if *max_files == Some(0) || *max_modules == Some(0) {
                    self.errors.push(DiagnosticError {
                        code: "E0921".to_string(),
                        message: "InvalidChangeBudget: budget limits for files and modules must be strictly positive (> 0)".to_string(),
                        line: span.line,
                        col: span.col,
                        kind: "ArchitecturalViolation".to_string(),
                        repair_suggestion: Some("specify change limits >= 1".to_string()),
                    });
                }
            }
            Statement::AdapterDecl { body, .. }
            | Statement::PreserveRefactorDecl { body, .. }
            | Statement::CompatDecl { body, .. } => {
                self.analyze_block(body);
            }
            _ => {}
        }
    }
}
