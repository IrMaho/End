use crate::ast::*;
use crate::semantic::analyzer::DiagnosticError;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub struct InheritanceHierarchy {
    pub class_parents: HashMap<String, Vec<String>>,
    pub class_mixins: HashMap<String, Vec<String>>,
    pub class_implements: HashMap<String, Vec<String>>,
    pub sealed_classes: HashSet<String>,
    pub abstract_classes: HashSet<String>,
    pub abstract_methods: HashMap<String, HashSet<String>>,
    pub declared_methods: HashMap<String, HashSet<String>>,
    pub conflict_resolutions: HashMap<String, String>, // left+right -> preferred
    pub anti_pattern_warnings: Vec<String>,
}

impl InheritanceHierarchy {
    pub fn new() -> Self {
        Self {
            class_parents: HashMap::new(),
            class_mixins: HashMap::new(),
            class_implements: HashMap::new(),
            sealed_classes: HashSet::new(),
            abstract_classes: HashSet::new(),
            abstract_methods: HashMap::new(),
            declared_methods: HashMap::new(),
            conflict_resolutions: HashMap::new(),
            anti_pattern_warnings: Vec::new(),
        }
    }

    pub fn register_class(&mut self, class_def: &ClassDef) {
        self.class_parents.insert(class_def.name.clone(), class_def.extends.clone());
        self.class_mixins.insert(class_def.name.clone(), class_def.mixins.clone());
        self.class_implements.insert(class_def.name.clone(), class_def.implements.clone());

        if class_def.is_sealed {
            self.sealed_classes.insert(class_def.name.clone());
        }
        if class_def.is_abstract {
            self.abstract_classes.insert(class_def.name.clone());
        }

        let mut abs_methods = HashSet::new();
        let mut decl_methods = HashSet::new();

        for m in &class_def.methods {
            decl_methods.insert(m.name.clone());
            if m.directives.iter().any(|d| d.name == "@abstract" || d.name == "abstract") {
                abs_methods.insert(m.name.clone());
            }
        }

        self.abstract_methods.insert(class_def.name.clone(), abs_methods);
        self.declared_methods.insert(class_def.name.clone(), decl_methods);

        // Anti-pattern architectural heuristics:
        // Flagging utility/cross-cutting inheritance (e.g. Logger, DBConnection, Metrics)
        for parent in &class_def.extends {
            let p_lower = parent.to_lowercase();
            if p_lower.contains("logger") || p_lower.contains("database") || p_lower.contains("connection") || p_lower.contains("metric") {
                self.anti_pattern_warnings.push(format!(
                    "Architectural Warning: '{}' inherits from infrastructure utility '{}'. Prefer 'equip {} with {}' or composition.",
                    class_def.name, parent, class_def.name, parent
                ));
            }
        }
    }

    pub fn check_cycles(&self) -> Result<(), DiagnosticError> {
        for (child, _) in &self.class_parents {
            let mut visited = HashSet::new();
            let mut current = child.clone();
            while let Some(parents) = self.class_parents.get(&current) {
                if parents.is_empty() {
                    break;
                }
                for p in parents {
                    if p == child {
                        return Err(
                            DiagnosticError::new(
                                "E_CYCLIC_INHERITANCE",
                                format!("Cyclic inheritance hierarchy detected involving class '{}'", child),
                                1,
                                1,
                                "SemanticError",
                            )
                            .with_suggestion(format!("Break the cyclic dependency between '{}' and '{}'", child, p)),
                        );
                    }
                    if !visited.insert(p.clone()) {
                        break;
                    }
                    current = p.clone();
                }
            }
        }
        Ok(())
    }

    pub fn compute_mro(&self, class_name: &str) -> Vec<String> {
        let mut order = vec![class_name.to_string()];
        let mut queue = vec![class_name.to_string()];
        let mut seen = HashSet::new();
        seen.insert(class_name.to_string());

        while let Some(curr) = queue.pop() {
            if let Some(parents) = self.class_parents.get(&curr) {
                for p in parents {
                    if seen.insert(p.clone()) {
                        order.push(p.clone());
                        queue.push(p.clone());
                    }
                }
            }
            if let Some(mixins) = self.class_mixins.get(&curr) {
                for m in mixins {
                    if seen.insert(m.clone()) {
                        order.push(m.clone());
                        queue.push(m.clone());
                    }
                }
            }
        }
        order
    }

    pub fn check_abstract_implementations(&self) -> Vec<DiagnosticError> {
        let mut errors = Vec::new();
        for (class_name, parents) in &self.class_parents {
            if self.abstract_classes.contains(class_name) {
                continue; // Abstract classes do not need to implement abstract methods
            }
            let declared = self.declared_methods.get(class_name).cloned().unwrap_or_default();
            for p in parents {
                if let Some(abs_set) = self.abstract_methods.get(p) {
                    for abs_m in abs_set {
                        if !declared.contains(abs_m) {
                            errors.push(
                                DiagnosticError::new(
                                    "E_UNIMPLEMENTED_ABSTRACT_METHOD",
                                    format!("Concrete class '{}' fails to implement abstract method '{}' from parent '{}'", class_name, abs_m, p),
                                    1,
                                    1,
                                    "SemanticError",
                                )
                                .with_suggestion(format!("Provide an implementation for 'fn {}()' in class '{}'", abs_m, class_name)),
                            );
                        }
                    }
                }
            }
        }
        errors
    }
}
