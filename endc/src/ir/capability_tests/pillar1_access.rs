use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::*;

fn parse_code(code: &str) -> Module {
    let mut lexer = Lexer::new("test.end", code);
    let tokens = lexer.tokenize_all().expect("Lexing failed");
    let mut parser = Parser::new("test.end", tokens);
    parser.parse_module("test_mod").expect("Parsing failed")
}

// ── Item 01: use <Entity> ──
#[test]
fn test_item01_semantic_use_basic() {
    let code = r#"
        use User;
        fn get_id() -> int { ret 42; }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    assert_eq!(module.functions.len(), 1);
    if let Statement::UseSurface { target, is_borrow, is_mut, .. } = &module.statements[0] {
        assert_eq!(target, "User");
        assert!(!is_borrow);
        assert!(!is_mut);
    } else {
        panic!("Expected UseSurface statement");
    }
}

#[test]
fn test_item01_semantic_use_multi_composition() {
    let code = r#"
        use User;
        use Payments;
        use Notification;
        fn notify() { }
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 3);
    assert_eq!(module.functions.len(), 1);
}

#[test]
fn test_item01_semantic_checker_integration() {
    let code = r#"
        use Payments;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::UseSurface { target, .. } = &module.statements[0] {
        assert_eq!(target, "Payments");
    } else {
        panic!("Expected UseSurface");
    }
}

// ── Item 02: use <Entity>.<Section>.<Symbol> ──
#[test]
fn test_item02_deep_dotted_surface() {
    let code = r#"
        use Payments.Gateway.Stripe.charge;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, .. } = &module.statements[0] {
        assert_eq!(target, "Payments.Gateway.Stripe.charge");
    } else {
        panic!("Expected UseSurface");
    }
}

#[test]
fn test_item02_multi_dotted_use() {
    let code = r#"
        use Security.Crypto.AES256.encrypt;
        use Security.Crypto.AES256.decrypt;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item02_deep_nested_composition() {
    let code = r#"
        use Cloud.AWS.S3.Bucket.upload_file;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, .. } = &module.statements[0] {
        assert_eq!(target, "Cloud.AWS.S3.Bucket.upload_file");
    } else {
        panic!("Expected UseSurface");
    }
}

// ── Item 03: use <Entity>.section("<Name>") ──
#[test]
fn test_item03_section_accessor() {
    let code = r#"
        use Payments.section("refund");
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, section, .. } = &module.statements[0] {
        assert_eq!(target, "Payments");
        assert_eq!(section.as_deref(), Some("refund"));
    } else {
        panic!("Expected UseSurface with section");
    }
}

#[test]
fn test_item03_multiple_sections() {
    let code = r#"
        use Storage.section("blob");
        use Storage.section("metadata");
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item03_section_with_symbols() {
    let code = r#"
        use Analytics.section("telemetry");
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { section, .. } = &module.statements[0] {
        assert_eq!(section.as_deref(), Some("telemetry"));
    } else {
        panic!("Expected section");
    }
}

// ── Item 04: use <Entity> only { a, b } ──
#[test]
fn test_item04_selective_import() {
    let code = r#"
        use Payments only { refund, RefundResult };
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, symbols, .. } = &module.statements[0] {
        assert_eq!(target, "Payments");
        assert_eq!(symbols, &vec!["refund".to_string(), "RefundResult".to_string()]);
    } else {
        panic!("Expected selective UseSurface");
    }
}

#[test]
fn test_item04_empty_only_block() {
    let code = r#"
        use Security only { };
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, symbols, .. } = &module.statements[0] {
        assert_eq!(target, "Security");
        assert!(symbols.is_empty());
    } else {
        panic!("Expected empty only");
    }
}

#[test]
fn test_item04_multiple_only_clauses() {
    let code = r#"
        use User only { id, name };
        use Order only { total, status };
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

// ── Item 05: use <Entity> as <Alias> ──
#[test]
fn test_item05_alias_entity() {
    let code = r#"
        use Payments.v1 as LegacyPayments;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, alias, .. } = &module.statements[0] {
        assert_eq!(target, "Payments.v1");
        assert_eq!(alias.as_deref(), Some("LegacyPayments"));
    } else {
        panic!("Expected UseSurface with alias");
    }
}

