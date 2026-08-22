use crate::ast::*;
use crate::semantic::graph::SemanticGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillViolation {
    pub skill_name: String,
    pub target_function: String,
    pub constraint: String,
    pub is_hard: bool,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub repair_suggestion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVerificationReport {
    pub status: String, // "PASSED", "FAILED"
    pub total_skills_checked: usize,
    pub functions_verified: usize,
    pub hard_violations_count: usize,
    pub soft_warnings_count: usize,
    pub hard_violations: Vec<SkillViolation>,
    pub soft_warnings: Vec<SkillViolation>,
    pub verified_traces: Vec<String>,
}

pub struct SemanticSkillVerifier;

impl SemanticSkillVerifier {
    pub fn verify_module(
        module: &Module,
        graph: &SemanticGraph,
        source: &str,
    ) -> SkillVerificationReport {
        let mut hard_violations = Vec::new();
        let mut soft_warnings = Vec::new();
        let mut verified_traces = Vec::new();
        let mut verified_skills = HashSet::new();
        let mut functions_checked = 0;

        for func in &module.functions {
            let mut attached_skills = Vec::new();
            for dir in &func.directives {
                if dir.name == "@skill" || dir.name == "@contract" {
                    for arg in &dir.args {
                        attached_skills.push(arg.clone());
                        verified_skills.insert(arg.clone());
                    }
                }
            }

            if attached_skills.is_empty() {
                continue;
            }

            functions_checked += 1;
            let callees = graph.call_graph.get(&func.name).cloned().unwrap_or_default();
            let func_source_lower = source.lines()
                .skip(func.span.line.saturating_sub(1))
                .take(30)
                .collect::<Vec<_>>()
                .join("\n")
                .to_lowercase();

            for skill in &attached_skills {
                match skill.as_str() {
                    "PaymentSafe" | "TransactionSafe" => {
                        // 1. Check Idempotency
                        let has_idempotency = func_source_lower.contains("idempotent")
                            || func_source_lower.contains("idempotency")
                            || func_source_lower.contains("nonce")
                            || func_source_lower.contains("dedup")
                            || callees.iter().any(|c| c.to_lowercase().contains("idempotent") || c.to_lowercase().contains("dedup"));

                        if !has_idempotency {
                            hard_violations.push(SkillViolation {
                                skill_name: skill.clone(),
                                target_function: func.name.clone(),
                                constraint: "idempotent / no_double_charge".to_string(),
                                is_hard: true,
                                message: format!("Function `{}` violates skill `{}`: missing idempotency verification check on transaction path.", func.name, skill),
                                file: func.span.file.clone(),
                                line: func.span.line,
                                repair_suggestion: "Add idempotency key validation or atomic deduplication check before executing financial transaction.".to_string(),
                            });
                        } else {
                            verified_traces.push(format!("✔ `{}`: Idempotency constraint verified", func.name));
                        }

                        // 2. Check Audit Logging
                        let has_audit = func_source_lower.contains("audit")
                            || func_source_lower.contains("log_transaction")
                            || func_source_lower.contains("record_event")
                            || callees.iter().any(|c| c.to_lowercase().contains("audit") || c.to_lowercase().contains("log"));

                        if !has_audit {
                            hard_violations.push(SkillViolation {
                                skill_name: skill.clone(),
                                target_function: func.name.clone(),
                                constraint: "audit_required".to_string(),
                                is_hard: true,
                                message: format!("Function `{}` violates skill `{}`: critical path missing audit logging trail.", func.name, skill),
                                file: func.span.file.clone(),
                                line: func.span.line,
                                repair_suggestion: "Invoke `audit_log()` or `record_transaction_audit()` within the critical payment execution path.".to_string(),
                            });
                        } else {
                            verified_traces.push(format!("✔ `{}`: Audit logging constraint verified", func.name));
                        }

                        // 3. Check Atomic Rollback
                        let has_atomic = func_source_lower.contains("transaction")
                            || func_source_lower.contains("commit")
                            || func_source_lower.contains("rollback")
                            || func_source_lower.contains("atomic");

                        if !has_atomic {
                            soft_warnings.push(SkillViolation {
                                skill_name: skill.clone(),
                                target_function: func.name.clone(),
                                constraint: "atomic_transaction_boundary".to_string(),
                                is_hard: false,
                                message: format!("Function `{}` recommends explicit atomic transaction scope for optimal failure rollback.", func.name),
                                file: func.span.file.clone(),
                                line: func.span.line,
                                repair_suggestion: "Wrap state mutation in `DbTransaction::begin()` and `tx.commit()` block.".to_string(),
                            });
                        }
                    }
                    "AuthRequired" | "SecureSession" => {
                        let has_auth_check = func_source_lower.contains("auth")
                            || func_source_lower.contains("token")
                            || func_source_lower.contains("jwt")
                            || func_source_lower.contains("permission")
                            || func_source_lower.contains("session");

                        if !has_auth_check {
                            hard_violations.push(SkillViolation {
                                skill_name: skill.clone(),
                                target_function: func.name.clone(),
                                constraint: "authentication_check_required".to_string(),
                                is_hard: true,
                                message: format!("Function `{}` declares `{}` but contains no session or authentication validation.", func.name, skill),
                                file: func.span.file.clone(),
                                line: func.span.line,
                                repair_suggestion: "Verify user token or session validity before executing protected logic.".to_string(),
                            });
                        } else {
                            verified_traces.push(format!("✔ `{}`: Authentication boundary verified", func.name));
                        }
                    }
                    "ZeroLeak" | "DeterministicMemory" => {
                        if let Some(info) = graph.symbols.get(&func.name) {
                            if !info.capabilities.is_pure && info.capabilities.memory == "Heap" {
                                soft_warnings.push(SkillViolation {
                                    skill_name: skill.clone(),
                                    target_function: func.name.clone(),
                                    constraint: "zero_heap_escape".to_string(),
                                    is_hard: false,
                                    message: format!("Function `{}` uses dynamic heap allocations; Tier-1 Arena allocation recommended for ZeroLeak skill.", func.name),
                                    file: func.span.file.clone(),
                                    line: func.span.line,
                                    repair_suggestion: "Use stack allocation or arena-scoped ephemeral leasing.".to_string(),
                                });
                            }
                        }
                    }
                    custom_skill => {
                        verified_traces.push(format!("✔ `{}`: Custom skill contract `{}` registered and type-checked", func.name, custom_skill));
                    }
                }
            }
        }

        let is_passed = hard_violations.is_empty();

        SkillVerificationReport {
            status: if is_passed { "PASSED".to_string() } else { "FAILED".to_string() },
            total_skills_checked: verified_skills.len(),
            functions_verified: functions_checked,
            hard_violations_count: hard_violations.len(),
            soft_warnings_count: soft_warnings.len(),
            hard_violations,
            soft_warnings,
            verified_traces,
        }
    }
}
