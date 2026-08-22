use crate::ast::Type;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Security Classification Levels for the Compiler Security Gate
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// Baseline memory and type safety (Standard End language mode)
    Standard,
    /// Strict checking: warnings converted to compile-time errors
    Strict,
    /// Paranoid: capability isolation, secret flow analysis, threat model compliance
    Paranoid,
    /// Critical: adversarial simulation, formal contract proofs, constant-time verification
    Critical,
    /// Absolute: zero-compromise verified build; binary prohibited without full cryptographic attestation
    Absolute,
}

impl SecurityLevel {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "strict" => SecurityLevel::Strict,
            "paranoid" => SecurityLevel::Paranoid,
            "critical" => SecurityLevel::Critical,
            "absolute" => SecurityLevel::Absolute,
            _ => SecurityLevel::Standard,
        }
    }
}

/// Authority Levels for Granular Capability and Resource Access
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityLevel {
    Read,
    Write,
    Execute,
    Admin,
    Custom(String),
}

/// Policy restrictions on URL endpoints
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrlPolicy {
    HttpsOnly,
    AllowLocalhost,
    DomainRestricted(Vec<String>),
    Custom(String),
}

/// Core First-Class Security-by-Construction Types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityTypeKind {
    /// External, untrusted input marked as tainted (Feature 1)
    Tainted(Box<Type>),
    /// Input that passed an approved sanitizer (Feature 2)
    Sanitized(Box<Type>),
    /// High-entropy secret (ApiKey, Password, PrivateKey) protected from leaks (Feature 3)
    Secret(Box<Type>),
    /// Private data classification preventing unauthorized domain transit (Feature 4)
    Private(Box<Type>),
    /// Formally trusted data validated against a trust boundary (Feature 5)
    Trusted(Box<Type>),
    /// Unverified external data requiring proof before promotion (Feature 6)
    Untrusted(Box<Type>),
    /// Capability token granting access to a specific subsystem (Feature 7)
    Capability(String),
    /// Hierarchical authority level token (Feature 8)
    Authority(String, AuthorityLevel),
    /// Post-validation sealed type that cannot be forged or mutated (Feature 9)
    Sealed(Box<Type>),
    /// Verifier-constructed type with embedded proof obligations (Feature 10)
    Verified(String, Vec<String>),
    /// Parameterized SQL safe value (Feature 11)
    SqlValue,
    /// HTML-escaped string safe from XSS (Feature 12)
    HtmlEscaped(Box<Type>),
    /// Shell-safe argument preventing argument injection (Feature 13)
    ShellArg,
    /// Canonicalized path safe from directory traversal (Feature 14)
    SafePath,
    /// URL safe under strict policy (Feature 15)
    TrustedUrl(UrlPolicy),
    /// Cryptographic key with compile-time bitwidth constraint (Feature 16)
    CryptoKey(usize),
    /// Cryptographic nonce with single-use linear lifetime (Feature 16 & 17)
    CryptoNonce(usize),
    /// Ciphertext bound to an approved algorithm (Feature 16)
    CryptoCiphertext(String),
    /// Security domain isolation envelope (Feature 30)
    SecurityDomain(String),
    /// Security state machine type tracking allowed state transitions (Feature 45)
    StateMachine {
        entity: String,
        current_state: String,
        allowed_transitions: Vec<(String, String)>,
    },
}

/// Vulnerability Sink Targets where untrusted data or secret leaks are dangerous
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VulnerabilitySinkKind {
    SqlExecution,
    HtmlRendering,
    ShellExecution,
    PathAccess,
    UrlFetch,
    SecretLogging,
    SecretSerialization,
    SecretErrorReflection,
    UnauthorizedDomainTransit,
    CovertTimingChannel,
}

impl VulnerabilitySinkKind {
    pub fn description(&self) -> &'static str {
        match self {
            VulnerabilitySinkKind::SqlExecution => "Database SQL Query Sink (CWE-89)",
            VulnerabilitySinkKind::HtmlRendering => "HTML/DOM Rendering Sink (CWE-79 / XSS)",
            VulnerabilitySinkKind::ShellExecution => "OS Shell/Command Execution Sink (CWE-78)",
            VulnerabilitySinkKind::PathAccess => "Filesystem Path Traversal Sink (CWE-22)",
            VulnerabilitySinkKind::UrlFetch => "Network URL Fetch Sink (CWE-918 / SSRF)",
            VulnerabilitySinkKind::SecretLogging => "Logging / Output Stream Sink (CWE-532)",
            VulnerabilitySinkKind::SecretSerialization => "Serialization / Export Sink (CWE-359)",
            VulnerabilitySinkKind::SecretErrorReflection => "Error Message Reflection Sink (CWE-209)",
            VulnerabilitySinkKind::UnauthorizedDomainTransit => "Security Domain Boundary Violation (CWE-285)",
            VulnerabilitySinkKind::CovertTimingChannel => "Secret-Dependent Timing Branch (CWE-208)",
        }
    }
}

/// Security Violation Record generated during compilation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    pub code: String,
    pub title: String,
    pub message: String,
    pub severity: String,
    pub line: usize,
    pub col: usize,
    pub file: String,
    pub cwe_id: Option<String>,
    pub sink_kind: Option<VulnerabilitySinkKind>,
    pub remediation: String,
}

/// Complete Security Audit Report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEngineReport {
    pub file: String,
    pub security_level: SecurityLevel,
    pub is_secure: bool,
    pub verified_build_permitted: bool,
    pub violations: Vec<SecurityViolation>,
    pub secrets_isolated: usize,
    pub nonces_consumed: usize,
    pub capability_checks_passed: usize,
    pub contracts_verified: usize,
    pub proofs_verified: usize,
    pub constant_time_functions_checked: usize,
    pub summary: String,
}
