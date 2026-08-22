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

// ── Item 36: project <Entity> { fields... } ──
#[test]
fn test_item36_project_surface() {
    let code = r#"
        project User { id, name, avatar };
    "#;
    let module = parse_code(code);
    if let Statement::ProjectSurface { entity, fields, .. } = &module.statements[0] {
        assert_eq!(entity, "User");
        assert_eq!(fields, &vec!["id", "name", "avatar"]);
    } else {
        panic!("Expected ProjectSurface");
    }
}

#[test]
fn test_item36_project_single_field() {
    let code = r#"
        project Account { balance };
    "#;
    let module = parse_code(code);
    if let Statement::ProjectSurface { fields, .. } = &module.statements[0] {
        assert_eq!(fields, &vec!["balance"]);
    } else {
        panic!("Expected ProjectSurface");
    }
}

#[test]
fn test_item36_multi_projects() {
    let code = r#"
        project User { email };
        project Product { title, price };
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 37: delegate <Entity>.<Method> to <Target> ──
#[test]
fn test_item37_delegate_behavior() {
    let code = r#"
        delegate Payment.refund to RefundProvider;
    "#;
    let module = parse_code(code);
    if let Statement::DelegateBehavior { entity, method, target, .. } = &module.statements[0] {
        assert_eq!(entity, "Payment");
        assert_eq!(method, "refund");
        assert_eq!(target, "RefundProvider");
    } else {
        panic!("Expected DelegateBehavior");
    }
}

#[test]
fn test_item37_delegate_deep_path() {
    let code = r#"
        delegate Service.Worker.process to TaskQueue;
    "#;
    let module = parse_code(code);
    if let Statement::DelegateBehavior { entity, method, target, .. } = &module.statements[0] {
        assert_eq!(entity, "Service.Worker");
        assert_eq!(method, "process");
        assert_eq!(target, "TaskQueue");
    } else {
        panic!("Expected DelegateBehavior");
    }
}

#[test]
fn test_item37_multi_delegates() {
    let code = r#"
        delegate Auth.login to OAuthClient;
        delegate Auth.logout to SessionStore;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 38: proxy <Target> through <Interceptor> ──
#[test]
fn test_item38_proxy_capability() {
    let code = r#"
        proxy Storage through SecureStorage;
    "#;
    let module = parse_code(code);
    if let Statement::ProxyCapability { target, interceptor, .. } = &module.statements[0] {
        assert_eq!(target, "Storage");
        assert_eq!(interceptor, "SecureStorage");
    } else {
        panic!("Expected ProxyCapability");
    }
}

#[test]
fn test_item38_proxy_pipeline() {
    let code = r#"
        proxy HttpClient through RateLimiter;
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert_eq!(report.proxied_targets.get("HttpClient").unwrap(), "RateLimiter");
}

#[test]
fn test_item38_multi_proxies() {
    let code = r#"
        proxy RawDb through CachedDb;
        proxy Network through Firewall;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 39: decorate <Entity> with { ... } ──
#[test]
fn test_item39_decorate_entity() {
    let code = r#"
        decorate Payment with { Logging, Metrics };
    "#;
    let module = parse_code(code);
    if let Statement::DecorateEntity { target, method, with_caps, .. } = &module.statements[0] {
        assert_eq!(target, "Payment");
        assert!(method.is_none());
        assert_eq!(with_caps.len(), 2);
    } else {
        panic!("Expected DecorateEntity");
    }
}

#[test]
fn test_item39_decorate_method() {
    let code = r#"
        decorate Payment.refund with Audit;
    "#;
    let module = parse_code(code);
    if let Statement::DecorateEntity { target, method, with_caps, .. } = &module.statements[0] {
        assert_eq!(target, "Payment");
        assert_eq!(method.as_deref(), Some("refund"));
        assert_eq!(with_caps, &vec!["Audit"]);
    } else {
        panic!("Expected DecorateEntity method");
    }
}

#[test]
fn test_item39_multi_decorators() {
    let code = r#"
        decorate Controller.action with RateLimit;
        decorate Controller.action with AuthCheck;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 40: intercept <Entity>.<Method> { before { ... } after { ... } } ──
#[test]
fn test_item40_intercept_before_after() {
    let code = r#"
        intercept Payment.refund {
            before {
                log_start();
            }
            after {
                audit_done();
            }
        }
    "#;
    let module = parse_code(code);
    if let Statement::InterceptMethod(i) = &module.statements[0] {
        assert_eq!(i.entity, "Payment");
        assert_eq!(i.method, "refund");
        assert!(i.before_block.is_some());
        assert!(i.after_block.is_some());
    } else {
        panic!("Expected InterceptMethod");
    }
}

#[test]
fn test_item40_intercept_before_only() {
    let code = r#"
        intercept Auth.login {
            before {
                check_rate_limit();
            }
        }
    "#;
    let module = parse_code(code);
    if let Statement::InterceptMethod(i) = &module.statements[0] {
        assert!(i.before_block.is_some());
        assert!(i.after_block.is_none());
    } else {
        panic!("Expected InterceptMethod");
    }
}

#[test]
fn test_item40_intercept_checker() {
    let code = r#"
        intercept Order.place {
            after {
                emit_event();
            }
        }
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert!(report.intercepted_methods.contains_key("Order.place"));
}

// ── Item 41: hook <Entity>.<EventPoint> { ... } ──
#[test]
fn test_item41_hook_event() {
    let code = r#"
        hook Payment.after_refund {
            notify_user();
        }
    "#;
    let module = parse_code(code);
    if let Statement::HookEvent(h) = &module.statements[0] {
        assert_eq!(h.entity, "Payment");
        assert_eq!(h.event_point, "after_refund");
        assert_eq!(h.body.statements.len(), 1);
    } else {
        panic!("Expected HookEvent");
    }
}

#[test]
fn test_item41_hook_nested_entity() {
    let code = r#"
        hook System.Lifecycle.on_shutdown {
            flush_logs();
        }
    "#;
    let module = parse_code(code);
    if let Statement::HookEvent(h) = &module.statements[0] {
        assert_eq!(h.entity, "System.Lifecycle");
        assert_eq!(h.event_point, "on_shutdown");
    } else {
        panic!("Expected HookEvent");
    }
}

#[test]
fn test_item41_multi_hooks() {
    let code = r#"
        hook Server.on_start { init_db(); }
        hook Server.on_stop { close_db(); }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 42: enable / disable <Capability> for <Entity> ──
#[test]
fn test_item42_enable_capability() {
    let code = r#"
        enable Audit for Payments;
    "#;
    let module = parse_code(code);
    if let Statement::EnableCapability { capability, entity, enabled, .. } = &module.statements[0] {
        assert_eq!(capability, "Audit");
        assert_eq!(entity, "Payments");
        assert!(enabled);
    } else {
        panic!("Expected EnableCapability");
    }
}

#[test]
fn test_item42_disable_capability() {
    let code = r#"
        disable Audit for Payments;
    "#;
    let module = parse_code(code);
    if let Statement::EnableCapability { capability, entity, enabled, .. } = &module.statements[0] {
        assert_eq!(capability, "Audit");
        assert_eq!(entity, "Payments");
        assert!(!enabled);
    } else {
        panic!("Expected Disable Capability");
    }
}

#[test]
fn test_item42_enable_disable_sequence() {
    let code = r#"
        enable DebugMode for Engine;
        disable DebugMode for Engine;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 43: scope <Name> { enable <Capability>; ... } ──
#[test]
fn test_item43_scope_boundary() {
    let code = r#"
        scope Admin {
            enable DangerousOperations for System;
        }
    "#;
    let module = parse_code(code);
    if let Statement::ScopeBoundary(s) = &module.statements[0] {
        assert_eq!(s.name, "Admin");
        assert_eq!(s.statements.len(), 1);
    } else {
        panic!("Expected ScopeBoundary");
    }
}

#[test]
fn test_item43_scope_multi_statements() {
    let code = r#"
        scope ProductionWorker {
            enable Metrics for Worker;
            disable VerboseLogging for Worker;
        }
    "#;
    let module = parse_code(code);
    if let Statement::ScopeBoundary(s) = &module.statements[0] {
        assert_eq!(s.statements.len(), 2);
    } else {
        panic!("Expected ScopeBoundary");
    }
}

#[test]
fn test_item43_nested_scopes() {
    let code = r#"
        scope Root {
            scope Child {
                enable FeatureX for ModuleY;
            }
        }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
}

// ── Item 44: context <Environment> { ... } ──
#[test]
fn test_item44_context_production() {
    let code = r#"
        context production {
            use Storage as PostgreSQL;
            enable Metrics for Core;
            deny DebugTools { dump };
        }
    "#;
    let module = parse_code(code);
    if let Statement::ContextEnv(c) = &module.statements[0] {
        assert_eq!(c.environment, "production");
        assert_eq!(c.statements.len(), 3);
    } else {
        panic!("Expected ContextEnv");
    }
}

#[test]
fn test_item44_context_testing() {
    let code = r#"
        context testing {
            use Storage as Memory;
            enable DebugTools for Core;
        }
    "#;
    let module = parse_code(code);
    if let Statement::ContextEnv(c) = &module.statements[0] {
        assert_eq!(c.environment, "testing");
        assert_eq!(c.statements.len(), 2);
    } else {
        panic!("Expected ContextEnv");
    }
}

#[test]
fn test_item44_context_checker() {
    let code = r#"
        context staging {
            attach StagingTelemetry to App;
        }
    "#;
    let module = parse_code(code);
    let mut checker = CapabilityCompositionChecker::new();
    let report = checker.analyze_module(&module);
    assert!(report.attached_capabilities.contains_key("App"));
}
