use std::path::{Path, PathBuf};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct LlvmBuildArtifacts {
    pub ir_path: PathBuf,
    pub ir_disassembly: String,
    pub ir_sha256: String,
    pub opt_stderr: String,
    pub llc_stderr: String,
    pub object_path: PathBuf,
    pub object_sha256: String,
    pub executable_path: PathBuf,
    pub executable_sha256: String,
    pub llvm_version: String,
}

pub struct LlvmBackend {
    pub target_triple: String,
    pub emit_debug_info: bool,
    pub opt_level: String,
}

impl LlvmBackend {
    pub fn new(target_triple: Option<&str>) -> Self {
        Self {
            target_triple: target_triple.unwrap_or(Self::detect_host_triple()).to_string(),
            emit_debug_info: true,
            opt_level: "-O2".to_string(),
        }
    }

    pub fn set_debug_info(&mut self, enabled: bool) {
        self.emit_debug_info = enabled;
    }

    pub fn set_opt_level(&mut self, opt: &str) {
        self.opt_level = opt.to_string();
    }

    pub fn detect_host_triple() -> &'static str {
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

    pub fn compute_sha256(bytes: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        format!("{:x}", hasher.finalize())
    }
}
