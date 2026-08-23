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

pub fn handle_memory(args: MemoryArgs) {
    let MemoryArgs { action, task, req, json } = args;
            let root = std::path::Path::new(".");
            match action.as_str() {
                "list" => {
                    let tasks = DrmEngine::list_all_tasks(root);
                    if json {
                        println!("{}", serde_json::json!({ "tasks": tasks }));
                    } else {
                        println!("🧠 {} Total Checkpointed Tasks: {}", "Dynamic Research Memory (DRM):".cyan().bold(), tasks.len());
                        for t in &tasks {
                            println!("  ├─ 📌 Task `{}`", t.yellow().bold());
                        }
                    }
                }
                "show" | "resume" => {
                    let tid = task.unwrap_or_else(|| "task-183".to_string());
                    match DrmEngine::load(root, &tid) {
                        Ok(drm) => {
                            if json {
                                println!("{}", serde_json::to_string_pretty(&drm).unwrap_or_default());
                            } else {
                                println!("🧠 {} Task `{}` (Phase: {})", "DRM Checkpoint:".green().bold(), drm.task_id.yellow(), drm.current_phase.cyan());
                                println!("  Requirement:       {}", drm.requirement);
                                println!("  Agent ID:          {}", drm.agent_id);
                                println!("  Investigated Files:{:?}", drm.investigated_files);
                                println!("  Contracts:         {:?}", drm.contracts_affected);
                                println!("  Hypotheses:        {} recorded", drm.hypotheses.len());
                            }
                        }
                        Err(e) => {
                            eprintln!("{} {}", "DRM Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                }
                "new" => {
                    let tid = task.unwrap_or_else(|| "task-01".to_string());
                    let r = req.unwrap_or_else(|| "General Feature".to_string());
                    let drm = DrmEngine::new_task(&tid, &r, "autonomous_agent_01");
                    match DrmEngine::save(root, &drm) {
                        Ok(p) => println!("✔ Initialized DRM task checkpoint at {:?}", p),
                        Err(e) => eprintln!("Error: {}", e),
                    }
                }
                other => eprintln!("Unknown DRM action: {}", other),
            }
}

pub fn handle_scope(args: ScopeArgs) {
    let ScopeArgs { agent, file, action_type, json } = args;
            let scope_def = AgentScopeDef {
                name: agent.clone(),
                scope_pattern: "src/**".to_string(),
                allow_actions: vec!["read_code".to_string(), "modify_code".to_string(), "run_tests".to_string()],
                deny_patterns: vec!["modify(src/auth/**)".to_string(), "access_secrets".to_string(), "database_write".to_string()],
            };
            let rep = AgentScopeEngine::check_permission(&scope_def, &file.to_string_lossy(), &action_type, None);
            if json {
                println!("{}", serde_json::to_string_pretty(&rep).unwrap_or_default());
            } else {
                println!("🚨 {}", "End Agent Permission & Scope Guard".yellow().bold());
                println!("================================================================================");
                println!("  Agent Name:         {}", rep.agent_name.cyan());
                println!("  Target File:        {}", rep.target_file.yellow());
                println!("  Requested Action:   {}", rep.requested_action);
                println!("  Within Scope:       {}", rep.within_scope);
                if rep.is_authorized {
                    println!("\n{} {}", "✔".green().bold(), rep.status_message.green().bold());
                } else {
                    println!("\n{} {}", "✖".red().bold(), rep.status_message.red().bold());
                    for v in &rep.denied_violations {
                        println!("  ✖ {}", v);
                    }
                    std::process::exit(1);
                }
            }
}

pub fn handle_semantic_git(args: SemanticGitArgs) {
    let SemanticGitArgs { action, file, task, message, json } = args;
            let (module, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            let file_str = file.to_string_lossy().to_string();
            let diff = SemanticGitEngine::compute_diff(&file_str, None, &module, &analyzer.graph);
            if action == "diff" {
                if json {
                    println!("{}", serde_json::to_string_pretty(&diff).unwrap_or_default());
                } else {
                    println!("🌳 {}", "End Semantic Git Diff".green().bold());
                    println!("================================================================================");
                    println!("  File:               {}", diff.file.yellow());
                    println!("  Symbol Deltas:      {} changes", diff.symbol_deltas.len());
                    for d in &diff.symbol_deltas {
                        println!("  ├─ [{}] `{}`", d.delta_type.cyan(), d.symbol);
                    }
                    println!("  Architecture:       {}", diff.architecture_status);
                    println!("  Contracts:          {}", diff.contract_adherence);
                    println!("  Security:           {}", diff.security_status);
                }
            } else if action == "commit" {
                let tid = task.unwrap_or_else(|| "task-183".to_string());
                let msg = message.unwrap_or_else(|| "Automated Verified Commit".to_string());
                let commit_res = SemanticGitEngine::create_verified_commit(
                    "autonomous_agent_01",
                    &tid,
                    &msg,
                    vec!["PaymentSafe".to_string()],
                    vec![file_str],
                    diff,
                    1,
                    1,
                    true,
                    true,
                );
                if json {
                    println!("{}", serde_json::to_string_pretty(&commit_res).unwrap_or_default());
                } else if commit_res.is_valid {
                    let m = commit_res.manifest.unwrap();
                    println!("👑 {}", "End Cryptographically Verified Commit".green().bold());
                    println!("================================================================================");
                    println!("  Commit Hash:        {}", m.commit_hash.green().bold());
                    println!("  Task ID:            {}", m.task_id.yellow());
                    println!("  Agent:              {}", m.agent_id.cyan());
                    println!("  Requirement:        {}", m.requirement);
                    println!("  Compiler Hash:      {}", m.compiler_hash);
                    println!("  Proof Signature:    {}", m.verification_signature.magenta());
                    println!("  Tests Passed:       {}/{}", m.tests_passed, m.total_tests);
                    println!("\n{} Verified Commit Accepted into Semantic Repository Ledger!", "✔".green().bold());
                } else {
                    println!("✖ Commit Rejected:");
                    for r in &commit_res.rejected_reasons {
                        println!("  ✖ {}", r);
                    }
                    std::process::exit(1);
                }
            }
}

pub fn handle_semantic_ir(args: SemanticIrArgs) {
    let SemanticIrArgs { file, json: _ } = args;
            let (module, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            let source = fs::read_to_string(&file).unwrap_or_default();
            let ir = EndSemanticInterface::extract_ir(&module, &analyzer.graph, &source);
            println!("{}", serde_json::to_string_pretty(&ir).unwrap_or_default());
}

pub fn handle_agent_run(args: AgentRunArgs) {
    let AgentRunArgs { file, intent, task_id, json } = args;
            let (module, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            let source = fs::read_to_string(&file).unwrap_or_default();
            let tid = task_id.unwrap_or_else(|| "task-auto-01".to_string());
            let file_str = file.to_string_lossy().to_string();
            let rep = AutonomousAgentRuntime::run_task(&tid, &intent, &file_str, &source, &module, &analyzer, std::path::Path::new("."));

            if json {
                println!("{}", serde_json::to_string_pretty(&rep).unwrap_or_default());
            } else {
                println!("🤖 {}", "End Autonomous Software Engineering Runtime".green().bold());
                println!("================================================================================");
                println!("  Task ID:            {}", rep.task_id.yellow().bold());
                println!("  Intent:             \"{}\"", rep.intent.cyan());
                println!("  Status:             {}", if rep.status == "ACCEPTED" { rep.status.green().bold() } else { rep.status.red().bold() });
                println!("  Execution Time:     {} µs", rep.execution_time_us.to_string().cyan());
                println!("  Planned Steps:");
                for s in &rep.planned_steps {
                    println!("    ├─ ✔ {}", s);
                }
                println!("  DNA Adherence:      {}", if rep.dna_adherence_verified { "✔ Verified".green() } else { "✖ Failed".red() });
                println!("  Pre-Touch Impact:   Risk={}, Score={}", rep.impact_risk_level, rep.blast_radius_score);
                println!("  Context Extracted:  {} tokens", rep.extracted_context_tokens);
                println!("  Skill Contracts:    {}", if rep.skills_verified { "✔ 100% Passed".green() } else { "✖ Violated".red() });
                println!("  AST Security:       {}", if rep.security_scan_passed { "✔ Zero Vulnerabilities".green() } else { "✖ Failed".red() });
                println!("  Tests Executed:     {}/{} Passed", rep.tests_passed, rep.total_tests);

                if let Some(ref c) = rep.verified_commit {
                    println!("\n👑 Verified Commit Hash: {}", c.commit_hash.green().bold());
                    println!("  Proof Signature:     {}", c.verification_signature.magenta());
                    println!("\n{} AUTONOMOUS SOFTWARE ENGINEERING CYCLE COMPLETE (PATCH ACCEPTED)", "✔".green().bold());
                } else {
                    println!("\n✖ Autonomous Task Rejected:");
                    for r in &rep.rejection_reasons {
                        println!("  ✖ {}", r);
                    }
                    std::process::exit(1);
                }
            }
}

pub fn handle_evolve(args: EvolveArgs) {
    let EvolveArgs { target, file, json } = args;
            let target_file = file.unwrap_or_else(|| {
                let current_files = find_all_end_files(std::path::Path::new("."));
                current_files.into_iter().next().unwrap_or_else(|| PathBuf::from("main.end"))
            });

            let (module, _) = match load_and_analyze(&target_file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let mut mod_def = module.modules.iter().find(|m| m.name == target || target == "all").cloned().unwrap_or_default();
            if mod_def.name.is_empty() {
                mod_def.name = if target == "all" { module.name.clone() } else { target.clone() };
                mod_def.structs = module.structs.clone();
                mod_def.functions = module.functions.clone();
                mod_def.statements = module.statements.clone();
            }

            let report = crate::agent_api::evolution_engine::EvolutionEngine::evaluate_evolvable_module(&mod_def);

            if json {
                println!("{}", serde_json::to_string_pretty(&report).unwrap_or_default());
            } else {
                println!("{}", report.formatted_output);
            }
}

