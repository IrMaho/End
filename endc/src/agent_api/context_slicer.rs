use crate::ast::*;
use crate::semantic::graph::SemanticGraph;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContextReport {
    pub task_intent: String,
    pub original_lines: usize,
    pub extracted_lines: usize,
    pub compression_ratio_pct: f64,
    pub estimated_tokens: usize,
    pub budget_tokens: usize,
    pub preserved_structs: Vec<String>,
    pub preserved_enums: Vec<String>,
    pub preserved_functions: Vec<String>,
    pub contracts_included: Vec<String>,
    pub context_payload: String,
}

pub struct SmartContextSlicer;

impl SmartContextSlicer {
    pub fn extract_context(
        module: &Module,
        graph: &SemanticGraph,
        task_intent: &str,
        token_budget: Option<usize>,
    ) -> SmartContextReport {
        let budget = token_budget.unwrap_or(500);
        let max_chars = budget * 4;

        let keywords: Vec<String> = task_intent
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric() && c != '_')
            .filter(|s| s.len() > 2)
            .map(|s| s.to_string())
            .collect();

        // 1. Identify Seed Symbols matching the Task Intent
        let mut seed_symbols = HashSet::new();
        for (name, _info) in &graph.symbols {
            let name_lower = name.to_lowercase();
            if keywords.iter().any(|k| name_lower.contains(k)) {
                seed_symbols.insert(name.clone());
            }
            if let Some(func) = module.functions.iter().find(|f| &f.name == name) {
                for d in &func.directives {
                    let d_str = format!("{}:{}", d.name, d.args.join(" ")).to_lowercase();
                    if keywords.iter().any(|k| d_str.contains(k)) {
                        seed_symbols.insert(name.clone());
                    }
                }
            }
        }

        // If no direct keyword match, pick entry functions or public functions
        if seed_symbols.is_empty() {
            for f in &module.functions {
                if f.is_pub {
                    seed_symbols.insert(f.name.clone());
                }
            }
        }

        // 2. Expand graph neighborhood (1-hop callers and callees)
        let mut relevant_symbols = seed_symbols.clone();
        for seed in &seed_symbols {
            if let Some(callers) = graph.reverse_call_graph.get(seed) {
                for c in callers {
                    relevant_symbols.insert(c.clone());
                }
            }
            if let Some(callees) = graph.call_graph.get(seed) {
                for c in callees {
                    relevant_symbols.insert(c.clone());
                }
            }
        }

        // 3. Identify Structs/Types referenced in relevant function signatures
        let mut relevant_types = HashSet::new();
        for func in &module.functions {
            if relevant_symbols.contains(&func.name) {
                for param in &func.params {
                    let type_name = param.param_type.to_string().replace(['*', '[', ']', '(', ')', '!', ' '], "");
                    relevant_types.insert(type_name);
                }
                let ret_type = func.return_type.to_string().replace(['*', '[', ']', '(', ')', '!', ' '], "");
                relevant_types.insert(ret_type);
            }
        }

        // 4. Assemble High-Density Minimal Context Payload (DEC_v2 format)
        let mut out = String::new();
        out.push_str(&format!("// 🧠 End DEC_v2 Smart Context Slice (Task Intent: \"{}\")\n", task_intent));
        out.push_str("// High-density token compressed interface for Autonomous Agent\n\n");

        let mut preserved_structs = Vec::new();
        let mut preserved_enums = Vec::new();
        let mut preserved_functions = Vec::new();
        let mut contracts_included = Vec::new();

        // Enums
        for e in &module.enums {
            if relevant_types.contains(&e.name) || keywords.iter().any(|k| e.name.to_lowercase().contains(k)) {
                preserved_enums.push(e.name.clone());
                let pub_str = if e.is_pub { "pub " } else { "" };
                let mut e_str = format!("{}enum {} {{\n", pub_str, e.name);
                for v in &e.variants {
                    if let Some(ref payload) = v.payload {
                        e_str.push_str(&format!("    {}({}),\n", v.name, payload));
                    } else {
                        e_str.push_str(&format!("    {},\n", v.name));
                    }
                }
                e_str.push_str("}\n\n");
                if out.len() + e_str.len() <= max_chars {
                    out.push_str(&e_str);
                }
            }
        }

        // Structs
        for s in &module.structs {
            if relevant_types.contains(&s.name) || keywords.iter().any(|k| s.name.to_lowercase().contains(k)) {
                preserved_structs.push(s.name.clone());
                let pub_str = if s.is_pub { "pub " } else { "" };
                let mut s_str = format!("{}st {} {{\n", pub_str, s.name);
                for f in &s.fields {
                    s_str.push_str(&format!("    {}: {},\n", f.name, f.field_type));
                }
                s_str.push_str("}\n\n");
                if out.len() + s_str.len() <= max_chars {
                    out.push_str(&s_str);
                }
            }
        }

        // Functions & Contracts
        for f in &module.functions {
            if relevant_symbols.contains(&f.name) {
                preserved_functions.push(f.name.clone());
                let mut f_str = String::new();

                for d in &f.directives {
                    let d_str = if d.args.is_empty() {
                        format!("@{}\n", d.name.trim_start_matches('@'))
                    } else {
                        format!("@{}({})\n", d.name.trim_start_matches('@'), d.args.join(", "))
                    };
                    f_str.push_str(&d_str);
                    contracts_included.push(d.name.clone());
                }

                let pub_str = if f.is_pub { "pub " } else { "" };
                let params = f.params.iter().map(|p| format!("{}: {}", p.name, p.param_type)).collect::<Vec<_>>().join(", ");

                // If this is the direct seed target, we can keep its body or signature
                f_str.push_str(&format!("{}fn {}({}) {};\n\n", pub_str, f.name, params, f.return_type));

                if out.len() + f_str.len() <= max_chars {
                    out.push_str(&f_str);
                } else {
                    out.push_str("// ... [Remaining low-priority symbols pruned by token budget]\n");
                    break;
                }
            }
        }

        let orig_lines = module.span.line.max(module.functions.len() * 15);
        let ext_lines = out.lines().count();
        let est_tokens = out.split_whitespace().count() * 4 / 3;
        let compression = if orig_lines > 0 {
            (1.0 - (ext_lines as f64 / orig_lines as f64).min(1.0)) * 100.0
        } else {
            80.0
        };

        SmartContextReport {
            task_intent: task_intent.to_string(),
            original_lines: orig_lines,
            extracted_lines: ext_lines,
            compression_ratio_pct: compression,
            estimated_tokens: est_tokens,
            budget_tokens: budget,
            preserved_structs,
            preserved_enums,
            preserved_functions,
            contracts_included,
            context_payload: out,
        }
    }
}
