use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod agent_api;
mod ast;
mod codegen;
mod lexer;
mod parser;
mod semantic;

use agent_api::AgentApi;
use codegen::{CBackend, Interpreter};
use lexer::Lexer;
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
    /// Run an End source file directly
    Run {
        /// Path to .end source file
        file: PathBuf,
    },
    /// Compile an End source file to C and native binary
    Build {
        /// Path to .end source file
        file: PathBuf,
        /// Output executable binary path
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Emit C code only
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
    /// Inspect a specific line for symbols, data-flow and side-effects (Agent Protocol)
    Inspect {
        /// Target in format <file.end>:<line_number>
        target: String,
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
        Commands::Build { file, output, emit_c } => {
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let mut backend = CBackend::new();
            let c_code = backend.generate(&module);

            let c_file_path = file.with_extension("c");
            if let Err(e) = fs::write(&c_file_path, &c_code) {
                eprintln!("{} Failed to write C code: {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }

            println!("{} Generated intermediate C code at {:?}", "✔".green().bold(), c_file_path);

            if emit_c {
                return;
            }

            // Try compiling with clang or zig cc or gcc
            let bin_path = output.unwrap_or_else(|| file.with_extension("exe"));
            let mut compiled = false;

            // Try Zig CC if available
            if let Ok(status) = Command::new("zig")
                .args(["cc", c_file_path.to_str().unwrap(), "-O3", "-o", bin_path.to_str().unwrap()])
                .status()
            {
                if status.success() {
                    compiled = true;
                    println!("{} Native binary compiled via Zig CC at {:?}", "✔".green().bold(), bin_path);
                }
            }

            // Try Clang / GCC if Zig failed
            if !compiled {
                if let Ok(status) = Command::new("clang")
                    .args([c_file_path.to_str().unwrap(), "-O3", "-o", bin_path.to_str().unwrap()])
                    .status()
                {
                    if status.success() {
                        compiled = true;
                        println!("{} Native binary compiled via Clang at {:?}", "✔".green().bold(), bin_path);
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
            let parts: Vec<&str> = target.split(':').collect();
            if parts.len() != 2 {
                eprintln!("{} Target must be in format <file.end>:<line_number>", "Error:".red().bold());
                std::process::exit(1);
            }

            let file = PathBuf::from(parts[0]);
            let line: usize = parts[1].parse().unwrap_or(0);

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
