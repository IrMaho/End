use super::types::*;
use super::SemanticAnalyzer;
use crate::ast::*;
use crate::semantic::graph::*;
use std::collections::HashSet;

impl SemanticAnalyzer {
    pub fn analyze_module(&mut self, module: &Module) -> Result<(), Vec<DiagnosticError>> {
        // 1. Register Enums
        for e in &module.enums {
            self.enums.insert(e.name.clone(), e.clone());
            let variant_names = e.variants.iter().map(|v| v.name.clone()).collect::<Vec<_>>();
            let info = SymbolInfo {
                name: e.name.clone(),
                kind: "enum".to_string(),
                type_signature: format!("enum {} {{ {} }}", e.name, variant_names.join(", ")),
                file: e.span.file.clone(),
                defined_at_line: e.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: true,
                memory_region: None,
                capabilities: CapabilityContract::default(),
            };
            self.graph.symbols.insert(e.name.clone(), info);
        }

        // Register Extensions
        for ext in &module.extensions {
            let info = SymbolInfo {
                name: format!("extend_{}", ext.target),
                kind: "extension".to_string(),
                type_signature: format!("extend {}", ext.target),
                file: ext.span.file.clone(),
                defined_at_line: ext.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: true,
                memory_region: None,
                capabilities: CapabilityContract::default(),
            };
            self.graph.symbols.insert(format!("extend_{}", ext.target), info);
            for f in &ext.functions {
                let mangled = format!("{}_{}", ext.target, f.name);
                let param_types = f.params.iter().map(|p| p.param_type.clone()).collect();
                self.function_signatures.insert(mangled.clone(), (param_types, f.return_type.clone(), true));
            }
        }

        // Register Features, Contracts, and Architecture Rules
        for feat in &module.features {
            self.features.insert(feat.name.clone(), feat.clone());

            // E0931: Uncontracted Feature Implementation
            if !feat.implementations.is_empty() && feat.contracts.is_empty() && feat.api.is_none() {
                self.errors.push(
                    DiagnosticError::new(
                        "E0931",
                        format!("UncontractedFeature: feature '{}' has implementations but no contract or API boundary", feat.name),
                        feat.span.line,
                        feat.span.col,
                        "ArchitecturalViolation",
                    )
                    .with_suggestion(format!("declare a 'contract' or 'api' block for feature '{}'", feat.name)),
                );
            }

            // E0937: 7-Pillar @evolvable Audit (must have extension points or contracts)
            if feat.is_evolvable && feat.extensions.is_empty() && feat.contracts.is_empty() {
                self.errors.push(
                    DiagnosticError::new(
                        "E0937",
                        format!("UnboundedEvolvableFeature: evolvable feature '{}' must define at least one extension point or contract clause", feat.name),
                        feat.span.line,
                        feat.span.col,
                        "ArchitecturalViolation",
                    )
                    .with_suggestion(format!("add an 'extension_point' or 'contract' to feature '{}'", feat.name)),
                );
            }

            // Register dependencies
            for dep in &feat.needs {
                self.module_depends.entry(feat.name.clone()).or_default().insert(dep.name.clone());
            }

            // Check circular dependencies
            for dep in &feat.needs {
                if let Some(other) = self.features.get(&dep.name) {
                    if other.needs.iter().any(|d| d.name == feat.name) {
                        self.errors.push(
                            DiagnosticError::new(
                                "E0934",
                                format!("CircularFeatureDependency: circular dependency detected between feature '{}' and '{}'", feat.name, dep.name),
                                feat.span.line,
                                feat.span.col,
                                "ArchitecturalViolation",
                            )
                            .with_suggestion(format!("decouple dependency between '{}' and '{}' using contracts or events", feat.name, dep.name)),
                        );
                    }
                }
            }
        }

        for ctr in &module.contracts {
            self.contracts.insert(ctr.name.clone(), ctr.clone());
        }

        for rule in &module.architecture_rules {
            self.architecture_rules.push(rule.clone());
        }

        // Register Modules
        for m in &module.modules {
            if let Some(ref r) = m.responsibility {
                self.module_responsibilities.insert(m.name.clone(), r.clone());
            }
            if !m.owns.is_empty() {
                self.module_owns.insert(m.name.clone(), m.owns.iter().cloned().collect());
            }
            if !m.exposes.is_empty() {
                self.module_exposes.insert(m.name.clone(), m.exposes.iter().cloned().collect());
            }
            if !m.depends.is_empty() {
                self.module_depends.insert(m.name.clone(), m.depends.iter().cloned().collect());
            }
            if let Some(ref d_only) = m.depends_only {
                self.module_depends_only.insert(m.name.clone(), d_only.iter().cloned().collect());
            }
            if !m.forbid.is_empty() {
                self.module_forbidden.insert(m.name.clone(), m.forbid.iter().cloned().collect());
            }
            if m.is_sealed {
                self.module_sealed.insert(m.name.clone());
                self.sealed_modules.insert(m.name.clone());
            }
            if let Some(ref p) = m.purity {
                self.module_purity.insert(m.name.clone(), p.clone());
            }
            if let Some(thresh) = m.cohesion {
                if thresh < 0.5 {
                    self.errors.push(
                        DiagnosticError::new(
                            "E0917",
                            format!("CohesionBelowThreshold: module '{}' cohesion ({:.2}) is below threshold (0.50)", m.name, thresh),
                            m.span.line,
                            m.span.col,
                            "ArchitecturalViolation",
                        )
                        .with_suggestion(format!("decompose module '{}' to improve cohesion", m.name)),
                    );
                }
            }

            let info = SymbolInfo {
                name: m.name.clone(),
                kind: "module".to_string(),
                type_signature: format!("mod {} derives {:?}", m.name, m.parent),
                file: m.span.file.clone(),
                defined_at_line: m.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: true,
                memory_region: None,
                capabilities: CapabilityContract::default(),
            };
            self.graph.symbols.insert(m.name.clone(), info);
            for f in &m.functions {
                let mangled = format!("{}_{}", m.name, f.name);
                let param_types = f.params.iter().map(|p| p.param_type.clone()).collect();
                self.function_signatures.insert(mangled, (param_types, f.return_type.clone(), true));
            }
            for ov in &m.overrides {
                let mangled = format!("{}_{}", m.name, ov.name);
                let param_types = ov.params.iter().map(|p| p.param_type.clone()).collect();
                self.function_signatures.insert(mangled, (param_types, ov.return_type.clone(), true));
            }
            for stmt in &m.statements {
                self.analyze_statement(stmt);
            }
        }

        // Process top-level module statements
        for stmt in &module.statements {
            self.analyze_statement(stmt);
        }

        for s in &module.structs {
            self.structs.insert(s.name.clone(), s.clone());
            if s.is_sealed {
                self.sealed_structs.insert(s.name.clone());
            }
            let info = SymbolInfo {
                name: s.name.clone(),
                kind: "struct".to_string(),
                type_signature: format!("struct {}", s.name),
                file: s.span.file.clone(),
                defined_at_line: s.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: true,
                memory_region: None,
                capabilities: CapabilityContract::default(),
            };
            self.graph.symbols.insert(s.name.clone(), info);
        }

        // 3. Register Function Signatures
        for f in &module.functions {
            let mut effect_set = HashSet::new();
            let is_pure = f.directives.iter().any(|d| d.name == "@pure");
            if is_pure {
                effect_set.insert("pure".to_string());
            }

            let param_types = f.params.iter().map(|p| p.param_type.clone()).collect();
            self.function_signatures.insert(f.name.clone(), (param_types, f.return_type.clone(), is_pure));
            self.function_effects.insert(f.name.clone(), effect_set);

            let mut cap = CapabilityContract::default();
            for dir in &f.directives {
                if dir.name == "@pure" {
                    cap.is_pure = true;
                } else if dir.name == "@capability" {
                    for arg in &dir.args {
                        let parts: Vec<&str> = arg.split('=').collect();
                        if parts.len() == 2 {
                            match parts[0].trim() {
                                "net" => cap.net = parts[1].trim().trim_matches('"') == "true",
                                "disk" => cap.disk = parts[1].trim().trim_matches('"') == "true",
                                "io" => cap.io = parts[1].trim().trim_matches('"') == "true",
                                "memory" => cap.memory = parts[1].trim().trim_matches('"').to_string(),
                                _ => {}
                            }
                        }
                    }
                }
            }

            let info = SymbolInfo {
                name: f.name.clone(),
                kind: "function".to_string(),
                type_signature: format!(
                    "fn {}({}) -> {}",
                    f.name,
                    f.params.iter().map(|p| format!("{}: {}", p.name, p.param_type)).collect::<Vec<_>>().join(", "),
                    f.return_type
                ),
                file: f.span.file.clone(),
                defined_at_line: f.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: cap.is_pure,
                memory_region: Some(cap.memory.clone()),
                capabilities: cap,
            };
            self.graph.symbols.insert(f.name.clone(), info);
        }

        // 4. Analyze Function Bodies
        for f in &module.functions {
            self.analyze_function(f);
        }

        // 5. Transitive Effect & Purity Verification
        self.verify_transitive_effects_and_purity();

        // 6. Security-by-Construction & Verified Build Audit
        let full_source = self.source_lines.join("\n");
        let (sec_report, _) = crate::security::SecurityByConstructionEngine::audit_module_and_source(
            &self.graph.filename,
            &full_source,
            module,
            self.security_level,
        );

        for v in sec_report.violations {
            self.errors.push(
                DiagnosticError::new(v.code, v.message, v.line, v.col, v.title)
                    .with_suggestion(v.remediation),
            );
        }

        // Gate 12: Validate that no variable in scope remains with unresolved Type::Unknown
        if self.errors.is_empty() {
            for var_sym in self.env.all_symbols() {
                if var_sym.var_type.is_unknown() {
                    self.errors.push(
                        DiagnosticError::new(
                            "E002",
                            format!("TypeInferenceFailure: could not infer concrete type for variable '{}'", var_sym.name),
                            var_sym.line_def,
                            1,
                            "TypeInferenceError",
                        )
                        .with_expected("concrete type")
                        .with_actual("unknown type")
                        .with_suggestion("provide an explicit type annotation"),
                    );
                }
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    pub(crate) fn analyze_function(&mut self, func: &FunctionDef) {
        self.current_function = Some(func.name.clone());
        self.borrow_checker.clear();
        self.push_scope();

        for p in &func.params {
            self.declare_var(&p.name, p.param_type.clone(), p.span.line, p.is_mut);
        }

        self.analyze_block(&func.body);

        self.pop_scope();
        self.current_function = None;
    }

    pub(crate) fn verify_transitive_effects_and_purity(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            let current_effects = self.function_effects.clone();
            for (caller, callees) in self.graph.symbols.iter().map(|(k, v)| (k.clone(), v.callees.clone())) {
                for callee in callees {
                    if let Some(callee_effects) = current_effects.get(&callee) {
                        if let Some(caller_effects) = self.function_effects.get_mut(&caller) {
                            for eff in callee_effects {
                                if caller_effects.insert(eff.clone()) {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        for (func_name, (_params, _ret, is_pure)) in &self.function_signatures {
            if *is_pure {
                if let Some(effects) = self.function_effects.get(func_name) {
                    let impure_effects: Vec<&String> = effects.iter().filter(|e| *e == "network" || *e == "io" || *e == "database" || *e == "filesystem").collect();
                    if !impure_effects.is_empty() {
                        if let Some(sym) = self.graph.symbols.get(func_name) {
                            self.errors.push(
                                DiagnosticError::new(
                                    "E0904",
                                    format!("PurityViolation: function '{}' is marked @pure but transitively invokes impure operations: {:?}", func_name, impure_effects),
                                    sym.defined_at_line,
                                    1,
                                    "PurityViolationError",
                                )
                                .with_suggestion("remove @pure directive or refactor to isolate side-effects"),
                            );
                        }
                    }
                }
            }
        }
    }
}
