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

// ── Item 16: adopt <Entity/Contract> ──
#[test]
fn test_item16_adopt_basic() {
    let code = r#"
        adopt PaymentProvider;
    "#;
    let module = parse_code(code);
    if let Statement::AdoptContract { target, alias, .. } = &module.statements[0] {
        assert_eq!(target, "PaymentProvider");
        assert!(alias.is_none());
    } else {
        panic!("Expected AdoptContract");
    }
}

#[test]
fn test_item16_adopt_with_alias() {
    let code = r#"
        adopt PaymentProvider as Stripe;
    "#;
    let module = parse_code(code);
    if let Statement::AdoptContract { target, alias, .. } = &module.statements[0] {
        assert_eq!(target, "PaymentProvider");
        assert_eq!(alias.as_deref(), Some("Stripe"));
    } else {
        panic!("Expected AdoptContract with alias");
    }
}

#[test]
fn test_item16_multi_adopt() {
    let code = r#"
        adopt StorageEngine as S3;
        adopt CacheProvider as Redis;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 17: implement <Contract> { ... } ──
#[test]
fn test_item17_implement_contract() {
    let code = r#"
        implement PaymentProvider {
            fn pay(amount: int) -> bool { ret true; }
        }
    "#;
    let module = parse_code(code);
    if let Statement::ImplementContract { contract, methods, .. } = &module.statements[0] {
        assert_eq!(contract, "PaymentProvider");
        assert_eq!(methods.len(), 1);
        assert_eq!(methods[0].name, "pay");
    } else {
        panic!("Expected ImplementContract");
    }
}

#[test]
fn test_item17_implement_for_target() {
    let code = r#"
        implement PaymentProvider for StripeAdapter {
            fn refund(id: string) -> bool { ret true; }
        }
    "#;
    let module = parse_code(code);
    if let Statement::ImplementContract { contract, target, methods, .. } = &module.statements[0] {
        assert_eq!(contract, "PaymentProvider");
        assert_eq!(target.as_deref(), Some("StripeAdapter"));
        assert_eq!(methods.len(), 1);
    } else {
        panic!("Expected ImplementContract for target");
    }
}

#[test]
fn test_item17_implement_multi_methods() {
    let code = r#"
        implement Storage {
            fn get(key: string) -> string { ret "val"; }
            fn set(key: string, val: string) { }
        }
    "#;
    let module = parse_code(code);
    if let Statement::ImplementContract { methods, .. } = &module.statements[0] {
        assert_eq!(methods.len(), 2);
    } else {
        panic!("Expected ImplementContract");
    }
}

// ── Item 18: extend <Entity> { ... } ──
#[test]
fn test_item18_extend_entity() {
    let code = r#"
        extend Payment {
            fn refund(id: string) -> bool { ret true; }
        }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::ImplementContract { contract, methods, .. } = &module.statements[0] {
        assert_eq!(contract, "Payment");
        assert_eq!(methods.len(), 1);
    } else {
        panic!("Expected ImplementContract");
    }
}

#[test]
fn test_item18_extend_with_multi_methods() {
    let code = r#"
        extend UserSession {
            fn refresh_token() { }
            fn terminate() { }
        }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::ImplementContract { contract, methods, .. } = &module.statements[0] {
        assert_eq!(contract, "UserSession");
        assert_eq!(methods.len(), 2);
    } else {
        panic!("Expected ImplementContract");
    }
}

#[test]
fn test_item18_extend_pipeline() {
    let code = r#"
        extend Pipeline {
            fn flush() { }
        }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
}

// ── Item 19: attach <Capability> to <Entity> ──
#[test]
fn test_item19_attach_single() {
    let code = r#"
        attach Logging to Payment;
    "#;
    let module = parse_code(code);
    if let Statement::AttachCapability { capabilities, target, .. } = &module.statements[0] {
        assert_eq!(capabilities, &vec!["Logging"]);
        assert_eq!(target, "Payment");
    } else {
        panic!("Expected AttachCapability");
    }
}

#[test]
fn test_item19_attach_multiple() {
    let code = r#"
        attach { Logging, Metrics, Tracing } to Payment;
    "#;
    let module = parse_code(code);
    if let Statement::AttachCapability { capabilities, target, .. } = &module.statements[0] {
        assert_eq!(capabilities.len(), 3);
        assert_eq!(target, "Payment");
    } else {
        panic!("Expected multi AttachCapability");
    }
}

#[test]
fn test_item19_attach_checker() {
    let code = r#"
        attach Audit to OrderService;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert!(report.attached_capabilities.get("OrderService").unwrap().contains(&"Audit".to_string()));
}

// ── Item 20: attach <Capability> to <Entity> when <Condition> ──
#[test]
fn test_item20_attach_when_condition() {
    let code = r#"
        attach AdminPanel to User when environment == "production";
    "#;
    let module = parse_code(code);
    if let Statement::AttachCapability { capabilities, target, when_cond, .. } = &module.statements[0] {
        assert_eq!(capabilities, &vec!["AdminPanel"]);
        assert_eq!(target, "User");
        assert!(when_cond.is_some());
    } else {
        panic!("Expected AttachCapability when");
    }
}

#[test]
fn test_item20_attach_when_multi_caps() {
    let code = r#"
        attach { Profiler, HeapDump } to Runtime when debug_mode == true;
    "#;
    let module = parse_code(code);
    if let Statement::AttachCapability { capabilities, when_cond, .. } = &module.statements[0] {
        assert_eq!(capabilities.len(), 2);
        assert!(when_cond.is_some());
    } else {
        panic!("Expected AttachCapability when");
    }
}

#[test]
fn test_item20_attach_when_checker() {
    let code = r#"
        attach RateLimiter to Gateway when traffic > 1000;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.attached_capabilities.get("Gateway").unwrap()[0], "RateLimiter");
}

