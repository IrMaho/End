use crate::ast::*;
use crate::semantic::graph::SemanticGraph;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub cwe_id: String,
    pub title: String,
    pub severity: String, // "CRITICAL", "HIGH", "MEDIUM", "LOW"
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub snippet_redacted: String,
    pub description: String,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditReport {
    pub file: String,
    pub total_findings: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub is_secure: bool,
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub summary: String,
}

pub struct AstSecurityScanner;

impl AstSecurityScanner {
    pub fn scan_source_and_ast(
        file_path: &str,
        source: &str,
        module: &Module,
        graph: &SemanticGraph,
    ) -> SecurityAuditReport {
        let mut vulns = Vec::new();

        // 1. Scan for Hardcoded Secrets and API Keys (CWE-798)
        let secret_patterns = [
            ("sk_live_", "Stripe Live API Key", "CRITICAL"),
            ("AKIA", "AWS Access Key ID", "CRITICAL"),
            ("ghp_", "GitHub Personal Access Token", "CRITICAL"),
            ("Bearer eyJ", "Hardcoded JWT Bearer Token", "HIGH"),
            ("password = \"", "Hardcoded Plaintext Password", "HIGH"),
            ("secret_key = \"", "Hardcoded Secret Key", "HIGH"),
            ("PRIVATE KEY-----", "Hardcoded Private Key", "CRITICAL"),
        ];

        for (line_idx, line) in source.lines().enumerate() {
            for (pat, desc, sev) in &secret_patterns {
                if line.contains(pat) {
                    let col = line.find(pat).unwrap_or(0) + 1;
                    vulns.push(SecurityVulnerability {
                        cwe_id: "CWE-798".to_string(),
                        title: format!("Use of Hardcoded Credentials: {}", desc),
                        severity: sev.to_string(),
                        file: file_path.to_string(),
                        line: line_idx + 1,
                        col,
                        snippet_redacted: format!("{} [REDACTED_SECRET]", pat),
                        description: format!("Found hardcoded credential pattern `{}` in source code.", pat),
                        remediation: "Extract credentials to environment variables (`std.env.get`) or secure vault.".to_string(),
                    });
                }
            }
        }

        // 2. Scan for Capability Boundary Breaches (CWE-285)
        for func in &module.functions {
            let is_declared_pure = func.directives.iter().any(|d| d.name == "@pure" || (d.name == "@capability" && d.args.contains(&"is_pure=true".to_string())));
            let has_net_denied = func.directives.iter().any(|d| d.name == "@capability" && d.args.contains(&"net=false".to_string()));

            if let Some(info) = graph.symbols.get(&func.name) {
                if is_declared_pure && (!info.capabilities.is_pure || info.capabilities.net || info.capabilities.disk) {
                    vulns.push(SecurityVulnerability {
                        cwe_id: "CWE-285".to_string(),
                        title: "Capability Boundary Violation in Pure Context".to_string(),
                        severity: "HIGH".to_string(),
                        file: file_path.to_string(),
                        line: func.span.line,
                        col: func.span.col,
                        snippet_redacted: format!("fn {}()", func.name),
                        description: format!("Function `{}` is declared `@pure` but performs unverified side-effects or I/O.", func.name),
                        remediation: "Remove side-effecting calls or update function capability contract.".to_string(),
                    });
                }

                if has_net_denied && info.capabilities.net {
                    vulns.push(SecurityVulnerability {
                        cwe_id: "CWE-285".to_string(),
                        title: "Unauthorized Network Access Attempt".to_string(),
                        severity: "CRITICAL".to_string(),
                        file: file_path.to_string(),
                        line: func.span.line,
                        col: func.span.col,
                        snippet_redacted: format!("fn {}() [net=false violated]", func.name),
                        description: format!("Function `{}` has `net=false` capability constraint but invokes network operations.", func.name),
                        remediation: "Isolate network calls into authorized capability gateways.".to_string(),
                    });
                }
            }
        }

        // 3. Scan for Raw Memory Escapes & Unchecked Pointer Dereferences (CWE-119)
        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.contains("*raw_ptr") || trimmed.contains("transmute_unchecked") || (trimmed.contains("unsafe {") && !trimmed.contains("// @audit_reviewed")) {
                vulns.push(SecurityVulnerability {
                    cwe_id: "CWE-119".to_string(),
                    title: "Unbounded Raw Pointer Dereference / Unmanaged Memory Escape".to_string(),
                    severity: "HIGH".to_string(),
                    file: file_path.to_string(),
                    line: line_idx + 1,
                    col: 1,
                    snippet_redacted: trimmed.to_string(),
                    description: "Direct unchecked pointer manipulation outside managed ZeroGC arenas.".to_string(),
                    remediation: "Use End managed pointers or arena-scoped references.".to_string(),
                });
            }
        }

        let crit = vulns.iter().filter(|v| v.severity == "CRITICAL").count();
        let high = vulns.iter().filter(|v| v.severity == "HIGH").count();
        let med = vulns.iter().filter(|v| v.severity == "MEDIUM").count();
        let low = vulns.iter().filter(|v| v.severity == "LOW").count();
        let is_sec = crit == 0 && high == 0;

        let summary = if is_sec {
            "✔ 100% Security Guard Passed: Zero critical or high AST vulnerabilities detected.".to_string()
        } else {
            format!("🚨 Security Breach: Found {} critical and {} high severity vulnerabilities.", crit, high)
        };

        SecurityAuditReport {
            file: file_path.to_string(),
            total_findings: vulns.len(),
            critical_count: crit,
            high_count: high,
            medium_count: med,
            low_count: low,
            is_secure: is_sec,
            vulnerabilities: vulns,
            summary,
        }
    }
}
