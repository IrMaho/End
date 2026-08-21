use crate::ast::*;
use serde_json::json;

pub struct SemanticCodeSlicer;

impl SemanticCodeSlicer {
    pub fn slice_module(
        module: &Module,
        interface_only: bool,
        types_only: bool,
        budget: Option<usize>,
    ) -> String {
        let max_chars = budget.map(|b| b * 4).unwrap_or(usize::MAX);
        let mut out = String::new();

        out.push_str(&format!("// ?? End Semantic Skeletal Slice: {}\n\n", module.name));

        // 1. Imports
        if !types_only && !module.imports.is_empty() {
            let mut imp_block = String::from("// --- Imports ---\n");
            for imp in &module.imports {
                match &imp.kind {
                    ImportKind::Standard => imp_block.push_str(&format!("import \"{}\"", imp.path)),
                    ImportKind::C(h) => imp_block.push_str(&format!("@import_c(\"{}\")", h)),
                    ImportKind::Zig(z) => imp_block.push_str(&format!("@import_zig(\"{}\")", z)),
                    ImportKind::Rust(r) => imp_block.push_str(&format!("@import_rust(\"{}\")", r)),
                    ImportKind::Go(g) => imp_block.push_str(&format!("@import_go(\"{}\")", g)),
                }
                if let Some(ref alias) = imp.alias {
                    imp_block.push_str(&format!(" as {}", alias));
                }
                imp_block.push('\n');
            }
            imp_block.push('\n');
            if out.len() + imp_block.len() <= max_chars {
                out.push_str(&imp_block);
            }
        }

        // 2. Enums
        if !module.enums.is_empty() {
            let mut enum_header_added = false;
            for e in &module.enums {
                let mut enum_str = String::new();
                if !enum_header_added {
                    enum_str.push_str("// --- Type Definitions: Enums ---\n");
                }
                for dir in &e.directives {
                    enum_str.push_str(&format!("@{}", dir.name.trim_start_matches('@')));
                    if !dir.args.is_empty() {
                        enum_str.push_str(&format!("({})", dir.args.join(", ")));
                    }
                    enum_str.push('\n');
                }
                let pub_prefix = if e.is_pub { "pub " } else { "" };
                enum_str.push_str(&format!("{}enum {} {{\n", pub_prefix, e.name));
                for v in &e.variants {
                    if let Some(ref payload) = v.payload {
                        enum_str.push_str(&format!("    {}({}),\n", v.name, payload));
                    } else {
                        enum_str.push_str(&format!("    {},\n", v.name));
                    }
                }
                enum_str.push_str("}\n\n");

                if out.len() + enum_str.len() <= max_chars {
                    out.push_str(&enum_str);
                    enum_header_added = true;
                } else {
                    out.push_str("// ... [Remaining enums omitted to fit token budget]\n\n");
                    break;
                }
            }
        }

        // 3. Structs
        if !module.structs.is_empty() {
            let mut struct_header_added = false;
            for s in &module.structs {
                let mut struct_str = String::new();
                if !struct_header_added {
                    struct_str.push_str("// --- Type Definitions: Structs ---\n");
                }
                for dir in &s.directives {
                    struct_str.push_str(&format!("@{}", dir.name.trim_start_matches('@')));
                    if !dir.args.is_empty() {
                        struct_str.push_str(&format!("({})", dir.args.join(", ")));
                    }
                    struct_str.push('\n');
                }
                let pub_prefix = if s.is_pub { "pub " } else { "" };
                struct_str.push_str(&format!("{}st {} {{\n", pub_prefix, s.name));
                for f in &s.fields {
                    let field_pub = if f.is_pub { "pub " } else { "" };
                    struct_str.push_str(&format!("    {}{}: {},\n", field_pub, f.name, f.field_type));
                }
                struct_str.push_str("}\n\n");

                if out.len() + struct_str.len() <= max_chars {
                    out.push_str(&struct_str);
                    struct_header_added = true;
                } else {
                    out.push_str("// ... [Remaining structs omitted to fit token budget]\n\n");
                    break;
                }
            }
        }

        // 4. Function Signatures (Skeletal)
        if !types_only && !module.functions.is_empty() {
            let mut fn_header_added = false;
            for f in &module.functions {
                if interface_only && !f.is_pub && !f.directives.iter().any(|d| d.name == "@test") {
                    continue;
                }

                let mut fn_str = String::new();
                if !fn_header_added {
                    fn_str.push_str("// --- Functional Interface & Capability Contracts ---\n");
                }

                for dir in &f.directives {
                    fn_str.push_str(&format!("@{}", dir.name.trim_start_matches('@')));
                    if !dir.args.is_empty() {
                        fn_str.push_str(&format!("({})", dir.args.join(", ")));
                    }
                    fn_str.push('\n');
                }

                let pub_prefix = if f.is_pub { "pub " } else { "" };
                let params = f
                    .params
                    .iter()
                    .map(|p| format!("{}{}: {}", if p.is_mut { "mut " } else { "" }, p.name, p.param_type))
                    .collect::<Vec<_>>()
                    .join(", ");

                fn_str.push_str(&format!("{}fn {}({}) {};\n\n", pub_prefix, f.name, params, f.return_type));

                if out.len() + fn_str.len() <= max_chars {
                    out.push_str(&fn_str);
                    fn_header_added = true;
                } else {
                    out.push_str("// ... [Remaining functions omitted to fit token budget cleanly]\n");
                    break;
                }
            }
        }

        out
    }

    pub fn slice_json(module: &Module, budget: Option<usize>) -> serde_json::Value {
        let text = Self::slice_module(module, true, false, budget);
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
            "budget_applied": budget,
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
