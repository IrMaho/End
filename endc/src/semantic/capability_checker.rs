use crate::ast::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub struct CapabilitySurfaceMeta {
    pub entity: String,
    pub name: String,
    pub condition: Option<String>,
    pub symbols: HashSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeMeta {
    pub entity: String,
    pub name: String,
    pub fields: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraitifyReport {
    pub entity: String,
    pub trait_name: String,
    pub is_conformant: bool,
    pub missing_symbols: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CapabilityCompositionReport {
    pub surfaces: HashMap<String, Vec<CapabilitySurfaceMeta>>,
    pub shapes: HashMap<String, Vec<ShapeMeta>>,
    pub attached_capabilities: HashMap<String, Vec<String>>,
    pub equipped_capabilities: HashMap<String, Vec<String>>,
    pub denied_capabilities: HashMap<String, HashSet<String>>,
    pub resolved_contracts: HashMap<String, String>,
    pub contextual_resolutions: HashMap<String, Vec<(String, String)>>,
    pub fused_features: HashMap<String, Vec<String>>,
    pub intercepted_methods: HashMap<String, Vec<String>>,
    pub proxied_targets: HashMap<String, String>,
    pub traitify_reports: Vec<TraitifyReport>,
    pub violations: Vec<String>,
}

impl CapabilityCompositionReport {
    pub fn new() -> Self {
        Self {
            surfaces: HashMap::new(),
            shapes: HashMap::new(),
            attached_capabilities: HashMap::new(),
            equipped_capabilities: HashMap::new(),
            denied_capabilities: HashMap::new(),
            resolved_contracts: HashMap::new(),
            contextual_resolutions: HashMap::new(),
            fused_features: HashMap::new(),
            intercepted_methods: HashMap::new(),
            proxied_targets: HashMap::new(),
            traitify_reports: Vec::new(),
            violations: Vec::new(),
        }
    }
}

pub struct CapabilityCompositionChecker {
    pub report: CapabilityCompositionReport,
    known_entities: HashSet<String>,
    known_capabilities: HashSet<String>,
}

impl CapabilityCompositionChecker {
    pub fn new() -> Self {
        Self {
            report: CapabilityCompositionReport::new(),
            known_entities: HashSet::new(),
            known_capabilities: HashSet::new(),
        }
    }

    pub fn analyze_module(&mut self, module: &Module) -> CapabilityCompositionReport {
        // Collect entities and structs first
        for s in &module.structs {
            self.known_entities.insert(s.name.clone());
        }
        for stmt in &module.statements {
            match stmt {
                Statement::FeatureStatement(f) => {
                    self.known_entities.insert(f.name.clone());
                }
                Statement::CapabilityDecl(c) => {
                    self.known_capabilities.insert(c.name.clone());
                }
                _ => {}
            }
        }

        // Analyze capability and surface statements
        for stmt in &module.statements {
            self.analyze_statement(stmt);
        }

        self.report.clone()
    }

    pub fn analyze_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::SurfaceDefinition(s) => {
                let meta = CapabilitySurfaceMeta {
                    entity: s.entity.clone(),
                    name: s.name.clone(),
                    condition: s.condition.clone(),
                    symbols: s.symbols.iter().cloned().collect(),
                };
                self.report
                    .surfaces
                    .entry(s.entity.clone())
                    .or_default()
                    .push(meta);
            }
            Statement::ShapeDefinition(s) => {
                let meta = ShapeMeta {
                    entity: s.entity.clone(),
                    name: s.name.clone(),
                    fields: s.fields.clone(),
                };
                self.report
                    .shapes
                    .entry(s.entity.clone())
                    .or_default()
                    .push(meta);
            }
            Statement::AttachCapability {
                capabilities,
                target,
                when_cond,
                if_pred,
                ..
            } => {
                for cap in capabilities {
                    self.report
                        .attached_capabilities
                        .entry(target.clone())
                        .or_default()
                        .push(cap.clone());
                }
                let _ = (when_cond, if_pred);
            }
            Statement::DetachCapability {
                capability,
                target,
                ..
            } => {
                if let Some(caps) = self.report.attached_capabilities.get_mut(target) {
                    caps.retain(|c| c != capability);
                }
                if let Some(caps) = self.report.equipped_capabilities.get_mut(target) {
                    caps.retain(|c| c != capability);
                }
            }
            Statement::EquipEntity {
                entity,
                capabilities,
                condition,
                ..
            } => {
                for cap in capabilities {
                    self.report
                        .equipped_capabilities
                        .entry(entity.clone())
                        .or_default()
                        .push(cap.clone());
                }
                let _ = condition;
            }
            Statement::DenyCapability {
                target,
                capabilities,
                ..
            } => {
                let entry = self
                    .report
                    .denied_capabilities
                    .entry(target.clone())
                    .or_default();
                for cap in capabilities {
                    entry.insert(cap.clone());
                }
            }
            Statement::ResolveContract {
                contract,
                implementation,
                condition,
                ..
            } => {
                if let Some(cond) = condition {
                    self.report
                        .contextual_resolutions
                        .entry(contract.clone())
                        .or_default()
                        .push((cond.clone(), implementation.clone()));
                } else {
                    self.report
                        .resolved_contracts
                        .insert(contract.clone(), implementation.clone());
                }
            }
            Statement::FuseFeatures {
                features,
                alias,
                ..
            } => {
                self.report
                    .fused_features
                    .insert(alias.clone(), features.clone());
                self.known_entities.insert(alias.clone());
            }
            Statement::InterceptMethod(i) => {
                let key = format!("{}.{}", i.entity, i.method);
                self.report
                    .intercepted_methods
                    .entry(key)
                    .or_default()
                    .push("interceptor".to_string());
            }
            Statement::ProxyCapability {
                target,
                interceptor,
                ..
            } => {
                self.report
                    .proxied_targets
                    .insert(target.clone(), interceptor.clone());
            }
            Statement::TraitifyCheck {
                entity,
                trait_name,
                ..
            } => {
                let is_conformant = true;
                let missing_symbols = Vec::new();
                self.report.traitify_reports.push(TraitifyReport {
                    entity: entity.clone(),
                    trait_name: trait_name.clone(),
                    is_conformant,
                    missing_symbols,
                });
            }
            Statement::ScopeBoundary(s) => {
                for inner_stmt in &s.statements {
                    self.analyze_statement(inner_stmt);
                }
            }
            Statement::ContextEnv(c) => {
                for inner_stmt in &c.statements {
                    self.analyze_statement(inner_stmt);
                }
            }
            _ => {}
        }
    }
}
