use crate::ast::*;
use crate::security::types::*;
use std::collections::{HashMap, HashSet};

/// Active capability token held in an execution scope
#[derive(Debug, Clone, PartialEq)]
pub struct ActiveCapabilityToken {
    pub capability_name: String,
    pub authority: AuthorityLevel,
    pub path_or_target: Option<String>,
    pub granted_line: usize,
    pub expires_at_ms: Option<u64>,
    pub is_revoked: bool,
    pub delegated_to: Option<String>,
}

/// Security Domain definition
#[derive(Debug, Clone, PartialEq)]
pub struct SecurityDomainMeta {
    pub name: String,
    pub allowed_capabilities: HashSet<String>,
    pub allowed_egress_domains: HashSet<String>,
}

pub struct CapabilityAndDomainEngine {
    pub filename: String,
    pub security_level: SecurityLevel,
    pub active_capabilities: HashMap<String, ActiveCapabilityToken>,
    pub revoked_capabilities: HashSet<String>,
    pub security_domains: HashMap<String, SecurityDomainMeta>,
    pub function_capability_requirements: HashMap<String, Vec<String>>,
    pub function_domains: HashMap<String, String>,
    pub violations: Vec<SecurityViolation>,
    pub capabilities_verified_count: usize,
    pub delegations_count: usize,
    pub revocations_count: usize,
}

impl CapabilityAndDomainEngine {
    pub fn new(filename: &str, security_level: SecurityLevel) -> Self {
        Self {
            filename: filename.to_string(),
            security_level,
            active_capabilities: HashMap::new(),
            revoked_capabilities: HashSet::new(),
            security_domains: HashMap::new(),
            function_capability_requirements: HashMap::new(),
            function_domains: HashMap::new(),
            violations: Vec::new(),
            capabilities_verified_count: 0,
            delegations_count: 0,
            revocations_count: 0,
        }
    }

    pub fn register_domain(&mut self, name: &str, allowed_caps: &[&str], egress_domains: &[&str]) {
        let meta = SecurityDomainMeta {
            name: name.to_string(),
            allowed_capabilities: allowed_caps.iter().map(|s| s.to_string()).collect(),
            allowed_egress_domains: egress_domains.iter().map(|s| s.to_string()).collect(),
        };
        self.security_domains.insert(name.to_string(), meta);
    }

    pub fn grant_capability(&mut self, cap_name: &str, authority: AuthorityLevel, target: Option<&str>, line: usize) {
        let token = ActiveCapabilityToken {
            capability_name: cap_name.to_string(),
            authority,
            path_or_target: target.map(|s| s.to_string()),
            granted_line: line,
            expires_at_ms: None,
            is_revoked: false,
            delegated_to: None,
        };
        self.active_capabilities.insert(cap_name.to_string(), token);
        self.capabilities_verified_count += 1;
    }

    pub fn revoke_capability(&mut self, cap_name: &str, line: usize, col: usize) {
        self.revoked_capabilities.insert(cap_name.to_string());
        if let Some(token) = self.active_capabilities.get_mut(cap_name) {
            token.is_revoked = true;
        }
        self.revocations_count += 1;
    }

    pub fn delegate_capability(&mut self, cap_name: &str, target_agent: &str, line: usize) -> Result<(), SecurityViolation> {
        if self.revoked_capabilities.contains(cap_name) {
            return Err(SecurityViolation {
                code: "E0932".to_string(),
                title: "Use of Revoked Capability".to_string(),
                message: format!("Cannot delegate revoked capability '{}' at line {}.", cap_name, line),
                severity: "CRITICAL".to_string(),
                line,
                col: 1,
                file: self.filename.clone(),
                cwe_id: Some("CWE-285".to_string()),
                sink_kind: None,
                remediation: "Obtain a fresh capability grant before delegation.".to_string(),
            });
        }

        if let Some(token) = self.active_capabilities.get_mut(cap_name) {
            token.delegated_to = Some(target_agent.to_string());
            self.delegations_count += 1;
            Ok(())
        } else {
            Err(SecurityViolation {
                code: "E0931".to_string(),
                title: "Capability Delegation Failure".to_string(),
                message: format!("Attempted to delegate unheld capability '{}' at line {}.", cap_name, line),
                severity: "HIGH".to_string(),
                line,
                col: 1,
                file: self.filename.clone(),
                cwe_id: Some("CWE-285".to_string()),
                sink_kind: None,
                remediation: "Ensure host scope holds the capability before delegating to workers.".to_string(),
            })
        }
    }

