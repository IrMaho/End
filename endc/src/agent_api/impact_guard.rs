use crate::ast::*;
use crate::semantic::graph::SemanticGraph;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreTouchImpactReport {
    pub target_symbol: String,
    pub risk_level: String, // "LOW", "MEDIUM", "HIGH", "CRITICAL"
    pub blast_radius_score: usize,
    pub direct_callers_count: usize,
    pub transitive_callers_count: usize,
    pub direct_callers: Vec<String>,
    pub transitive_hierarchy: Vec<String>,
    pub database_flows: Vec<String>,
    pub network_boundaries: Vec<String>,
    pub impacted_test_suites: Vec<String>,
    pub required_skills: Vec<String>,
    pub capabilities_affected: Vec<String>,
    pub critical_paths: Vec<String>,
    pub can_proceed_safely: bool,
    pub blocking_reasons: Vec<String>,
}

pub struct ImpactGuard;

impl ImpactGuard {
    pub fn analyze(
        target_symbol: &str,
        module: &Module,
        graph: &SemanticGraph,
    ) -> PreTouchImpactReport {
        let mut direct_callers = Vec::new();
        let mut transitive_callers = HashSet::new();
        let mut hierarchy = Vec::new();

        // 1. Direct & Transitive Callers via BFS
        let mut queue = VecDeque::new();
        if let Some(callers) = graph.reverse_call_graph.get(target_symbol) {
            for caller in callers {
                direct_callers.push(caller.clone());
                transitive_callers.insert(caller.clone());
                queue.push_back((caller.clone(), 1usize));
                hierarchy.push(format!("{} (Level 1) -> calls -> {}", caller, target_symbol));
            }
        }

        while let Some((curr, level)) = queue.pop_front() {
            if level >= 5 {
                continue; // Depth cap
            }
            if let Some(callers) = graph.reverse_call_graph.get(&curr) {
                for caller in callers {
                    if transitive_callers.insert(caller.clone()) {
                        queue.push_back((caller.clone(), level + 1));
                        hierarchy.push(format!("{} (Level {}) -> calls -> {}", caller, level + 1, curr));
                    }
                }
            }
        }

        // 2. Discover Database Flows & Network Boundaries in the blast radius
        let mut db_flows = Vec::new();
        let mut net_boundaries = Vec::new();
        let mut capabilities_affected = HashSet::new();
        let mut critical_paths = Vec::new();

        let mut all_affected = transitive_callers.clone();
        all_affected.insert(target_symbol.to_string());

        // Also add direct callees of target and affected symbols
        let mut callees_to_add = Vec::new();
        for sym in &all_affected {
            if let Some(callees) = graph.call_graph.get(sym) {
                for callee in callees {
                    callees_to_add.push(callee.clone());
                }
            }
        }
        for callee in callees_to_add {
            all_affected.insert(callee);
        }

        for sym_name in &all_affected {
            if let Some(info) = graph.symbols.get(sym_name) {
                if info.capabilities.disk || sym_name.to_lowercase().contains("db") || sym_name.to_lowercase().contains("repo") || sym_name.to_lowercase().contains("store") || sym_name.to_lowercase().contains("transaction") {
                    db_flows.push(format!("{}: Database Data Flow (disk=true)", sym_name));
                    capabilities_affected.insert("disk_io".to_string());
                }
                if info.capabilities.net || sym_name.to_lowercase().contains("api") || sym_name.to_lowercase().contains("client") || sym_name.to_lowercase().contains("http") || sym_name.to_lowercase().contains("net") || sym_name.to_lowercase().contains("stripe") {
                    net_boundaries.push(format!("{}: External Network Boundary (net=true)", sym_name));
                    capabilities_affected.insert("network".to_string());
                }
                if sym_name.to_lowercase().contains("pay") || sym_name.to_lowercase().contains("auth") || sym_name.to_lowercase().contains("security") || sym_name.to_lowercase().contains("token") || sym_name.to_lowercase().contains("money") {
                    critical_paths.push(format!("Critical Path: `{}` touches sensitive business domain", sym_name));
                }
            }
        }

        // 3. Impacted Test Suites
        let mut impacted_tests = Vec::new();
        for func in &module.functions {
            let is_test = func.directives.iter().any(|d| d.name == "@test" || d.name == "@scenario" || d.name == "@bench") || func.name.starts_with("test_");
            if is_test {
                // Check if this test calls any of the affected symbols
                let calls_affected = all_affected.iter().any(|aff| {
                    graph.call_graph.get(&func.name).map(|callees| callees.contains(aff)).unwrap_or(false)
                });
                if calls_affected {
                    impacted_tests.push(func.name.clone());
                }
            }
        }

        // 4. Required Skills
        let mut required_skills = Vec::new();
        for func in &module.functions {
            if all_affected.contains(&func.name) {
                for d in &func.directives {
                    if d.name == "@skill" || d.name == "@contract" {
                        required_skills.extend(d.args.clone());
                    }
                }
            }
        }
        required_skills.sort();
        required_skills.dedup();

        // 5. Blast Radius & Risk Score Calculation
        let blast_score = direct_callers.len() * 2 + transitive_callers.len() + db_flows.len() * 3 + net_boundaries.len() * 3 + critical_paths.len() * 5;

        let risk_level = if blast_score == 0 {
            "LOW"
        } else if blast_score <= 10 {
            "MEDIUM"
        } else if blast_score <= 30 {
            "HIGH"
        } else {
            "CRITICAL"
        };

        // 6. Pre-Touch Safety Gate
        let mut blocking_reasons = Vec::new();
        let target_info = graph.symbols.get(target_symbol);

        if let Some(info) = target_info {
            if info.capabilities.is_pure && (db_flows.contains(&target_symbol.to_string()) || net_boundaries.contains(&target_symbol.to_string())) {
                blocking_reasons.push(format!("Symbol '{}' is declared pure but attempts side-effecting operations.", target_symbol));
            }
        }

        let can_proceed = blocking_reasons.is_empty();

        PreTouchImpactReport {
            target_symbol: target_symbol.to_string(),
            risk_level: risk_level.to_string(),
            blast_radius_score: blast_score,
            direct_callers_count: direct_callers.len(),
            transitive_callers_count: transitive_callers.len(),
            direct_callers,
            transitive_hierarchy: hierarchy,
            database_flows: db_flows,
            network_boundaries: net_boundaries,
            impacted_test_suites: impacted_tests,
            required_skills,
            capabilities_affected: capabilities_affected.into_iter().collect(),
            critical_paths,
            can_proceed_safely: can_proceed,
            blocking_reasons,
        }
    }
}