// ── Item 21: attach <Capability> to <Entity> if <Predicate> ──
#[test]
fn test_item21_attach_if_predicate() {
    let code = r#"
        attach Cache to Product if Product.isExpensive;
    "#;
    let module = parse_code(code);
    if let Statement::AttachCapability { capabilities, target, if_pred, .. } = &module.statements[0] {
        assert_eq!(capabilities, &vec!["Cache"]);
        assert_eq!(target, "Product");
        assert_eq!(if_pred.as_deref(), Some("Product.isExpensive"));
    } else {
        panic!("Expected AttachCapability if");
    }
}

#[test]
fn test_item21_attach_if_dynamic() {
    let code = r#"
        attach EncryptedStorage to Vault if Vault.requires_fips;
    "#;
    let module = parse_code(code);
    if let Statement::AttachCapability { if_pred, .. } = &module.statements[0] {
        assert!(if_pred.is_some());
    } else {
        panic!("Expected if_pred");
    }
}

#[test]
fn test_item21_attach_if_multi() {
    let code = r#"
        attach { Compression, Encryption } to NetworkStream if NetworkStream.is_external;
    "#;
    let module = parse_code(code);
    if let Statement::AttachCapability { capabilities, .. } = &module.statements[0] {
        assert_eq!(capabilities.len(), 2);
    } else {
        panic!("Expected AttachCapability if");
    }
}

// ── Item 22: detach <Capability> from <Entity> ──
#[test]
fn test_item22_detach_single() {
    let code = r#"
        detach Logging from Payment;
    "#;
    let module = parse_code(code);
    if let Statement::DetachCapability { capability, target, .. } = &module.statements[0] {
        assert_eq!(capability, "Logging");
        assert_eq!(target, "Payment");
    } else {
        panic!("Expected DetachCapability");
    }
}

#[test]
fn test_item22_attach_then_detach() {
    let code = r#"
        attach Logging to Payment;
        detach Logging from Payment;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert!(!report.attached_capabilities.get("Payment").unwrap().contains(&"Logging".to_string()));
}

#[test]
fn test_item22_detach_security_guard() {
    let code = r#"
        detach DebugTools from ProductionEnvironment;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
}

// ── Item 23: compose <NewCapability> { ... } ──
#[test]
fn test_item23_compose_capability() {
    let code = r#"
        compose SecurePayment {
            Authentication,
            Encryption,
            Audit,
            RateLimit
        }
    "#;
    let module = parse_code(code);
    if let Statement::ComposeCapability { name, capabilities, .. } = &module.statements[0] {
        assert_eq!(name, "SecurePayment");
        assert_eq!(capabilities.len(), 4);
    } else {
        panic!("Expected ComposeCapability");
    }
}

#[test]
fn test_item23_compose_microservices() {
    let code = r#"
        compose ResilientClient {
            Retry,
            CircuitBreaker,
            Fallback
        }
    "#;
    let module = parse_code(code);
    if let Statement::ComposeCapability { capabilities, .. } = &module.statements[0] {
        assert_eq!(capabilities.len(), 3);
    } else {
        panic!("Expected ComposeCapability");
    }
}

#[test]
fn test_item23_compose_and_attach() {
    let code = r#"
        compose FullAudit { FileLog, RemoteSink }
        attach FullAudit to Payment;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 24: mixin <Name> ──
#[test]
fn test_item24_mixin_declaration() {
    let code = r#"
        mixin Timestamped {
            fn get_timestamp() -> int { ret 100; }
        }
    "#;
    let module = parse_code(code);
    if let Statement::MixinDecl(m) = &module.statements[0] {
        assert_eq!(m.name, "Timestamped");
        assert_eq!(m.methods.len(), 1);
    } else {
        panic!("Expected MixinDecl");
    }
}

#[test]
fn test_item24_multi_mixins() {
    let code = r#"
        mixin Auditable {
            fn audit_event(name: string) { }
        }
        mixin Serializable {
            fn serialize() -> string { ret "{}"; }
        }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item24_empty_mixin() {
    let code = r#"
        mixin MarkerMixin;
    "#;
    let module = parse_code(code);
    if let Statement::MixinDecl(m) = &module.statements[0] {
        assert_eq!(m.name, "MarkerMixin");
    } else {
        panic!("Expected MarkerMixin");
    }
}

// ── Item 25: feature <Entity> with { ... } ──
#[test]
fn test_item25_feature_with_capabilities() {
    let code = r#"
        attach { Logging, Metrics } to Payment;
    "#;
    let module = parse_code(code);
    if let Statement::AttachCapability { capabilities, target, .. } = &module.statements[0] {
        assert_eq!(target, "Payment");
        assert_eq!(capabilities.len(), 2);
    } else {
        panic!("Expected AttachCapability");
    }
}

#[test]
fn test_item25_feature_with_checker() {
    let code = r#"
        attach { Caching, Compression } to WebServer;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.attached_capabilities.get("WebServer").unwrap().len(), 2);
}

#[test]
fn test_item25_multi_feature_attachments() {
    let code = r#"
        attach Auth to ApiGateway;
        attach RateLimit to ApiGateway;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}
