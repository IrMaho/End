use crate::ast::*;
use serde_json::json;

pub struct SemanticCodeSlicer;

impl SemanticCodeSlicer {
    pub fn slice_module(
        module: &Module,
        interface_only: bool,
        types_only: bool,
    ) -> String {
        let mut out = String::new();

        out.push_str(&format!("// 🧩 End Semantic Skeletal Slice: {}\n\n", module.name));

        // 1. Imports
        if !types_only && !module.imports.is_empty() {
            out.push_str("// --- Imports ---\n");
            for imp in &module.imports {
                match &imp.kind {
                    ImportKind::Standard => {
                        out.push_str(&format!("import \"{}\"", imp.path));
                    }
                    ImportKind::C(h) => {
                        out.push_str(&format!("@import_c(\"{}\")", h));
                    }
                    ImportKind::Zig(z) => {
                        out.push_str(&format!("@import_zig(\"{}\")", z));
                    }
                    ImportKind::Rust(r) => {
                        out.push_str(&format!("@import_rust(\"{}\")", r));
                    }
                    ImportKind::Go(g) => {
                        out.push_str(&format!("@import_go(\"{}\")", g));
                    }
                }
                if let Some(ref alias) = imp.alias {
                    out.push_str(&format!(" as {}", alias));
                }
                out.push('\n');
            }
            out.push('\n');
        }

        // 2. Enums
        if !module.enums.is_empty() {
            out.push_str("// --- Type Definitions: Enums ---\n");
            for e in &module.enums {
                for dir in &e.directives {
                    out.push_str(&format!("@{}", dir.name.trim_start_matches('@')));
                    if !dir.args.is_empty() {
                        out.push_str(&format!("({})", dir.args.join(", ")));
                    }
                    out.push('\n');
                }
                let pub_prefix = if e.is_pub { "pub " } else { "" };
                out.push_str(&format!("{}enum {} {{\n", pub_prefix, e.name));
                for v in &e.variants {
                    if let Some(ref payload) = v.payload {
                        out.push_str(&format!("    {}({}),\n", v.name, payload));
                    } else {
                        out.push_str(&format!("    {},\n", v.name));
                    }
                }
                out.push_str("}\n\n");
            }
        }

        // 3. Structs
        if !module.structs.is_empty() {
            out.push_str("// --- Type Definitions: Structs ---\n");
            for s in &module.structs {
                for dir in &s.directives {
                    out.push_str(&format!("@{}", dir.name.trim_start_matches('@')));
                    if !dir.args.is_empty() {
                        out.push_str(&format!("({})", dir.args.join(", ")));
                    }
                    out.push('\n');
                }
                let pub_prefix = if s.is_pub { "pub " } else { "" };
                out.push_str(&format!("{}st {} {{\n", pub_prefix, s.name));
                for f in &s.fields {
                    let field_pub = if f.is_pub { "pub " } else { "" };
                    out.push_str(&format!("    {}{}: {},\n", field_pub, f.name, f.field_type));
                }
                out.push_str("}\n\n");
            }
        }

        // 4. Function Signatures (Skeletal)
        if !types_only && !module.functions.is_empty() {
            out.push_str("// --- Functional Interface & Capability Contracts ---\n");
            for f in &module.functions {
                if interface_only && !f.is_pub && !f.directives.iter().any(|d| d.name == "@test") {
                    continue;
                }

                for dir in &f.directives {
                    out.push_str(&format!("@{}", dir.name.trim_start_matches('@')));
                    if !dir.args.is_empty() {
                        out.push_str(&format!("({})", dir.args.join(", ")));
                    }
                    out.push('\n');
                }

                let pub_prefix = if f.is_pub { "pub " } else { "" };
                let params = f
                    .params
                    .iter()
                    .map(|p| format!("{}{}: {}", if p.is_mut { "mut " } else { "" }, p.name, p.param_type))
                    .collect::<Vec<_>>()
                    .join(", ");

                out.push_str(&format!("{}fn {}({}) {};\n\n", pub_prefix, f.name, params, f.return_type));
            }
        }

        out
    }

    pub fn slice_json(module: &Module) -> serde_json::Value {
        let text = Self::slice_module(module, true, false);
        let token_estimate = text.split_whitespace().count() * 4 / 3;

        let struct_names = module.structs.iter().map(|s| s.name.clone()).collect::<Vec<_>>();
        let enum_names = module.enums.iter().map(|e| e.name.clone()).collect::<Vec<_>>();
        let fn_signatures = module
            .functions
            .iter()
            .map(|f| {
                json!({
                    "name": f.name,
                    "is_pub": f.is_pub,
                    "params": f.params.iter().map(|p| format!("{}: {}", p.name, p.param_type)).collect::<Vec<_>>(),
                    "return_type": f.return_type.to_string(),
                    "directives": f.directives.iter().map(|d| d.name.clone()).collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>();

        json!({
            "status": "success",
            "module": module.name,
            "estimated_tokens": token_estimate,
            "structs_count": module.structs.len(),
            "enums_count": module.enums.len(),
            "functions_count": module.functions.len(),
            "structs": struct_names,
            "enums": enum_names,
            "functions": fn_signatures,
            "skeletal_code": text
        })
    }
}
