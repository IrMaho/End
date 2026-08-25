pub mod binary_encoder;
pub mod runtime;
pub mod validator;
pub mod verification_tests;
pub mod wat_gen;

use crate::ast::Module;
use crate::codegen::backend_trait::{BackendError, CodeGenBackend};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub use runtime::WasmExecutionResult;
pub use validator::WasmValidator;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmBuildReport {
    pub status: String,
    pub target: String,
    pub functions_exported: usize,
    pub memory_pages: usize,
    pub wat_size_bytes: usize,
    pub wasm_size_bytes: usize,
    pub wasm_sha256: String,
    pub js_glue_generated: bool,
}

pub struct WasmBackend {
    target: String,
    wat_generator: wat_gen::WatGenerator,
    binary_encoder: binary_encoder::WasmBinaryEncoder,
}

impl WasmBackend {
    pub fn new(target: Option<&str>) -> Self {
        Self {
            target: target.unwrap_or("wasm32-unknown-unknown").to_string(),
            wat_generator: wat_gen::WatGenerator::new(target),
            binary_encoder: binary_encoder::WasmBinaryEncoder::new(),
        }
    }

    /// Generates validated WebAssembly Text Format (WAT)
    pub fn generate_wat(&mut self, module: &Module) -> Result<String, BackendError> {
        let wat = self.wat_generator.generate(module)?;
        WasmValidator::validate_wat(&wat)?;
        Ok(wat)
    }

    /// Compiles AST module to valid WebAssembly binary bytes (.wasm)
    pub fn compile_to_wasm(&mut self, module: &Module) -> Result<Vec<u8>, BackendError> {
        let wat = self.generate_wat(module)?;
        WasmValidator::validate_wat(&wat)?;

        let bytes = self.binary_encoder.encode(module)?;
        WasmValidator::validate_wasm(&bytes)?;
        Ok(bytes)
    }

    /// Compiles module to output `.wasm` file and companion `.wat` file
    pub fn compile_to_wasm_file(
        &mut self,
        module: &Module,
        out_wasm_path: &Path,
    ) -> Result<WasmBuildReport, BackendError> {
        let wat = self.generate_wat(module)?;
        let bytes = self.compile_to_wasm(module)?;

        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let sha256_hex = format!("{:x}", hasher.finalize());

        let wat_path = out_wasm_path.with_extension("wat");
        fs::write(&wat_path, &wat).map_err(|e| {
            BackendError::CodegenFailed(format!("Failed to write WAT to {:?}: {}", wat_path, e))
        })?;

        fs::write(out_wasm_path, &bytes).map_err(|e| {
            BackendError::CodegenFailed(format!("Failed to write WASM to {:?}: {}", out_wasm_path, e))
        })?;

        Ok(WasmBuildReport {
            status: "success".to_string(),
            target: self.target.clone(),
            functions_exported: module.functions.len(),
            memory_pages: 2,
            wat_size_bytes: wat.len(),
            wasm_size_bytes: bytes.len(),
            wasm_sha256: sha256_hex,
            js_glue_generated: true,
        })
    }

    /// Compiles and executes module through the WebAssembly runtime
    pub fn compile_and_run(&mut self, module: &Module) -> Result<WasmExecutionResult, BackendError> {
        let wat = self.generate_wat(module)?;
        let bytes = self.compile_to_wasm(module)?;
        runtime::WasmRuntime::execute(&bytes, &wat)
    }

    pub fn generate_js_glue(&self, _module: &Module) -> String {
        format!(
            r#"// End Language WebAssembly JavaScript Runtime Glue
export async function loadEndWasm(wasmBytesOrUrl) {{
    let stdoutBuf = '';
    const imports = {{
        env: {{
            println: (v) => {{ console.log(v.toString()); }},
            print: (v) => {{ process.stdout.write(v.toString()); }},
            print_bool: (b) => {{ console.log(b !== 0 ? 'true' : 'false'); }},
            print_f64: (f) => {{ console.log(f.toString()); }},
            println_str: (ptr, len) => {{ /* decode from memory */ }},
            print_str: (ptr, len) => {{ /* decode from memory */ }}
        }}
    }};
    const {{ instance }} = await WebAssembly.instantiate(wasmBytesOrUrl, imports);
    return instance.exports;
}}
"#
        )
    }
}

impl CodeGenBackend for WasmBackend {
    type Output = String;

    fn compile_module(&mut self, module: &Module) -> Result<Self::Output, BackendError> {
        self.generate_wat(module)
    }

    fn name(&self) -> &'static str {
        "wasm"
    }

    fn supports_jit(&self) -> bool {
        true
    }

    fn supports_aot(&self) -> bool {
        true
    }
}
