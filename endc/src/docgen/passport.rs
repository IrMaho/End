use crate::ast::*;
use crate::semantic::analyzer::SemanticAnalyzer;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPassport {
    pub metadata: ProjectMetadata,
    pub structs: Vec<StructPassport>,
    pub enums: Vec<EnumPassport>,
    pub functions: Vec<FunctionPassport>,
    pub modules: Vec<ModulePassport>,
    pub extensions: Vec<ExtensionPassport>,
    pub endpoints: Vec<ApiEndpointPassport>,
    pub memory_safety_summary: MemorySafetySummary,
    pub capability_summary: CapabilitySummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub name: String,
    pub entry_file: String,
    pub total_lines: usize,
    pub total_structs: usize,
    pub total_enums: usize,
    pub total_functions: usize,
    pub total_modules: usize,
    pub total_endpoints: usize,
    pub compiler_version: String,
    pub generation_timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructFieldInfo {
    pub name: String,
    pub field_type: String,
    pub byte_offset: usize,
    pub byte_size: usize,
    pub alignment: usize,
    pub doc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructPassport {
    pub name: String,
    pub is_pub: bool,
    pub total_size_bytes: usize,
    pub alignment_bytes: usize,
    pub fields: Vec<StructFieldInfo>,
    pub is_concurrency_safe: bool,
    pub doc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariantInfo {
    pub name: String,
    pub payload_type: Option<String>,
    pub tag_value: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumPassport {
    pub name: String,
    pub is_pub: bool,
    pub variants: Vec<EnumVariantInfo>,
    pub memory_size_bytes: usize,
    pub doc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionParamInfo {
    pub name: String,
    pub param_type: String,
    pub is_mut: bool,
    pub is_ref: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionPassport {
    pub name: String,
    pub is_pub: bool,
    pub signature: String,
    pub params: Vec<FunctionParamInfo>,
    pub return_type: String,
    pub memory_tier: String, // "Tier 1 (Arena Scoped / Zero-Alloc)", "Tier 2 (Automatic Reference Counting)", "Tier 3 (Bare-Metal Raw Pointer)"
    pub purity: String,    // "Pure (Deterministic)", "Side-Effectful"
    pub capabilities: Vec<String>, // "io", "net", "disk", "concurrency_safe"
    pub callers: Vec<String>,
    pub callees: Vec<String>,
    pub suggested_test_hints: Vec<String>,
    pub invariants: Vec<String>,
    pub doc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePassport {
    pub name: String,
    pub is_pub: bool,
    pub parent_module: Option<String>,
    pub functions: Vec<String>,
    pub overrides: Vec<String>,
    pub doc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPassport {
    pub target_struct: String,
    pub extension_methods: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiEndpointPassport {
    pub path: String,
    pub http_method: String, // GET, POST, PUT, DELETE, PATCH
    pub summary: String,
    pub tag: String,
    pub handler_name: String,
    pub request_body_type: Option<String>,
    pub response_type: String,
    pub status_code: u16,
    pub is_authenticated: bool,
    pub doc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySafetySummary {
    pub tier1_arena_symbols_count: usize,
    pub tier2_arc_symbols_count: usize,
    pub tier3_bare_metal_symbols_count: usize,
    pub zero_overhead_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitySummary {
    pub pure_functions_count: usize,
    pub io_restricted_functions_count: usize,
    pub network_access_functions_count: usize,
    pub disk_access_functions_count: usize,
    pub concurrency_safe_percentage: f64,
}

pub struct PassportBuilder;

impl PassportBuilder {
    pub fn build(module: &Module, analyzer: &SemanticAnalyzer, source_code: &str) -> ProjectPassport {
        let lines: Vec<&str> = source_code.lines().collect();
        let total_lines = lines.len();

        let mut structs = Vec::new();
        for s in &module.structs {
            let mut current_offset = 0;
            let mut max_align = 1;
            let mut fields = Vec::new();

            for f in &s.fields {
                let (sz, al) = Self::type_size_align(&f.field_type);
                if al > max_align { max_align = al; }
                // Align offset
                if current_offset % al != 0 {
                    current_offset += al - (current_offset % al);
                }
                let f_doc = Self::extract_doc_above(&lines, f.span.line);
                fields.push(StructFieldInfo {
                    name: f.name.clone(),
                    field_type: format!("{:?}", f.field_type),
                    byte_offset: current_offset,
                    byte_size: sz,
                    alignment: al,
                    doc: f_doc,
                });
                current_offset += sz;
            }

            if current_offset % max_align != 0 {
                current_offset += max_align - (current_offset % max_align);
            }

            let s_doc = Self::extract_doc_above(&lines, s.span.line);
            structs.push(StructPassport {
                name: s.name.clone(),
                is_pub: s.is_pub,
                total_size_bytes: current_offset,
                alignment_bytes: max_align,
                fields,
                is_concurrency_safe: true,
                doc: s_doc,
            });
        }

        let mut enums = Vec::new();
        for e in &module.enums {
            let mut variants = Vec::new();
            for (idx, v) in e.variants.iter().enumerate() {
                variants.push(EnumVariantInfo {
                    name: v.name.clone(),
                    payload_type: v.payload.as_ref().map(|p| format!("{:?}", p)),
                    tag_value: idx,
                });
            }
            let e_doc = Self::extract_doc_above(&lines, e.span.line);
            enums.push(EnumPassport {
                name: e.name.clone(),
                is_pub: e.is_pub,
                variants,
                memory_size_bytes: 16, // Tag (8 bytes) + max payload
                doc: e_doc,
            });
        }

        let mut functions = Vec::new();
        let mut endpoints = Vec::new();

        let mut tier1_count = 0;
        let mut tier2_count = 0;
        let mut tier3_count = 0;
        let mut pure_count = 0;
        let mut io_count = 0;
        let mut net_count = 0;
        let mut disk_count = 0;
        let mut concurrency_safe_count = 0;

        for f in &module.functions {
            let mut params = Vec::new();
            for p in &f.params {
                params.push(FunctionParamInfo {
                    name: p.name.clone(),
                    param_type: format!("{:?}", p.param_type),
                    is_mut: p.is_mut,
                    is_ref: p.name.starts_with('&'),
                });
            }

            let sym_info = analyzer.graph.symbols.get(&f.name);
            let mem_tier = if let Some(sym) = sym_info {
                if sym.effects.iter().any(|eff| eff.contains("malloc") || eff.contains("raw_pointer")) {
                    tier3_count += 1;
                    "Tier 3 (Bare-Metal Raw Pointer)".to_string()
                } else if sym.effects.iter().any(|eff| eff.contains("rc") || eff.contains("arc")) {
                    tier2_count += 1;
                    "Tier 2 (Automatic Reference Counting)".to_string()
                } else {
                    tier1_count += 1;
                    "Tier 1 (Arena Scoped / Zero-Alloc)".to_string()
                }
            } else {
                tier1_count += 1;
                "Tier 1 (Arena Scoped / Zero-Alloc)".to_string()
            };

            let mut caps = Vec::new();
            let is_pure = sym_info.map_or(true, |s| s.is_pure);
            if is_pure {
                pure_count += 1;
                caps.push("pure".to_string());
            }
            if sym_info.map_or(false, |s| s.capabilities.io) {
                io_count += 1;
                caps.push("io".to_string());
            }
            if sym_info.map_or(false, |s| s.capabilities.net) {
                net_count += 1;
                caps.push("net".to_string());
            }
            if sym_info.map_or(false, |s| s.capabilities.disk) {
                disk_count += 1;
                caps.push("disk".to_string());
            }
            if sym_info.map_or(true, |s| s.capabilities.concurrency_safe) {
                concurrency_safe_count += 1;
                caps.push("concurrency_safe".to_string());
            }

            let callers = sym_info.map_or(Vec::new(), |s| s.callers.clone());
            let callees = sym_info.map_or(Vec::new(), |s| s.callees.clone());

            let mut invariants = Vec::new();
            let mut test_hints = Vec::new();
            if is_pure {
                invariants.push("Idempotent: Identical inputs guarantee identical output with zero side-effects.".to_string());
                test_hints.push(format!("Fuzz test with boundary values for params: {:?}", f.params.iter().map(|p| &p.name).collect::<Vec<_>>()));
            } else {
                invariants.push("Performs stateful or I/O operations; requires isolated mock runtime.".to_string());
                test_hints.push("Integration test with mock network/disk environment.".to_string());
            }

            let f_doc = Self::extract_doc_above(&lines, f.span.line);

            // Check if function is an API endpoint from directives OR doc annotations
            let mut detected_endpoint = None;

            // 1. Directives check
            for d in &f.directives {
                if d.name == "@route" || d.name == "@get" || d.name == "@post" || d.name == "@put" || d.name == "@delete" || d.name == "@api" {
                    let mut path = "/api".to_string();
                    let method = if d.name == "@get" { "GET".to_string() }
                        else if d.name == "@post" { "POST".to_string() }
                        else if d.name == "@put" { "PUT".to_string() }
                        else if d.name == "@delete" { "DELETE".to_string() }
                        else { "GET".to_string() };

                    if let Some(first_arg) = d.args.first() {
                        path = first_arg.clone();
                    }

                    let summary = f.directives.iter()
                        .find(|d| d.name == "@summary")
                        .and_then(|d| d.args.first())
                        .cloned()
                        .unwrap_or_else(|| format!("Handler for {}", f.name));

                    let tag = f.directives.iter()
                        .find(|d| d.name == "@tag")
                        .and_then(|d| d.args.first())
                        .cloned()
                        .unwrap_or_else(|| "Default".to_string());

                    let req_body = if method == "POST" || method == "PUT" || method == "PATCH" {
                        f.params.first().map(|p| format!("{:?}", p.param_type))
                    } else {
                        None
                    };

                    detected_endpoint = Some(ApiEndpointPassport {
                        path,
                        http_method: method,
                        summary,
                        tag,
                        handler_name: f.name.clone(),
                        request_body_type: req_body,
                        response_type: format!("{:?}", f.return_type),
                        status_code: 200,
                        is_authenticated: f.directives.iter().any(|d| d.name == "@auth"),
                        doc: f_doc.clone(),
                    });
                    break;
                }
            }

            // 2. Doc comments annotation check (e.g. /// @get("/api/...") or /// @route(...))
            if detected_endpoint.is_none() && !f_doc.is_empty() {
                for line in f_doc.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("@get(") || trimmed.starts_with("@post(") || trimmed.starts_with("@put(") || trimmed.starts_with("@delete(") || trimmed.starts_with("@route(") {
                        let method = if trimmed.starts_with("@get") { "GET".to_string() }
                            else if trimmed.starts_with("@post") { "POST".to_string() }
                            else if trimmed.starts_with("@put") { "PUT".to_string() }
                            else if trimmed.starts_with("@delete") { "DELETE".to_string() }
                            else { "GET".to_string() };

                        let path = trimmed.split('(').nth(1)
                            .and_then(|s| s.split(')').next())
                            .map(|s| s.trim_matches('"').trim_matches('\'').to_string())
                            .unwrap_or_else(|| "/api".to_string());

                        let mut summary = format!("Handler for {}", f.name);
                        let mut tag = "Default".to_string();

                        for l in f_doc.lines() {
                            let lt = l.trim();
                            if lt.starts_with("@summary(") {
                                if let Some(s) = lt.split('(').nth(1).and_then(|x| x.split(')').next()) {
                                    summary = s.trim_matches('"').trim_matches('\'').to_string();
                                }
                            } else if lt.starts_with("@tag(") {
                                if let Some(t) = lt.split('(').nth(1).and_then(|x| x.split(')').next()) {
                                    tag = t.trim_matches('"').trim_matches('\'').to_string();
                                }
                            }
                        }

                        let req_body = if method == "POST" || method == "PUT" || method == "PATCH" {
                            f.params.first().map(|p| format!("{:?}", p.param_type))
                        } else {
                            None
                        };

                        detected_endpoint = Some(ApiEndpointPassport {
                            path,
                            http_method: method,
                            summary,
                            tag,
                            handler_name: f.name.clone(),
                            request_body_type: req_body,
                            response_type: format!("{:?}", f.return_type),
                            status_code: 200,
                            is_authenticated: f_doc.contains("@auth"),
                            doc: f_doc.clone(),
                        });
                        break;
                    }
                }
            }

            if let Some(ep) = detected_endpoint {
                endpoints.push(ep);
            }

            let sig = format!("fn {}({}) -> {:?}", f.name, f.params.iter().map(|p| format!("{}: {:?}", p.name, p.param_type)).collect::<Vec<_>>().join(", "), f.return_type);

            functions.push(FunctionPassport {
                name: f.name.clone(),
                is_pub: f.is_pub,
                signature: sig,
                params,
                return_type: format!("{:?}", f.return_type),
                memory_tier: mem_tier,
                purity: if is_pure { "Pure (Deterministic)".to_string() } else { "Side-Effectful".to_string() },
                capabilities: caps,
                callers,
                callees,
                suggested_test_hints: test_hints,
                invariants,
                doc: f_doc,
            });
        }

        let mut modules_passport = Vec::new();
        for m in &module.modules {
            let m_doc = Self::extract_doc_above(&lines, m.span.line);
            modules_passport.push(ModulePassport {
                name: m.name.clone(),
                is_pub: m.is_pub,
                parent_module: m.parent.clone(),
                functions: m.functions.iter().map(|f| f.name.clone()).collect(),
                overrides: m.overrides.iter().map(|ov| ov.name.clone()).collect(),
                doc: m_doc,
            });
        }

        let mut extensions_passport = Vec::new();
        for ext in &module.extensions {
            extensions_passport.push(ExtensionPassport {
                target_struct: ext.target.clone(),
                extension_methods: ext.functions.iter().map(|f| f.name.clone()).collect(),
            });
        }

        let total_fns = functions.len().max(1);
        let mem_summary = MemorySafetySummary {
            tier1_arena_symbols_count: tier1_count,
            tier2_arc_symbols_count: tier2_count,
            tier3_bare_metal_symbols_count: tier3_count,
            zero_overhead_percentage: (tier1_count as f64 / total_fns as f64) * 100.0,
        };

        let cap_summary = CapabilitySummary {
            pure_functions_count: pure_count,
            io_restricted_functions_count: io_count,
            network_access_functions_count: net_count,
            disk_access_functions_count: disk_count,
            concurrency_safe_percentage: (concurrency_safe_count as f64 / total_fns as f64) * 100.0,
        };

        ProjectPassport {
            metadata: ProjectMetadata {
                name: module.name.clone(),
                entry_file: module.span.file.clone(),
                total_lines,
                total_structs: structs.len(),
                total_enums: enums.len(),
                total_functions: functions.len(),
                total_modules: modules_passport.len(),
                total_endpoints: endpoints.len(),
                compiler_version: "0.4.0-alpha (Enterprise Vibe Coding Edition)".to_string(),
                generation_timestamp: "2026-08-21T21:30:00Z".to_string(),
            },
            structs,
            enums,
            functions,
            modules: modules_passport,
            extensions: extensions_passport,
            endpoints,
            memory_safety_summary: mem_summary,
            capability_summary: cap_summary,
        }
    }

    fn type_size_align(ty: &Type) -> (usize, usize) {
        match ty {
            Type::I8 | Type::U8 | Type::Bool => (1, 1),
            Type::I16 | Type::U16 => (2, 2),
            Type::I32 | Type::U32 | Type::F32 => (4, 4),
            Type::I64 | Type::U64 | Type::F64 | Type::Str | Type::Pointer(_) | Type::Box(_) | Type::Rc(_) | Type::Arc(_) => (8, 8),
            Type::Array(inner, count) => {
                let (elem_sz, elem_al) = Self::type_size_align(inner);
                (elem_sz * count, elem_al)
            }
            _ => (8, 8),
        }
    }

    fn extract_doc_above(lines: &[&str], line_num: usize) -> String {
        if line_num == 0 || line_num > lines.len() { return String::new(); }
        let mut doc_lines = Vec::new();
        let mut cur = line_num.saturating_sub(2);
        while cur < lines.len() {
            let trimmed = lines[cur].trim();
            if trimmed.starts_with("///") {
                doc_lines.push(trimmed.trim_start_matches("///").trim().to_string());
            } else if trimmed.starts_with("//") {
                doc_lines.push(trimmed.trim_start_matches("//").trim().to_string());
            } else {
                break;
            }
            if cur == 0 { break; }
            cur -= 1;
        }
        doc_lines.reverse();
        doc_lines.join("\\n")
    }
}
