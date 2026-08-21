use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSource {
    pub symbol: String,
    #[serde(rename = "type")]
    pub symbol_type: String,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolDestination {
    pub symbol: String,
    #[serde(rename = "type")]
    pub symbol_type: String,
    pub lifetime: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlow {
    pub from: Vec<SymbolSource>,
    pub to: Vec<SymbolDestination>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffects {
    pub memory_allocated: bool,
    pub allocator_used: Option<String>,
    pub io_performed: bool,
    pub can_panic: bool,
    pub possible_errors: Vec<String>,
    pub effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineSemantics {
    pub line: usize,
    pub code: String,
    pub flow: DataFlow,
    pub side_effects: SideEffects,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String, // "function", "struct", "variable"
    pub type_signature: String,
    pub file: String,
    pub defined_at_line: usize,
    pub callers: Vec<String>,
    pub callees: Vec<String>,
    pub effects: Vec<String>,
    pub is_pure: bool,
    pub memory_region: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub target_symbol: String,
    pub directly_affected_functions: Vec<String>,
    pub call_hierarchy: Vec<String>,
    pub estimated_risk: String, // "LOW", "MEDIUM", "HIGH"
    pub requires_recompilation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemanticGraph {
    pub filename: String,
    pub lines: HashMap<usize, LineSemantics>,
    pub symbols: HashMap<String, SymbolInfo>,
    pub call_graph: HashMap<String, HashSet<String>>, // caller -> set of callees
    pub reverse_call_graph: HashMap<String, HashSet<String>>, // callee -> set of callers
}

impl SemanticGraph {
    pub fn new(filename: impl Into<String>) -> Self {
        Self {
            filename: filename.into(),
            lines: HashMap::new(),
            symbols: HashMap::new(),
            call_graph: HashMap::new(),
            reverse_call_graph: HashMap::new(),
        }
    }

    pub fn add_line(&mut self, line: usize, semantics: LineSemantics) {
        self.lines.insert(line, semantics);
    }

    pub fn inspect_line(&self, line: usize) -> Option<&LineSemantics> {
        self.lines.get(&line)
    }

    pub fn get_symbol(&self, name: &str) -> Option<&SymbolInfo> {
        self.symbols.get(name)
    }

    pub fn add_call(&mut self, caller: &str, callee: &str) {
        self.call_graph
            .entry(caller.to_string())
            .or_default()
            .insert(callee.to_string());

        self.reverse_call_graph
            .entry(callee.to_string())
            .or_default()
            .insert(caller.to_string());
    }

    pub fn impact_analysis(&self, target_symbol: &str) -> ImpactReport {
        let mut affected = Vec::new();
        let mut hierarchy = Vec::new();

        if let Some(callers) = self.reverse_call_graph.get(target_symbol) {
            for caller in callers {
                affected.push(caller.clone());
                hierarchy.push(format!("{} -> calls -> {}", caller, target_symbol));
            }
        }

        let risk = if affected.is_empty() {
            "LOW"
        } else if affected.len() <= 3 {
            "MEDIUM"
        } else {
            "HIGH"
        };

        ImpactReport {
            target_symbol: target_symbol.to_string(),
            directly_affected_functions: affected.clone(),
            call_hierarchy: hierarchy,
            estimated_risk: risk.to_string(),
            requires_recompilation: if affected.is_empty() {
                vec![self.filename.clone()]
            } else {
                affected
            },
        }
    }
}
