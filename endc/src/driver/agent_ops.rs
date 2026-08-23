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

pub fn handle_ui(args: UiArgs) {
    let UiArgs { file, output, dev, serve, port, open, target: _ } = args;
            let out_dir = output.unwrap_or_else(|| PathBuf::from("ui_build"));
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            match UiOrchestrator::build_ui(&module, &out_dir, dev) {
                Ok(index_path) => {
                    println!("✨ {} Rendered EndUI Declarative Application at {:?}", "EndUI:".green().bold(), out_dir);
                    println!("  ├─ 🌐 {} (120 FPS Reactive Declarative HTML5 DOM)", "index.html".cyan().bold());
                    if dev {
                        println!("  └─ 🤖 {} (Visual Pin Drop, Image Mockup, Task Board & Bug Reporter)", "AI DevMode Overlay: ACTIVE".green().bold());
                    }

                    if open {
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd").args(["/C", "start", &index_path.to_string_lossy()]).spawn();
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open").arg(&index_path).spawn();
                        #[cfg(target_os = "linux")]
                        let _ = std::process::Command::new("xdg-open").arg(&index_path).spawn();
                    }

                    if serve {
                        if let Err(e) = UiOrchestrator::serve_ui(&out_dir, port) {
                            eprintln!("{} Failed to start EndUI DevServer: {}", "Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
}

pub fn handle_agent(args: AgentArgs) {
    let AgentArgs { action, id, message, status, json } = args;
            let base_dir = std::path::Path::new(".");
            match action.as_str() {
                "list" | "feedback" | "sync" => {
                    let items = FeedbackManager::list_all(base_dir);
                    if json {
                        println!("{}", serde_json::to_string_pretty(&items).unwrap_or_default());
                    } else {
                        println!("🤖 {} Found {} active feedback items / tasks in .end/agent_feedback/:", "AI Agent Feedback Channel:".green().bold(), items.len());
                        for item in &items {
                            let prio_color = if item.priority.contains("P0") { item.priority.red().bold() }
                                else if item.priority.contains("P1") { item.priority.yellow().bold() }
                                else { item.priority.green().bold() };
                            println!("  ┌─ [{}] {} | Target: {} ({}:{})", prio_color, item.id.cyan().bold(), item.widget_name.yellow(), item.source_file, item.source_line);
                            println!("  │  Note: {}", item.developer_note);
                            if let Some(ref img) = item.attached_image {
                                println!("  │  🖼️ Mockup Image: {}", img.cyan().underline());
                            }
                            println!("  │  Status: {} | Replies: {}", item.status.magenta(), item.agent_replies.len());
                            for r in &item.agent_replies {
                                println!("  │    └─ 🤖 {}: {}", r.agent_name.green(), r.message);
                            }
                            println!("  └───────────────────────────────────────────────");
                        }
                    }
                }
                "reply" => {
                    let target_id = match id {
                        Some(i) => i,
                        None => {
                            eprintln!("{} Please provide --id <feedback_id>", "Error:".red().bold());
                            std::process::exit(1);
                        }
                    };
                    let msg = message.unwrap_or_else(|| "Resolved by AI Coding Agent".to_string());
                    let new_st = status.as_deref().unwrap_or("Resolved");
                    match FeedbackManager::add_reply(base_dir, &target_id, "Antigravity Agent", &msg, Some(new_st)) {
                        Ok(updated) => {
                            println!("✔ {} Replied to feedback `{}` and updated status to `{}`", "Agent Protocol:".green().bold(), updated.id.cyan(), updated.status.green().bold());
                        }
                        Err(e) => {
                            eprintln!("{} {}", "Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                }
                "board" => {
                    match FeedbackManager::update_board_summary(base_dir) {
                        Ok(board) => {
                            println!("📋 {} Total Tasks: {} | Open: {} | Resolved: {}", "Task Board:".green().bold(), board.total_tasks, board.open_count, board.resolved_count);
                        }
                        Err(e) => {
                            eprintln!("{} {}", "Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                }
                other => {
                    eprintln!("Unknown action: {}", other);
                }
            }
}

pub fn handle_skill(args: SkillArgs) {
    let SkillArgs { action } = args;
            match action.as_str() {
                "init" => {
                    let dest_dir = std::path::Path::new(".agents").join("skills").join("end-language");
                    if let Err(e) = std::fs::create_dir_all(&dest_dir) {
                        eprintln!("❌ Failed to create directory: {}", e);
                        std::process::exit(1);
                    }
                    let skill_content = include_str!("../../../.agents/skills/end-language/SKILL.md");
                    let dest_file = dest_dir.join("SKILL.md");
                    if let Err(e) = std::fs::write(&dest_file, skill_content) {
                        eprintln!("❌ Failed to write SKILL.md: {}", e);
                        std::process::exit(1);
                    }
                    println!("✔ Initialized End Language Skill at {}", dest_file.display());
                    println!("🚀 AI assistants in this workspace will now automatically pair program in End!");
                }
                "path" => {
                    let exe_path = std::env::current_exe().unwrap_or_default();
                    let skill_path = exe_path.parent().unwrap_or(std::path::Path::new("")).join("..").join(".agents").join("skills").join("end-language").join("SKILL.md");
                    println!("{}", skill_path.display());
                }
                other => {
                    eprintln!("Unknown action '{}'. Use 'end skill init' or 'end skill path'.", other);
                }
            }
}

pub fn handle_dna(args: DnaArgs) {
    let DnaArgs { path, prompt, json } = args;
            let files = if path.is_file() {
                vec![path.clone()]
            } else {
                find_all_end_files(&path)
            };

            let mut modules = Vec::new();
            let mut file_strs = Vec::new();
            for f in &files {
                file_strs.push(f.to_string_lossy().to_string());
                if let Ok((m, _)) = load_and_analyze(f) {
                    modules.push(m);
                }
            }

            let dna = ProjectDnaEngine::mine_dna(&modules, &file_strs, std::path::Path::new("."));

            if prompt {
                let p = ProjectDnaEngine::generate_agent_prompt(&dna);
                println!("{}", p);
            } else if json {
                println!("{}", serde_json::to_string_pretty(&dna).unwrap_or_default());
            } else {
                println!("🧬 {}", "End Project DNA & Architectural Signal Engine".cyan().bold());
                println!("================================================================================");
                println!("  Project Name:       {}", dna.project_name.green().bold());
                println!("  Architecture Style: {}", dna.architecture_style.yellow().bold());
                println!("  Layers:             {}", dna.layer_structure.join(" -> ").cyan());
                println!("  Function Style:     {}", dna.naming_conventions.function_style.green());
                println!("  Struct Style:       {}", dna.naming_conventions.struct_style.green());
                println!("  Error Handling:     {}", dna.error_handling_pattern.magenta());
                println!("  Concurrency:        {}", dna.concurrency_model.cyan());
                println!("  Memory Strategy:    {}", dna.memory_strategy.yellow());
                println!("  Files Scanned:      {}", dna.scanned_files_count);
                println!("  Confidence Score:   {:.0}%", dna.confidence_score * 100.0);
                println!("================================================================================");
            }
}

pub fn handle_context(args: ContextArgs) {
    let ContextArgs { file, intent, budget, json } = args;
            let (module, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            let rep = SmartContextSlicer::extract_context(&module, &analyzer.graph, &intent, Some(budget));
            if json {
                println!("{}", serde_json::to_string_pretty(&rep).unwrap_or_default());
            } else {
                println!("🧠 {}", "DEC_v2 Smart Context Slicing Engine".green().bold());
                println!("================================================================================");
                println!("  Task Intent:        \"{}\"", rep.task_intent.yellow());
                println!("  Original Lines:     {}", rep.original_lines);
                println!("  Extracted Lines:    {}", rep.extracted_lines);
                println!("  Compression Ratio:  {:.1}% Token Reduction", rep.compression_ratio_pct.to_string().green().bold());
                println!("  Estimated Tokens:   {} (Budget: {})", rep.estimated_tokens.to_string().cyan().bold(), rep.budget_tokens);
                println!("  Preserved Structs:  {:?}", rep.preserved_structs);
                println!("  Preserved Enums:    {:?}", rep.preserved_enums);
                println!("  Preserved Functions:{:?}", rep.preserved_functions);
                println!("--------------------------------------------------------------------------------");
                println!("{}", rep.context_payload);
            }
}

pub fn handle_precheck(args: PrecheckArgs) {
    let PrecheckArgs { file, symbol, json } = args;
            let (module, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            let rep = ImpactGuard::analyze(&symbol, &module, &analyzer.graph);
            if json {
                println!("{}", serde_json::to_string_pretty(&rep).unwrap_or_default());
            } else {
                println!("🛡️ {}", "Pre-Touch Blast Radius & Impact Guard".cyan().bold());
                println!("================================================================================");
                println!("  Target Symbol:        {}", rep.target_symbol.yellow().bold());
                let risk_col = match rep.risk_level.as_str() {
                    "LOW" => rep.risk_level.green().bold(),
                    "MEDIUM" => rep.risk_level.yellow().bold(),
                    _ => rep.risk_level.red().bold(),
                };
                println!("  Blast Radius Risk:    {}", risk_col);
                println!("  Direct Callers:       {} ({:?})", rep.direct_callers_count, rep.direct_callers);
                println!("  Transitive Callers:   {}", rep.transitive_callers_count);
                println!("  Database Flows:       {}", rep.database_flows.len());
                println!("  Network Boundaries:   {}", rep.network_boundaries.len());
                println!("  Impacted Test Suites: {:?}", rep.impacted_test_suites);
                println!("  Required Skills:      {:?}", rep.required_skills);
                if rep.can_proceed_safely {
                    println!("\n{} Safe to modify. Pre-touch verification passed.", "✔".green().bold());
                } else {
                    println!("\n{} Blocked by Pre-Touch Guard:", "✖".red().bold());
                    for b in &rep.blocking_reasons {
                        println!("  ✖ {}", b);
                    }
                    std::process::exit(1);
                }
            }
}

