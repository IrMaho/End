pub mod blast_radius;
pub mod const_eval;
pub mod expr_analyzer;
pub mod module_analyzer;
pub mod stmt_analyzer;
pub mod stmt_architectural;
pub mod stmt_control_flow;
pub mod types;

pub use types::*;

use crate::ast::*;
use crate::semantic::graph::*;
use std::collections::{HashMap, HashSet};

pub struct SemanticAnalyzer {
    pub graph: SemanticGraph,
    pub source_lines: Vec<String>,
    pub errors: Vec<DiagnosticError>,
    pub enums: HashMap<String, EnumDef>,
    pub structs: HashMap<String, StructDef>,
    pub function_signatures: HashMap<String, (Vec<Type>, Type, bool)>, // params, ret, is_pure
    pub function_effects: HashMap<String, HashSet<String>>,
    pub strict_leaks: bool,
    pub(crate) current_function: Option<String>,
    pub(crate) region_depth: usize,
    pub(crate) region_allocations: Vec<HashSet<String>>, // track pointers allocated inside each region depth
    pub(crate) var_scopes: Vec<HashMap<String, (Type, usize, bool)>>, // name -> (Type, line_def, is_mut)
    pub(crate) ownership_scopes: Vec<HashMap<String, OwnershipState>>,
    pub(crate) active_loans: Vec<ActiveLoan>,
    pub frozen_symbols: HashSet<String>,
    pub domain_ownership: HashMap<String, String>,
    pub in_race_free_block: bool,
    pub module_responsibilities: HashMap<String, String>,
    pub module_owns: HashMap<String, HashSet<String>>,
    pub module_exposes: HashMap<String, HashSet<String>>,
    pub module_depends: HashMap<String, HashSet<String>>,
    pub module_depends_only: HashMap<String, HashSet<String>>,
    pub module_forbidden: HashMap<String, HashSet<String>>,
    pub module_sealed: HashSet<String>,
    pub module_purity: HashMap<String, String>,
    pub module_friends: HashMap<String, HashSet<String>>,
    pub private_to_symbols: HashMap<String, String>,
    pub arch_layers: HashMap<String, HashSet<String>>,
    pub arch_directions: Vec<(String, String)>,
    pub arch_cycle_free: bool,
    pub arch_max_depth: Option<usize>,
    pub sealed_modules: HashSet<String>,
    pub sealed_structs: HashSet<String>,
    pub arch_locked: bool,
    pub security_level: crate::security::SecurityLevel,
    pub features: HashMap<String, FeatureDef>,
    pub contracts: HashMap<String, ContractDef>,
    pub architecture_rules: Vec<ArchitectureRuleDef>,
}

impl SemanticAnalyzer {
    pub fn new(filename: &str, source: &str) -> Self {
        Self {
            graph: SemanticGraph::new(filename),
            source_lines: source.lines().map(|s| s.to_string()).collect(),
            errors: Vec::new(),
            enums: HashMap::new(),
            structs: HashMap::new(),
            function_signatures: HashMap::new(),
            function_effects: HashMap::new(),
            strict_leaks: false,
            current_function: None,
            region_depth: 0,
            region_allocations: vec![HashSet::new()],
            var_scopes: vec![HashMap::new()],
            ownership_scopes: vec![HashMap::new()],
            active_loans: Vec::new(),
            frozen_symbols: HashSet::new(),
            domain_ownership: HashMap::new(),
            in_race_free_block: false,
            module_responsibilities: HashMap::new(),
            module_owns: HashMap::new(),
            module_exposes: HashMap::new(),
            module_depends: HashMap::new(),
            module_depends_only: HashMap::new(),
            module_forbidden: HashMap::new(),
            module_sealed: HashSet::new(),
            module_purity: HashMap::new(),
            module_friends: HashMap::new(),
            private_to_symbols: HashMap::new(),
            arch_layers: HashMap::new(),
            arch_directions: Vec::new(),
            arch_cycle_free: false,
            arch_max_depth: None,
            sealed_modules: HashSet::new(),
            sealed_structs: HashSet::new(),
            arch_locked: false,
            security_level: crate::security::SecurityLevel::Standard,
            features: HashMap::new(),
            contracts: HashMap::new(),
            architecture_rules: Vec::new(),
        }
    }

    pub(crate) fn push_scope(&mut self) {
        self.var_scopes.push(HashMap::new());
        self.ownership_scopes.push(HashMap::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.var_scopes.pop();
        self.ownership_scopes.pop();
    }

    pub(crate) fn declare_var(&mut self, name: &str, ty: Type, line: usize, is_mut: bool) {
        if let Some(scope) = self.var_scopes.last_mut() {
            scope.insert(name.to_string(), (ty, line, is_mut));
        }
        if let Some(o_scope) = self.ownership_scopes.last_mut() {
            o_scope.insert(name.to_string(), OwnershipState::Owned);
        }
    }

    pub(crate) fn lookup_var(&self, name: &str) -> Option<(Type, usize, bool)> {
        for scope in self.var_scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    pub(crate) fn get_ownership_state(&self, name: &str) -> Option<OwnershipState> {
        for scope in self.ownership_scopes.iter().rev() {
            if let Some(state) = scope.get(name) {
                return Some(state.clone());
            }
        }
        None
    }

    pub(crate) fn set_ownership_state(&mut self, name: &str, new_state: OwnershipState) {
        for scope in self.ownership_scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), new_state);
                return;
            }
        }
    }
}
