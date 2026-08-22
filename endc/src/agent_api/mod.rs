use crate::semantic::graph::SemanticGraph;
use serde_json::json;

pub mod self_healing;
pub mod code_slicing;
pub mod ast_patch;
pub mod evaluator;

// End Intelligence Platform (EIP) Modules
pub mod semantic_interface;
pub mod impact_guard;
pub mod context_slicer;
pub mod skill_verifier;
pub mod dna;
pub mod live_graph;
pub mod auto_heal;
pub mod agent_scope;
pub mod security_scan;
pub mod research_memory;
pub mod semantic_git;
pub mod autonomous_agent;
pub mod universal_resource;
pub mod adaptive_intent;
pub mod type_security;
pub mod reality_types;

pub use self_healing::SelfHealingEngine;
pub use code_slicing::SemanticCodeSlicer;
pub use ast_patch::StructuredAstPatcher;
pub use evaluator::MicroEvaluator;

pub use semantic_interface::{EndSemanticInterface, EndSemanticIR};
pub use impact_guard::{ImpactGuard, PreTouchImpactReport};
pub use context_slicer::{SmartContextSlicer, SmartContextReport};
pub use skill_verifier::{SemanticSkillVerifier, SkillVerificationReport, SkillViolation};
pub use dna::{ProjectDnaEngine, ProjectDNA, DnaAuditReport};
pub use live_graph::{LiveSemanticGraphEngine, GraphDeltaReport, LiveGraphEvent};
pub use auto_heal::{AutonomousSelfHealingEngine, AutoHealLoopReport};
pub use agent_scope::{AgentScopeEngine, AgentScopeDef, AgentScopeAuditReport};
pub use security_scan::{AstSecurityScanner, SecurityAuditReport, SecurityVulnerability};
pub use research_memory::{DrmEngine, DynamicResearchMemory};
pub use semantic_git::{SemanticGitEngine, SemanticGitDiff, VerifiedCommitManifest};
pub use autonomous_agent::{AutonomousAgentRuntime, AutonomousAgentExecutionReport};
pub use universal_resource::{UniversalResourceManager, UniversalResourceReport, ResourceLease, ResourceKind};
pub use adaptive_intent::{IntentOptimizationEngine, OptimizationIntent, PerformanceBudget, ParetoOptimizationResult};
pub use type_security::{TypeLevelSecurityEngine, TypeSecurityAuditReport, TaintFlowViolation};
pub use reality_types::{RealityAwareEngine, PhysicalDataLayout, StateMachineTransition, IntentProofBinaryReport};

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

    pub fn explain_line(&self, line: usize) -> serde_json::Value {
        if let Some(line_sem) = self.graph.inspect_line(line) {
            let mut summary_parts = Vec::new();
            if !line_sem.flow.from.is_empty() {
                let inputs = line_sem
                    .flow
                    .from
                    .iter()
                    .map(|f| format!("{}: {}", f.symbol, f.symbol_type))
                    .collect::<Vec<_>>()
                    .join(", ");
                summary_parts.push(format!("Consumes input values [{}]", inputs));
            }
            if !line_sem.flow.to.is_empty() {
                let outputs = line_sem
                    .flow
                    .to
                    .iter()
                    .map(|t| format!("{}: {} (lifetime: {})", t.symbol, t.symbol_type, t.lifetime))
                    .collect::<Vec<_>>()
                    .join(", ");
                summary_parts.push(format!("Produces outputs [{}]", outputs));
            }
            if line_sem.side_effects.memory_allocated {
                summary_parts.push("Allocates dynamic memory".to_string());
            }
            if line_sem.side_effects.io_performed {
                summary_parts.push("Performs I/O operations".to_string());
            }

            let explanation = if summary_parts.is_empty() {
                format!("Executes pure expression '{}'", line_sem.code)
            } else {
                summary_parts.join(" | ")
            };

            json!({
                "status": "success",
                "line": line,
                "code": line_sem.code,
                "semantic_explanation": explanation,
                "is_pure": !line_sem.side_effects.memory_allocated && !line_sem.side_effects.io_performed,
                "can_panic": line_sem.side_effects.can_panic
            })
        } else {
            json!({
                "status": "not_found",
                "line": line,
                "message": format!("Line {} has no registered AST facts", line)
            })
        }
    }

    pub fn trace_symbol(&self, symbol_name: &str) -> serde_json::Value {
        let mut occurrences = Vec::new();
        for (line, sem) in &self.graph.lines {
            let from_match = sem.flow.from.iter().any(|f| f.symbol == symbol_name);
            let to_match = sem.flow.to.iter().any(|t| t.symbol == symbol_name);
            if from_match || to_match {
                occurrences.push(json!({
                    "line": line,
                    "code": sem.code,
                    "role": if to_match { "definition / assignment" } else { "usage / operand" },
                }));
            }
        }

        occurrences.sort_by_key(|a| a["line"].as_u64().unwrap_or(0));

        json!({
            "status": "success",
            "symbol": symbol_name,
            "trace_count": occurrences.len(),
            "timeline": occurrences
        })
    }

    pub fn query_effects(&self, symbol_name: &str) -> serde_json::Value {
        if let Some(info) = self.graph.get_symbol(symbol_name) {
            json!({
                "status": "success",
                "symbol": symbol_name,
                "kind": info.kind,
                "signature": info.type_signature,
                "file": info.file,
                "line": info.defined_at_line,
                "purity": if info.capabilities.is_pure { "Pure (No I/O or Mutation)" } else { "Impure (Side Effects / Mutation)" },
                "memory_arena": info.capabilities.memory,
                "capabilities": {
                    "net": info.capabilities.net,
                    "disk": info.capabilities.disk,
                    "io": info.capabilities.io,
                    "memory": info.capabilities.memory,
                    "is_pure": info.capabilities.is_pure,
                    "can_panic": info.capabilities.can_panic,
                    "concurrency_safe": info.capabilities.concurrency_safe
                },
                "declared_directives": info.effects
            })
        } else {
            json!({
                "status": "not_found",
                "symbol": symbol_name,
                "message": format!("Symbol '{}' not found in semantic database", symbol_name)
            })
        }
    }

    pub fn impact_analysis(&self, symbol: &str) -> serde_json::Value {
        let report = self.graph.impact_analysis(symbol);
        json!({
            "status": "success",
            "impact": report
        })
    }

    pub fn query_symbol(&self, symbol_name: &str) -> serde_json::Value {
        if let Some(info) = self.graph.get_symbol(symbol_name) {
            let callers = self.graph.reverse_call_graph.get(symbol_name).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();
            let callees = self.graph.call_graph.get(symbol_name).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();

            json!({
                "status": "success",
                "symbol": info.name,
                "kind": info.kind,
                "signature": info.type_signature,
                "file": info.file,
                "line": info.defined_at_line,
                "callers": callers,
                "callees": callees,
                "memory_arena": info.capabilities.memory,
                "purity": if info.capabilities.is_pure { "Pure (No I/O)" } else { "Impure" },
                "capabilities": {
                    "net": info.capabilities.net,
                    "disk": info.capabilities.disk,
                    "io": info.capabilities.io,
                    "is_pure": info.capabilities.is_pure,
                    "concurrency_safe": info.capabilities.concurrency_safe
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

    pub fn query_callers(&self, symbol_name: &str) -> serde_json::Value {
        let callers = self.graph.reverse_call_graph.get(symbol_name).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();
        if let Some(info) = self.graph.get_symbol(symbol_name) {
            json!({
                "symbol": symbol_name,
                "callers": callers,
                "signature": info.type_signature,
                "memory_arena": info.capabilities.memory,
                "purity": if info.capabilities.is_pure { "Pure (No I/O)" } else { "Impure" }
            })
        } else {
            json!({
                "symbol": symbol_name,
                "callers": callers,
                "signature": "unknown",
                "memory_arena": "ZeroGC-Local",
                "purity": "Pure (No I/O)"
            })
        }
    }

    pub fn query_callees(&self, symbol_name: &str) -> serde_json::Value {
        let callees = self.graph.call_graph.get(symbol_name).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();
        if let Some(info) = self.graph.get_symbol(symbol_name) {
            json!({
                "symbol": symbol_name,
                "callees": callees,
                "signature": info.type_signature,
                "memory_arena": info.capabilities.memory,
                "purity": if info.capabilities.is_pure { "Pure (No I/O)" } else { "Impure" }
            })
        } else {
            json!({
                "symbol": symbol_name,
                "callees": callees,
                "signature": "unknown",
                "memory_arena": "ZeroGC-Local",
                "purity": "Pure (No I/O)"
            })
        }
    }

    pub fn knowledge_graph(&self) -> serde_json::Value {
        self.graph.generate_knowledge_graph()
    }
}
