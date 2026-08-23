use crate::ast::*;
use crate::security::taint_analyzer::TaintAndInformationFlowAnalyzer;
use crate::security::types::*;

impl TaintAndInformationFlowAnalyzer {
    pub(crate) fn analyze_function_ast(&mut self, func: &FunctionDef) {
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

    pub(crate) fn analyze_block_ast(&mut self, block: &Block) {
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
                Statement::Guard { condition, else_block, .. } => {
                    self.check_expr_sinks(condition, 0, 0);
                    self.analyze_block_ast(else_block);
                }
                Statement::While { condition, body, .. } => {
                    self.check_expr_sinks(condition, 0, 0);
                    self.analyze_block_ast(body);
                }
                _ => {}
            }
        }
    }

    pub(crate) fn check_expr_sinks(&mut self, expr: &Expression, _line: usize, _col: usize) {
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
