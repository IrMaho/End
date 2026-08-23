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
