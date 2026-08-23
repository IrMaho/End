use crate::ast::{EnumDef, Type};
use std::collections::{HashMap, HashSet};

pub struct CBackend {
    pub(crate) output: String,
    pub(crate) header_output: String,
    pub(crate) indent_level: usize,
    pub(crate) enums: Vec<EnumDef>,
    pub(crate) is_lib: bool,
    pub var_types: HashMap<String, Type>,
    pub active_regions: Vec<String>,
    pub struct_methods: HashMap<String, HashSet<String>>,
    pub module_methods: HashMap<String, HashSet<String>>,
    pub scope_vars: Vec<HashSet<String>>,
}

pub fn escape_c_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 16);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            other => out.push(other),
        }
    }
    out
}

impl CBackend {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            header_output: String::new(),
            indent_level: 0,
            enums: Vec::new(),
            is_lib: false,
            var_types: HashMap::new(),
            active_regions: Vec::new(),
            struct_methods: HashMap::new(),
            module_methods: HashMap::new(),
            scope_vars: vec![HashSet::new()],
        }
    }

    pub(crate) fn push_c_scope(&mut self) {
        self.scope_vars.push(HashSet::new());
    }

    pub(crate) fn pop_c_scope(&mut self) {
        self.scope_vars.pop();
    }

    pub(crate) fn declare_c_var(&mut self, name: &str, ty: Type) {
        self.var_types.insert(name.to_string(), ty);
        if let Some(top) = self.scope_vars.last_mut() {
            top.insert(name.to_string());
        }
    }

    pub(crate) fn get_active_visible_vars(&self) -> Vec<(String, Type)> {
        let mut vars = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        for scope in &self.scope_vars {
            for v in scope {
                if !seen.contains(v) {
                    seen.insert(v.clone());
                    let ty = self.var_types.get(v).cloned().unwrap_or(Type::I64);
                    vars.push((v.clone(), ty));
                }
            }
        }
        vars
    }

    pub(crate) fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    pub(crate) fn find_enum_for_variant(&self, variant_name: &str) -> String {
        for e in &self.enums {
            if e.variants.iter().any(|v| v.name == variant_name) {
                return e.name.clone();
            }
        }
        "Enum".to_string()
    }
}