    pub fn analyze_module_capabilities(&mut self, source: &str, module: &Module) {
        // 1. Scan source lines for capability directives, requires, and ambient authority breaches
        for (idx, line) in source.lines().enumerate() {
            let l_num = idx + 1;
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Capability grants
            if trimmed.starts_with("grant capability") || trimmed.contains("borrow capability<") {
                self.capabilities_verified_count += 1;
            }

            // Capability revocations
            if trimmed.starts_with("revoke ") {
                let cap_name = trimmed.trim_start_matches("revoke ").trim_end_matches(';').trim();
                self.revoke_capability(cap_name, l_num, 1);
            }

            // Capability delegations
            if trimmed.contains("delegate ") && trimmed.contains(" to ") {
                self.delegations_count += 1;
            }

            // Zero Ambient Authority Violation Check:
            // Disallow ambient direct system I/O, socket, or process without capability annotation or grant
            if (trimmed.contains("std.fs.write") || trimmed.contains("std.net.connect") || trimmed.contains("std.process.spawn"))
                && !source.contains("requires FileWrite")
                && !source.contains("requires Network")
                && !source.contains("requires ProcessSpawn")
                && !source.contains("@capability")
            {
                self.violations.push(SecurityViolation {
                    code: "E0933".to_string(),
                    title: "Ambient Authority Violation (No Ambient Authority Principle)".to_string(),
                    message: format!(
                        "Ambient access to ungranted system resource at line {}. All I/O and process actions require explicit capability declarations.",
                        l_num
                    ),
                    severity: "CRITICAL".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-285".to_string()),
                    sink_kind: None,
                    remediation: "Annotate function with `requires FileWrite(...)` or `requires Network` capability.".to_string(),
                });
            }

            // Privilege Escalation Check:
            // A function in read-only / low-privilege context trying to call high-privilege write/admin operation
            if (trimmed.contains("admin_escalate(") || ((trimmed.contains("low_privilege") || trimmed.contains("guest_user") || trimmed.contains("reader"))
                && (trimmed.contains("db_write(") || trimmed.contains("delete_all("))))
                && !trimmed.contains("elevation_proof")
            {
                self.violations.push(SecurityViolation {
                    code: "E0934".to_string(),
                    title: "Privilege Escalation Detected (CWE-269)".to_string(),
                    message: format!(
                        "Unverified privilege escalation attempt at line {}: low-privilege caller invoked high-authority write operation without formal elevation proof.",
                        l_num
                    ),
                    severity: "CRITICAL".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-269".to_string()),
                    sink_kind: None,
                    remediation: "Provide explicit authorization proof token or route through privilege-isolated boundary.".to_string(),
                });
            }

            // Dangerous API Quarantine Check:
            // Low-level C/ASM or raw unsafe API called without `unsafe { reason: "...", proof: ... }`
            if (trimmed.contains("unsafe.raw_mem_write") || trimmed.contains("unsafe.ptr_cast") || trimmed.contains("inline_c"))
                && !trimmed.contains("reason:")
            {
                self.violations.push(SecurityViolation {
                    code: "E0936".to_string(),
                    title: "Dangerous API Quarantine Violation (CWE-242)".to_string(),
                    message: format!(
                        "Direct call to dangerous unquarantined API at line {}. Dangerous APIs must be enclosed in `unsafe {{{{ reason: \"...\", proof: ... }}}}`.",
                        l_num
                    ),
                    severity: "HIGH".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-242".to_string()),
                    sink_kind: None,
                    remediation: "Enclose raw operation inside explicit `unsafe { reason: \"...\", proof: ... }` block.".to_string(),
                });
            }
        }

        // 2. Scan functions for capability requirements and capability intersection (`requires CapA & CapB`)
        for func in &module.functions {
            let mut req_caps = Vec::new();
            for dir in &func.directives {
                if dir.name == "@requires" || dir.name == "requires" {
                    req_caps.extend(dir.args.clone());
                }
            }

            if !req_caps.is_empty() {
                self.function_capability_requirements.insert(func.name.clone(), req_caps);
                self.capabilities_verified_count += 1;
            }
        }
    }
}
