pub mod contract;
pub mod evidence;
pub mod lifecycle;
pub mod provenance;
pub mod stale;
pub mod verifier;

#[cfg(test)]
pub mod tests;

pub use contract::{AgentContract, ContractError, CONTRACT_REL_PATH};
pub use evidence::{ContractEvidence, TestExecutionRecord, EVIDENCE_REL_PATH};
pub use lifecycle::{LifecycleError, LifecycleState};
pub use provenance::{Provenance, ProvenanceError};
pub use stale::{compute_file_hash, check_stale_against_disk, StaleCheckResult};
pub use verifier::{ContractVerifier, VerificationOutcome};

use colored::*;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::agent_args::ContractArgs;

/// CLI handler for `endc contract [action]`
pub fn handle_contract_cli(args: ContractArgs) {
    let ContractArgs { action, file, json } = args;

    let contract_path = file.unwrap_or_else(|| {
        let default_p = PathBuf::from(CONTRACT_REL_PATH);
        if default_p.exists() {
            default_p
        } else if let Some(found) = AgentContract::find_contract_file(Path::new(".")) {
            found
        } else {
            default_p
        }
    });

    match action.as_str() {
        "verify" => {
            if !contract_path.exists() {
                if json {
                    println!("{}", serde_json::json!({
                        "status": "error",
                        "error": format!("Contract file not found at {:?}", contract_path)
                    }));
                } else {
                    eprintln!(
                        "{} Contract file not found at {:?}",
                        "✖ Error:".red().bold(),
                        contract_path
                    );
                    eprintln!("  Create one with `endc contract init` or place it at `.agents/contract.toml`");
                }
                std::process::exit(1);
            }

            let project_root = contract_path
                .parent()
                .and_then(|p| {
                    if p.file_name().and_then(|s| s.to_str()) == Some(".agents") {
                        p.parent()
                    } else {
                        Some(p)
                    }
                })
                .unwrap_or_else(|| Path::new("."));

            let outcome = ContractVerifier::verify(&contract_path, Some(project_root), true);

            if json {
                match &outcome {
                    VerificationOutcome::Verified(ev) => {
                        println!("{}", ev.to_json_pretty());
                    }
                    VerificationOutcome::Rejected { evidence, reasons } => {
                        println!("{}", serde_json::json!({
                            "status": "REJECTED",
                            "evidence": evidence,
                            "reasons": reasons
                        }));
                    }
                    VerificationOutcome::Stale { evidence, modified_files, missing_files } => {
                        println!("{}", serde_json::json!({
                            "status": "STALE",
                            "evidence": evidence,
                            "modified_files": modified_files,
                            "missing_files": missing_files
                        }));
                    }
                    VerificationOutcome::InvalidContract(err) => {
                        println!("{}", serde_json::json!({
                            "status": "INVALID_CONTRACT",
                            "error": err
                        }));
                    }
                    VerificationOutcome::VerificationError(err) => {
                        println!("{}", serde_json::json!({
                            "status": "VERIFICATION_ERROR",
                            "error": err
                        }));
                    }
                }
            } else {
                println!(
                    "🤖 {}",
                    "End Agent Contract Verification System (Foundation)".green().bold()
                );
                println!("================================================================================");
                println!("  Contract File:   {:?}", contract_path);
                println!("  Project Root:    {:?}", project_root);

                match &outcome {
                    VerificationOutcome::Verified(ev) => {
                        println!("  Task ID:         {}", ev.contract_id.cyan().bold());
                        println!("  Intent:          {}", ev.intent.yellow());
                        println!("  Agent:           {}", ev.provenance.agent.bold());
                        println!("  Model:           {}", ev.provenance.model_version.bold());
                        println!("  Tests Executed:  {}/{} passed", ev.passed_tests.to_string().green().bold(), ev.total_tests);
                        println!("  Execution Time:  {} ms", ev.total_duration_ms.to_string().cyan());
                        println!("  Artifact Hashes: {} files tracked", ev.artifact_hashes.len());
                        println!("================================================================================");
                        println!("\n{} [CONTRACT VERIFIED] State: VERIFIED", "✔".green().bold());
                        println!("  Evidence persisted to: .agents/evidence.json");
                    }
                    VerificationOutcome::Rejected { evidence, reasons } => {
                        println!("  Task ID:         {}", evidence.contract_id.cyan().bold());
                        println!("  Intent:          {}", evidence.intent.yellow());
                        println!("  Tests Executed:  {}/{} passed ({} failed)", evidence.passed_tests, evidence.total_tests, evidence.failed_tests.to_string().red().bold());
                        println!("================================================================================");
                        println!("\n{} [CONTRACT REJECTED] State: REJECTED", "✖".red().bold());
                        println!("  Failure Reasons:");
                        for r in reasons {
                            println!("    ✖ {}", r.red());
                        }
                    }
                    VerificationOutcome::Stale { evidence, modified_files, missing_files } => {
                        println!("  Task ID:         {}", evidence.contract_id.cyan().bold());
                        println!("================================================================================");
                        println!("\n{} [CONTRACT STALE] State: STALE", "⚠".yellow().bold());
                        println!("  Artifacts have been modified since last verification:");
                        for m in modified_files {
                            println!("    ⚠ Modified: {}", m.yellow());
                        }
                        for miss in missing_files {
                            println!("    ✖ Missing:  {}", miss.red());
                        }
                        println!("  Run `endc contract verify` after repairing to re-verify.");
                    }
                    VerificationOutcome::InvalidContract(err) => {
                        println!("\n{} [INVALID CONTRACT] {}", "✖".red().bold(), err);
                    }
                    VerificationOutcome::VerificationError(err) => {
                        println!("\n{} [VERIFICATION ERROR] {}", "✖".red().bold(), err);
                    }
                }
            }

            std::process::exit(outcome.exit_code());
        }
        "show" | "status" => {
            if !contract_path.exists() {
                eprintln!("{} Contract not found at {:?}", "Error:".red().bold(), contract_path);
                std::process::exit(1);
            }
            let contract = match AgentContract::from_file(&contract_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("{} Failed to parse contract: {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            if json {
                println!("{}", serde_json::to_string_pretty(&contract).unwrap());
            } else {
                println!("📄 {}", "Agent Contract Specification".cyan().bold());
                println!("================================================================================");
                println!("  Task ID:         {}", contract.task_id.bold());
                println!("  Intent:          {}", contract.intent.yellow());
                println!("  Lifecycle State: {}", contract.lifecycle.to_string().green().bold());
                println!("  Agent:           {}", contract.provenance.agent);
                println!("  Prompt Hash:     {}", contract.provenance.prompt_hash);
                println!("  Model Version:   {}", contract.provenance.model_version);
                println!("  Requirements:    {} items", contract.requirements.len());
                println!("  Required Tests:  {:?}", contract.required_tests);
                println!("  Allowed Ops:     {:?}", contract.allowed_operations);
                println!("  Boundaries:      {:?}", contract.security_boundaries);
                println!("  Tracked Hashes:  {} files", contract.artifact_hashes.len());
            }
        }
        "init" => {
            if contract_path.exists() {
                eprintln!("{} Contract file already exists at {:?}", "Warning:".yellow().bold(), contract_path);
                std::process::exit(1);
            }
            let sample_contract = AgentContract {
                task_id: "task-001".to_string(),
                intent: "Implement core module functionality".to_string(),
                requirements: vec![
                    "Module exports entrypoint function".to_string(),
                    "Deterministic execution with zero memory leaks".to_string(),
                ],
                preconditions: vec!["Compiler environment initialized".to_string()],
                postconditions: vec!["All unit tests pass".to_string()],
                allowed_operations: vec![
                    "file_read".to_string(),
                    "crypto_hash".to_string(),
                    "time_read".to_string(),
                ],
                required_tests: vec!["tests/test_main.end".to_string()],
                evidence_requirements: vec!["test_output".to_string(), "artifact_hash".to_string()],
                security_boundaries: vec!["no_outbound_network".to_string(), "no_exec_subprocess".to_string()],
                target_files: vec!["src/main.end".to_string()],
                artifact_hashes: HashMap::new(),
                provenance: Provenance::new("antigravity-pair-programmer-v2", "Initial prompt", "gemini-3.7-flash"),
                lifecycle: LifecycleState::Draft,
            };

            if let Err(e) = sample_contract.save_to_file(&contract_path) {
                eprintln!("{} Failed to initialize contract: {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
            println!("✨ {} Initialized new contract at {:?}", "Success:".green().bold(), contract_path);
        }
        _ => {
            eprintln!("Unknown contract action '{}'. Supported: verify, show, status, init", action);
            std::process::exit(1);
        }
    }
}
