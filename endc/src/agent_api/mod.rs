use crate::semantic::graph::SemanticGraph;
use serde_json::json;

pub struct AgentApi<'a> {
    graph: &'a SemanticGraph,
}

impl<'a> AgentApi<'a> {
    pub fn new(graph: &'a SemanticGraph) -> Self {
        Self { graph }
    }

    pub fn inspect_line(&self, line: usize) -> serde_json::Value {
        if let Some(line_sem) = self.graph.inspect_line(line) {
            json!({
                "status": "success",
                "file": self.graph.filename,
                "line": line_sem.line,
                "code": line_sem.code,
                "flow": {
                    "from": line_sem.flow.from,
                    "to": line_sem.flow.to
                },
                "side_effects": {
                    "memory_allocated": line_sem.side_effects.memory_allocated,
                    "allocator_used": line_sem.side_effects.allocator_used,
                    "io_performed": line_sem.side_effects.io_performed,
                    "can_panic": line_sem.side_effects.can_panic,
                    "possible_errors": line_sem.side_effects.possible_errors,
                    "effects": line_sem.side_effects.effects
                }
            })
        } else {
            json!({
                "status": "not_found",
                "file": self.graph.filename,
                "line": line,
                "message": format!("No specific semantic facts recorded for line {}", line)
            })
        }
    }

    pub fn impact_analysis(&self, symbol: &str) -> serde_json::Value {
        let report = self.graph.impact_analysis(symbol);
        json!({
            "status": "success",
            "report": report
        })
    }

    pub fn query_symbol(&self, symbol_name: &str) -> serde_json::Value {
        if let Some(info) = self.graph.get_symbol(symbol_name) {
            let callers = self.graph.reverse_call_graph.get(symbol_name).cloned().unwrap_or_default();
            let callees = self.graph.call_graph.get(symbol_name).cloned().unwrap_or_default();

            json!({
                "status": "success",
                "symbol": {
                    "name": info.name,
                    "kind": info.kind,
                    "signature": info.type_signature,
                    "file": info.file,
                    "line": info.defined_at_line,
                    "is_pure": info.is_pure,
                    "effects": info.effects,
                    "callers": callers.into_iter().collect::<Vec<_>>(),
                    "callees": callees.into_iter().collect::<Vec<_>>()
                }
            })
        } else {
            json!({
                "status": "not_found",
                "symbol": symbol_name,
                "message": format!("Symbol '{}' not found in semantic database", symbol_name)
            })
        }
    }
}
