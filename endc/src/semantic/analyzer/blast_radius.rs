use super::SemanticAnalyzer;
use crate::ast::BlastRadiusReport;
use std::collections::HashSet;

impl SemanticAnalyzer {
    pub fn detect_dependency_cycle(&self) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in self.module_depends.keys() {
            if !visited.contains(node) {
                if self.dfs_cycle(node, &mut visited, &mut rec_stack, &mut path) {
                    return Some(path);
                }
            }
        }
        None
    }

    pub(crate) fn dfs_cycle(&self, node: &str, visited: &mut HashSet<String>, rec_stack: &mut HashSet<String>, path: &mut Vec<String>) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.module_depends.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.dfs_cycle(neighbor, visited, rec_stack, path) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    path.push(neighbor.to_string());
                    return true;
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
        false
    }

    pub fn calculate_blast_radius(&self, target_symbol: &str) -> BlastRadiusReport {
        let mut affected_modules = Vec::new();
        let mut affected_features = Vec::new();
        let mut affected_symbols = Vec::new();
        let mut affected_public_apis = Vec::new();
        let mut required_migrations = Vec::new();

        // 1. Direct dependencies: any module or feature depending on target_symbol
        for (caller, deps) in &self.module_depends {
            if deps.contains(target_symbol) && caller != target_symbol {
                if self.features.contains_key(caller) {
                    if !affected_features.contains(caller) {
                        affected_features.push(caller.clone());
                    }
                } else if !affected_modules.contains(caller) {
                    affected_modules.push(caller.clone());
                }
            }
        }
        for (feat_name, feat) in &self.features {
            for dep in &feat.needs {
                if dep.name == target_symbol && feat_name != target_symbol && !affected_features.contains(feat_name) {
                    affected_features.push(feat_name.clone());
                }
            }
        }

        // 2. Transitive dependencies
        let mut queue: Vec<String> = affected_modules.iter().chain(affected_features.iter()).cloned().collect();
        let mut visited: HashSet<String> = queue.iter().cloned().collect();
        visited.insert(target_symbol.to_string());

        while let Some(current) = queue.pop() {
            for (caller, deps) in &self.module_depends {
                if deps.contains(&current) && !visited.contains(caller) {
                    visited.insert(caller.clone());
                    if self.features.contains_key(caller) {
                        if !affected_features.contains(caller) {
                            affected_features.push(caller.clone());
                        }
                    } else if !affected_modules.contains(caller) {
                        affected_modules.push(caller.clone());
                    }
                    queue.push(caller.clone());
                }
            }
            for (feat_name, feat) in &self.features {
                for dep in &feat.needs {
                    if dep.name == current && !visited.contains(feat_name) {
                        visited.insert(feat_name.clone());
                        if !affected_features.contains(feat_name) {
                            affected_features.push(feat_name.clone());
                        }
                        queue.push(feat_name.clone());
                    }
                }
            }
        }

        // 3. Affected symbols & public APIs
        if let Some(feat) = self.features.get(target_symbol) {
            if let Some(ref api) = feat.api {
                for f in &api.functions {
                    affected_public_apis.push(f.name.clone());
                }
            }
            if let Some(ref lc) = feat.lifecycle {
                if let Some(ref mig) = lc.migration_path {
                    required_migrations.push(mig.clone());
                }
            }
        }

        affected_symbols.extend(affected_modules.clone());
        affected_symbols.extend(affected_features.clone());

        BlastRadiusReport {
            target_symbol: target_symbol.to_string(),
            affected_features,
            affected_modules,
            affected_symbols,
            affected_public_apis,
            required_migrations,
        }
    }
}
