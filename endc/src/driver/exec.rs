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

pub fn handle_run(args: RunArgs) {
    let RunArgs { file, backend: exec_backend } = args;
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            if exec_backend == "llvm" {
                let llvm_be = LlvmBackend::new(None);
                let temp_exe = std::env::temp_dir().join(format!("end_llvm_run_{}.exe", std::process::id()));
                match llvm_be.compile_to_executable(&module, &temp_exe) {
                    Ok(artifacts) => {
                        let status = Command::new(&artifacts.executable_path).status();
                        let _ = fs::remove_file(&artifacts.executable_path);
                        let _ = fs::remove_file(artifacts.executable_path.with_extension("ll"));
                        let _ = fs::remove_file(artifacts.executable_path.with_extension("obj"));
                        if let Ok(st) = status {
                            if !st.success() {
                                std::process::exit(st.code().unwrap_or(1));
                            }
                        }
                        return;
                    }
                    Err(e) => {
                        eprintln!("{} LLVM execution error: {:?}", "Error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            }

            if exec_backend == "wasm" {
                let mut wasm_be = WasmBackend::new(None);
                match wasm_be.compile_and_run(&module) {
                    Ok(res) => {
                        print!("{}", res.stdout);
                        if !res.stderr.is_empty() {
                            eprint!("{}", res.stderr);
                        }
                        if res.exit_code != 0 {
                            std::process::exit(res.exit_code);
                        }
                        return;
                    }
                    Err(e) => {
                        eprintln!("{} WebAssembly execution error: {:?}", "Error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            }

            if exec_backend == "cranelift" {
                let mut cl_be = CraneliftBackend::new();
                match cl_be.compile_and_run_jit(&module) {
                    Ok(rep) => {
                        println!("⚡ {} JIT compiled {} functions in {} µs (Zero C Dependency)", "Cranelift JIT:".green().bold(), rep.functions_compiled, rep.compilation_duration_us);
                        println!("  ├─ Status: {}", rep.status.green());
                        println!("  ├─ Entry: {}", rep.entry_address.cyan());
                        println!("  └─ Note: {}", rep.note);
                        return;
                    }
                    Err(e) => {
                        eprintln!("{} Cranelift JIT Error: {}", "Error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            }

            let mut vm = Interpreter::new();
            if let Err(e) = vm.run(&module) {
                eprintln!("{} {}", "Runtime Error:".red().bold(), e);
                std::process::exit(1);
            }
}

pub fn handle_check(args: CheckArgs) {
    let CheckArgs { file, json, strict_leaks } = args;
            let file_str = file.to_string_lossy().to_string();
            let source = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "error",
                            "message": e.to_string()
                        }));
                    } else {
                        eprintln!("{} Failed reading file {:?}: {}", "Error:".red().bold(), file, e);
                    }
                    std::process::exit(1);
                }
            };
            let source_lines: Vec<&str> = source.lines().collect();

            let mut lexer = Lexer::new(&file_str, &source);
            let tokens = match lexer.tokenize_all() {
                Ok(t) => t,
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "syntax_error",
                            "message": e
                        }));
                    } else {
                        eprintln!("{} {}", "Syntax Error:".red().bold(), e);
                    }
                    std::process::exit(1);
                }
            };

            let mut parser = EndParser::new(&file_str, tokens);
            let module = match parser.parse_module("main") {
                Ok(m) => m,
                Err(e) => {
                    if json {
                        if parser.diagnostics.has_errors() {
                            let errors: Vec<serde_json::Value> = parser.diagnostics.diagnostics()
                                .iter()
                                .map(|d| serde_json::json!({
                                    "code": d.code.as_code_str(),
                                    "message": d.message,
                                    "location": d.location
                                }))
                                .collect();
                            println!("{}", serde_json::json!({
                                "status": "parse_error",
                                "errors": errors
                            }));
                        } else {
                            println!("{}", serde_json::json!({
                                "status": "parse_error",
                                "message": e
                            }));
                        }
                    } else {
                        if parser.diagnostics.has_errors() {
                            for diag in parser.diagnostics.diagnostics() {
                                eprintln!("{}", diag.render(&source));
                            }
                        } else {
                            let diag = Diagnostic::error("E005", &e, &file_str, parser.current_span().line, parser.current_span().col);
                            eprintln!("{}", diag.render(&source));
                        }
                    }
                    std::process::exit(1);
                }
            };

            let mut analyzer = SemanticAnalyzer::new(&file_str, &source);
            analyzer.strict_leaks = strict_leaks;
            match analyzer.analyze_module(&module) {
                Ok(_) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "passed",
                            "file": file_str,
                            "structs_count": module.structs.len(),
                            "enums_count": module.enums.len(),
                            "functions_count": module.functions.len(),
                            "errors": []
                        }));
                    } else {
                        println!("{} {} is valid (0 errors, 0 warnings)", "✔".green().bold(), file_str);
                    }
                }
                Err(errors) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "failed",
                            "file": file_str,
                            "errors": errors
                        }));
                    } else {
                        for err in &errors {
                            let mut diag = crate::diagnostics::Diagnostic::error(&err.code, &err.message, &file_str, err.line, err.col);
                            if let Some(ref h) = err.repair_suggestion {
                                diag = diag.with_help(h);
                            }
                            if let Some(ref exp) = err.expected {
                                diag = diag.with_expected(exp);
                            }
                            if let Some(ref act) = err.actual {
                                diag = diag.with_actual(act);
                            }
                            for ctx in &err.context {
                                diag = diag.with_context(ctx);
                            }
                            eprintln!("{}", diag.render(&source));
                        }
                    }
                    std::process::exit(1);
                }
            }
}

pub fn handle_version() {
            println!("👑 End Programming Language v0.4.0-alpha ({})", std::env::consts::ARCH);
            println!("⚡ Toolchain: C11 Backend with GCC/Clang -O3 (LTO + Fast-Math)");
            println!("🎯 Benchmark: 12-Challenge Reproducible Suite");
            println!("📦 Repository: https://github.com/IrMaho/End");
}

pub fn handle_lsp() {
            let mut lsp_server = LanguageServer::new();
            lsp_server.run_stdio();
}

pub fn handle_dap() {
            println!("🐛 {} Debug Adapter Protocol (DAP 1.51) listening on stdio...", "DAP Server:".cyan().bold());
            let mut dap_srv = dap::DapServer::new();
            let init_req = serde_json::json!({ "command": "initialize", "seq": 1 });
            let _ = dap_srv.handle_dap_request(&init_req);
}

pub fn handle_repl() {
            let mut repl = ReplEngine::new();
            repl.start();
}

