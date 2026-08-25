pub mod contract;
pub mod evidence;
pub mod lifecycle;
pub mod provenance;
pub mod signing;
pub mod stale;
pub mod verifier;

#[cfg(test)]
pub mod tests;

pub use contract::{AgentContract, ContractError, CONTRACT_REL_PATH};
pub use evidence::{
    ArtifactBundle, AssertionDetail, CoverageInfo, EnvironmentInfo, EvidenceBundle, EvidenceError,
    RepairAttempt, SuggestedFixArea, TestExecutionRecord, DEFAULT_SCHEMA_VERSION,
    SUPPORTED_SCHEMA_VERSIONS,
};
pub use lifecycle::{LifecycleError, LifecycleState};
pub use provenance::{Provenance, ProvenanceError};
pub use signing::{
    compute_signature_string, get_or_create_project_key, hmac_sha256, SECRET_KEY_REL_PATH,
};
pub use stale::{check_stale_against_disk, compute_file_hash, StaleCheckResult};
pub use verifier::{ContractVerifier, VerificationOutcome};

use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

use crate::cli::agent_args::ContractArgs;

/// CLI handler for `endc contract [action]`
pub fn handle_contract_cli(args: ContractArgs) {
    let ContractArgs {
        action,
        file,
        evidence: evidence_arg,
        json,
    } = args;

    // Handle --evidence <path> mode directly
    if let Some(evidence_path) = evidence_arg {
        let project_root = evidence_path
            .parent()
            .and_then(|p| {
                if p.file_name().and_then(|s| s.to_str()) == Some("evidence") {
                    p.parent().and_then(|p2| {
                        if p2.file_name().and_then(|s| s.to_str()) == Some(".agents") {
                            p2.parent()
                        } else {
                            Some(p2)
                        }
                    })
                } else if p.file_name().and_then(|s| s.to_str()) == Some(".agents") {
                    p.parent()
                } else {
                    Some(p)
                }
            })
            .unwrap_or_else(|| Path::new("."));

        if !evidence_path.exists() {
            if json {
                println!("{}", serde_json::json!({
                    "status": "error",
                    "error": format!("Evidence file not found at {:?}", evidence_path)
                }));
            } else {
                eprintln!("{} Evidence file not found at {:?}", "✖ Error:".red().bold(), evidence_path);
            }
            std::process::exit(1);
        }

        let secret_key = match get_or_create_project_key(project_root) {
            Ok(k) => k,
            Err(e) => {
                eprintln!("{} Failed to read secret key: {}", "✖ Error:".red().bold(), e);
                std::process::exit(1);
            }
        };

        match EvidenceBundle::load_and_verify(&evidence_path, &secret_key) {
            Ok(bundle) => {
                // Check for source drift / stale status
                let stale_check = check_stale_against_disk(project_root, &bundle.artifacts.source_files);
                match stale_check {
                    StaleCheckResult::Stale { modified_files, missing_files, .. } => {
                        if json {
                            println!("{}", serde_json::json!({
                                "status": "STALE",
                                "evidence_file": evidence_path,
                                "contract_id": bundle.contract_id,
                                "modified_files": modified_files,
                                "missing_files": missing_files
                            }));
                        } else {
                            println!("STALE");
                            eprintln!("⚠ [EVIDENCE STALE] Evidence is out of date with disk artifacts:");
                            for m in modified_files {
                                eprintln!("  ⚠ Modified: {}", m.yellow());
                            }
                            for miss in missing_files {
                                eprintln!("  ✖ Missing:  {}", miss.red());
                            }
                        }
                        std::process::exit(1);
                    }
                    _ => {}
                }

                if bundle.state == LifecycleState::Rejected {
                    if json {
                        println!("{}", serde_json::to_string_pretty(&bundle).unwrap());
                    } else {
                        println!("REJECTED");
                        eprintln!("✖ [CONTRACT REJECTED] Task: {}", bundle.contract_id);
                        if let Some(last_attempt) = bundle.repair_attempts.last() {
                            eprintln!("  Failure Reason: {}", last_attempt.failure_reason.red());
                            if let Some(test) = &last_attempt.failed_test {
                                eprintln!("  Failed Test:    {}", test);
                            }
                            if let Some(fix) = &last_attempt.suggested_fix_area {
                                eprintln!("  Fix Area:       {}:{}-{} ({})", fix.file, fix.line_start, fix.line_end, fix.hint);
                            }
                        }
                    }
                    std::process::exit(1);
                }

                if json {
                    println!("{}", serde_json::to_string_pretty(&bundle).unwrap());
                } else {
                    println!("EVIDENCE_VERIFIED");
                    println!("✔ Evidence for task '{}' is genuine, HMAC-signed, and up-to-date.", bundle.contract_id.cyan().bold());
                }
                std::process::exit(0);
            }
            Err(EvidenceError::IncompatibleSchemaVersion { version, supported }) => {
                if json {
                    println!("{}", serde_json::json!({
                        "status": "SCHEMA_VERSION_INCOMPATIBLE",
                        "version": version,
                        "supported": supported
                    }));
                } else {
                    println!("SCHEMA_VERSION_INCOMPATIBLE");
                    eprintln!("✖ [SCHEMA INCOMPATIBLE] Evidence schema version '{}' is not supported. Supported: {:?}", version, supported);
                }
                std::process::exit(1);
            }
            Err(EvidenceError::Tampered { .. } | EvidenceError::MissingSignature) => {
                if json {
                    println!("{}", serde_json::json!({
                        "status": "EVIDENCE_TAMPERED",
                        "error": "HMAC signature mismatch or missing signature"
                    }));
                } else {
                    println!("EVIDENCE_TAMPERED");
                    eprintln!("✖ [SECURITY ALERT] Evidence bundle signature verification failed! File has been tampered with.");
                }
                std::process::exit(1);
            }
            Err(e) => {
                if json {
                    println!("{}", serde_json::json!({
                        "status": "ERROR",
                        "error": e.to_string()
                    }));
                } else {
                    eprintln!("✖ [ERROR] Failed to verify evidence: {}", e);
                }
                std::process::exit(1);
            }
        }
    }

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
                        println!("{}", serde_json::to_string_pretty(ev).unwrap());
                    }
                    VerificationOutcome::Rejected {
                        evidence,
                        feedback,
                        reasons,
                    } => {
                        println!("{}", serde_json::json!({
                            "status": "REJECTED",
                            "contract_id": evidence.contract_id,
                            "evidence": evidence,
                            "feedback": feedback,
                            "reasons": reasons
                        }));
                    }
                    VerificationOutcome::Stale {
                        evidence,
                        modified_files,
                        missing_files,
                    } => {
                        println!("{}", serde_json::json!({
                            "status": "STALE",
                            "evidence": evidence,
                            "modified_files": modified_files,
                            "missing_files": missing_files
                        }));
                    }
                    VerificationOutcome::Tampered { file, error } => {
                        println!("{}", serde_json::json!({
                            "status": "EVIDENCE_TAMPERED",
                            "file": file,
                            "error": error.to_string()
                        }));
                    }
                    VerificationOutcome::IncompatibleSchema { file, version } => {
                        println!("{}", serde_json::json!({
                            "status": "SCHEMA_VERSION_INCOMPATIBLE",
                            "file": file,
                            "version": version
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
                        println!("  State:           {}", "VERIFIED".green().bold());
                        println!("  Schema Version:  {}", ev.schema_version);
                        println!("  Signed At:       {}", ev.signed_at);
                        println!("  Signature:       {}", ev.signature.green());
                        println!("  Tests Executed:  {}/{} passed", ev.tests.iter().filter(|t| t.pass).count().to_string().green().bold(), ev.tests.len());
                        println!("  Rebuild Determ.: {}", if ev.rebuild_deterministic { "true (verified)".green() } else { "false".red() });
                        println!("  Artifact Hashes: {} source, {} C, {} bin", ev.artifacts.source_files.len(), ev.artifacts.generated_c.len(), ev.artifacts.binaries.len());
                        println!("================================================================================");
                        println!("\n{} [CONTRACT VERIFIED] State: VERIFIED", "✔".green().bold());
                        println!("  Evidence persisted to: .agents/evidence/{}.json", ev.contract_id);
                    }
                    VerificationOutcome::Rejected {
                        evidence,
                        feedback,
                        reasons,
                    } => {
                        println!("  Task ID:         {}", evidence.contract_id.cyan().bold());
                        println!("  State:           {}", "REJECTED".red().bold());
                        println!("  Tests Executed:  {}/{} passed", evidence.tests.iter().filter(|t| t.pass).count(), evidence.tests.len());
                        println!("================================================================================");
                        println!("\n{} [CONTRACT REJECTED] State: REJECTED", "✖".red().bold());
                        println!("  Failure Reasons:");
                        for r in reasons {
                            println!("    ✖ {}", r.red());
                        }
                        if let Some(fb) = feedback {
                            println!("\n  📋 Structured Repair Feedback (Attempt #{})", fb.attempt_number);
                            println!("    Failure Reason:     {}", fb.failure_reason.yellow());
                            if let Some(test) = &fb.failed_test {
                                println!("    Failed Test:        {}", test.cyan());
                            }
                            if let Some(assertion) = &fb.assertion {
                                println!("    Assertion Expected: {}", assertion.expected);
                                println!("    Assertion Actual:   {}", assertion.actual);
                            }
                            if let Some(fix) = &fb.suggested_fix_area {
                                println!("    Suggested Fix Area: {}:{}-{}", fix.file, fix.line_start, fix.line_end);
                                println!("    Hint:               {}", fix.hint);
                            }
                        }
                    }
                    VerificationOutcome::Stale {
                        evidence: _,
                        modified_files,
                        missing_files,
                    } => {
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
                    VerificationOutcome::Tampered { error, .. } => {
                        println!("\n{} [EVIDENCE TAMPERED] {}", "✖".red().bold(), error);
                    }
                    VerificationOutcome::IncompatibleSchema { version, .. } => {
                        println!("\n{} [SCHEMA INCOMPATIBLE] Version '{}' unsupported", "✖".red().bold(), version);
                    }
                    VerificationOutcome::InvalidContract(err) => {
                        println!("\n{} [INVALID CONTRACT] {}", "✖".red().bold(), err);
                    }
                    VerificationOutcome::VerificationError(err) => {
                        println!("\n{} [VERIFICATION ERROR] {}", "✖".red().bold(), err);
                    }
                }
            }

            let exit_code = match &outcome {
                VerificationOutcome::Verified(_) => 0,
                _ => 1,
            };
            std::process::exit(exit_code);
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
                artifact_hashes: std::collections::HashMap::new(),
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
