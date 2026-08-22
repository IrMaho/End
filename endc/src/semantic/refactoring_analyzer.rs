use std::collections::{HashMap, HashSet};
use crate::ast::decl::refactoring_engine::*;

#[derive(Debug, Clone, Default)]
pub struct RefactoringAuditReport {
    pub is_valid: bool,
    pub original_symbols_count: usize,
    pub accounted_symbols_count: usize,
    pub unaccounted_symbols_count: usize,
    pub max_submodule_loc: usize,
    pub line_limit_violations: Vec<String>,
    pub circular_dependencies: Vec<String>,
    pub solid_violations: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct RefactoringAnalyzer {
    pub sessions: HashMap<String, RefactorSessionDef>,
    pub plans: HashMap<String, DecompositionPlanDef>,
    pub audits: Vec<ConservationAuditDef>,
    pub solid_audits: Vec<SolidAuditDef>,
    pub inventories: HashMap<String, SymbolInventoryDef>,
    pub trace_maps: HashMap<String, TraceableMapDef>,
}

impl RefactoringAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_session(&mut self, def: &RefactorSessionDef) {
        self.sessions.insert(def.agent_name.clone(), def.clone());
    }

    pub fn register_plan(&mut self, def: &DecompositionPlanDef) {
        self.plans.insert(def.source.clone(), def.clone());
    }

    pub fn register_conservation_audit(&mut self, def: &ConservationAuditDef) {
        self.audits.push(def.clone());
    }

    pub fn register_solid_audit(&mut self, def: &SolidAuditDef) {
        self.solid_audits.push(def.clone());
    }

    pub fn register_inventory(&mut self, def: &SymbolInventoryDef) {
        self.inventories.insert(def.module_name.clone(), def.clone());
    }

    pub fn register_traceable_map(&mut self, def: &TraceableMapDef) {
        self.trace_maps.insert(def.source_module.clone(), def.clone());
    }

    pub fn run_full_audit(&self) -> RefactoringAuditReport {
        let mut report = RefactoringAuditReport {
            is_valid: true,
            original_symbols_count: 0,
            accounted_symbols_count: 0,
            unaccounted_symbols_count: 0,
            max_submodule_loc: 0,
            line_limit_violations: vec![],
            circular_dependencies: vec![],
            solid_violations: vec![],
            warnings: vec![],
        };

        // 1. Audit Conservation & Unaccounted Symbols
        for audit in &self.audits {
            report.original_symbols_count += audit.original_symbols.len();
            report.accounted_symbols_count += audit.accounted_symbols.len();
            report.unaccounted_symbols_count += audit.unaccounted_count;

            if audit.unaccounted_count > 0 && !audit.allow_semantic_deletion {
                report.is_valid = false;
                report.warnings.push(format!(
                    "Lossless Violation in '{}': {} symbols unaccounted for without approved deletion.",
                    audit.original_source, audit.unaccounted_count
                ));
            }
        }

        // 2. Audit Submodule Line Limits (Hard constraint: <= 500 LOC)
        for (src, plan) in &self.plans {
            for sub in &plan.submodules {
                if sub.max_loc > report.max_submodule_loc {
                    report.max_submodule_loc = sub.max_loc;
                }
                if sub.max_loc > 500 {
                    report.is_valid = false;
                    report.line_limit_violations.push(format!(
                        "Submodule '{}' in plan for '{}' specifies max_loc={} which exceeds hard limit 500.",
                        sub.name, src, sub.max_loc
                    ));
                }
            }
        }

        // 3. Audit SOLID Principles
        for solid in &self.solid_audits {
            if solid.verify_srp && solid.max_responsibilities > 1 {
                report.solid_violations.push(format!(
                    "SRP Warning for module '{}': has {} responsibilities (expected <= 1).",
                    solid.module_name, solid.max_responsibilities
                ));
            }
        }

        // 4. Audit Traceable Destination Mappings
        for (src, map) in &self.trace_maps {
            let mut mapped_set = HashSet::new();
            for (sym, dest) in &map.mappings {
                if dest.is_empty() {
                    report.is_valid = false;
                    report.warnings.push(format!(
                        "Symbol '{}' in '{}' has empty destination module.", sym, src
                    ));
                }
                mapped_set.insert(sym);
            }
        }

        report
    }

    pub fn compute_line_differential(&self, original_loc: usize, new_loc: usize) -> (isize, f64) {
        let diff = (new_loc as isize) - (original_loc as isize);
        let pct = if original_loc > 0 {
            (diff as f64) / (original_loc as f64) * 100.0
        } else {
            0.0
        };
        (diff, pct)
    }
}
