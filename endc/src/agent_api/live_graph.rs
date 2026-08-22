use crate::semantic::graph::SemanticGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LiveGraphEvent {
    SymbolAdded {
        symbol: String,
        kind: String,
        signature: String,
    },
    SymbolModified {
        symbol: String,
        old_signature: String,
        new_signature: String,
        is_breaking: bool,
    },
    SymbolDeleted {
        symbol: String,
    },
    CallEdgeAdded {
        caller: String,
        callee: String,
    },
    CallEdgeRemoved {
        caller: String,
        callee: String,
    },
    BreakingChange {
        symbol: String,
        impacted_callers_count: usize,
        reason: String,
    },
    ContractInvalidated {
        contract: String,
        affected_functions: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphDeltaReport {
    pub file: String,
    pub timestamp_epoch_ms: u128,
    pub has_breaking_changes: bool,
    pub events_count: usize,
    pub events: Vec<LiveGraphEvent>,
}

pub struct LiveSemanticGraphEngine;

impl LiveSemanticGraphEngine {
    pub fn compute_delta(
        old_graph: &SemanticGraph,
        new_graph: &SemanticGraph,
    ) -> GraphDeltaReport {
        let mut events = Vec::new();
        let mut has_breaking = false;

        let old_symbols: HashSet<&String> = old_graph.symbols.keys().collect();
        let new_symbols: HashSet<&String> = new_graph.symbols.keys().collect();

        // 1. Added Symbols
        for added in new_symbols.difference(&old_symbols) {
            if let Some(info) = new_graph.symbols.get(*added) {
                events.push(LiveGraphEvent::SymbolAdded {
                    symbol: (*added).clone(),
                    kind: info.kind.clone(),
                    signature: info.type_signature.clone(),
                });
            }
        }

        // 2. Deleted Symbols
        for deleted in old_symbols.difference(&new_symbols) {
            let callers_count = old_graph.reverse_call_graph.get(*deleted).map(|c| c.len()).unwrap_or(0);
            events.push(LiveGraphEvent::SymbolDeleted {
                symbol: (*deleted).clone(),
            });
            if callers_count > 0 {
                has_breaking = true;
                events.push(LiveGraphEvent::BreakingChange {
                    symbol: (*deleted).clone(),
                    impacted_callers_count: callers_count,
                    reason: format!("Symbol `{}` was deleted while {} active callers depend on it.", deleted, callers_count),
                });
            }
        }

        // 3. Modified Symbols
        for common in old_symbols.intersection(&new_symbols) {
            let old_info = &old_graph.symbols[*common];
            let new_info = &new_graph.symbols[*common];

            if old_info.type_signature != new_info.type_signature {
                let callers_count = new_graph.reverse_call_graph.get(*common).map(|c| c.len()).unwrap_or(0);
                let is_breaking = callers_count > 0;
                if is_breaking {
                    has_breaking = true;
                }

                events.push(LiveGraphEvent::SymbolModified {
                    symbol: (*common).clone(),
                    old_signature: old_info.type_signature.clone(),
                    new_signature: new_info.type_signature.clone(),
                    is_breaking,
                });

                if is_breaking {
                    events.push(LiveGraphEvent::BreakingChange {
                        symbol: (*common).clone(),
                        impacted_callers_count: callers_count,
                        reason: format!("Signature of `{}` changed from `{}` to `{}`, affecting {} callers.", common, old_info.type_signature, new_info.type_signature, callers_count),
                    });
                }
            }
        }

        // 4. Call Edge Changes
        for (caller, callees) in &new_graph.call_graph {
            let old_callees = old_graph.call_graph.get(caller).cloned().unwrap_or_default();
            for new_callee in callees.difference(&old_callees) {
                events.push(LiveGraphEvent::CallEdgeAdded {
                    caller: caller.clone(),
                    callee: new_callee.clone(),
                });
            }
        }

        for (caller, old_callees) in &old_graph.call_graph {
            let new_callees = new_graph.call_graph.get(caller).cloned().unwrap_or_default();
            for removed_callee in old_callees.difference(&new_callees) {
                events.push(LiveGraphEvent::CallEdgeRemoved {
                    caller: caller.clone(),
                    callee: removed_callee.clone(),
                });
            }
        }

        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();

        GraphDeltaReport {
            file: new_graph.filename.clone(),
            timestamp_epoch_ms: now_ms,
            has_breaking_changes: has_breaking,
            events_count: events.len(),
            events,
        }
    }
}
