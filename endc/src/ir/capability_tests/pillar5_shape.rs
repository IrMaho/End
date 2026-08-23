use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::*;
use crate::semantic::CapabilityCompositionChecker;

fn parse_code(code: &str) -> Module {
    let mut lexer = Lexer::new("test.end", code);
    let tokens = lexer.tokenize_all().expect("Lexing failed");
    let mut parser = Parser::new("test.end", tokens);
    parser.parse_module("test_mod").expect("Parsing failed")
}

// ── Item 45: feature_switch <Name> { enabled <Env> } ──
#[test]
fn test_item45_feature_switch_basic() {
    let code = r#"
        feature_switch NewCheckout { enabled production }
    "#;
    let module = parse_code(code);
    if let Statement::FeatureSwitchDecl { name, enabled_env, .. } = &module.statements[0] {
        assert_eq!(name, "NewCheckout");
        assert_eq!(enabled_env, "production");
    } else {
        panic!("Expected FeatureSwitchDecl");
    }
}

#[test]
fn test_item45_feature_switch_staging() {
    let code = r#"
        feature_switch ExperimentalV2 { enabled staging }
    "#;
    let module = parse_code(code);
    if let Statement::FeatureSwitchDecl { name, enabled_env, .. } = &module.statements[0] {
        assert_eq!(name, "ExperimentalV2");
        assert_eq!(enabled_env, "staging");
    } else {
        panic!("Expected FeatureSwitchDecl");
    }
}

#[test]
fn test_item45_multi_feature_switches() {
    let code = r#"
        feature_switch DarkMode { enabled all }
        feature_switch FastCheckout { enabled beta }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 46: augment <Entity> { capability <Name> } ──
#[test]
fn test_item46_augment_single_capability() {
    let code = r#"
        augment User {
            capability Searchable;
        }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::AttachCapability { capabilities, target, .. } = &module.statements[0] {
        assert_eq!(target, "User");
        assert_eq!(capabilities, &vec!["Searchable"]);
    } else {
        panic!("Expected AttachCapability from augment");
    }
}

#[test]
fn test_item46_augment_multi_capabilities() {
    let code = r#"
        augment Product {
            capability Searchable;
            capability Indexable;
        }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::AttachCapability { capabilities, target, .. } = &module.statements[0] {
        assert_eq!(target, "Product");
        assert_eq!(capabilities.len(), 2);
    } else {
        panic!("Expected augment multi");
    }
}

#[test]
fn test_item46_augment_checker() {
    let code = r#"
        augment Order {
            capability Exportable;
        }
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert!(report.attached_capabilities.get("Order").unwrap().contains(&"Exportable".to_string()));
}

// ── Item 47: traitify <Entity> as <Trait> ──
#[test]
fn test_item47_traitify_basic() {
    let code = r#"
        traitify User as Auditable;
    "#;
    let module = parse_code(code);
    if let Statement::TraitifyCheck { entity, trait_name, .. } = &module.statements[0] {
        assert_eq!(entity, "User");
        assert_eq!(trait_name, "Auditable");
    } else {
        panic!("Expected TraitifyCheck");
    }
}

#[test]
fn test_item47_traitify_deep_path() {
    let code = r#"
        traitify Core.Account as Security.Traits.Verifiable;
    "#;
    let module = parse_code(code);
    if let Statement::TraitifyCheck { entity, trait_name, .. } = &module.statements[0] {
        assert_eq!(entity, "Core.Account");
        assert_eq!(trait_name, "Security.Traits.Verifiable");
    } else {
        panic!("Expected TraitifyCheck");
    }
}

#[test]
fn test_item47_traitify_checker_report() {
    let code = r#"
        traitify Payment as Refundable;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.traitify_reports.len(), 1);
    assert!(report.traitify_reports[0].is_conformant);
}

// ── Item 48: equip <Entity> with { ... } ──
#[test]
fn test_item48_equip_with_capabilities() {
    let code = r#"
        equip Payment with {
            Refundable,
            Auditable,
            Retryable
        };
    "#;
    let module = parse_code(code);
    if let Statement::EquipEntity { entity, capabilities, condition, .. } = &module.statements[0] {
        assert_eq!(entity, "Payment");
        assert_eq!(capabilities.len(), 3);
        assert!(condition.is_none());
    } else {
        panic!("Expected EquipEntity");
    }
}

#[test]
fn test_item48_equip_when_condition() {
    let code = r#"
        equip Payment when environment == "production" with FraudProtected;
    "#;
    let module = parse_code(code);
    if let Statement::EquipEntity { entity, capabilities, condition, .. } = &module.statements[0] {
        assert_eq!(entity, "Payment");
        assert_eq!(capabilities, &vec!["FraudProtected"]);
        assert!(condition.is_some());
    } else {
        panic!("Expected EquipEntity when");
    }
}

#[test]
fn test_item48_equip_checker() {
    let code = r#"
        equip WebService with { RateLimiter, Compression };
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.equipped_capabilities.get("WebService").unwrap().len(), 2);
}

// ── Item 49: fuse { ... } as <NewFeature> ──
#[test]
fn test_item49_fuse_features() {
    let code = r#"
        fuse {
            Authentication,
            Authorization,
            Audit
        } as SecureAccess;
    "#;
    let module = parse_code(code);
    if let Statement::FuseFeatures { features, alias, .. } = &module.statements[0] {
        assert_eq!(alias, "SecureAccess");
        assert_eq!(features.len(), 3);
    } else {
        panic!("Expected FuseFeatures");
    }
}

#[test]
fn test_item49_fuse_then_equip() {
    let code = r#"
        fuse { MetricCollector, TracingLogger } as Observability;
        equip AdminPanel with Observability;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item49_fuse_checker() {
    let code = r#"
        fuse { Storage, Cache, Replication } as UnifiedStorage;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.fused_features.get("UnifiedStorage").unwrap().len(), 3);
}

// ── Item 50: shape <Entity>.<Name> { fields... } ──
#[test]
fn test_item50_shape_public() {
    let code = r#"
        shape User.Public {
            id,
            profile
        };
    "#;
    let module = parse_code(code);
    if let Statement::ShapeDefinition(s) = &module.statements[0] {
        assert_eq!(s.entity, "User");
        assert_eq!(s.name, "Public");
        assert_eq!(s.fields, vec!["id", "profile"]);
    } else {
        panic!("Expected ShapeDefinition");
    }
}

#[test]
fn test_item50_multi_shapes_on_entity() {
    let code = r#"
        shape User.Public { id, profile };
        shape User.Admin { id, email, profile, permissions };
        shape User.Authentication { email, password };
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 3);
}

#[test]
fn test_item50_shape_checker() {
    let code = r#"
        shape Payment.Summary { id, amount, status };
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.shapes.get("Payment").unwrap()[0].name, "Summary");
    assert_eq!(report.shapes.get("Payment").unwrap()[0].fields.len(), 3);
}
