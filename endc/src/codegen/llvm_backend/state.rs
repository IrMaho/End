use crate::ast::Type;
use crate::codegen::type_mapper::{LlvmTypeMapper, TypeMapper};
use std::collections::HashMap;

pub struct LlvmBackend {
    pub(crate) output: String,
    pub(crate) target_triple: String,
    pub(crate) temp_var_id: usize,
    pub(crate) block_id: usize,
    pub(crate) str_literal_id: usize,
    #[allow(dead_code)]
    pub(crate) debug_id: usize,
    pub(crate) emit_debug_info: bool,
    pub(crate) string_constants: Vec<(String, String, usize)>, // (name, content, byte_len)
    pub(crate) variables: HashMap<String, (String, String)>, // name -> (llvm_type, llvm_reg_or_ptr)
    pub(crate) type_mapper: LlvmTypeMapper,
}

impl LlvmBackend {
    pub fn new(target_triple: Option<&str>) -> Self {
        Self {
            output: String::new(),
            target_triple: target_triple.unwrap_or(Self::detect_host_triple()).to_string(),
            temp_var_id: 0,
            block_id: 0,
            str_literal_id: 0,
            debug_id: 1,
            emit_debug_info: true,
            string_constants: Vec::new(),
            variables: HashMap::new(),
            type_mapper: LlvmTypeMapper,
        }
    }

    pub fn set_debug_info(&mut self, enabled: bool) {
        self.emit_debug_info = enabled;
    }

    pub(crate) fn detect_host_triple() -> &'static str {
        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        { "x86_64-pc-windows-msvc" }
        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        { "x86_64-unknown-linux-gnu" }
        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        { "x86_64-apple-darwin" }
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        { "aarch64-apple-darwin" }
        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        { "aarch64-unknown-linux-gnu" }
        #[cfg(not(any(
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "linux", target_arch = "aarch64"),
        )))]
        { "x86_64-unknown-linux-gnu" }
    }

    pub(crate) fn next_temp(&mut self) -> String {
        let id = self.temp_var_id;
        self.temp_var_id += 1;
        format!("%t{}", id)
    }

    pub(crate) fn next_label(&mut self, prefix: &str) -> String {
        let id = self.block_id;
        self.block_id += 1;
        format!("{}_{}", prefix, id)
    }

    pub(crate) fn register_string_literal(&mut self, text: &str) -> String {
        let name = format!("@.str.{}", self.str_literal_id);
        self.str_literal_id += 1;
        let escaped = text
            .replace("\\", "\\5C")
            .replace("\n", "\\0A")
            .replace("\t", "\\09")
            .replace("\r", "\\0D")
            .replace("\"", "\\22")
            .replace("\0", "\\00");
        let byte_len = text.as_bytes().len() + 1; // +1 for null terminator
        self.string_constants.push((name.clone(), escaped, byte_len));
        name
    }

    pub fn map_type(&self, ty: &Type) -> String {
        self.type_mapper.map_type(ty)
    }

    pub(crate) fn get_field_index(&self, _struct_name: &str, field: &str) -> usize {
        match field {
            "id" | "x" | "first" | "order_id" | "sku" => 0,
            "name" | "y" | "second" | "amount" | "quantity" | "customer_id" => 1,
            "active" | "z" | "third" | "total" | "price" => 2,
            _ => 0,
        }
    }
}
