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

pub fn handle_inspect(args: InspectArgs) {
    let InspectArgs { target } = args;
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

pub fn handle_explain(args: ExplainArgs) {
    let ExplainArgs { target } = args;
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

pub fn handle_trace(args: TraceArgs) {
    let TraceArgs { file, symbol } = args;
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

pub fn handle_effects(args: EffectsArgs) {
    let EffectsArgs { file, symbol } = args;
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

pub fn handle_impact(args: ImpactArgs) {
    let ImpactArgs { file, symbol } = args;
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

pub fn handle_graph(args: GraphArgs) {
    let GraphArgs { file, json } = args;
            let (_, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let api = AgentApi::new(&analyzer.graph);
            let result = api.knowledge_graph();
            if json {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("🗺️ {} for {:?}", "End Machine Knowledge Graph".cyan().bold(), file);
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            }
}

pub fn handle_query(args: QueryArgs) {
    let QueryArgs { file, symbol, callers, callees, json } = args;
            let (_, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let api = AgentApi::new(&analyzer.graph);
            let result = if callers {
                api.query_callers(&symbol)
            } else if callees {
                api.query_callees(&symbol)
            } else {
                api.query_symbol(&symbol)
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            } else {
                println!("{}", serde_json::to_string_pretty(&result).unwrap());
            }
}

pub fn handle_slice(args: SliceArgs) {
    let SliceArgs { file, interface_only, types_only, budget, json } = args;
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            if json {
                let json_slice = SemanticCodeSlicer::slice_json(&module, budget);
                println!("{}", serde_json::to_string_pretty(&json_slice).unwrap());
            } else {
                let text_slice = SemanticCodeSlicer::slice_module(&module, interface_only, types_only, budget);
                println!("{}", text_slice);
            }
}

pub fn handle_patch(args: PatchArgs) {
    let PatchArgs { file, ast_patch, json_input, apply } = args;
            let file_str = file.to_string_lossy().to_string();
            let source = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{} Failed reading file {:?}: {}", "Error:".red().bold(), file, e);
                    std::process::exit(1);
                }
            };

            let patch_json_str = if let Some(ref p) = ast_patch {
                match fs::read_to_string(p) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{} Failed reading patch file {:?}: {}", "Error:".red().bold(), p, e);
                        std::process::exit(1);
                    }
                }
            } else if let Some(ref s) = json_input {
                s.clone()
            } else {
                eprintln!("{} Must provide either --ast-patch <file.json> or --json-input <json>", "Error:".red().bold());
                std::process::exit(1);
            };

            match StructuredAstPatcher::apply_patch_json(&source, &patch_json_str) {
                Ok(report) => {
                    println!("==================================================");
                    println!("🛠️ {} for `{}`", "Structured AST Patch Report".green().bold(), file_str.yellow());
                    println!("==================================================");
                    println!("  Action:  {}", report.action.cyan());
                    println!("  Target:  {}", report.target.cyan());
                    println!("  Lines:   {} -> {}", report.original_lines_count, report.patched_lines_count);
                    println!("  Status:  {}", "✔ Validated AST Node".green());

                    if apply {
                        if let Err(e) = fs::write(&file, &report.patched_source) {
                            eprintln!("{} Failed to apply patch to {:?}: {}", "Error:".red().bold(), file, e);
                            std::process::exit(1);
                        }
                        println!("\n{} Successfully applied patch to `{}`", "✔".green().bold(), file_str.cyan());
                    } else {
                        println!("\n{} Proposed patch is valid. Run with {} to apply directly.", "ℹ".blue().bold(), "--apply".yellow());
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "AST Patch Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
}

pub fn handle_eval(args: EvalArgs) {
    let EvalArgs { expression, json } = args;
            match MicroEvaluator::eval_expression(&expression) {
                Ok(eval_res) => {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&eval_res).unwrap());
                    } else {
                        println!("⚡ {} {}", "Evaluated:".green().bold(), eval_res.result.cyan().bold());
                        println!("  Type:     {}", eval_res.value_type);
                        println!("  Duration: {} µs", eval_res.duration_us.to_string().yellow());
                    }
                }
                Err(e) => {
                    if json {
                        println!("{}", serde_json::json!({
                            "status": "error",
                            "error": e
                        }));
                    } else {
                        eprintln!("{} {}", "Eval Error:".red().bold(), e);
                    }
                    std::process::exit(1);
                }
            }
}

pub fn handle_arch(args: ArchArgs) {
    let ArchArgs { action: _, config, json } = args;
            let arch_cfg = match ArchitectureEngine::load_config(config.as_deref()) {
                Ok(cfg) => cfg,
                Err(e) => {
                    eprintln!("{} {}", "Architecture Config Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let report = match ArchitectureEngine::check_project(&arch_cfg, std::path::Path::new(".")) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("{} {}", "Architecture Check Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap());
            } else {
                println!("🛡️ {}", "End Enterprise Architecture Invariant Validator".cyan().bold());
                println!("================================================================================");
                println!("  Rules Checked:  {}", report.rules_checked);
                println!("  Files Scanned:  {}", report.files_scanned);
                println!("  Violations:     {}", report.violations_count);

                if report.violations.is_empty() {
                    println!("\n{} 100% Architectural Invariants Respected (0 Violations)!", "✔".green().bold());
                } else {
                    println!("\n{} {} Architecture Violation(s) Found:\n", "✖".red().bold(), report.violations.len());
                    for v in &report.violations {
                        println!("  ✖ [{}] Rule: `{}` in `{}:{}`", v.violation_type.red().bold(), v.rule_pattern.yellow(), v.file.cyan(), v.line);
                        println!("    Message: {}", v.message);
                        println!("    Suggested Fix: {}\n", v.suggested_alternative.green());
                    }
                    std::process::exit(1);
                }
            }
}

pub fn handle_fix(args: FixArgs) {
    let FixArgs { file, apply } = args;
            let file_str = file.to_string_lossy().to_string();
            let source = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("{} Failed reading file {:?}: {}", "Error:".red().bold(), file, e);
                    std::process::exit(1);
                }
            };
            let report = AutonomousSelfHealingEngine::heal_file(&file_str, &source, apply);
            println!("==================================================");
            println!("🤖 {} for `{}`", "AI Autonomous Self-Healing Report".cyan().bold(), report.file.yellow());
            println!("==================================================");
            println!("  Status:             {}", if report.status == "HEALED" || report.status == "ALREADY_HEALTHY" { report.status.green().bold() } else { report.status.red().bold() });
            println!("  Original Errors:    {}", report.original_errors_count);
            println!("  Iterations Tested:  {}", report.iterations_attempted);
            println!("  Proof Summary:      {}", report.proof_summary.cyan());
            if let Some(ref cand) = report.accepted_candidate {
                println!("  Accepted Patch:     {}", cand.description.green().bold());
            }
            if report.was_applied_to_disk {
                println!("\n{} Successfully applied verified heal patch to `{}`", "✔".green().bold(), report.file.cyan());
            } else if report.status == "HEALED" {
                println!("\n{} Verified patch ready. Run with {} to apply to disk.", "ℹ".blue().bold(), "--apply".yellow());
            }
}

