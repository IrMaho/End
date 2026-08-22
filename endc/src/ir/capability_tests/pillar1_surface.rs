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

// ── Item 09: access <Entity>.<Capability> ──
#[test]
fn test_item09_explicit_access_request() {
    let code = r#"
        access Network.Http;
    "#;
    let module = parse_code(code);
    if let Statement::AccessCapability { entity, capability, .. } = &module.statements[0] {
        assert_eq!(entity, "Network");
        assert_eq!(capability, "Http");
    } else {
        panic!("Expected AccessCapability");
    }
}

#[test]
fn test_item09_multi_access() {
    let code = r#"
        access FileSystem.Read;
        access FileSystem.Write;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item09_access_in_checker() {
    let code = r#"
        access Hardware.GpuCompute;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::AccessCapability { entity, capability, .. } = &module.statements[0] {
        assert_eq!(entity, "Hardware");
        assert_eq!(capability, "GpuCompute");
    } else {
        panic!("Expected AccessCapability");
    }
}

// ── Item 10: grant <Entity> { <Capability> } ──
#[test]
fn test_item10_grant_single_capability() {
    let code = r#"
        grant PaymentService { NetworkAccess };
    "#;
    let module = parse_code(code);
    if let Statement::GrantCapability { target, capabilities, .. } = &module.statements[0] {
        assert_eq!(target, "PaymentService");
        assert_eq!(capabilities, &vec!["NetworkAccess".to_string()]);
    } else {
        panic!("Expected GrantCapability");
    }
}

#[test]
fn test_item10_grant_multiple_capabilities() {
    let code = r#"
        grant AuthService {
            TokenSigner,
            KeyManager,
            AuditLogger
        };
    "#;
    let module = parse_code(code);
    if let Statement::GrantCapability { capabilities, .. } = &module.statements[0] {
        assert_eq!(capabilities.len(), 3);
    } else {
        panic!("Expected grant multiple");
    }
}

#[test]
fn test_item10_grant_checker_security() {
    let code = r#"
        grant WorkerThread { SharedMemory };
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::GrantCapability { target, capabilities, .. } = &module.statements[0] {
        assert_eq!(target, "WorkerThread");
        assert_eq!(capabilities, &vec!["SharedMemory".to_string()]);
    } else {
        panic!("Expected GrantCapability");
    }
}

// ── Item 11: deny <Entity> { <Capability> } ──
#[test]
fn test_item11_deny_single_capability() {
    let code = r#"
        deny UntrustedPlugin { RawSocketAccess };
    "#;
    let module = parse_code(code);
    if let Statement::DenyCapability { target, capabilities, .. } = &module.statements[0] {
        assert_eq!(target, "UntrustedPlugin");
        assert_eq!(capabilities, &vec!["RawSocketAccess".to_string()]);
    } else {
        panic!("Expected DenyCapability");
    }
}

#[test]
fn test_item11_deny_multi_capabilities() {
    let code = r#"
        deny SandboxEnv {
            FileWrite,
            NetworkListen,
            ProcessSpawn
        };
    "#;
    let module = parse_code(code);
    if let Statement::DenyCapability { capabilities, .. } = &module.statements[0] {
        assert_eq!(capabilities.len(), 3);
    } else {
        panic!("Expected deny multi");
    }
}

#[test]
fn test_item11_deny_checker_enforcement() {
    let code = r#"
        deny GuestRole { AdminConsole };
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert!(report.denied_capabilities.get("GuestRole").unwrap().contains(&"AdminConsole".to_string()));
}

// ── Item 12: expose <Entity>.<Surface> ──
#[test]
fn test_item12_expose_public_surface() {
    let code = r#"
        expose Payments.PublicApi;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, symbols, .. } = &module.statements[0] {
        assert_eq!(target, "Payments.PublicApi");
        assert_eq!(symbols, &vec!["__expose__".to_string()]);
    } else {
        panic!("Expected expose UseSurface");
    }
}

#[test]
fn test_item12_multi_expose() {
    let code = r#"
        expose User.Profile;
        expose User.Settings;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item12_expose_deep_entity() {
    let code = r#"
        expose Core.Security.TokenVerifier.v2;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, .. } = &module.statements[0] {
        assert_eq!(target, "Core.Security.TokenVerifier.v2");
    } else {
        panic!("Expected deep expose");
    }
}

// ── Item 13: hide <Entity>.<Surface> ──
#[test]
fn test_item13_hide_internal_surface() {
    let code = r#"
        hide Payments.InternalLedger;
    "#;
    let module = parse_code(code);
    if let Statement::DenyCapability { target, capabilities, .. } = &module.statements[0] {
        assert_eq!(target, "Payments");
        assert_eq!(capabilities, &vec!["InternalLedger".to_string()]);
    } else {
        panic!("Expected hide DenyCapability");
    }
}

#[test]
fn test_item13_multi_hide() {
    let code = r#"
        hide SecretManager.master_key;
        hide SecretManager.salt;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item13_hide_checker() {
    let code = r#"
        hide Kernel.raw_pointers;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert!(report.denied_capabilities.get("Kernel").unwrap().contains(&"raw_pointers".to_string()));
}

// ── Item 14: surface <Entity>.<Name> { ... } ──
#[test]
fn test_item14_surface_declaration() {
    let code = r#"
        surface Payments.Public { pay, refund };
    "#;
    let module = parse_code(code);
    if let Statement::SurfaceDefinition(s) = &module.statements[0] {
        assert_eq!(s.entity, "Payments");
        assert_eq!(s.name, "Public");
        assert_eq!(s.symbols, vec!["pay", "refund"]);
        assert!(s.condition.is_none());
    } else {
        panic!("Expected SurfaceDefinition");
    }
}

#[test]
fn test_item14_multi_surfaces() {
    let code = r#"
        surface User.PublicProfile { username, avatar };
        surface User.PrivateSettings { email, billing_address };
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item14_surface_checker() {
    let code = r#"
        surface Storage.ReadSurface { get_item, list_all };
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.surfaces.get("Storage").unwrap()[0].name, "ReadSurface");
}

// ── Item 15: surface <Entity>.<Name> when <Condition> { ... } ──
#[test]
fn test_item15_conditional_surface() {
    let code = r#"
        surface Payments.Admin when environment == "staging" {
            force_refund,
            override_limit
        };
    "#;
    let module = parse_code(code);
    if let Statement::SurfaceDefinition(s) = &module.statements[0] {
        assert_eq!(s.entity, "Payments");
        assert_eq!(s.name, "Admin");
        assert_eq!(s.condition.as_deref(), Some("environment == \"staging\""));
        assert_eq!(s.symbols.len(), 2);
    } else {
        panic!("Expected conditional SurfaceDefinition");
    }
}

#[test]
fn test_item15_surface_role_condition() {
    let code = r#"
        surface AdminPanel.Debug when role == "superadmin" {
            dump_heap,
            kill_process
        };
    "#;
    let module = parse_code(code);
    if let Statement::SurfaceDefinition(s) = &module.statements[0] {
        assert_eq!(s.condition.as_deref(), Some("role == \"superadmin\""));
    } else {
        panic!("Expected conditional surface");
    }
}

#[test]
fn test_item15_surface_checker_condition() {
    let code = r#"
        surface Metrics.Exporter when mode == "prod" { push_telemetry };
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert!(report.surfaces.get("Metrics").unwrap()[0].condition.is_some());
}
