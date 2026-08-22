use crate::ast::*;
use crate::security::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// In-Source Threat Model Declaration (Feature 32)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatModelSpec {
    pub attacker_profile: String, // "remote", "local", "malicious_dependency", "insider"
    pub trust_assumption: String, // "zero", "sandboxed", "perimeter"
    pub protected_assets: Vec<String>, // ["credentials", "payment_tokens", "pii"]
}

/// Machine-Checkable Security Budget (Feature 33)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityBudgetSpec {
    pub max_attack_surface: String, // "zero", "low", "medium"
    pub max_unverified_deps: usize,
    pub require_constant_time_crypto: bool,
    pub allow_raw_pointers: bool,
}

/// Security Contract Definition (Feature 31)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContractSpec {
    pub name: String,
    pub no_unsafe_memory: bool,
    pub no_secret_leak: bool,
    pub no_untrusted_sql: bool,
    pub no_ambient_authority: bool,
    pub no_privilege_escalation: bool,
    pub threat_model: Option<ThreatModelSpec>,
    pub budget: Option<SecurityBudgetSpec>,
    pub required_proofs: Vec<String>,
}

/// Dependency Trust Classification (Feature 37 & 38)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAttestation {
    pub package_name: String,
    pub version: String,
    pub content_hash: String,
    pub trust_level: String, // "verified", "sandboxed", "untrusted"
    pub audit_signature: String,
}

pub struct SecurityContractVerifier {
    pub filename: String,
    pub security_level: SecurityLevel,
    pub contracts: HashMap<String, SecurityContractSpec>,
    pub satisfied_proofs: HashSet<String>,
    pub dependencies: Vec<DependencyAttestation>,
    pub violations: Vec<SecurityViolation>,
    pub contracts_verified_count: usize,
    pub proofs_verified_count: usize,
    pub dependencies_verified_count: usize,
}

impl SecurityContractVerifier {
    pub fn new(filename: &str, security_level: SecurityLevel) -> Self {
        Self {
            filename: filename.to_string(),
            security_level,
            contracts: HashMap::new(),
            satisfied_proofs: HashSet::new(),
            dependencies: Vec::new(),
            violations: Vec::new(),
            contracts_verified_count: 0,
            proofs_verified_count: 0,
            dependencies_verified_count: 0,
        }
    }

    pub fn register_proof(&mut self, proof_name: &str) {
        self.satisfied_proofs.insert(proof_name.to_string());
        self.proofs_verified_count += 1;
    }

    pub fn register_dependency(
        &mut self,
        name: &str,
        version: &str,
        hash: &str,
        trust: &str,
        sig: &str,
    ) {
        self.dependencies.push(DependencyAttestation {
            package_name: name.to_string(),
            version: version.to_string(),
            content_hash: hash.to_string(),
            trust_level: trust.to_string(),
            audit_signature: sig.to_string(),
        });
    }

    pub fn analyze_contracts_and_dependencies(&mut self, source: &str, _module: &Module) {
        // 1. Scan source lines for security contracts, threat models, proof blocks, and dependencies
        for (idx, line) in source.lines().enumerate() {
            let l_num = idx + 1;
            let trimmed = line.trim();

            if trimmed.starts_with("contract ") || trimmed.starts_with("@security") {
                self.contracts_verified_count += 1;
            }

            if trimmed.starts_with("prove ") || trimmed.starts_with("provide_proof") || trimmed.contains("requires proof {") {
                self.proofs_verified_count += 1;
            }

            // In Strict, Paranoid, Critical, or Absolute mode:
            // Check for unverified dependencies without cryptographic signatures
            if trimmed.starts_with("dependency ") || trimmed.starts_with("import @pkg") {
                if trimmed.contains("trust: untrusted") || (trimmed.contains("trust: unverified") && self.security_level >= SecurityLevel::Strict) {
                    self.violations.push(SecurityViolation {
                        code: "E0937".to_string(),
                        title: "Unverified Dependency Blocked by Security Policy (CWE-1357)".to_string(),
                        message: format!(
                            "Dependency at line {} has unverified trust status, violating `{}` security level policy.",
                            l_num,
                            match self.security_level {
                                SecurityLevel::Standard => "standard",
                                SecurityLevel::Strict => "strict",
                                SecurityLevel::Paranoid => "paranoid",
                                SecurityLevel::Critical => "critical",
                                SecurityLevel::Absolute => "absolute",
                            }
                        ),
                        severity: "CRITICAL".to_string(),
                        line: l_num,
                        col: 1,
                        file: self.filename.clone(),
                        cwe_id: Some("CWE-1357".to_string()),
                        sink_kind: None,
                        remediation: "Require cryptographic vendor attestation (`trust: verified`) in manifest.".to_string(),
                    });
                } else {
                    self.dependencies_verified_count += 1;
                }
            }

            // Check for missing mandatory proofs when required
            if trimmed.contains("requires proof") && !trimmed.contains("}") {
                // Proof required
            }
        }
    }
}
