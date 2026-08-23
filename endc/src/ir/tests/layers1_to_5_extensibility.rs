use crate::ast::*;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ir::hir::*;
use crate::ir::lowering::*;
use crate::ir::mir_lowering::*;
use crate::semantic::analyzer::*;
use crate::codegen::interpreter::{Interpreter, Value};
use crate::agent_api::evolution_engine::*;

fn parse_str(code: &str) -> Result<Module, String> {
    let mut lexer = Lexer::new("test.end", code);
    let tokens = lexer.tokenize_all()?;
    let mut parser = Parser::new("test.end", tokens);
    parser.parse_module("test_mod")
}

    // =========================================================================
    // LAYER 1: DNA of Code Itself (Tests 1 - 3)
    // =========================================================================

    #[test]
    fn test_layer1_partial_augment_composition() {
        let code = r#"
        partial struct User {
            pub id: i64,
            pub name: str,
        }

        partial struct User {
            pub email: str,
            pub is_active: bool,
        }

        augment struct User {
            pub fn get_display_name() -> str {
                return "UserDisplay";
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert!(!module.statements.is_empty() || !module.structs.is_empty() || !module.extensions.is_empty());
        let has_partial = module.statements.iter().any(|s| matches!(s, Statement::PartialDecl { kind, name, .. } if kind == "struct" && name == "User"));
        let has_augment = module.statements.iter().any(|s| matches!(s, Statement::AugmentDecl(_))) || module.extensions.iter().any(|e| e.is_augment);
        assert!(has_partial || has_augment);
    }

    #[test]
    fn test_layer1_override_and_extension_points() {
        let code = r#"
        extension_point User { before_save, after_save, validation }

        extend User at validation {
            fn validate_payload() -> bool {
                return true;
            }
        }

        override User.save() {
            val status = 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_ext_point = module.statements.iter().any(|s| matches!(s, Statement::ExtensionPointDecl { target, points, .. } if target == "User" && points.contains(&"validation".to_string())));
        let has_at_hook = module.extensions.iter().any(|e| e.target == "User" && e.at_hook == Some("validation".to_string()));
        let has_override = module.statements.iter().any(|s| matches!(s, Statement::OverrideDecl { target, .. } if target == "User.save"));
        assert!(has_ext_point);
        assert!(has_at_hook);
        assert!(has_override);
    }

    #[test]
    fn test_layer1_sealed_and_friend_protection() {
        let code = r#"
        sealed struct SecurityToken;
        friend module Authentication;

        mod Security {
            sealed: true;
            owns: ["SecurityToken"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        analyzer.analyze_module(&module);
        assert!(analyzer.sealed_structs.contains("SecurityToken") || analyzer.module_sealed.contains("Security"));
        assert!(analyzer.module_friends.contains_key("global") || analyzer.module_friends.contains_key("Security"));
    }

    // =========================================================================
    // LAYER 2: Super Module System (Tests 4 - 6)
    // =========================================================================

    #[test]
    fn test_layer2_multidimensional_facets() {
        let code = r#"
        mod payments {
            api {
                pub fn pay(amount: i64) -> bool {
                    return true;
                }
            }
            implementation {
                fn process_gateway(amount: i64) -> bool {
                    return true;
                }
            }
            tests {
                fn test_pay_flow() -> bool {
                    return true;
                }
            }
            extension {
                fn refund(amount: i64) -> bool {
                    return true;
                }
            }
            architecture {
                "payments -> payment_gateway";
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.modules.len(), 1);
        let payments = &module.modules[0];
        assert!(payments.facets.is_some());
        let facets = payments.facets.as_ref().unwrap();
        assert_eq!(facets.api.len(), 1);
        assert_eq!(facets.implementation.len(), 1);
        assert_eq!(facets.tests.len(), 1);
        assert_eq!(facets.extension.len(), 1);
        assert_eq!(facets.architecture.len(), 1);
    }

    #[test]
    fn test_layer2_contracts_and_dependencies() {
        let code = r#"
        mod payments {
            requires: ["database", "auth"];
            provides: ["PaymentService", "RefundService"];
            guarantees: ["no_network_after_commit", "idempotent_charge"];
        }
        "#;
        let module = parse_str(code).unwrap();
        let payments = &module.modules[0];
        assert!(payments.contract.is_some());
        let contract = payments.contract.as_ref().unwrap();
        assert_eq!(contract.requires, vec!["database", "auth"]);
        assert_eq!(contract.provides, vec!["PaymentService", "RefundService"]);
        assert_eq!(contract.guarantees, vec!["no_network_after_commit", "idempotent_charge"]);
    }

    #[test]
    fn test_layer2_replace_overlay_and_compose() {
        let code = r#"
        replace payments.database with postgres_adapter;

        migration payments 3 -> 4 {
            rename "Payment.id" -> "Payment.payment_id";
        }

        overlay production {
            val endpoint = "https://prod.gateway.com";
        }

        compose { Auth, Database, Payments, Observability }
        "#;
        let module = parse_str(code).unwrap();
        let has_replace = module.statements.iter().any(|s| matches!(s, Statement::ReplaceModuleDecl { target, replacement, .. } if target == "payments.database" && replacement == "postgres_adapter"));
        let has_migration = module.statements.iter().any(|s| matches!(s, Statement::ModuleMigrationDecl { module_name, from_version, to_version, .. } if module_name == "payments" && *from_version == 3 && *to_version == 4));
        let has_overlay = module.statements.iter().any(|s| matches!(s, Statement::ModuleOverlayDecl { target_env, .. } if target_env == "production"));
        let has_compose = module.statements.iter().any(|s| matches!(s, Statement::ModuleComposeDecl { modules, .. } if modules.len() == 4));
        assert!(has_replace);
        assert!(has_migration);
        assert!(has_overlay);
        assert!(has_compose);
    }

    // =========================================================================
    // LAYER 3: Type System for Extensibility (Tests 7 - 9)
    // =========================================================================

    #[test]
    fn test_layer3_open_closed_types() {
        let code = r#"
        open type User;
        closed type SecureVault;
        "#;
        let module = parse_str(code).unwrap();
        let has_open = module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: true, name, .. } if name == "User"));
        let has_closed = module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: false, name, .. } if name == "SecureVault"));
        assert!(has_open);
        assert!(has_closed);
    }

    #[test]
    fn test_layer3_conditional_and_generic_extensions() {
        let code = r#"
        extend User when Feature("social") {
            fn get_followers() -> i64 {
                return 42;
            }
        }

        extend User requires capability("user.profile") {
            fn get_bio() -> str {
                return "developer";
            }
        }

        extend List<T> {
            fn is_empty_collection() -> bool {
                return true;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 3);
        assert_eq!(module.extensions[0].when_feature, Some("social".to_string()));
        assert_eq!(module.extensions[1].required_capability, Some("user.profile".to_string()));
        assert_eq!(module.extensions[2].generic_params, vec!["T".to_string()]);
    }

    #[test]
    fn test_layer3_extension_conflict_resolution() {
        let code = r#"
        extend User owned_by IdentityTeam {
            fn notify() -> bool {
                return true;
            }
        }

        extend User owned_by AnalyticsTeam {
            fn notify() -> bool {
                return false;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 2);
        assert_eq!(module.extensions[0].owned_by, Some("IdentityTeam".to_string()));
        assert_eq!(module.extensions[1].owned_by, Some("AnalyticsTeam".to_string()));
    }

    // =========================================================================
    // LAYER 4: Syntax Extensibility (Tests 10 - 12)
    // =========================================================================

    #[test]
    fn test_layer4_custom_syntax_definition() {
        let code = r#"
        syntax query {
            val q = 100;
        }

        syntax route(path: str) -> Route {
            val r = 200;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_query_syntax = module.statements.iter().any(|s| matches!(s, Statement::SyntaxDecl { name, .. } if name == "query"));
        let has_route_syntax = module.statements.iter().any(|s| matches!(s, Statement::SyntaxDecl { name, params, .. } if name == "route" && params.len() == 1));
        assert!(has_query_syntax);
        assert!(has_route_syntax);
    }

    #[test]
    fn test_layer4_namespaced_versioned_syntax() {
        let code = r#"
        use syntax web@3;
        use feature("authentication");
        "#;
        let module = parse_str(code).unwrap();
        let has_use_syntax = module.statements.iter().any(|s| matches!(s, Statement::UseSyntaxDecl { namespace, version, .. } if namespace == "web" && *version == Some(3)));
        let has_use_feature = module.statements.iter().any(|s| matches!(s, Statement::SemanticImportDecl { feature_intent, .. } if feature_intent == "authentication"));
        assert!(has_use_syntax);
        assert!(has_use_feature);
    }

    #[test]
    fn test_layer4_syntax_safety_and_composition() {
        let code = r#"
        syntax web::graphql(schema: str) -> Schema {
            val loaded = true;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_namespaced = module.statements.iter().any(|s| matches!(s, Statement::SyntaxDecl { namespace, name, .. } if namespace == &Some("web".to_string()) && name == "graphql"));
        assert!(has_namespaced);
    }

    // =========================================================================
    // LAYER 5: Compile-time Extensibility (Tests 13 - 15)
    // =========================================================================

    #[test]
    fn test_layer5_custom_linters_and_analyzers() {
        let code = r#"
        lint no_database_in_ui {
            "forbid UI -> Database";
            "require View -> ViewModel";
        }

        analyzer detect_blocking_io {
            "forbid sync_read in async_fn";
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_lint = module.statements.iter().any(|s| matches!(s, Statement::CustomLinterDecl { name, rules, .. } if name == "no_database_in_ui" && rules.len() == 2));
        let has_analyzer = module.statements.iter().any(|s| matches!(s, Statement::CustomAnalyzerDecl { name, checks, .. } if name == "detect_blocking_io" && checks.len() == 1));
        assert!(has_lint);
        assert!(has_analyzer);
    }

    #[test]
    fn test_layer5_type_rules_and_optimizers() {
        let code = r#"
        type_rule Money {
            "forbid_float_conversion";
            "require_explicit_currency";
        }

        optimizer vectorize_math {
            val pass_id = 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let has_type_rule = module.statements.iter().any(|s| matches!(s, Statement::CustomTypeRuleDecl { target_type, rules, .. } if target_type == "Money" && rules.len() == 2));
        let has_opt = module.statements.iter().any(|s| matches!(s, Statement::CustomOptimizerDecl { name, .. } if name == "vectorize_math"));
        assert!(has_type_rule);
        assert!(has_opt);
    }

    #[test]
    fn test_layer5_reflection_and_generators() {
        let code = r#"
        generator grpc {
            "proto3";
        }

        reflect User { fields, methods, attributes }
        "#;
        let module = parse_str(code).unwrap();
        let has_gen = module.statements.iter().any(|s| matches!(s, Statement::GeneratorDecl { name, .. } if name == "grpc"));
        let has_reflect = module.statements.iter().any(|s| matches!(s, Statement::ReflectDecl { target_type, queries, .. } if target_type == "User" && queries.len() == 3));
        assert!(has_gen);
        assert!(has_reflect);
    }