#[test]
fn test_item05_deep_alias() {
    let code = r#"
        use Internal.Engine.Core.Pipeline as PipelineEngine;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, alias, .. } = &module.statements[0] {
        assert_eq!(target, "Internal.Engine.Core.Pipeline");
        assert_eq!(alias.as_deref(), Some("PipelineEngine"));
    } else {
        panic!("Expected deep alias");
    }
}

#[test]
fn test_item05_alias_in_checker() {
    let code = r#"
        use Math.Matrix as Mat;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::UseSurface { alias, .. } = &module.statements[0] {
        assert_eq!(alias.as_deref(), Some("Mat"));
    } else {
        panic!("Expected UseSurface with alias");
    }
}

// ── Item 06: use <Entity> as { a, b } ──
#[test]
fn test_item06_shape_destructuring() {
    let code = r#"
        use User as { id, email };
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, shape_fields, .. } = &module.statements[0] {
        assert_eq!(target, "User");
        assert_eq!(shape_fields, &vec!["id".to_string(), "email".to_string()]);
    } else {
        panic!("Expected shape destructuring");
    }
}

#[test]
fn test_item06_shape_multi_fields() {
    let code = r#"
        use Order as { id, customer_id, total, status, items };
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { shape_fields, .. } = &module.statements[0] {
        assert_eq!(shape_fields.len(), 5);
    } else {
        panic!("Expected shape multi fields");
    }
}

#[test]
fn test_item06_checker_shape_fields() {
    let code = r#"
        use Account as { balance, currency };
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::UseSurface { shape_fields, .. } = &module.statements[0] {
        assert_eq!(shape_fields, &vec!["balance".to_string(), "currency".to_string()]);
    } else {
        panic!("Expected UseSurface");
    }
}

// ── Item 07: borrow <Entity>.<Capability> ──
#[test]
fn test_item07_immutable_borrow() {
    let code = r#"
        borrow User.profile;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, is_borrow, is_mut, .. } = &module.statements[0] {
        assert_eq!(target, "User.profile");
        assert!(is_borrow);
        assert!(!is_mut);
    } else {
        panic!("Expected immutable borrow");
    }
}

#[test]
fn test_item07_multi_borrow() {
    let code = r#"
        borrow Database.read_connection;
        borrow Cache.read_handle;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 2);
}

#[test]
fn test_item07_scoped_borrow() {
    let code = r#"
        borrow Hardware.temperature_sensor;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::UseSurface { is_borrow, .. } = &module.statements[0] {
        assert!(is_borrow);
    } else {
        panic!("Expected UseSurface borrow");
    }
}

// ── Item 08: borrow mut <Entity>.<Capability> ──
#[test]
fn test_item08_mutable_borrow() {
    let code = r#"
        borrow mut User.session;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, is_borrow, is_mut, .. } = &module.statements[0] {
        assert_eq!(target, "User.session");
        assert!(is_borrow);
        assert!(is_mut);
    } else {
        panic!("Expected mutable borrow");
    }
}

#[test]
fn test_item08_exclusive_mutable_borrow() {
    let code = r#"
        borrow mut PaymentGateway.transaction_lock;
    "#;
    let module = parse_code(code);
    if let Statement::UseSurface { target, is_mut, .. } = &module.statements[0] {
        assert_eq!(target, "PaymentGateway.transaction_lock");
        assert!(is_mut);
    } else {
        panic!("Expected exclusive mut borrow");
    }
}

#[test]
fn test_item08_mutable_borrow_in_function() {
    let code = r#"
        borrow mut StateMachine.active_state;
    "#;
    let module = parse_code(code);
    assert_eq!(module.statements.len(), 1);
    if let Statement::UseSurface { is_mut, .. } = &module.statements[0] {
        assert!(is_mut);
    } else {
        panic!("Expected UseSurface mut");
    }
}
