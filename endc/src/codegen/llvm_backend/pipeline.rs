use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use inkwell::context::Context;
use crate::ast::Module;
use crate::codegen::backend_trait::BackendError;
use super::state::{LlvmBackend, LlvmBuildArtifacts};
use super::module_gen::LlvmLoweringContext;

impl LlvmBackend {
    pub fn generate_llvm_ir(&self, module: &Module) -> Result<String, BackendError> {
        let context = Context::create();
        let llvm_module = context.create_module(&module.name);
        llvm_module.set_triple(&inkwell::targets::TargetTriple::create(&self.target_triple));
        let builder = context.create_builder();

        let mut lowering_ctx = LlvmLoweringContext::new(
            &context,
            &llvm_module,
            &builder,
            self.emit_debug_info,
        );

        lowering_ctx.lower_module(module)?;

        Ok(llvm_module.print_to_string().to_string())
    }

    pub fn compile_to_object(
        &self,
        module: &Module,
        output_obj: &Path,
    ) -> Result<PathBuf, BackendError> {
        let ir_content = self.generate_llvm_ir(module)?;
        let temp_ll = output_obj.with_extension("ll");
        fs::write(&temp_ll, &ir_content).map_err(|e| {
            BackendError::CodegenFailed(format!("Failed to write LLVM IR to {:?}: {}", temp_ll, e))
        })?;

        let clang_bin = Self::find_clang_binary();
        let mut cmd = Command::new(&clang_bin);
        cmd.arg(&self.opt_level)
            .arg("-c")
            .arg(&temp_ll)
            .arg("-o")
            .arg(output_obj);

        let output = cmd.output().map_err(|e| {
            BackendError::CodegenFailed(format!("Failed to invoke clang for object generation: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(BackendError::CodegenFailed(format!(
                "clang object generation failed (code {:?}):\n{}",
                output.status.code(),
                stderr
            )));
        }

        Ok(output_obj.to_path_buf())
    }

    pub fn compile_to_executable(
        &self,
        module: &Module,
        output_exe: &Path,
    ) -> Result<LlvmBuildArtifacts, BackendError> {
        let ir_content = self.generate_llvm_ir(module)?;
        let ir_path = output_exe.with_extension("ll");
        fs::write(&ir_path, &ir_content).map_err(|e| {
            BackendError::CodegenFailed(format!("Failed to write LLVM IR to {:?}: {}", ir_path, e))
        })?;

        let ir_sha256 = Self::compute_sha256(ir_content.as_bytes());

        #[cfg(windows)]
        let obj_path = output_exe.with_extension("obj");
        #[cfg(not(windows))]
        let obj_path = output_exe.with_extension("o");

        let clang_bin = Self::find_clang_binary();

        // 1. Lower LLVM IR to Object file (opt + llc pipeline inside clang)
        let mut opt_llc_cmd = Command::new(&clang_bin);
        opt_llc_cmd
            .arg(&self.opt_level)
            .arg("-c")
            .arg(&ir_path)
            .arg("-o")
            .arg(&obj_path);

        let opt_llc_res = opt_llc_cmd.output().map_err(|e| {
            BackendError::CodegenFailed(format!("Failed to invoke clang compiler driver: {}", e))
        })?;

        let opt_stderr = String::from_utf8_lossy(&opt_llc_res.stderr).to_string();

        if !opt_llc_res.status.success() {
            return Err(BackendError::CodegenFailed(format!(
                "LLVM optimization & lowering failed (code {:?}):\n{}",
                opt_llc_res.status.code(),
                opt_stderr
            )));
        }

        let obj_bytes = fs::read(&obj_path).unwrap_or_default();
        let object_sha256 = Self::compute_sha256(&obj_bytes);

        // 2. Link Object file into Native Executable
        let mut link_cmd = Command::new(&clang_bin);
        link_cmd
            .arg(&obj_path)
            .arg("-o")
            .arg(output_exe);

        #[cfg(windows)]
        {
            link_cmd.arg("-lws2_32").arg("-lgdi32").arg("-luser32");
        }

        let link_res = link_cmd.output().map_err(|e| {
            BackendError::LinkerFailed(format!("Failed to invoke linker: {}", e))
        })?;

        let link_stderr = String::from_utf8_lossy(&link_res.stderr).to_string();

        if !link_res.status.success() {
            // Fallback to gcc if clang linker fails in MinGW environment
            let mut gcc_cmd = Command::new("gcc");
            gcc_cmd
                .arg(&obj_path)
                .arg("-o")
                .arg(output_exe);

            #[cfg(windows)]
            {
                gcc_cmd.arg("-lws2_32").arg("-lgdi32").arg("-luser32");
            }

            let gcc_res = gcc_cmd.output().map_err(|e| {
                BackendError::LinkerFailed(format!("Fallback GCC linker failed to execute: {}", e))
            })?;

            if !gcc_res.status.success() {
                return Err(BackendError::LinkerFailed(format!(
                    "Linker failed (code {:?}):\nClang: {}\nGCC: {}",
                    link_res.status.code(),
                    link_stderr,
                    String::from_utf8_lossy(&gcc_res.stderr)
                )));
            }
        }

        let exe_bytes = fs::read(output_exe).unwrap_or_default();
        let executable_sha256 = Self::compute_sha256(&exe_bytes);

        let llvm_version = Self::get_toolchain_version(&clang_bin);

        Ok(LlvmBuildArtifacts {
            ir_path,
            ir_disassembly: ir_content,
            ir_sha256,
            opt_stderr,
            llc_stderr: link_stderr,
            object_path: obj_path,
            object_sha256,
            executable_path: output_exe.to_path_buf(),
            executable_sha256,
            llvm_version,
        })
    }

    pub fn find_clang_binary() -> String {
        let candidates = [
            "C:\\Program Files\\LLVM\\bin\\clang.exe",
            "C:\\Program Files (x86)\\LLVM\\bin\\clang.exe",
            "clang.exe",
            "clang",
        ];

        for c in &candidates {
            if let Ok(meta) = fs::metadata(c) {
                if meta.is_file() {
                    return c.to_string();
                }
            }
        }

        "clang".to_string()
    }

    pub fn get_toolchain_version(bin: &str) -> String {
        if let Ok(out) = Command::new(bin).arg("--version").output() {
            let s = String::from_utf8_lossy(&out.stdout);
            if let Some(line) = s.lines().next() {
                return line.trim().to_string();
            }
        }
        "LLVM 22.1.8".to_string()
    }
}
