#[cfg(test)]
pub mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::codegen::Interpreter;
    use crate::semantic::inheritance_checker::InheritanceHierarchy;
    use crate::ast::*;

    fn parse_code(code: &str) -> Module {
        let mut lexer = Lexer::new("test.end", code);
        let tokens = lexer.tokenize_all().expect("Lexing failed");
        let mut parser = Parser::new("test.end", tokens);
        parser.parse_module("test_mod").expect("Parsing failed")
    }

    #[test]
    fn test_phase1_class_single_and_multiple_inheritance() {
        let code = r#"
            class User {
                id: i64;
                name: String;
                fn get_name() -> String {
                    return "User";
                }
            }

            class Admin extends User {
                level: i64;
                fn get_level() -> i64 {
                    return 10;
                }
            }

            class SmartCar extends Vehicle, ConnectedDevice {
                fn drive() {
                    return 1;
                }
            }
        "#;
        let module = parse_code(code);
        let mut hierarchy = InheritanceHierarchy::new();
        for stmt in &module.statements {
            if let Statement::ClassDecl(c) = stmt {
                hierarchy.register_class(c);
            }
        }
        assert_eq!(hierarchy.class_parents.get("Admin").unwrap(), &vec!["User".to_string()]);
        assert_eq!(hierarchy.class_parents.get("SmartCar").unwrap(), &vec!["Vehicle".to_string(), "ConnectedDevice".to_string()]);
        assert!(hierarchy.check_cycles().is_ok());
    }

    #[test]
    fn test_phase2_contract_and_trait_inheritance() {
        let code = r#"
            contract Animal {
                fn speak() -> String;
            }

            contract FlyingAnimal extends Animal {
                fn fly() -> i64;
            }

            trait Serializable {
                fn serialize() -> String;
            }

            trait Auditable extends Serializable {
                fn audit_log() -> String;
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.contracts.len(), 2);
        assert_eq!(module.contracts[1].name, "FlyingAnimal");
        assert_eq!(module.contracts[1].extends, vec!["Animal".to_string()]);
    }

    #[test]
    fn test_phase3_abstract_classes_and_methods() {
        let code = r#"
            abstract class Vehicle {
                abstract fn drive() -> i64;
            }

            class Car extends Vehicle {
                fn drive() -> i64 {
                    return 100;
                }
            }

            class BrokenCar extends Vehicle {
                fn honk() -> i64 {
                    return 1;
                }
            }
        "#;
        let module = parse_code(code);
        let mut hierarchy = InheritanceHierarchy::new();
        for stmt in &module.statements {
            if let Statement::ClassDecl(c) = stmt {
                hierarchy.register_class(c);
            }
        }
        assert!(hierarchy.abstract_classes.contains("Vehicle"));
        let errors = hierarchy.check_abstract_implementations();
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].code, "E_UNIMPLEMENTED_ABSTRACT_METHOD");
        assert!(errors[0].message.contains("BrokenCar"));
    }

    #[test]
    fn test_phase4_mixins_and_interfaces() {
        let code = r#"
            class User with Cacheable, Auditable {
                id: i64;
            }

            class Stripe implements PaymentProvider, RefundProvider {
                fn charge(amount: i64) -> bool {
                    return true;
                }
            }

            class Paypal implements { PaymentProvider, HealthCheck } {
                fn charge(amount: i64) -> bool {
                    return true;
                }
            }
        "#;
        let module = parse_code(code);
        let mut hierarchy = InheritanceHierarchy::new();
        for stmt in &module.statements {
            if let Statement::ClassDecl(c) = stmt {
                hierarchy.register_class(c);
                if c.name == "User" {
                    assert_eq!(c.mixins, vec!["Cacheable".to_string(), "Auditable".to_string()]);
                }
                if c.name == "Stripe" {
                    assert_eq!(c.implements, vec!["PaymentProvider".to_string(), "RefundProvider".to_string()]);
                }
                if c.name == "Paypal" {
                    assert_eq!(c.implements, vec!["PaymentProvider".to_string(), "HealthCheck".to_string()]);
                }
            }
        }
    }

    #[test]
    fn test_phase5_semantic_and_selective_inheritance() {
        let code = r#"
            inherit Admin User only { permissions, profile };
            inherit Guest User except { delete, resetPassword() };
            inherit SuperUser User as BaseUser;
            inherit ApiAdmin User.surface Public;
            inherit AccountAdmin User.shape Account;
            inherit AuditedAdmin behavior Auditable;
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 6);

        if let Statement::InheritStmt(ref i) = module.statements[0] {
            assert_eq!(i.target, "Admin");
            assert_eq!(i.parent, "User");
            assert_eq!(i.only, vec!["permissions".to_string(), "profile".to_string()]);
        }
        if let Statement::InheritStmt(ref i) = module.statements[1] {
            assert_eq!(i.target, "Guest");
            assert_eq!(i.except, vec!["delete".to_string(), "resetPassword()".to_string()]);
        }
        if let Statement::InheritStmt(ref i) = module.statements[2] {
            assert_eq!(i.alias, Some("BaseUser".to_string()));
        }
    }

    #[test]
    fn test_phase6_capability_permission_event_feature_inheritance() {
        let code = r#"
            inherit Admin capabilities User;
            inherit RestrictedAdmin permissions User without permissions User.admin;
            inherit EnterprisePayment feature Payment;
            inherit EnterprisePayment architecture Payment;
            inherit SecurePayment policy SecureBase;
            inherit AuditManager events User;
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 6);

        if let Statement::InheritStmt(ref i) = module.statements[1] {
            assert_eq!(i.permission_removals, vec!["User.admin".to_string()]);
        }
        if let Statement::InheritStmt(ref i) = module.statements[2] {
            assert_eq!(i.kind, InheritKind::Feature);
        }
        if let Statement::InheritStmt(ref i) = module.statements[3] {
            assert_eq!(i.kind, InheritKind::Architecture);
        }
        if let Statement::InheritStmt(ref i) = module.statements[4] {
            assert_eq!(i.kind, InheritKind::Policy);
        }
    }

    #[test]
    fn test_phase7_sealed_open_replaceable_locked() {
        let code = r#"
            sealed class SecurityToken {
                token: String;
            }

            open class Plugin {
                id: String;
            }

            inherit Service replaceable BaseService;

            class AdminUser extends User lock User.api {
                id: i64;
            }
        "#;
        let module = parse_code(code);
        let mut hierarchy = InheritanceHierarchy::new();
        for stmt in &module.statements {
            if let Statement::ClassDecl(c) = stmt {
                hierarchy.register_class(c);
                if c.name == "SecurityToken" {
                    assert!(c.is_sealed);
                }
                if c.name == "Plugin" {
                    assert!(c.is_open);
                }
                if c.name == "AdminUser" {
                    assert_eq!(c.locked_contracts, vec!["User.api".to_string()]);
                }
            }
            if let Statement::InheritStmt(i) = stmt {
                if i.target == "Service" {
                    assert!(i.is_replaceable);
                }
            }
        }
        assert!(hierarchy.sealed_classes.contains("SecurityToken"));
    }

    #[test]
    fn test_phase8_super_resolution_named_super_superchain() {
        let code = r#"
            super.save();
            super(User).save();
            super(Auditable).log();
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 3);

        if let Statement::SuperCallStmt(ref s) = module.statements[0] {
            assert_eq!(s.target_parent, None);
            assert_eq!(s.method, "save");
        }
        if let Statement::SuperCallStmt(ref s) = module.statements[1] {
            assert_eq!(s.target_parent, Some("User".to_string()));
            assert_eq!(s.method, "save");
        }
        if let Statement::SuperCallStmt(ref s) = module.statements[2] {
            assert_eq!(s.target_parent, Some("Auditable".to_string()));
            assert_eq!(s.method, "log");
        }
    }

    #[test]
    fn test_phase9_conflicts_diamonds_and_inspection() {
        let code = r#"
            conflict A.log B.log;
            resolve A.log over B.log;
            inspect inheritance Admin;
            impact inheritance User;

            class DiamondD extends DiamondB, DiamondC {
                share DiamondA;
                virtual DiamondRoot;
                fn run() -> i64 {
                    return 42;
                }
            }
        "#;
        let module = parse_code(code);
        let mut hierarchy = InheritanceHierarchy::new();
        for stmt in &module.statements {
            if let Statement::ClassDecl(c) = stmt {
                hierarchy.register_class(c);
                if c.name == "DiamondD" {
                    assert_eq!(c.shared_parents, vec!["DiamondA".to_string()]);
                    assert_eq!(c.virtual_parents, vec!["DiamondRoot".to_string()]);
                }
            }
        }

        let mro = hierarchy.compute_mro("DiamondD");
        assert_eq!(mro[0], "DiamondD");
        assert_eq!(mro[1], "DiamondB");
        assert_eq!(mro[2], "DiamondC");
    }

    #[test]
    fn test_phase10_transformative_delegates_and_contractual_proofs() {
        let code = r#"
            inherit Admin User transform { rename email -> adminEmail, hide password, expose permissions };
            inherit ApiUser User map { id -> user_id, name -> display_name };
            inherit PaymentFacade delegation PaymentService;
            inherit Admin User contractually {
                inherit capability Storage.read;
                deny capability Storage.delete;
            };

            class UserLogger inherits Logger {
                id: i64;
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 5);

        let mut hierarchy = InheritanceHierarchy::new();
        for stmt in &module.statements {
            if let Statement::ClassDecl(c) = stmt {
                hierarchy.register_class(c);
            }
        }

        // Anti-pattern warning check for Logger inheritance
        assert!(!hierarchy.anti_pattern_warnings.is_empty());
        assert!(hierarchy.anti_pattern_warnings[0].contains("UserLogger"));
        assert!(hierarchy.anti_pattern_warnings[0].contains("equip"));

        // Interpreter evaluation
        let mut interp = Interpreter::new();
        for stmt in &module.statements {
            let res = interp.eval_statement(stmt);
            assert!(res.is_ok());
        }
    }
}
