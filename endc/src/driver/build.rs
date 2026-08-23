use colored::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::agent_api::*;
use crate::architecture::*;
use crate::ast;
use crate::bindgen;
use crate::bindgen::*;
use crate::codegen;
use crate::codegen::*;
use crate::config::CompilerConfig;
use crate::dap;
use crate::dap::*;
use crate::diagnostics::Diagnostic;
use crate::docgen;
use crate::docgen::server::DocServer;
use crate::docgen::*;
use crate::formatter;
use crate::formatter::*;
use crate::fuzz;
use crate::fuzz::*;
use crate::lexer::Lexer;
use crate::linter;
use crate::linter::Linter;
use crate::loader::*;
use crate::lsp;
use crate::lsp::*;
use crate::mobile;
use crate::mobile::*;
use crate::package;
use crate::package::*;
use crate::parser::Parser as EndParser;
use crate::profiler;
use crate::repl;
use crate::repl::*;
use crate::runtime::*;
use crate::security;
use crate::security::*;
use crate::semantic::analyzer::SemanticAnalyzer;
use crate::semantic::*;
use crate::ui::feedback::FeedbackManager;
use crate::ui::*;

use crate::cli::agent_args::*;
use crate::cli::build_args::*;
use crate::cli::dev_args::*;
use crate::cli::package_args::*;
use crate::cli::query_args::*;
use crate::cli::security_args::*;

