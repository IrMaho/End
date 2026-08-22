use crate::ast::*;
use crate::semantic::graph::SemanticGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Official End Semantic Interface Specification
/// The decoupled IR bridging the End Native Compiler and DeepSift Intelligence Engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndSemanticIR {
    pub version: String,
    pub module_name: String,
    pub filename: String,
    pub project_info: ProjectSemanticInfo,
    pub type_graph: TypeGraphIR,
    pub symbol_graph: SymbolGraphIR,
    pub contract_graph: ContractGraphIR,
    pub resource_graph: ResourceGraphIR,
    pub agent_graph: AgentGraphIR,
    pub source_stats: SourceStatsIR,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSemanticInfo {
    pub name: String,
    pub target_arch: String,
    pub memory_model: String,
    pub capability_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeNodeIR {
    pub name: String,
    pub kind: String, // "struct", "enum", "trait", "primitive", "alias"
    pub is_pub: bool,
    pub generic_params: Vec<String>,
    pub fields_or_variants: Vec<String>,
    pub memory_size_bytes: usize,
    pub is_zero_copy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeGraphIR {
    pub types: HashMap<String, TypeNodeIR>,
    pub hierarchy: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolNodeIR {
    pub name: String,
    pub kind: String,
    pub signature: String,
    pub file: String,
    pub line: usize,
    pub callers: Vec<String>,
    pub callees: Vec<String>,
    pub is_pure: bool,
    pub memory_region: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolGraphIR {
    pub total_symbols: usize,
    pub symbols: HashMap<String, SymbolNodeIR>,
    pub call_matrix: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillRuleIR {
    pub skill_name: String,
    pub scope: String,
    pub rules: Vec<String>,
    pub hard_constraints: Vec<String>,
    pub soft_constraints: Vec<String>,
    pub required_capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractGraphIR {
    pub skills: Vec<SkillRuleIR>,
    pub function_contracts: HashMap<String, Vec<String>>,
    pub intent_declarations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceNodeIR {
    pub symbol: String,
    pub net_access: bool,
    pub disk_access: bool,
    pub raw_io: bool,
    pub memory_arena: String,
    pub can_panic: bool,
    pub concurrency_safe: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceGraphIR {
    pub resources: HashMap<String, ResourceNodeIR>,
    pub pure_symbols_count: usize,
    pub io_symbols_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGraphIR {
    pub tasks_defined: Vec<String>,
    pub skills_attached: Vec<String>,
    pub evidence_registered: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStatsIR {
    pub total_lines: usize,
    pub structs_count: usize,
    pub enums_count: usize,
    pub traits_count: usize,
    pub functions_count: usize,
    pub tests_count: usize,
}

pub struct EndSemanticInterface;

impl EndSemanticInterface {
    pub fn extract_ir(module: &Module, graph: &SemanticGraph, source: &str) -> EndSemanticIR {
        // 1. Type Graph
        let mut types = HashMap::new();
        for s in &module.structs {
            let fields = s.fields.iter().map(|f| format!("{}: {}", f.name, f.field_type)).collect();
            types.insert(
                s.name.clone(),
                TypeNodeIR {
                    name: s.name.clone(),
                    kind: "struct".to_string(),
                    is_pub: s.is_pub,
                    generic_params: s.generic_params.clone(),
                    fields_or_variants: fields,
                    memory_size_bytes: s.fields.len() * 8, // Approximate 64-bit alignment
                    is_zero_copy: true,
                },
            );
        }
        for e in &module.enums {
            let variants = e.variants.iter().map(|v| {
                if let Some(ref p) = v.payload {
                    format!("{}({})", v.name, p)
                } else {
                    v.name.clone()
                }
            }).collect();
            types.insert(
                e.name.clone(),
                TypeNodeIR {
                    name: e.name.clone(),
                    kind: "enum".to_string(),
                    is_pub: e.is_pub,
                    generic_params: e.generic_params.clone(),
                    fields_or_variants: variants,
                    memory_size_bytes: 8,
                    is_zero_copy: true,
                },
            );
        }

        // 2. Symbol Graph
        let mut symbols = HashMap::new();
        let mut call_matrix = Vec::new();
        for (name, info) in &graph.symbols {
            let callers = graph.reverse_call_graph.get(name).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();
            let callees = graph.call_graph.get(name).cloned().unwrap_or_default().into_iter().collect::<Vec<_>>();

            for callee in &callees {
                call_matrix.push((name.clone(), callee.clone()));
            }

            symbols.insert(
                name.clone(),
                SymbolNodeIR {
                    name: name.clone(),
                    kind: info.kind.clone(),
                    signature: info.type_signature.clone(),
                    file: info.file.clone(),
                    line: info.defined_at_line,
                    callers,
                    callees,
                    is_pure: info.capabilities.is_pure,
                    memory_region: info.capabilities.memory.clone(),
                },
            );
        }

        // 3. Contract & Skill Graph
        let mut skills = Vec::new();
        let mut function_contracts = HashMap::new();
        let mut intent_declarations = HashMap::new();

        for f in &module.functions {
            let mut contracts = Vec::new();
            for d in &f.directives {
                if d.name == "@skill" || d.name == "@contract" || d.name == "@requires" || d.name == "@guarantees" {
                    contracts.push(format!("{}({})", d.name, d.args.join(", ")));
                }
                if d.name == "@intent" {
                    intent_declarations.insert(f.name.clone(), d.args.join(" "));
                }
            }
            if !contracts.is_empty() {
                function_contracts.insert(f.name.clone(), contracts);
            }
        }

        // Extract skills from directives or comments
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("skill ") {
                let name = trimmed.split_whitespace().nth(1).unwrap_or("Unknown").trim_matches('{').to_string();
                skills.push(SkillRuleIR {
                    skill_name: name,
                    scope: "module".to_string(),
                    rules: vec!["no_unhandled_exceptions".to_string(), "deterministic_flow".to_string()],
                    hard_constraints: vec!["zero_dirty_reads".to_string(), "atomic_execution".to_string()],
                    soft_constraints: vec!["max_query_latency_20ms".to_string()],
                    required_capabilities: vec!["database".to_string()],
                });
            }
        }

        // 4. Resource Graph
        let mut resources = HashMap::new();
        let mut pure_count = 0;
        let mut io_count = 0;
        for (name, info) in &graph.symbols {
            if info.capabilities.is_pure {
                pure_count += 1;
            } else {
                io_count += 1;
            }
            resources.insert(
                name.clone(),
                ResourceNodeIR {
                    symbol: name.clone(),
                    net_access: info.capabilities.net,
                    disk_access: info.capabilities.disk,
                    raw_io: info.capabilities.io,
                    memory_arena: info.capabilities.memory.clone(),
                    can_panic: info.capabilities.can_panic,
                    concurrency_safe: info.capabilities.concurrency_safe,
                },
            );
        }

        // 5. Agent Graph
        let tests_count = module.functions.iter().filter(|f| {
            f.directives.iter().any(|d| d.name == "@test" || d.name == "@scenario" || d.name == "@bench") || f.name.starts_with("test_")
        }).count();

        EndSemanticIR {
            version: "2.0.0".to_string(),
            module_name: module.name.clone(),
            filename: graph.filename.clone(),
            project_info: ProjectSemanticInfo {
                name: module.name.clone(),
                target_arch: std::env::consts::ARCH.to_string(),
                memory_model: "ZeroGC Tier-1 Arena & Ephemeral Leasing".to_string(),
                capability_profile: "Deterministic Capability-Restricted".to_string(),
            },
            type_graph: TypeGraphIR {
                types,
                hierarchy: vec!["Domain -> Data -> Presentation".to_string()],
            },
            symbol_graph: SymbolGraphIR {
                total_symbols: symbols.len(),
                symbols,
                call_matrix,
            },
            contract_graph: ContractGraphIR {
                skills,
                function_contracts,
                intent_declarations,
            },
            resource_graph: ResourceGraphIR {
                resources,
                pure_symbols_count: pure_count,
                io_symbols_count: io_count,
            },
            agent_graph: AgentGraphIR {
                tasks_defined: vec![],
                skills_attached: vec![],
                evidence_registered: vec![],
            },
            source_stats: SourceStatsIR {
                total_lines: source.lines().count(),
                structs_count: module.structs.len(),
                enums_count: module.enums.len(),
                traits_count: module.traits.len(),
                functions_count: module.functions.len(),
                tests_count,
            },
        }
    }
}
