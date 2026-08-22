use crate::ast::*;
use crate::security::types::*;
use std::collections::{HashMap, HashSet};

/// Tracks symbol security metadata in lexical scopes
#[derive(Debug, Clone, PartialEq)]
pub struct SymbolSecurityMeta {
    pub name: String,
    pub kind: SecurityTypeKind,
    pub declared_line: usize,
    pub is_consumed: bool, // For linear nonce / single-use resources
    pub zeroize_on_drop: bool,
    pub declassified_with_proof: Option<String>,
}

pub struct TaintAndInformationFlowAnalyzer {
    pub filename: String,
    pub security_level: SecurityLevel,
    pub scopes: Vec<HashMap<String, SymbolSecurityMeta>>,
    pub violations: Vec<SecurityViolation>,
    pub secrets_count: usize,
    pub taints_count: usize,
    pub sanitizations_count: usize,
    pub nonces_consumed_count: usize,
    pub constant_time_checked_count: usize,
    pub declassifications_count: usize,
}

impl TaintAndInformationFlowAnalyzer {
    pub fn new(filename: &str, security_level: SecurityLevel) -> Self {
        Self {
            filename: filename.to_string(),
            security_level,
            scopes: vec![HashMap::new()],
            violations: Vec::new(),
            secrets_count: 0,
            taints_count: 0,
            sanitizations_count: 0,
            nonces_consumed_count: 0,
            constant_time_checked_count: 0,
            declassifications_count: 0,
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if let Some(scope) = self.scopes.pop() {
            for (_, meta) in scope {
                if meta.zeroize_on_drop {
                    // Scope ended: memory zeroized safely
                }
            }
        }
    }

    pub fn declare_security_symbol(&mut self, name: &str, kind: SecurityTypeKind, line: usize) {
        match &kind {
            SecurityTypeKind::Secret(_) => self.secrets_count += 1,
            SecurityTypeKind::Tainted(_) | SecurityTypeKind::Untrusted(_) => self.taints_count += 1,
            SecurityTypeKind::Sanitized(_) | SecurityTypeKind::Trusted(_) | SecurityTypeKind::Verified(_, _) => {
                self.sanitizations_count += 1;
            }
            SecurityTypeKind::CryptoNonce(_) => {}
            _ => {}
        }

        let is_zeroize = matches!(kind, SecurityTypeKind::Secret(_));
        let meta = SymbolSecurityMeta {
            name: name.to_string(),
            kind,
            declared_line: line,
            is_consumed: false,
            zeroize_on_drop: is_zeroize,
            declassified_with_proof: None,
        };

        if let Some(current_scope) = self.scopes.last_mut() {
            current_scope.insert(name.to_string(), meta);
        }
    }

    pub fn lookup_security_meta(&self, name: &str) -> Option<SymbolSecurityMeta> {
        for scope in self.scopes.iter().rev() {
            if let Some(meta) = scope.get(name) {
                return Some(meta.clone());
            }
        }
        None
    }

    pub fn mark_nonce_consumed(&mut self, name: &str, line: usize, col: usize) -> Result<(), SecurityViolation> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(meta) = scope.get_mut(name) {
                if let SecurityTypeKind::CryptoNonce(_) = meta.kind {
                    if meta.is_consumed {
                        let violation = SecurityViolation {
                            code: "E0930".to_string(),
                            title: "Nonce Reuse Violation (Linear Type Consumed)".to_string(),
                            message: format!(
                                "Linear cryptographic nonce '{}' was already consumed and cannot be reused at line {}. Nonces must be strictly single-use.",
                                name, line
                            ),
                            severity: "CRITICAL".to_string(),
                            line,
                            col,
                            file: self.filename.clone(),
                            cwe_id: Some("CWE-323".to_string()),
                            sink_kind: None,
                            remediation: "Generate a fresh cryptographic nonce for each operation.".to_string(),
                        };
                        return Err(violation);
                    } else {
                        meta.is_consumed = true;
                        self.nonces_consumed_count += 1;
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn declassify_symbol(
        &mut self,
        name: &str,
        proof: &str,
        line: usize,
        col: usize,
    ) -> Result<(), SecurityViolation> {
        if proof.trim().is_empty() {
            let violation = SecurityViolation {
                code: "E0928".to_string(),
                title: "Illegal Declassification Without Proof".to_string(),
                message: format!(
                    "Cannot declassify secret '{}' at line {} without explicit verification proof contract.",
                    name, line
                ),
                severity: "CRITICAL".to_string(),
                line,
                col,
                file: self.filename.clone(),
                cwe_id: Some("CWE-285".to_string()),
                sink_kind: None,
                remediation: "Provide formal proof contract (e.g. `declassify secret requires proof<Redaction>`)".to_string(),
            };
            return Err(violation);
        }

        for scope in self.scopes.iter_mut().rev() {
            if let Some(meta) = scope.get_mut(name) {
                meta.declassified_with_proof = Some(proof.to_string());
                self.declassifications_count += 1;
                return Ok(());
            }
        }

        Ok(())
    }

    pub fn analyze_source_and_ast(&mut self, source: &str, module: &Module) {
        let mut active_tainted_vars: HashSet<String> = HashSet::new();
        let mut active_secret_vars: HashSet<String> = HashSet::new();

        // 1. Scan source lines for security type declarations and obvious leak patterns
        for (idx, line) in source.lines().enumerate() {
            let l_num = idx + 1;
            let trimmed = line.trim();
            if trimmed.starts_with("//") {
                continue;
            }

            // Detect function parameters
            if trimmed.starts_with("pub fn ") || trimmed.starts_with("fn ") {
                if let Some(open_p) = trimmed.find('(') {
                    if let Some(close_p) = trimmed.find(')') {
                        let params_str = &trimmed[open_p + 1..close_p];
                        for p in params_str.split(',') {
                            let p_trim = p.trim();
                            if let Some(colon) = p_trim.find(':') {
                                let v_name = p_trim[..colon].trim();
                                let v_ty = p_trim[colon + 1..].trim();
                                if v_ty.contains("tainted<") || v_ty.contains("untrusted<") {
                                    active_tainted_vars.insert(v_name.to_string());
                                }
                                if v_ty.contains("secret<") {
                                    active_secret_vars.insert(v_name.to_string());
                                }
                            }
                        }
                    }
                }
            }

            // Detect local variables
            if trimmed.starts_with("val ") || trimmed.starts_with("mut ") {
                if let Some(colon) = trimmed.find(':') {
                    let v_name = trimmed[4..colon].trim();
                    let rest = &trimmed[colon + 1..];
                    if rest.contains("tainted<") || rest.contains("untrusted<") {
                        active_tainted_vars.insert(v_name.to_string());
                    }
                    if rest.contains("secret<") {
                        active_secret_vars.insert(v_name.to_string());
                    }
                }
            }

            // Check Declassification without proof
            if trimmed.contains("declassify") && !trimmed.contains("proof") {
                self.violations.push(SecurityViolation {
                    code: "E0928".to_string(),
                    title: "Illegal Declassification Without Proof (CWE-285)".to_string(),
                    message: format!(
                        "Attempted declassification of protected secret at line {} without explicit verification proof contract.",
                        l_num
                    ),
                    severity: "HIGH".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-285".to_string()),
                    sink_kind: None,
                    remediation: "Provide explicit `proof<ProofType>` when calling declassify.".to_string(),
                });
            }

            // Taint declarations
            if trimmed.contains("tainted<") || trimmed.contains("untrusted<") {
                self.taints_count += 1;
            }
            if trimmed.contains("sanitized<") || trimmed.contains("trusted<") || trimmed.contains("sealed<") {
                self.sanitizations_count += 1;
            }
            if trimmed.contains("secret<") {
                self.secrets_count += 1;
            }

            // Check SQL Sink with unparameterized raw string
            let is_sql_call = trimmed.contains("query(") || trimmed.contains("sql_exec(") || trimmed.contains("db.execute(");
            let has_taint_sql = trimmed.contains("tainted") || trimmed.contains("user_input") || trimmed.contains("raw_req")
                || active_tainted_vars.iter().any(|v| trimmed.contains(&format!("({}", v)) || trimmed.contains(&format!(" {},", v)));
            if is_sql_call && has_taint_sql && !trimmed.contains("SqlValue") && !trimmed.contains("param") && !trimmed.contains("sanitized") {
                self.violations.push(SecurityViolation {
                    code: "E0921".to_string(),
                    title: "Unsanitized Tainted Flow to SQL Sink (CWE-89)".to_string(),
                    message: format!(
                        "Tainted input flows directly into database SQL query sink at line {} without SqlValue parameterization.",
                        l_num
                    ),
                    severity: "CRITICAL".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-89".to_string()),
                    sink_kind: Some(VulnerabilitySinkKind::SqlExecution),
                    remediation: "Wrap query arguments in `SqlValue` or use parameterized prepared statements.".to_string(),
                });
            }

            // Check HTML Sink with raw string
            let is_html_call = trimmed.contains("render_html(") || trimmed.contains("inner_html =") || trimmed.contains("response.html(");
            let has_html_taint = trimmed.contains("tainted") || trimmed.contains("user_input") || trimmed.contains("raw_content")
                || active_tainted_vars.iter().any(|v| trimmed.contains(&format!("({}", v)));
            if is_html_call && has_html_taint && !trimmed.contains("HtmlEscaped") && !trimmed.contains("escape(") {
                self.violations.push(SecurityViolation {
                    code: "E0922".to_string(),
                    title: "Cross-Site Scripting (XSS) in HTML Sink (CWE-79)".to_string(),
                    message: format!(
                        "Unescaped string flows into HTML rendering sink at line {} without HtmlEscaped<String> type proof.",
                        l_num
                    ),
                    severity: "HIGH".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-79".to_string()),
                    sink_kind: Some(VulnerabilitySinkKind::HtmlRendering),
                    remediation: "Pass `HtmlEscaped<String>` or apply standard HTML entity sanitization.".to_string(),
                });
            }

            // Check Shell Sink
            let is_shell_call = trimmed.contains("system(") || trimmed.contains("exec(") || trimmed.contains("spawn_process(") || trimmed.contains("shell(");
            let has_shell_taint = trimmed.contains("tainted") || trimmed.contains("user_arg") || trimmed.contains("cmd_input")
                || active_tainted_vars.iter().any(|v| trimmed.contains(&format!("({}", v)));
            if is_shell_call && has_shell_taint && !trimmed.contains("ShellArg") {
                self.violations.push(SecurityViolation {
                    code: "E0923".to_string(),
                    title: "Command Injection in OS Shell Sink (CWE-78)".to_string(),
                    message: format!(
                        "Unsanitized input supplied to system execution sink at line {} without ShellArg encapsulation.",
                        l_num
                    ),
                    severity: "CRITICAL".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-78".to_string()),
                    sink_kind: Some(VulnerabilitySinkKind::ShellExecution),
                    remediation: "Use `ShellArg` array and direct argv execution instead of shell interpolation.".to_string(),
                });
            }

            // Check Path Traversal Sink
            let is_fs_call = trimmed.contains("read_file(") || trimmed.contains("open_file(") || trimmed.contains("fs.read(");
            let has_fs_taint = trimmed.contains("tainted") || trimmed.contains("path_param") || trimmed.contains("user_path")
                || active_tainted_vars.iter().any(|v| trimmed.contains(&format!("({}", v)));
            if is_fs_call && has_fs_taint && !trimmed.contains("SafePath") && !trimmed.contains("canonicalize") {
                self.violations.push(SecurityViolation {
                    code: "E0924".to_string(),
                    title: "Path Traversal Risk (CWE-22)".to_string(),
                    message: format!(
                        "Unvalidated path string flows into filesystem sink at line {} without SafePath validation.",
                        l_num
                    ),
                    severity: "HIGH".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-22".to_string()),
                    sink_kind: Some(VulnerabilitySinkKind::PathAccess),
                    remediation: "Wrap in `SafePath` to enforce root sandboxing and prevent `../` traversal.".to_string(),
                });
            }

            // Check Secret Logging / Output
            let is_log_call = trimmed.contains("println(") || trimmed.contains("log(") || trimmed.contains("console.log(") || trimmed.contains("debug(");
            let has_secret_log = trimmed.contains("secret<") || trimmed.contains("secret_key") || trimmed.contains("api_key") || trimmed.contains("password")
                || active_secret_vars.iter().any(|v| trimmed.contains(&format!("({}", v)));
            if !trimmed.starts_with("pub fn") && !trimmed.starts_with("fn ")
                && is_log_call && has_secret_log
                && !trimmed.contains("declassify") && !trimmed.contains("redact")
            {
                self.violations.push(SecurityViolation {
                    code: "E0926".to_string(),
                    title: "Secret Exposure in Logging Sink (CWE-532)".to_string(),
                    message: format!(
                        "Illegal secret emission to public logger/stdout at line {}. Secret types must never be logged.",
                        l_num
                    ),
                    severity: "CRITICAL".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-532".to_string()),
                    sink_kind: Some(VulnerabilitySinkKind::SecretLogging),
                    remediation: "Remove logging of secret values or use explicit masked/redacted representation.".to_string(),
                });
            }

            // Check Secret Serialization
            let is_serial_call = trimmed.contains("to_json(") || trimmed.contains("serialize(") || trimmed.contains("encode_json(");
            let has_secret_serial = trimmed.contains("secret<") || trimmed.contains("private_key") || trimmed.contains("auth_token")
                || active_secret_vars.iter().any(|v| trimmed.contains(&format!("({}", v)));
            if !trimmed.starts_with("pub fn") && !trimmed.starts_with("fn ")
                && is_serial_call && has_secret_serial
                && !trimmed.contains("declassify")
            {
                self.violations.push(SecurityViolation {
                    code: "E0927".to_string(),
                    title: "Non-Serializable Secret Serialization Attempt (CWE-359)".to_string(),
                    message: format!(
                        "Attempt to serialize protected secret type at line {}. Secrets are non-serializable by default.",
                        l_num
                    ),
                    severity: "HIGH".to_string(),
                    line: l_num,
                    col: 1,
                    file: self.filename.clone(),
                    cwe_id: Some("CWE-359".to_string()),
                    sink_kind: Some(VulnerabilitySinkKind::SecretSerialization),
                    remediation: "Derive custom public DTO without secret fields before serialization.".to_string(),
                });
            }

            // Constant-time functions inspection
            if trimmed.contains("constant_time") || trimmed.contains("@constant_time") {
                self.constant_time_checked_count += 1;
                // Check if function contains secret-dependent branching
                if (trimmed.contains("if secret") || trimmed.contains("if key") || trimmed.contains("match token"))
                    && !trimmed.contains("constant_time_cmp")
                {
                    self.violations.push(SecurityViolation {
                        code: "E0929".to_string(),
                        title: "Constant-Time Verification Failure (Timing Side-Channel CWE-208)".to_string(),
                        message: format!(
                            "Secret-dependent conditional branching detected in constant_time function at line {}.",
                            l_num
                        ),
                        severity: "CRITICAL".to_string(),
                        line: l_num,
                        col: 1,
                        file: self.filename.clone(),
                        cwe_id: Some("CWE-208".to_string()),
                        sink_kind: Some(VulnerabilitySinkKind::CovertTimingChannel),
                        remediation: "Replace conditional branch with constant-time bitwise operations or `crypto.ct_eq`.".to_string(),
                    });
                }
            }
        }

        // 2. Walk AST to analyze statements, linear nonces, constant-time functions, and security state machines
        for f in &module.functions {
            self.analyze_function_ast(f);
        }
    }

    fn analyze_function_ast(&mut self, func: &FunctionDef) {
        self.push_scope();

        let is_constant_time = func.directives.iter().any(|d| d.name == "@constant_time" || d.name == "constant_time");
        if is_constant_time {
            self.constant_time_checked_count += 1;
            for stmt in &func.body.statements {
                if let Statement::If { span, .. } | Statement::Match { span, .. } = stmt {
                    self.violations.push(SecurityViolation {
                        code: "E0929".to_string(),
                        title: "Constant-Time Verification Failure (Timing Side-Channel CWE-208)".to_string(),
                        message: format!(
                            "Secret-dependent conditional branching detected in constant_time function '{}' at line {}.",
                            func.name, span.line
                        ),
                        severity: "CRITICAL".to_string(),
                        line: span.line,
                        col: span.col,
                        file: self.filename.clone(),
                        cwe_id: Some("CWE-208".to_string()),
                        sink_kind: Some(VulnerabilitySinkKind::CovertTimingChannel),
                        remediation: "Replace conditional branch with constant-time bitwise operations or `crypto.ct_eq`.".to_string(),
                    });
                }
            }
        }

        for p in &func.params {
            let ty_str = p.param_type.to_string();
            if ty_str.contains("tainted") {
                self.declare_security_symbol(&p.name, SecurityTypeKind::Tainted(Box::new(p.param_type.clone())), p.span.line);
            } else if ty_str.contains("secret") {
                self.declare_security_symbol(&p.name, SecurityTypeKind::Secret(Box::new(p.param_type.clone())), p.span.line);
            } else if ty_str.contains("Nonce") {
                self.declare_security_symbol(&p.name, SecurityTypeKind::CryptoNonce(96), p.span.line);
            }
        }

        self.analyze_block_ast(&func.body);
        self.pop_scope();
    }

    fn analyze_block_ast(&mut self, block: &Block) {
        for stmt in &block.statements {
            match stmt {
                Statement::VarDecl { name, var_type, initializer, span, .. } => {
                    let ty_str = var_type.as_ref().map(|t| t.to_string()).unwrap_or_default();
                    if ty_str.contains("tainted") {
                        self.declare_security_symbol(name, SecurityTypeKind::Tainted(Box::new(var_type.clone().unwrap())), span.line);
                    } else if ty_str.contains("secret") {
                        self.declare_security_symbol(name, SecurityTypeKind::Secret(Box::new(var_type.clone().unwrap())), span.line);
                    } else if ty_str.contains("Nonce") {
                        self.declare_security_symbol(name, SecurityTypeKind::CryptoNonce(96), span.line);
                    } else if ty_str.contains("sanitized") {
                        self.declare_security_symbol(name, SecurityTypeKind::Sanitized(Box::new(var_type.clone().unwrap())), span.line);
                    }

                    if let Some(init) = initializer {
                        self.check_expr_sinks(init, span.line, span.col);
                    }
                }
                Statement::Expression(expr) => {
                    self.check_expr_sinks(expr, 0, 0);
                }
                Statement::If { condition, then_block, else_block, .. } => {
                    self.check_expr_sinks(condition, 0, 0);
                    self.analyze_block_ast(then_block);
                    if let Some(eb) = else_block {
                        self.analyze_block_ast(eb);
                    }
                }
                Statement::While { condition, body, .. } => {
                    self.check_expr_sinks(condition, 0, 0);
                    self.analyze_block_ast(body);
                }
                _ => {}
            }
        }
    }

    fn check_expr_sinks(&mut self, expr: &Expression, line: usize, col: usize) {
        if let Expression::Call { callee, args, span } = expr {
            let callee_name = match callee.as_ref() {
                Expression::Ident(n, _) => n.clone(),
                _ => String::new(),
            };

            // Linear Nonce Consumption tracking
            if callee_name == "consume_nonce" || callee_name == "encrypt_with_nonce" || callee_name == "use_nonce" {
                for arg in args {
                    if let Expression::Ident(nonce_var, _) = arg {
                        if let Err(err) = self.mark_nonce_consumed(nonce_var, span.line, span.col) {
                            self.violations.push(err);
                        }
                    }
                }
            }

            // Declassification call validation
            if callee_name == "declassify" {
                if let Some(first_arg) = args.get(0) {
                    if let Expression::Ident(sec_var, _) = first_arg {
                        let proof_name = if args.len() > 1 { "RedactionProof" } else { "" };
                        if let Err(err) = self.declassify_symbol(sec_var, proof_name, span.line, span.col) {
                            self.violations.push(err);
                        }
                    }
                }
            }
        }
    }
}
