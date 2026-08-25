use clap::Args;
use std::path::PathBuf;

    /// Run an End source file directly (Instant Interpreter VM or JIT Backend)
        /// Path to .end source file
        /// Execution backend: vm, cranelift, llvm
#[derive(Args, Debug, Clone)]
pub struct RunArgs {
        pub file: PathBuf,
        #[arg(long, default_value = "vm")]
        pub backend: String,
}

    /// Compile an End source file to an ultra-optimized native binary, DLL, or cross-platform target
        /// Path to .end source file
        /// Output binary or library path (e.g. -o mylib.dll or -o app.exe)
        /// Target architecture & OS triple (e.g. x86_64-windows, x86_64-linux, aarch64-macos, aarch64-linux, wasm32-wasi)
        /// Compile as a shared library / dynamic library (.dll, .so, .dylib) with C-ABI header
        /// Compile as a library (alias for --dll / shared library)
        /// Strip all debug symbols for absolute minimum bare-metal binary size
        /// Emit generated C code and header only
        /// Emit LLVM IR (.ll) code directly (Zero C Dependency)
        /// Emit WebAssembly WAT and glue code (.wat)
        /// Dump WebAssembly WAT format to .wat file
        /// Include full DWARF / CodeView debug symbols in LLVM IR
        /// Dump LLVM IR to .ll file
        /// Dump Cranelift CLIF IR to .clif file
        /// Code generation backend (c, llvm, cranelift, wasm)
        /// Perform binary tree-shaking & dead-code elimination (micro-binary optimization)
        /// Enable AddressSanitizer & UndefinedBehaviorSanitizer
#[derive(Args, Debug, Clone)]
pub struct BuildArgs {
        pub file: PathBuf,
        #[arg(short, long)]
        pub output: Option<PathBuf>,
        #[arg(short, long)]
        pub target: Option<String>,
        #[arg(long, default_value_t = false)]
        pub dll: bool,
        #[arg(long, default_value_t = false)]
        pub lib: bool,
        #[arg(long, default_value_t = true)]
        pub strip: bool,
        #[arg(long)]
        pub emit_c: bool,
        #[arg(long)]
        pub emit_llvm: bool,
        #[arg(long)]
        pub emit_wasm: bool,
        #[arg(long, default_value_t = false)]
        pub dump_wasm_wat: bool,
        #[arg(long, default_value_t = false)]
        pub debug_info: bool,
        #[arg(long, default_value_t = false)]
        pub dump_llvm_ir: bool,
        #[arg(long, default_value_t = false)]
        pub dump_cranelift_clif: bool,
        #[arg(long, default_value = "c")]
        pub backend: String,
        #[arg(long, default_value_t = true)]
        pub tree_shake: bool,
        #[arg(long, default_value_t = false)]
        pub sanitize: bool,
        #[arg(long, default_value_t = false)]
        pub release: bool,
}

    /// Perform fast semantic check and return machine-readable diagnostics
        /// Path to .end source file
        /// Format as JSON for AI Agent
        /// Enforce strict zero memory leaks (fails on escaping pointers and unmanaged allocations)
#[derive(Args, Debug, Clone)]
pub struct CheckArgs {
        pub file: PathBuf,
        #[arg(long, default_value_t = false)]
        pub json: bool,
        #[arg(long, default_value_t = false)]
        pub strict_leaks: bool,
}

