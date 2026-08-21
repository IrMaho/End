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
pub struct CapabilityContract {
    pub net: bool,
    pub disk: bool,
    pub io: bool,
    pub memory: String, // "ZeroGC-Arena", "ArenaScoped", "Heap", "StackOnly"
    pub is_pure: bool,
    pub can_panic: bool,
    pub concurrency_safe: bool,
}

impl Default for CapabilityContract {
    fn default() -> Self {
        Self {
            net: false,
            disk: false,
            io: false,
            memory: "ArenaScoped".to_string(),
            is_pure: true,
            can_panic: false,
            concurrency_safe: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String, // "function", "struct", "enum", "variable"
    pub type_signature: String,
    pub file: String,
    pub defined_at_line: usize,
    pub callers: Vec<String>,
    pub callees: Vec<String>,
    pub effects: Vec<String>,
    pub is_pure: bool,
    pub memory_region: Option<String>,
    pub capabilities: CapabilityContract,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AffectedFileLocation {
    pub path: String,
    pub lines: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactReport {
    pub target: String,
    pub risk_level: String, // "LOW", "MEDIUM", "HIGH"
    pub directly_affected_functions: Vec<String>,
    pub call_hierarchy: Vec<String>,
    pub affected_files: Vec<AffectedFileLocation>,
    pub breaking_changes: bool,
    pub safe_to_modify: bool,
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
        let mut affected_lines = Vec::new();

        if let Some(callers) = self.reverse_call_graph.get(target_symbol) {
            for caller in callers {
                affected.push(caller.clone());
                hierarchy.push(format!("{} -> calls -> {}", caller, target_symbol));

                if let Some(caller_info) = self.symbols.get(caller) {
                    affected_lines.push(caller_info.defined_at_line);
                }
            }
        }

        // Also check if lines contain the target symbol
        for (line, sem) in &self.lines {
            if sem.code.contains(target_symbol) && !affected_lines.contains(line) {
                affected_lines.push(*line);
            }
        }
        affected_lines.sort();

        let risk = if affected.is_empty() {
            "LOW"
        } else if affected.len() <= 3 {
            "MEDIUM"
        } else {
            "HIGH"
        };

        let breaking_changes = affected.len() > 5;
        let safe_to_modify = risk == "LOW" || risk == "MEDIUM";

        let affected_files = if affected_lines.is_empty() {
            vec![]
        } else {
            vec![AffectedFileLocation {
                path: self.filename.clone(),
                lines: affected_lines,
            }]
        };

        ImpactReport {
            target: target_symbol.to_string(),
            risk_level: risk.to_string(),
            directly_affected_functions: affected.clone(),
            call_hierarchy: hierarchy,
            affected_files,
            breaking_changes,
            safe_to_modify,
            requires_recompilation: if affected.is_empty() {
                vec![self.filename.clone()]
            } else {
                affected
            },
        }
    }

    pub fn generate_knowledge_graph(&self) -> serde_json::Value {
        let mut symbol_list = Vec::new();

        for (name, info) in &self.symbols {
            let callers = self.reverse_call_graph.get(name).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();
            let callees = self.call_graph.get(name).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();

            symbol_list.push(serde_json::json!({
                "symbol": name,
                "kind": info.kind,
                "signature": info.type_signature,
                "file": info.file,
                "line": info.defined_at_line,
                "callers": callers,
                "callees": callees,
                "memory_arena": info.capabilities.memory,
                "purity": if info.capabilities.is_pure { "Pure (No I/O)" } else { "Impure (I/O or State Mutation)" },
                "capabilities": {
                    "net": info.capabilities.net,
                    "disk": info.capabilities.disk,
                    "io": info.capabilities.io,
                    "is_pure": info.capabilities.is_pure,
                    "concurrency_safe": info.capabilities.concurrency_safe
                }
            }));
        }

        serde_json::json!({
            "status": "success",
            "file": self.filename,
            "total_symbols": self.symbols.len(),
            "symbols": symbol_list
        })
    }
}
