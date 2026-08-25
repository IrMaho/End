use crate::codegen::backend_trait::BackendError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmExecutionResult {
    pub wat_disassembly: String,
    pub wasm_bytes_sha256: String,
    pub wasm_version: String,
    pub executed: bool,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub return_value: Option<i64>,
}

pub struct WasmRuntime;

static WASM_RUN_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

impl WasmRuntime {
    pub fn execute(wasm_bytes: &[u8], wat_text: &str) -> Result<WasmExecutionResult, BackendError> {
        let mut hasher = Sha256::new();
        hasher.update(wasm_bytes);
        let sha256_hex = format!("{:x}", hasher.finalize());

        let temp_dir = std::env::temp_dir().join("end_wasm_runtime");
        let _ = fs::create_dir_all(&temp_dir);

        let run_id = WASM_RUN_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let wasm_file = temp_dir.join(format!("run_{}_{}.wasm", std::process::id(), run_id));
        let runner_script = temp_dir.join(format!("runner_{}_{}.js", std::process::id(), run_id));

        fs::write(&wasm_file, wasm_bytes).map_err(|e| {
            BackendError::CodegenFailed(format!("Failed to write temp WASM binary: {}", e))
        })?;

        let js_runner = r#"
const fs = require('fs');
const wasmPath = process.argv[2];
const wasmBytes = fs.readFileSync(wasmPath);

let stdoutBuf = '';
let stderrBuf = '';

let instanceRef = null;

const imports = {
    env: {
        println: (v) => { stdoutBuf += v.toString() + '\n'; },
        print: (v) => { stdoutBuf += v.toString(); },
        print_bool: (b) => { stdoutBuf += (b !== 0 ? 'true\n' : 'false\n'); },
        print_f64: (f) => { stdoutBuf += f.toString() + '\n'; },
        println_str: (ptr, len) => {
            try {
                const mem = instanceRef ? instanceRef.exports.memory : null;
                if (mem) {
                    const u8 = new Uint8Array(mem.buffer, Number(ptr), Number(len));
                    const s = Buffer.from(u8).toString('utf8');
                    stdoutBuf += s + '\n';
                }
            } catch (e) {
                stderrBuf += e.toString() + '\n';
            }
        },
        print_str: (ptr, len) => {
            try {
                const mem = instanceRef ? instanceRef.exports.memory : null;
                if (mem) {
                    const u8 = new Uint8Array(mem.buffer, Number(ptr), Number(len));
                    const s = Buffer.from(u8).toString('utf8');
                    stdoutBuf += s;
                }
            } catch (e) {
                stderrBuf += e.toString() + '\n';
            }
        }
    }
};

WebAssembly.instantiate(wasmBytes, imports).then(({ instance }) => {
    instanceRef = instance;
    let ret = 0;
    if (typeof instance.exports.main === 'function') {
        ret = instance.exports.main();
    }
    process.stdout.write(stdoutBuf);
    process.stderr.write(stderrBuf);
    process.exit(typeof ret === 'number' || typeof ret === 'bigint' ? Number(ret) : 0);
}).catch(err => {
    process.stderr.write('WASM Trap/Error: ' + err.toString() + '\n');
    process.exit(1);
});
"#;

        fs::write(&runner_script, js_runner).map_err(|e| {
            BackendError::CodegenFailed(format!("Failed to write WASM runner script: {}", e))
        })?;

        let output = Command::new("node")
            .arg(&runner_script)
            .arg(&wasm_file)
            .output()
            .map_err(|e| {
                BackendError::CodegenFailed(format!("Failed to invoke WebAssembly runtime: {}", e))
            })?;

        // Cleanup temp files
        let _ = fs::remove_file(&wasm_file);
        let _ = fs::remove_file(&runner_script);

        let stdout_str = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr_str = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(0);

        if exit_code != 0 && !stderr_str.is_empty() {
            eprintln!("[WASM Runtime Error]: {}", stderr_str);
        }

        Ok(WasmExecutionResult {
            wat_disassembly: wat_text.to_string(),
            wasm_bytes_sha256: sha256_hex,
            wasm_version: "WebAssembly 1.0 (MVP)".to_string(),
            executed: true,
            stdout: stdout_str,
            stderr: stderr_str,
            exit_code,
            return_value: Some(exit_code as i64),
        })
    }
}
