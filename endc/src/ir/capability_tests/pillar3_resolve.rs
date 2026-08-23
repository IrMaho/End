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

// ── Item 26: capability <Name> { ... } ──
#[test]
fn test_item26_capability_definition() {
    let code = r#"
        capability Cache { get, set, invalidate };
    "#;
    let module = parse_code(code);
    if let Statement::CapabilityDecl(c) = &module.statements[0] {
        assert_eq!(c.name, "Cache");
        assert_eq!(c.methods, vec!["get", "set", "invalidate"]);
    } else {
        panic!("Expected CapabilityDecl");
    }
}

#[test]
fn test_item26_capability_empty() {
    let code = r#"
        capability Marker;
    "#;
    let module = parse_code(code);
    if let Statement::CapabilityDecl(c) = &module.statements[0] {
        assert_eq!(c.name, "Marker");
        assert!(c.methods.is_empty());
    } else {
        panic!("Expected Marker CapabilityDecl");
    }
}

#[test]
fn test_item26_multi_capabilities() {
    let code = r#"
        capability Persistence { save, load };
        capability Searchable { search, index };
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 27: provide <Capability> ──
#[test]
fn test_item27_provide_single() {
    let code = r#"
        provide Persistence;
    "#;
    let module = parse_code(code);
    if let Statement::ProvideCapability { capability, .. } = &module.statements[0] {
        assert_eq!(capability, "Persistence");
    } else {
        panic!("Expected ProvideCapability");
    }
}

#[test]
fn test_item27_provide_nested_path() {
    let code = r#"
        provide Storage.Relational.PostgreSQL;
    "#;
    let module = parse_code(code);
    if let Statement::ProvideCapability { capability, .. } = &module.statements[0] {
        assert_eq!(capability, "Storage.Relational.PostgreSQL");
    } else {
        panic!("Expected ProvideCapability");
    }
}

#[test]
fn test_item27_multi_provides() {
    let code = r#"
        provide Caching;
        provide Metrics;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 28: require <Contract/Capability> ──
#[test]
fn test_item28_require_single() {
    let code = r#"
        require PaymentProvider;
    "#;
    let module = parse_code(code);
    if let Statement::RequireCapability { contract, alias, .. } = &module.statements[0] {
        assert_eq!(contract, "PaymentProvider");
        assert!(alias.is_none());
    } else {
        panic!("Expected RequireCapability");
    }
}

#[test]
fn test_item28_require_deep_path() {
    let code = r#"
        require Security.Authentication.OAuth2;
    "#;
    let module = parse_code(code);
    if let Statement::RequireCapability { contract, .. } = &module.statements[0] {
        assert_eq!(contract, "Security.Authentication.OAuth2");
    } else {
        panic!("Expected RequireCapability");
    }
}

#[test]
fn test_item28_multi_requires() {
    let code = r#"
        require DatabaseConnection;
        require LoggingSink;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 29: require <Contract> as <Alias> ──
#[test]
fn test_item29_require_with_alias() {
    let code = r#"
        require PaymentProvider as Provider;
    "#;
    let module = parse_code(code);
    if let Statement::RequireCapability { contract, alias, .. } = &module.statements[0] {
        assert_eq!(contract, "PaymentProvider");
        assert_eq!(alias.as_deref(), Some("Provider"));
    } else {
        panic!("Expected RequireCapability with alias");
    }
}

#[test]
fn test_item29_require_alias_avoid_clash() {
    let code = r#"
        require Service.V1.Client as OldClient;
        require Service.V2.Client as NewClient;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item29_require_alias_structure() {
    let code = r#"
        require CloudStore as Store;
    "#;
    let module = parse_code(code);
    if let Statement::RequireCapability { alias, .. } = &module.statements[0] {
        assert_eq!(alias.as_deref(), Some("Store"));
    } else {
        panic!("Expected alias");
    }
}

// ── Item 30: resolve <Contract> -> <Implementation> ──
#[test]
fn test_item30_resolve_static() {
    let code = r#"
        resolve PaymentProvider -> Stripe;
    "#;
    let module = parse_code(code);
    if let Statement::ResolveContract { contract, implementation, condition, .. } = &module.statements[0] {
        assert_eq!(contract, "PaymentProvider");
        assert_eq!(implementation, "Stripe");
        assert!(condition.is_none());
    } else {
        panic!("Expected ResolveContract");
    }
}

#[test]
fn test_item30_resolve_in_memory() {
    let code = r#"
        resolve Storage -> InMemory;
    "#;
    let module = parse_code(code);
    if let Statement::ResolveContract { implementation, .. } = &module.statements[0] {
        assert_eq!(implementation, "InMemory");
    } else {
        panic!("Expected InMemory");
    }
}

#[test]
fn test_item30_resolve_checker() {
    let code = r#"
        resolve Cache -> Redis;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.resolved_contracts.get("Cache").unwrap(), "Redis");
}

// ── Item 31: resolve <Contract> -> <Implementation> when <Condition> ──
#[test]
fn test_item31_resolve_conditional_environment() {
    let code = r#"
        resolve Storage -> Redis when environment == production;
    "#;
    let module = parse_code(code);
    if let Statement::ResolveContract { contract, implementation, condition, .. } = &module.statements[0] {
        assert_eq!(contract, "Storage");
        assert_eq!(implementation, "Redis");
        assert!(condition.is_some());
    } else {
        panic!("Expected ResolveContract when");
    }
}

#[test]
fn test_item31_resolve_multi_conditions() {
    let code = r#"
        resolve Storage -> Redis when environment == production;
        resolve Storage -> Memory when environment == test;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item31_resolve_checker_conditions() {
    let code = r#"
        resolve Database -> Postgres when env == "prod";
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.contextual_resolutions.get("Database").unwrap().len(), 1);
}

// ── Item 32: select <Contract> { ... } ──
#[test]
fn test_item32_select_candidates() {
    let code = r#"
        select PaymentProvider { Stripe, PayPal, Crypto };
    "#;
    let module = parse_code(code);
    if let Statement::SelectContract { contract, candidates, .. } = &module.statements[0] {
        assert_eq!(contract, "PaymentProvider");
        assert_eq!(candidates.len(), 3);
    } else {
        panic!("Expected SelectContract");
    }
}

#[test]
fn test_item32_select_single_candidate() {
    let code = r#"
        select DatabaseDriver { SQLite };
    "#;
    let module = parse_code(code);
    if let Statement::SelectContract { candidates, .. } = &module.statements[0] {
        assert_eq!(candidates, &vec!["SQLite"]);
    } else {
        panic!("Expected SelectContract");
    }
}

#[test]
fn test_item32_select_multi_blocks() {
    let code = r#"
        select Transport { TCP, UDP, QUIC };
        select Serializer { JSON, Protobuf };
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 33: use <Contract><Implementation> ──
#[test]
fn test_item33_generic_contract_consumption() {
    let code = r#"
        use PaymentProvider<Stripe>;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, is_generic, .. } = &module.statements[0] {
        assert_eq!(target, "PaymentProvider");
        assert_eq!(is_generic.as_deref(), Some("Stripe"));
    } else {
        panic!("Expected generic UseSurface");
    }
}

#[test]
fn test_item33_generic_nested_target() {
    let code = r#"
        use Storage.KV<RedisStore>;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { is_generic, .. } = &module.statements[0] {
        assert_eq!(is_generic.as_deref(), Some("RedisStore"));
    } else {
        panic!("Expected generic param");
    }
}

#[test]
fn test_item33_multi_generic_use() {
    let code = r#"
        use Service<MockService>;
        use Database<InMemoryDb>;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 34: use <Entity> as <Shape/Contract> ──
#[test]
fn test_item34_use_as_shape_contract() {
    let code = r#"
        use User as AdminUser;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, alias, .. } = &module.statements[0] {
        assert_eq!(target, "User");
        assert_eq!(alias.as_deref(), Some("AdminUser"));
    } else {
        panic!("Expected UseSurface as shape");
    }
}

#[test]
fn test_item34_use_payment_as_refundable() {
    let code = r#"
        use Payment as Refundable;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { alias, .. } = &module.statements[0] {
        assert_eq!(alias.as_deref(), Some("Refundable"));
    } else {
        panic!("Expected Refundable alias");
    }
}

#[test]
fn test_item34_multi_shape_casting() {
    let code = r#"
        use Document as Printable;
        use Document as Exportable;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 35: view <Entity> as <ViewShape> ──
#[test]
fn test_item35_view_projection() {
    let code = r#"
        view User as PublicUser;
    "#;
    let module = parse_code(code);
    if let Statement::ViewProjection { entity, view_shape, .. } = &module.statements[0] {
        assert_eq!(entity, "User");
        assert_eq!(view_shape, "PublicUser");
    } else {
        panic!("Expected ViewProjection");
    }
}

#[test]
fn test_item35_view_payment_projection() {
    let code = r#"
        view Payment as RefundablePayment;
    "#;
    let module = parse_code(code);
    if let Statement::ViewProjection { entity, view_shape, .. } = &module.statements[0] {
        assert_eq!(entity, "Payment");
        assert_eq!(view_shape, "RefundablePayment");
    } else {
        panic!("Expected ViewProjection");
    }
}

#[test]
fn test_item35_multi_views() {
    let code = r#"
        view Order as OrderSummary;
        view Order as OrderAudit;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}
