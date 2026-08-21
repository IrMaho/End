use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod agent_api;
mod ast;
mod codegen;
mod lexer;
mod lsp;
mod parser;
mod semantic;

use agent_api::AgentApi;
use codegen::{CBackend, Interpreter};
use lexer::Lexer;
use lsp::LanguageServer;
use parser::Parser as EndParser;
use semantic::SemanticAnalyzer;

#[derive(Parser)]
#[command(name = "end")]
#[command(about = "The End Programming Language Toolchain & Agent Knowledge Engine", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an End source file directly (Instant Interpreter VM)
    Run {
        /// Path to .end source file
        file: PathBuf,
    },
    /// Compile an End source file to an ultra-optimized native binary, DLL, or cross-platform target
    Build {
        /// Path to .end source file
        file: PathBuf,
        /// Output binary or library path (e.g. -o mylib.dll or -o app.exe)
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Target architecture & OS triple (e.g. x86_64-windows, x86_64-linux, aarch64-macos, aarch64-linux, wasm32-wasi)
        #[arg(short, long)]
        target: Option<String>,
        /// Compile as a shared library / dynamic library (.dll, .so, .dylib) with C-ABI header
        #[arg(long, default_value_t = false)]
        dll: bool,
        /// Compile as a library (alias for --dll / shared library)
        #[arg(long, default_value_t = false)]
        lib: bool,
        /// Strip all debug symbols for absolute minimum bare-metal binary size
        #[arg(long, default_value_t = true)]
        strip: bool,
        /// Emit generated C code and header only
        #[arg(long)]
        emit_c: bool,
    },
    /// Perform fast semantic check and return machine-readable diagnostics
    Check {
        /// Path to .end source file
        file: PathBuf,
        /// Format as JSON for AI Agent
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    /// Start Language Server Protocol (LSP) for VS Code and IDE integrations
    Lsp,
    /// Inspect a specific line for symbols, data-flow and side-effects (Agent Protocol)
    Inspect {
        /// Target in format <file.end>:<line_number>
        target: String,
    },
    /// Explain a specific line with human & agent-friendly semantic breakdown
    Explain {
        /// Target in format <file.end>:<line_number>
        target: String,
    },
    /// Trace timeline and data-flow of a symbol across its entire lifecycle
    Trace {
        /// Path to .end source file
        file: PathBuf,
        /// Symbol to trace
        symbol: String,
    },
    /// Query side-effects and capabilities for a symbol
    Effects {
        /// Path to .end source file
        file: PathBuf,
        /// Symbol name
        symbol: String,
    },
    /// Perform impact analysis before modifying a symbol (Agent Protocol)
    Impact {
        /// Path to .end source file
        file: PathBuf,
        /// Symbol to analyze (e.g. function or struct name)
        symbol: String,
    },
    /// Query semantic information about a symbol
    Query {
        /// Path to .end source file
        file: PathBuf,
        /// Symbol name to query
        symbol: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Lsp => {
            let mut lsp_server = LanguageServer::new();
            lsp_server.run_stdio();
        }
        Commands::Run { file } => {
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let mut vm = Interpreter::new();
            if let Err(e) = vm.run(&module) {
                eprintln!("{} {}", "Runtime Error:".red().bold(), e);
                std::process::exit(1);
            }
        }
        Commands::Build {
            file,
            output,
            target,
            dll,
            lib,
            strip,
            emit_c,
        } => {
            let is_library_mode = dll || lib;
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

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
                c_file_path.to_str().unwrap().to_string(),
                "-O3".to_string(),
                "-funroll-loops".to_string(),
                "-fomit-frame-pointer".to_string(),
            ];

            if is_library_mode {
                zig_args.push("-shared".to_string());
                zig_args.push("-fPIC".to_string());
            }

            if strip {
                zig_args.push("-s".to_string());
            }

            if let Some(ref t) = target {
                zig_args.push("-target".to_string());
                zig_args.push(t.clone());
            } else {
                zig_args.push("-march=native".to_string());
            }

            zig_args.push("-o".to_string());
            zig_args.push(bin_path.to_str().unwrap().to_string());

            let mut compiled = false;

            // Execute Zig CC for cross-platform bare-metal compilation
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

            if !compiled {
                println!(
                    "{} C code is ready at {:?}. To compile natively, run: `zig cc {:?} -o {:?}`",
                    "ℹ".cyan().bold(),
                    c_file_path,
                    c_file_path,
                    bin_path
                );
            }
        }
        Commands::Check { file, json } => {
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
                        eprintln!("{} Cannot read file: {}", "Error:".red().bold(), e);
                    }
                    std::process::exit(1);
                }
            };

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
                        println!("{}", serde_json::json!({
                            "status": "parse_error",
                            "message": e
                        }));
                    } else {
                        eprintln!("{} {}", "Parse Error:".red().bold(), e);
                    }
                    std::process::exit(1);
                }
            };

            let mut analyzer = SemanticAnalyzer::new(&file_str, &source);
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
                        eprintln!("{} {} semantic error(s) found in {}:", "✖".red().bold(), errors.len(), file_str);
                        for err in errors {
                            eprintln!("  - [{}:{}:{}] {}: {}", err.code, err.line, err.col, err.kind, err.message);
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
        Commands::Inspect { target } => {
            let (file, line) = parse_file_line(&target);
            let (_, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let api = AgentApi::new(&analyzer.graph);
            let result = api.inspect_line(line);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Commands::Explain { target } => {
            let (file, line) = parse_file_line(&target);
            let (_, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let api = AgentApi::new(&analyzer.graph);
            let result = api.explain_line(line);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Commands::Trace { file, symbol } => {
            let (_, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let api = AgentApi::new(&analyzer.graph);
            let result = api.trace_symbol(&symbol);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Commands::Effects { file, symbol } => {
            let (_, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let api = AgentApi::new(&analyzer.graph);
            let result = api.query_effects(&symbol);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Commands::Impact { file, symbol } => {
            let (_, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let api = AgentApi::new(&analyzer.graph);
            let result = api.impact_analysis(&symbol);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
        Commands::Query { file, symbol } => {
            let (_, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let api = AgentApi::new(&analyzer.graph);
            let result = api.query_symbol(&symbol);
            println!("{}", serde_json::to_string_pretty(&result).unwrap());
        }
    }
}

fn parse_file_line(target: &str) -> (PathBuf, usize) {
    let parts: Vec<&str> = target.split(':').collect();
    if parts.len() != 2 {
        eprintln!("{} Target must be in format <file.end>:<line_number>", "Error:".red().bold());
        std::process::exit(1);
    }
    (PathBuf::from(parts[0]), parts[1].parse().unwrap_or(0))
}

fn load_and_analyze(file: &PathBuf) -> Result<(ast::Module, SemanticAnalyzer), String> {
    let file_str = file.to_string_lossy().to_string();
    let source = fs::read_to_string(file).map_err(|e| format!("Failed to read file: {}", e))?;

    let mut lexer = Lexer::new(&file_str, &source);
    let tokens = lexer.tokenize_all()?;

    let mut parser = EndParser::new(&file_str, tokens);
    let module = parser.parse_module("main")?;

    let mut analyzer = SemanticAnalyzer::new(&file_str, &source);
    analyzer
        .analyze_module(&module)
        .map_err(|errs| format!("Semantic errors: {:?}", errs))?;

    Ok((module, analyzer))
}