pub fn handle_build(args: BuildArgs) {
    let BuildArgs { file,
            output,
            target,
            dll,
            lib,
            strip,
            emit_c,
            emit_llvm,
            emit_wasm,
            dump_wasm_wat,
            debug_info,
            dump_llvm_ir,
            dump_cranelift_clif,
            backend: backend_choice,
            tree_shake,
            sanitize, } = args;
            let is_library_mode = dll || lib;
            let (raw_module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let module = if tree_shake {
                let (pruned, count) = TreeShaker::prune_unreachable(&raw_module);
                if count > 0 {
                    println!("✂️  {} Tree-shaking eliminated {} unused symbols (<15KB micro-binary)", "Optimizer:".green().bold(), count);
                }
                pruned
            } else {
                raw_module
            };

            if dump_cranelift_clif {
                let mut cl_be = CraneliftBackend::new();
                match cl_be.generate_clif_ir(&module) {
                    Ok(clif_ir) => {
                        let clif_file_path = file.with_extension("clif");
                        if let Err(e) = fs::write(&clif_file_path, &clif_ir) {
                            eprintln!("{} Failed to write Cranelift CLIF IR: {}", "Error:".red().bold(), e);
                        } else {
                            println!("{} Dumped Cranelift CLIF IR at {:?}", "✔".green().bold(), clif_file_path);
                        }
                    }
                    Err(e) => {
                        eprintln!("{} Cranelift IR Generation Error: {}", "Error:".red().bold(), e);
                    }
                }
            }

            if emit_llvm || dump_llvm_ir || backend_choice == "llvm" {
                let mut llvm_be = LlvmBackend::new(target.as_deref());
                llvm_be.set_debug_info(debug_info);
                match llvm_be.generate_llvm_ir(&module) {
                    Ok(llvm_ir) => {
                        let ll_file_path = file.with_extension("ll");
                        if let Err(e) = fs::write(&ll_file_path, &llvm_ir) {
                            eprintln!("{} Failed to write LLVM IR: {}", "Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                        println!("{} Generated direct LLVM IR at {:?}", "✔".green().bold(), ll_file_path);
                        if emit_llvm || dump_llvm_ir {
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("{} LLVM Codegen Error: {}", "Error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            }

            if emit_wasm || dump_wasm_wat || backend_choice == "wasm" {
                let mut wasm_be = WasmBackend::new(target.as_deref());
                match wasm_be.generate_wat(&module) {
                    Ok(wat_content) => {
                        let wat_file_path = file.with_extension("wat");
                        if let Err(e) = fs::write(&wat_file_path, &wat_content) {
                            eprintln!("{} Failed to write WebAssembly WAT: {}", "Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                        let js_glue = wasm_be.generate_js_glue(&module);
                        let js_file_path = file.with_extension("js");
                        let _ = fs::write(&js_file_path, &js_glue);
                        println!("{} Generated WebAssembly WAT at {:?} (and JS runtime glue)", "✔".green().bold(), wat_file_path);
                        if emit_wasm || dump_wasm_wat {
                            return;
                        }
                    }
                    Err(e) => {
                        eprintln!("{} WebAssembly Error: {}", "Error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            }

            if backend_choice == "cranelift" {
                let mut cl_be = CraneliftBackend::new();
                match cl_be.compile_and_run_jit(&module) {
                    Ok(rep) => {
                        println!("⚡ {} JIT compiled {} functions in {} µs (Zero C Dependency)", "Cranelift:".green().bold(), rep.functions_compiled, rep.compilation_duration_us);
                        return;
                    }
                    Err(e) => {
                        eprintln!("{} Cranelift error: {}", "Error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            }

            let mut backend = CBackend::new();
            let (c_code, header_code) = backend.generate_with_options(&module, is_library_mode);

            let c_file_path = file.with_extension("c");
            if let Err(e) = fs::write(&c_file_path, &c_code) {
                eprintln!("{} Failed to write C code: {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
            println!("{} Generated C source at {:?}", "✔".green().bold(), c_file_path);

            if let Some(hdr) = header_code {
                let h_file_path = file.with_extension("h");
                if let Err(e) = fs::write(&h_file_path, &hdr) {
                    eprintln!("{} Failed to write C header: {}", "Error:".red().bold(), e);
                } else {
                    println!("{} Generated C Header (FFI API) at {:?}", "✔".green().bold(), h_file_path);
                }
            }

            if emit_c {
                return;
            }

            // Determine default output extension
            let default_ext = if is_library_mode {
                if let Some(ref t) = target {
                    if t.contains("windows") {
                        "dll"
                    } else if t.contains("macos") || t.contains("darwin") {
                        "dylib"
                    } else {
                        "so"
                    }
                } else {
                    #[cfg(target_os = "windows")]
                    let ext = "dll";
                    #[cfg(target_os = "macos")]
                    let ext = "dylib";
                    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
                    let ext = "so";
                    ext
                }
            } else {
                #[cfg(target_os = "windows")]
                let ext = "exe";
                #[cfg(not(target_os = "windows"))]
                let ext = "";
                ext
            };

            let bin_path = output.unwrap_or_else(|| {
                if default_ext.is_empty() {
                    file.with_extension("")
                } else {
                    file.with_extension(default_ext)
                }
            });

            // Build compiler args
            let mut zig_args: Vec<String> = vec![
                "cc".to_string(),
                #[cfg(windows)]
                "-target".to_string(),
                #[cfg(windows)]
                "x86_64-windows-gnu".to_string(),
                c_file_path.to_str().unwrap().to_string(),
                "-O3".to_string(),
                "-march=native".to_string(),
                "-funroll-loops".to_string(),
                "-fomit-frame-pointer".to_string(),
                "-finline-functions".to_string(),
                "-fno-math-errno".to_string(),
                "-fno-trapping-math".to_string(),
                "-ffp-contract=fast".to_string(),
                "-freciprocal-math".to_string(),
                "-fwrapv".to_string(),
            ];

            if is_library_mode {
                zig_args.push("-shared".to_string());
                zig_args.push("-fPIC".to_string());
            }

            if strip {
                zig_args.push("-s".to_string());
                zig_args.push("-ffunction-sections".to_string());
                zig_args.push("-fdata-sections".to_string());
            }

            if sanitize {
                zig_args.push("-fsanitize=address,undefined".to_string());
            }

            if let Some(ref t) = target {
                zig_args.push("-target".to_string());
                zig_args.push(t.clone());
                if t.contains("windows") {
                    zig_args.push("-lgdi32".to_string());
                    zig_args.push("-luser32".to_string());
                    zig_args.push("-lws2_32".to_string());
                }
            } else {
                #[cfg(windows)]
                {
                    zig_args.push("-lgdi32".to_string());
                    zig_args.push("-luser32".to_string());
                    zig_args.push("-lws2_32".to_string());
                }
                #[cfg(not(windows))]
                {
                    zig_args.push("-march=native".to_string());
                }
            }

            zig_args.push("-o".to_string());
            zig_args.push(bin_path.to_str().unwrap().to_string());

            let mut compiled = false;

            // 1. Try native GCC with whole-program LTO for peak bare-metal performance
            if target.is_none() {
                let mut gcc_args = vec![
                    "-O3".to_string(),
                    "-march=native".to_string(),
                    "-flto".to_string(),
                    "-funroll-loops".to_string(),
                    "-fomit-frame-pointer".to_string(),
                    "-finline-functions".to_string(),
                    "-Wno-incompatible-pointer-types".to_string(),
                    "-fno-math-errno".to_string(),
                    "-ffast-math".to_string(),
                    c_file_path.to_str().unwrap().to_string(),
                ];
                if is_library_mode {
                    gcc_args.push("-shared".to_string());
                    gcc_args.push("-fPIC".to_string());
                }
                if strip {
                    gcc_args.push("-s".to_string());
                }
                #[cfg(windows)]
                {
                    gcc_args.push("-lws2_32".to_string());
                    gcc_args.push("-lgdi32".to_string());
                    gcc_args.push("-luser32".to_string());
                }
                gcc_args.push("-o".to_string());
                gcc_args.push(bin_path.to_str().unwrap().to_string());

                let gcc_refs: Vec<&str> = gcc_args.iter().map(|s| s.as_str()).collect();
                if let Ok(status) = Command::new("gcc").args(&gcc_refs).status() {
                    if status.success() {
                        compiled = true;
                        let target_name = "Host Native (GCC LTO)";
                        if is_library_mode {
                            println!(
                                "{} Shared Library / DLL compiled for [{}] at {:?}",
                                "👑".green().bold(),
                                target_name.cyan().bold(),
                                bin_path
                            );
                        } else {
                            println!(
                                "{} Native Binary compiled for [{}] (Ultra-Optimized) at {:?}",
                                "👑".green().bold(),
                                target_name.cyan().bold(),
                                bin_path
                            );
                        }
                    }
                }
            }

            // 2. Fallback to Zig CC for cross-platform bare-metal compilation
            if !compiled {
                let zig_args_refs: Vec<&str> = zig_args.iter().map(|s| s.as_str()).collect();
                if let Ok(status) = Command::new("zig").args(&zig_args_refs).status() {
                    if status.success() {
                        compiled = true;
                        let target_name = target.as_deref().unwrap_or("Host Native");
                        if is_library_mode {
                            println!(
                                "{} Shared Library / DLL compiled for [{}] at {:?}",
                                "👑".green().bold(),
                                target_name.cyan().bold(),
                                bin_path
                            );
                        } else {
                            println!(
                                "{} Native Binary compiled for [{}] (Ultra-Optimized) at {:?}",
                                "👑".green().bold(),
                                target_name.cyan().bold(),
                                bin_path
                            );
                        }
                    }
                }
            }

            // Fallback to Clang if Zig CC was not found
            if !compiled {
                let mut clang_args = vec![
                    c_file_path.to_str().unwrap().to_string(),
                    "-O3".to_string(),
                    "-funroll-loops".to_string(),
                    "-fomit-frame-pointer".to_string(),
                ];
                if is_library_mode {
                    clang_args.push("-shared".to_string());
                }
                if strip {
                    clang_args.push("-s".to_string());
                    clang_args.push("-flto".to_string());
                }
                if sanitize {
                    clang_args.push("-fsanitize=address,undefined".to_string());
                }
                clang_args.push("-o".to_string());
                clang_args.push(bin_path.to_str().unwrap().to_string());

                let clang_refs: Vec<&str> = clang_args.iter().map(|s| s.as_str()).collect();
                if let Ok(status) = Command::new("clang").args(&clang_refs).status() {
                    if status.success() {
                        compiled = true;
                        println!("{} Compiled via Clang at {:?}", "✔".green().bold(), bin_path);
                    }
                }
            }

            // Fallback to GCC if Clang / Zig CC failed
            if !compiled {
                let mut gcc_args = vec![
                    "-O3".to_string(),
                    "-march=native".to_string(),
                    "-flto".to_string(),
                    "-funroll-loops".to_string(),
                    "-fomit-frame-pointer".to_string(),
                    "-finline-functions".to_string(),
                    "-fno-math-errno".to_string(),
                    "-Wno-incompatible-pointer-types".to_string(),
                    c_file_path.to_str().unwrap().to_string(),
                ];
                if is_library_mode {
                    gcc_args.push("-shared".to_string());
                }
                if strip {
                    gcc_args.push("-s".to_string());
                }
                #[cfg(windows)]
                {
                    gcc_args.push("-lws2_32".to_string());
                    gcc_args.push("-luser32".to_string());
                    gcc_args.push("-lgdi32".to_string());
                }
                gcc_args.push("-o".to_string());
                gcc_args.push(bin_path.to_str().unwrap().to_string());

                let gcc_refs: Vec<&str> = gcc_args.iter().map(|s| s.as_str()).collect();
                if let Ok(status) = Command::new("gcc").args(&gcc_refs).status() {
                    if status.success() {
                        compiled = true;
                        println!("{} Native Binary compiled via GCC (Ultra-Optimized) at {:?}", "👑".green().bold(), bin_path);
                    }
                }
            }

            if !compiled {
                println!(
                    "{} C code is ready at {:?}. To compile natively, run: `gcc -O3 {:?} -o {:?}`",
                    "ℹ".cyan().bold(),
                    c_file_path,
                    c_file_path,
                    bin_path
                );
            }
}

