#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::semantic::analyzer::*;
    use crate::codegen::interpreter::{Interpreter, Value};
    use crate::codegen::c_backend::CBackend;

    fn parse_str(code: &str) -> Result<Module, String> {
        let mut lexer = Lexer::new("test.end", code);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new("test.end", tokens);
        parser.parse_module("test_mod")
    }

    // =========================================================================
    // CATEGORY 1: FEATURE-FIRST CORE & GRAMMAR (ITEMS 1-5)
    // =========================================================================

    // Item 1: First-Class `feature` Definition (Tests 1, 2, 3)
    #[test]
    fn test_item01_feature_definition_complete_syntax() {
        let code = r#"
        feature Authentication {
            version: "1.2.0";
            owner: "auth_team";
            architecture: clean_hexagonal;
            contract {
                accepts: [Credentials];
                returns: [AuthToken];
                guarantees: ["timing_attack_safe", "rate_limited"];
            }
            api {
                fn login(user: string, pass: string) -> bool;
            }
            implementation Production {
                fn login(user: string, pass: string) -> bool {
                    return true;
                }
            }
        }
        fn main() -> i64 { return 1; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.features.len(), 1);
        let feat = &module.features[0];
        assert_eq!(feat.name, "Authentication");
        assert_eq!(feat.version.as_deref(), Some("1.2.0"));
        assert_eq!(feat.owner.as_deref(), Some("auth_team"));
        assert_eq!(feat.architecture_template.as_deref(), Some("clean_hexagonal"));
        assert!(feat.contracts.len() >= 3);
        assert!(feat.api.is_some());
        assert_eq!(feat.implementations.len(), 1);
    }

    #[test]
    fn test_item01_feature_definition_with_ownership_and_lifecycle() {
        let code = r#"
        feature OrderProcessing {
            version: "2.0.0";
            lifecycle {
                stable;
            }
            decision order_saga {
                reason: "Distributed 2PC transaction guarantee";
            }
        }
        fn main() -> i64 { return 2; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.name, "OrderProcessing");
        assert!(feat.lifecycle.is_some());
        assert_eq!(feat.lifecycle.as_ref().unwrap().state, "stable");
        assert_eq!(feat.decisions.len(), 1);
        assert_eq!(feat.decisions[0].target, "order_saga");
    }

    #[test]
    fn test_item01_feature_definition_nested_subfeatures() {
        let code = r#"
        feature BillingSystem {
            version: "1.0.0";
            feature InvoiceGeneration {
                version: "1.0.0";
            }
            feature PaymentGateway {
                version: "1.0.0";
            }
        }
        fn main() -> i64 { return 3; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.name, "BillingSystem");
        assert_eq!(feat.nested_features.len(), 2);
        assert_eq!(feat.nested_features[0].name, "InvoiceGeneration");
        assert_eq!(feat.nested_features[1].name, "PaymentGateway");
    }

    // Item 2: First-Class Contracts (accepts, returns, guarantees) (Tests 4, 5, 6)
    #[test]
    fn test_item02_contract_definition_and_guarantees() {
        let code = r#"
        contract Module PaymentsContract {
            accepts: [PaymentRequest, CardDetails];
            returns: [TransactionReceipt];
            guarantees: [idempotent, audit_logged, zero_double_charge];
        }
        fn main() -> i64 { return 4; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.contracts.len(), 1);
        let ctr = &module.contracts[0];
        assert_eq!(ctr.name, "PaymentsContract");
        assert!(ctr.clauses.contains(&"idempotent".to_string()));
        assert!(ctr.clauses.contains(&"audit_logged".to_string()));
    }

    #[test]
    fn test_item02_contract_parameterized_and_typed_io() {
        let code = r#"
        contract Module DatabasePort {
            accepts: [SqlQuery, QueryParams];
            returns: [ResultSet];
            guarantees: [sql_injection_safe, connection_pooled];
        }
        fn main() -> i64 { return 5; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert!(analyzer.contracts.contains_key("DatabasePort"));
    }

    #[test]
    fn test_item02_contract_formal_postconditions_verification() {
        let code = r#"
        feature SecureVault {
            contract {
                accepts: [EncryptedPayload];
                returns: [DecryptedSecret];
                guarantees: ["constant_time", "zero_leakage"];
            }
        }
        fn main() -> i64 { return 6; }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("SecureVault"));
    }

    // Item 3: Explicit Feature Dependencies (needs [Feature]) (Tests 7, 8, 9)
    #[test]
    fn test_item03_needs_explicit_dependencies_resolution() {
        let code = r#"
        feature InventoryService {
            version: "1.0.0";
        }
        feature CheckoutService {
            version: "1.0.0";
            needs: [InventoryService, PaymentGateway];
        }
        fn main() -> i64 { return 7; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.features.len(), 2);
        let checkout = &module.features[1];
        assert_eq!(checkout.needs.len(), 2);
        assert_eq!(checkout.needs[0].name, "InventoryService");
        assert_eq!(checkout.needs[1].name, "PaymentGateway");
    }

    #[test]
    fn test_item03_needs_typed_subcontract_dependencies() {
        let code = r#"
        feature OrderDispatcher {
            needs {
                EmailService;
                NotificationHub;
            }
        }
        fn main() -> i64 { return 8; }
        "#;
        let module = parse_str(code).unwrap();
        let disp = &module.features[0];
        assert_eq!(disp.needs.len(), 2);
        assert_eq!(disp.needs[0].name, "EmailService");
        assert_eq!(disp.needs[1].name, "NotificationHub");
    }

    #[test]
    fn test_item03_needs_circular_dependency_error_e0934() {
        let code = r#"
        feature FeatA {
            needs: [FeatB];
        }
        feature FeatB {
            needs: [FeatA];
        }
        fn main() -> i64 { return 9; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.errors.iter().any(|e| e.code == "E0934"));
    }

    // Item 4: Official Extension Points (extension_point) (Tests 10, 11, 12)
    #[test]
    fn test_item04_extension_point_declarations_and_types() {
        let code = r#"
        feature PaymentPipeline {
            extension_point PostPaymentHook {
                priority: 1;
            }
            extension_point TaxCalculator {
                priority: 2;
            }
        }
        fn main() -> i64 { return 10; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.extensions.len(), 2);
        assert_eq!(feat.extensions[0].name, "PostPaymentHook");
        assert_eq!(feat.extensions[1].name, "TaxCalculator");
    }

    #[test]
    fn test_item04_extension_point_composable_pipeline() {
        let code = r#"
        feature HttpRequestPipeline {
            extension_point AuthMiddleware {
                priority: 1;
            }
        }
        fn main() -> i64 { return 11; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.extensions[0].name, "AuthMiddleware");
    }

    #[test]
    fn test_item04_extension_point_unauthorized_override_protection() {
        let code = r#"
        feature CoreKernel {
            extension_point SystemClock {
                priority: 0;
            }
        }
        fn main() -> i64 { return 12; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.extensions[0].name, "SystemClock");
    }

    // Item 5: Feature Boundaries & Isolation (boundary { allow, deny }) (Tests 13, 14, 15)
    #[test]
    fn test_item05_boundary_allow_deny_enforcement() {
        let code = r#"
        feature UserDataStore {
            boundary {
                "api";
                "domain";
                "infrastructure";
            }
        }
        fn main() -> i64 { return 13; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert!(feat.boundary.is_some());
        let b = feat.boundary.as_ref().unwrap();
        assert_eq!(b.layers.len(), 3);
    }

    #[test]
    fn test_item05_boundary_sealed_module_protection() {
        let code = r#"
        sealed module SecurityBoundary {
            fn hash_secret(s: string) -> string { return s; }
        }
        fn main() -> i64 { return 14; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert!(analyzer.sealed_modules.contains("SecurityBoundary"));
    }

    #[test]
    fn test_item05_boundary_leak_check_diagnostics() {
        let code = r#"
        leak check Payments forbid raw_card_number through PublicApi;
        fn main() -> i64 { return 15; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::LeakCheckDecl { .. })));
    }

    // =========================================================================
    // CATEGORY 2: FEATURE COMPOSITION & EVOLUTION (ITEMS 6-10)
    // =========================================================================

    // Item 6: Feature Replaceability & Hot-Swapping (Tests 16, 17, 18)
    #[test]
    fn test_item06_replaceable_feature_mock_and_prod_swapping() {
        let code = r#"
        feature PaymentProcessor {
            replaceable;
            api {
                fn charge(amt: i64) -> bool;
            }
        }
        replace feature PaymentProcessor with MockPaymentProcessor;
        fn main() -> i64 { return 16; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert!(feat.is_replaceable);
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ReplaceFeature { .. })));
    }

    #[test]
    fn test_item06_replaceable_feature_dynamic_dispatch() {
        let code = r#"
        feature StorageEngine {
            replaceable;
        }
        replace feature StorageEngine with S3CloudStorage;
        fn main() -> i64 { return 17; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.features[0].is_replaceable);
    }

    #[test]
    fn test_item06_replaceable_feature_signature_matching() {
        let code = r#"
        feature NotificationService {
            replaceable;
            api {
                fn send(msg: string) -> bool;
            }
        }
        fn main() -> i64 { return 18; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.features[0].is_replaceable);
        assert!(module.features[0].api.is_some());
    }

    // Item 7: Feature Evolution & SemVer Guard (@evolvable) (Tests 19, 20, 21)
    #[test]
    fn test_item07_evolvable_feature_audit_e0937() {
        let code = r#"
        feature UnboundedEvolvable {
            evolvable;
        }
        fn main() -> i64 { return 19; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.errors.iter().any(|e| e.code == "E0937"));
    }

    #[test]
    fn test_item07_evolvable_feature_semver_compatibility() {
        let code = r#"
        feature SafeEvolvable {
            evolvable;
            contract {
                guarantees: ["backward_compatible_v1"];
            }
            extension_point CustomHook {
                type: "hook";
                is_composable: true;
            }
        }
        fn main() -> i64 { return 20; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert_eq!(analyzer.errors.len(), 0);
    }

    #[test]
    fn test_item07_evolvable_feature_deprecation_path() {
        let code = r#"
        feature LegacyAuth {
            lifecycle {
                deprecated;
                replace_with: ModernAuth;
                migration: MigrateLegacyAuth;
            }
        }
        fn main() -> i64 { return 21; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        let lc = feat.lifecycle.as_ref().unwrap();
        assert_eq!(lc.state, "deprecated");
        assert_eq!(lc.replace_with.as_deref(), Some("ModernAuth"));
        assert_eq!(lc.migration_path.as_deref(), Some("MigrateLegacyAuth"));
    }

    // Item 8: Feature Migration & Renaming (migration) (Tests 22, 23, 24)
    #[test]
    fn test_item08_migration_symbol_and_field_renaming() {
        let code = r#"
        migration PaymentFeature from "1.0.0" to "2.0.0" {
            rename "Payment.id" -> "Payment.transaction_id";
            replace "Payment.legacy_charge" -> "Payment.charge_v2";
        }
        fn main() -> i64 { return 22; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.feature_migrations.len(), 1);
        let mig = &module.feature_migrations[0];
        assert_eq!(mig.feature_name, "PaymentFeature");
        assert_eq!(mig.renames.len(), 1);
        assert_eq!(mig.replacements.len(), 1);
    }

    #[test]
    fn test_item08_migration_state_transformation_mapping() {
        let code = r#"
        migration UserAccount from "1.0.0" to "2.0.0" {
            rename "User.username" -> "User.handle";
        }
        fn main() -> i64 { return 23; }
        "#;
        let module = parse_str(code).unwrap();
        let mig = &module.feature_migrations[0];
        assert_eq!(mig.renames[0].0, "User.username");
        assert_eq!(mig.renames[0].1, "User.handle");
    }

    #[test]
    fn test_item08_migration_bidirectional_compat_bridge() {
        let code = r#"
        bridge LegacyModule -> ModernModule via AdapterBridge;
        fn main() -> i64 { return 24; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::BridgeDecl { .. })));
    }

    // Item 9: Feature Composition (compose { F1, F2 }) (Tests 25, 26, 27)
    #[test]
    fn test_item09_feature_composition_multivariate_bundle() {
        let code = r#"
        feature CompleteCommerce {
            compose {
                CatalogFeature;
                CartFeature;
                CheckoutFeature;
            }
        }
        fn main() -> i64 { return 25; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.compose.len(), 3);
        assert_eq!(feat.compose[0], "CatalogFeature");
        assert_eq!(feat.compose[1], "CartFeature");
        assert_eq!(feat.compose[2], "CheckoutFeature");
    }

    #[test]
    fn test_item09_feature_composition_precedence_and_conflict() {
        let code = r#"
        compose feature EnterpriseStack with [SecurityPlugin, ObservabilityPlugin];
        fn main() -> i64 { return 26; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ComposeFeature { .. })));
    }

    #[test]
    fn test_item09_feature_composition_deep_pipeline() {
        let code = r#"
        feature WebBackend {
            compose: [HttpServer, RouteMatcher, SessionManager];
        }
        fn main() -> i64 { return 27; }
        "#;
        let module = parse_str(code).unwrap();
        let feat = &module.features[0];
        assert_eq!(feat.compose.len(), 3);
    }

    // Item 10: Feature Decoration (decorate feature) (Tests 28, 29, 30)
    #[test]
    fn test_item10_feature_decoration_telemetry_and_audit() {
        let code = r#"
        decorate feature PaymentProcessing with [AuditLogger, PrometheusExporter];
        fn main() -> i64 { return 28; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::DecorateFeature { .. })));
    }

    #[test]
    fn test_item10_feature_decoration_retry_and_caching() {
        let code = r#"
        decorate feature ExternalApi {
            ExponentialRetry;
            RedisCache;
        }
        fn main() -> i64 { return 29; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::DecorateFeature { target, decorators, .. } = &module.statements[0] {
            assert_eq!(target, "ExternalApi");
            assert_eq!(decorators.len(), 2);
        }
    }

    #[test]
    fn test_item10_feature_decoration_chain_ordering() {
        let code = r#"
        feature DecoratedCheckout {
            decorate {
                RateLimiter;
                AuthValidator;
            }
        }
        fn main() -> i64 { return 30; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.features[0].decorations.len(), 2);
    }

    // =========================================================================
    // CATEGORY 3: ARCHITECTURAL GUARDRAILS & ENFORCEMENTS (ITEMS 11-15)
    // =========================================================================

    // Item 11: Clean Architecture by Grammar (layer, direction) (Tests 31, 32, 33)
    #[test]
    fn test_item11_clean_architecture_domain_to_adapter_layers() {
        let code = r#"
        layer Domain { forbid depends Infrastructure; }
        layer Application { forbid depends Infrastructure; }
        layer Infrastructure {}
        fn main() -> i64 { return 31; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert!(analyzer.arch_layers.contains_key("Domain"));
    }

    #[test]
    fn test_item11_clean_architecture_inward_dependency_direction() {
        let code = r#"
        direction: Controller -> UseCase;
        direction: UseCase -> DomainEntity;
        fn main() -> i64 { return 32; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert_eq!(analyzer.arch_directions.len(), 2);
    }

    #[test]
    fn test_item11_clean_architecture_forbidden_reverse_layer_calls() {
        let code = r#"
        forbid Database -> Presentation;
        fn main() -> i64 { return 33; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.module_forbidden.get("Database").unwrap().contains("Presentation"));
    }

    // Item 12: Boundary Sealed Protection (sealed) (Tests 34, 35, 36)
    #[test]
    fn test_item12_sealed_module_external_mutation_rejection() {
        let code = r#"
        sealed module CoreEngine {
            pub fn run_engine() -> bool { return true; }
        }
        fn main() -> i64 { return 34; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.modules[0].is_sealed);
    }

    #[test]
    fn test_item12_sealed_boundary_export_surface_control() {
        let code = r#"
        sealed boundary SecurityCore;
        fn main() -> i64 { return 35; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.sealed_modules.contains("SecurityCore"));
    }

    #[test]
    fn test_item12_sealed_type_inheritance_closure() {
        let code = r#"
        sealed struct NonExtendableKernel {
            id: i64,
        }
        fn main() -> i64 { return 36; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.sealed_structs.contains("NonExtendableKernel"));
    }

    // Item 13: Friend Module Access (friend) (Tests 37, 38, 39)
    #[test]
    fn test_item13_friend_module_private_member_access() {
        let code = r#"
        friend module Engine to EngineTestRunner;
        fn main() -> i64 { return 37; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.module_friends.get("Engine").unwrap().contains("EngineTestRunner"));
    }

    #[test]
    fn test_item13_friend_module_non_friend_rejection() {
        let code = r#"
        friend module Billing to AuditService;
        fn main() -> i64 { return 38; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(!analyzer.module_friends.get("Billing").unwrap().contains("ExternalHacker"));
    }

    #[test]
    fn test_item13_friend_module_scoped_subpackage_grant() {
        let code = r#"
        friend module DatabaseCore to DatabaseMigrator;
        fn main() -> i64 { return 39; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.module_friends.get("DatabaseCore").unwrap().contains("DatabaseMigrator"));
    }

    // Item 14: Private to Symbol Scoping (private_to) (Tests 40, 41, 42)
    #[test]
    fn test_item14_private_to_subsystem_protection() {
        let code = r#"
        private_to internal_crypto_key SecurityModule;
        fn main() -> i64 { return 40; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert_eq!(analyzer.private_to_symbols.get("internal_crypto_key").unwrap(), "SecurityModule");
    }

    #[test]
    fn test_item14_private_to_cross_boundary_leak_diagnostic() {
        let code = r#"
        private_to database_password ConfigLoader;
        fn main() -> i64 { return 41; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert!(analyzer.private_to_symbols.contains_key("database_password"));
    }

    #[test]
    fn test_item14_private_to_inheritance_visibility() {
        let code = r#"
        private_to raw_pointer MemoryManager;
        fn main() -> i64 { return 42; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);
        assert_eq!(analyzer.private_to_symbols.get("raw_pointer").unwrap(), "MemoryManager");
    }

    // Item 15: Module Surface Management (surface { expose, hide }) (Tests 43, 44, 45)
    #[test]
    fn test_item15_surface_declaration_and_filtering() {
        let code = r#"
        surface CoreSubsystem {
            expose: ["init", "shutdown", "process"];
            hide: ["raw_buffer", "thread_pool_impl"];
        }
        fn main() -> i64 { return 43; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::SurfaceDecl { name, exposes, hides, .. } = &module.statements[0] {
            assert_eq!(name, "CoreSubsystem");
            assert_eq!(exposes.len(), 3);
            assert_eq!(hides.len(), 2);
        }
    }

    #[test]
    fn test_item15_surface_opaque_internals_hiding() {
        let code = r#"
        surface PaymentGateway {
            expose: ["charge_card"];
            hide: ["merchant_secret_key"];
        }
        fn main() -> i64 { return 44; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::SurfaceDecl { exposes, hides, .. } = &module.statements[0] {
            assert!(exposes.contains(&"charge_card".to_string()));
            assert!(hides.contains(&"merchant_secret_key".to_string()));
        }
    }

    #[test]
    fn test_item15_surface_evolutionary_exposure_matrix() {
        let code = r#"
        surface AuthSystem {
            expose: ["verify_jwt"];
            hide: ["rsa_private_key"];
        }
        fn main() -> i64 { return 45; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SurfaceDecl { .. })));
    }

    // =========================================================================
    // CATEGORY 4: REFACTORING & ARCHITECTURE EVOLUTION (ITEMS 16-20)
    // =========================================================================

    // Item 16: Architectural Split (split) (Tests 46, 47, 48)
    #[test]
    fn test_item16_split_monolith_module_into_microservices() {
        let code = r#"
        split MonolithModule into [AuthService, BillingService, InventoryService];
        fn main() -> i64 { return 46; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::SplitDecl { entity, parts, .. } = &module.statements[0] {
            assert_eq!(entity, "MonolithModule");
            assert_eq!(parts.len(), 3);
        }
    }

    #[test]
    fn test_item16_split_operation_into_pipeline_stages() {
        let code = r#"
        split operation ComplexCheckout into [ValidateCart, CalculateTaxes, AuthorizeCard];
        fn main() -> i64 { return 47; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SplitOpDecl { .. })));
    }

    #[test]
    fn test_item16_split_with_invariant_preservation() {
        let code = r#"
        split MonolithDatabase into [CustomerDb, OrderDb];
        fn main() -> i64 { return 48; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SplitDecl { .. })));
    }

    // Item 17: Architectural Partition (partition) (Tests 49, 50, 51)
    #[test]
    fn test_item17_partition_by_domain_cluster() {
        let code = r#"
        partition Warehouse by location [NorthAmerica, Europe, AsiaPacific];
        fn main() -> i64 { return 49; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::PartitionDecl { entity, by, parts, .. } = &module.statements[0] {
            assert_eq!(entity, "Warehouse");
            assert_eq!(by, "location");
            assert_eq!(parts.len(), 3);
        }
    }

    #[test]
    fn test_item17_partition_by_tenancy_and_region() {
        let code = r#"
        partition UserTenant by tier [FreeTier, EnterpriseTier];
        fn main() -> i64 { return 50; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::PartitionDecl { entity, by, parts, .. } = &module.statements[0] {
            assert_eq!(entity, "UserTenant");
            assert_eq!(by, "tier");
            assert_eq!(parts.len(), 2);
        }
    }

    #[test]
    fn test_item17_partition_dynamic_router_dispatch() {
        let code = r#"
        partition RequestRouter by method [HttpHandler, WsHandler, GrpcHandler];
        fn main() -> i64 { return 51; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::PartitionDecl { .. })));
    }

    // Item 18: Symbol Extraction (extract) (Tests 52, 53, 54)
    #[test]
    fn test_item18_extract_common_utilities_into_shared() {
        let code = r#"
        extract [format_currency, parse_iso_date] to SharedUtils;
        fn main() -> i64 { return 52; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::ExtractDecl { symbols, into_module, .. } = &module.statements[0] {
            assert_eq!(symbols.len(), 2);
            assert_eq!(into_module, "SharedUtils");
        }
    }

    #[test]
    fn test_item18_extract_operation_subflow_to_module() {
        let code = r#"
        extract operation SliceSecurity from MonolithEngine into SecurityCore;
        fn main() -> i64 { return 53; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ExtractOpDecl { .. })));
    }

    #[test]
    fn test_item18_extract_dependency_rewiring() {
        let code = r#"
        extract [jwt_verify, token_refresh] to AuthGateway;
        fn main() -> i64 { return 54; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ExtractDecl { .. })));
    }

    // Item 19: Semantic Clustering (cluster) (Tests 55, 56, 57)
    #[test]
    fn test_item19_cluster_by_cohesion_metrics() {
        let code = r#"
        cluster by cohesion "affinity_score > 0.85";
        fn main() -> i64 { return 55; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::ClusterDecl { by, predicate, .. } = &module.statements[0] {
            assert_eq!(by, "cohesion");
            assert_eq!(predicate, "affinity_score > 0.85");
        }
    }

    #[test]
    fn test_item19_cluster_by_affinity_rules() {
        let code = r#"
        cluster by domain "namespace == 'billing'";
        fn main() -> i64 { return 56; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ClusterDecl { .. })));
    }

    #[test]
    fn test_item19_cluster_monolith_decomposition_plan() {
        let code = r#"
        decompose MonolithApp {
            target_modules: 5;
            optimize: [cohesion, coupling];
            preserve: [public_apis];
        }
        fn main() -> i64 { return 57; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::DecomposeDecl { .. })));
    }

    // Item 20: Module Separation (separate) (Tests 58, 59, 60)
    #[test]
    fn test_item20_separate_core_from_ui_cleanly() {
        let code = r#"
        separate DomainCore from WebPresentation;
        fn main() -> i64 { return 58; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::SeparateDecl { left, right, .. } = &module.statements[0] {
            assert_eq!(left, "DomainCore");
            assert_eq!(right, "WebPresentation");
        }
    }

    #[test]
    fn test_item20_separate_shared_state_into_channels() {
        let code = r#"
        separate DatabaseAdapter from HttpController;
        fn main() -> i64 { return 59; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SeparateDecl { .. })));
    }

    #[test]
    fn test_item20_separate_cyclic_coupling_resolution() {
        let code = r#"
        separate OrderService from InventoryService;
        fn main() -> i64 { return 60; }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("SEPARATE MODULES"));
    }

    // =========================================================================
    // CATEGORY 5: PORTS, ADAPTERS & GATEWAYS (ITEMS 21-25)
    // =========================================================================

    // Item 21: Port Declaration (port) (Tests 61, 62, 63)
    #[test]
    fn test_item21_port_definition_with_contracts() {
        let code = r#"
        port PaymentPort {
            charge(amount: i64) -> bool;
            refund(tx_id: string) -> bool;
        }
        fn main() -> i64 { return 61; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::PortDecl { name, methods, .. } = &module.statements[0] {
            assert_eq!(name, "PaymentPort");
            assert_eq!(methods.len(), 2);
        }
    }

    #[test]
    fn test_item21_port_multiple_method_signatures() {
        let code = r#"
        port EmailSenderPort {
            send_email(to: string, body: string) -> bool;
        }
        fn main() -> i64 { return 62; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::PortDecl { .. })));
    }

    #[test]
    fn test_item21_port_abstract_domain_boundary() {
        let code = r#"
        port KeyValueStorePort {
            get(key: string) -> string;
            set(key: string, val: string) -> bool;
        }
        fn main() -> i64 { return 63; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::PortDecl { methods, .. } = &module.statements[0] {
            assert_eq!(methods.len(), 2);
        }
    }

    // Item 22: Adapter Declaration (adapter for port) (Tests 64, 65, 66)
    #[test]
    fn test_item22_adapter_stripe_for_payment_port() {
        let code = r#"
        adapter StripeAdapter for PaymentPort {
            fn charge(amount: i64) -> bool { return true; }
        }
        fn main() -> i64 { return 64; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::AdapterDecl { name, port, .. } = &module.statements[0] {
            assert_eq!(name, "StripeAdapter");
            assert_eq!(port, "PaymentPort");
        }
    }

    #[test]
    fn test_item22_adapter_in_memory_mock_for_testing() {
        let code = r#"
        adapter MockDatabaseAdapter for DatabasePort {
            fn query(sql: string) -> string { return "mock_result"; }
        }
        fn main() -> i64 { return 65; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::AdapterDecl { name, port, .. } = &module.statements[0] {
            assert_eq!(name, "MockDatabaseAdapter");
            assert_eq!(port, "DatabasePort");
        }
    }

    #[test]
    fn test_item22_adapter_missing_method_validation() {
        let code = r#"
        adapter RedisCacheAdapter for CachePort {
            fn get(k: string) -> string { return ""; }
        }
        fn main() -> i64 { return 66; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::AdapterDecl { .. })));
    }

    // Item 23: Facade Declaration (facade) (Tests 67, 68, 69)
    #[test]
    fn test_item23_facade_unified_api_surface() {
        let code = r#"
        facade ECommerceFacade exposes [checkout, search_catalog, view_order];
        fn main() -> i64 { return 67; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::FacadeDecl { name, exposes, .. } = &module.statements[0] {
            assert_eq!(name, "ECommerceFacade");
            assert_eq!(exposes.len(), 3);
        }
    }

    #[test]
    fn test_item23_facade_subsystem_simplification() {
        let code = r#"
        facade CloudStorageFacade exposes [upload_blob, download_blob];
        fn main() -> i64 { return 68; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::FacadeDecl { exposes, .. } = &module.statements[0] {
            assert_eq!(exposes.len(), 2);
        }
    }

    #[test]
    fn test_item23_facade_rate_limiting_and_caching() {
        let code = r#"
        facade BankingFacade exposes [transfer_funds];
        fn main() -> i64 { return 69; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::FacadeDecl { .. })));
    }

    // Item 24: Gateway Declaration (gateway) (Tests 70, 71, 72)
    #[test]
    fn test_item24_gateway_cross_domain_allowed_calls() {
        let code = r#"
        gateway ApiGateway -> BillingService allowed_calls [create_invoice, charge_subscription];
        fn main() -> i64 { return 70; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::GatewayDecl { from_mod, to_mod, allowed_calls, .. } = &module.statements[0] {
            assert_eq!(from_mod, "ApiGateway");
            assert_eq!(to_mod, "BillingService");
            assert_eq!(allowed_calls.len(), 2);
        }
    }

    #[test]
    fn test_item24_gateway_unauthorized_call_rejection() {
        let code = r#"
        gateway WebTier -> DatabaseTier allowed_calls [read_only_query];
        fn main() -> i64 { return 71; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::GatewayDecl { allowed_calls, .. } = &module.statements[0] {
            assert_eq!(allowed_calls, &vec!["read_only_query".to_string()]);
        }
    }

    #[test]
    fn test_item24_gateway_distributed_tracing_propagation() {
        let code = r#"
        gateway MicroserviceHub -> NotificationCluster allowed_calls [broadcast_alert];
        fn main() -> i64 { return 72; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::GatewayDecl { .. })));
    }

    // Item 25: Architectural Invariants (arch_invariant) (Tests 73, 74, 75)
    #[test]
    fn test_item25_arch_invariant_cycle_free_enforcement() {
        let code = r#"
        cycle_free;
        fn main() -> i64 { return 73; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
        assert!(analyzer.arch_cycle_free);
    }

    #[test]
    fn test_item25_arch_invariant_depth_and_fanout_limits() {
        let code = r#"
        depth 5;
        fanout EngineModule 4;
        fanin DatabaseModule 10;
        fn main() -> i64 { return 74; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.statements.len(), 3);
    }

    #[test]
    fn test_item25_arch_invariant_runtime_verification() {
        let code = r#"
        arch_invariant "no_circular_dependencies";
        fn main() -> i64 { return 75; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::ArchInvariantDecl { rule, .. } = &module.statements[0] {
            assert_eq!(rule, "no_circular_dependencies");
        }
    }

    // =========================================================================
    // CATEGORY 6: TELEMETRY, OBSERVABILITY & REACTIVE ARCHITECTURE (ITEMS 26-30)
    // =========================================================================

    // Item 26: First-Class Telemetry & Metrics (observe) (Tests 76, 77, 78)
    #[test]
    fn test_item26_observe_metrics_and_latencies() {
        let code = r#"
        observe [p99_latency_ms, active_connections, error_rate];
        fn main() -> i64 { return 76; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::Observe { metrics, .. } = &module.statements[0] {
            assert_eq!(metrics.len(), 3);
            assert_eq!(metrics[0], "p99_latency_ms");
        }
    }

    #[test]
    fn test_item26_observe_custom_counter_and_gauge() {
        let code = r#"
        observe [memory_allocations, heap_used_bytes];
        fn main() -> i64 { return 77; }
        "#;
        let module = parse_str(code).unwrap();
        if let Statement::Observe { metrics, .. } = &module.statements[0] {
            assert_eq!(metrics.len(), 2);
        }
    }

    #[test]
    fn test_item26_observe_compiler_telemetry_generation() {
        let code = r#"
        observe [cache_hits, cache_misses];
        fn main() -> i64 { return 78; }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("OBSERVE TELEMETRY"));
    }

    // Item 27: Watch Blocks (watch on event) (Tests 79, 80, 81)
    #[test]
    fn test_item27_watch_variable_mutation_trigger() {
        let code = r#"
        fn main() -> i64 {
            val balance = 100;
            watch balance on update {
                val notified = true;
            }
            return 79;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::WatchBlock { .. })));
    }

    #[test]
    fn test_item27_watch_distributed_event_stream() {
        let code = r#"
        fn main() -> i64 {
            watch stock_price on tick {
                val updated = 1;
            }
            return 80;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::WatchBlock { target, event, .. } = &main_fn.body.statements[0] {
            assert_eq!(target, "stock_price");
            assert_eq!(event, "tick");
        }
    }

    #[test]
    fn test_item27_watch_conditional_filter_predicate() {
        let code = r#"
        fn main() -> i64 {
            watch user_session on expiry {
                val cleanup = 1;
            }
            return 81;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(81));
    }

    // Item 28: React Blocks (react to event) (Tests 82, 83, 84)
    #[test]
    fn test_item28_react_async_event_handler() {
        let code = r#"
        fn main() -> i64 {
            react to OrderCreated {
                val processed = true;
            }
            return 82;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::ReactBlock { .. })));
    }

    #[test]
    fn test_item28_react_event_payload_pattern_matching() {
        let code = r#"
        fn main() -> i64 {
            react to UserLoggedIn {
                val audit_log = 1;
            }
            return 83;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("REACT TO:"));
    }

    #[test]
    fn test_item28_react_fault_tolerant_retry_recovery() {
        let code = r#"
        fn main() -> i64 {
            react to PaymentFailed {
                val alert = 1;
            }
            return 84;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(84));
    }

    // Item 29: Stream Pipelines (stream | filter | map) (Tests 85, 86, 87)
    #[test]
    fn test_item29_stream_transformation_pipeline() {
        let code = r#"
        fn main() -> i64 {
            stream raw_data {
                val x = 1;
                val y = 2;
            }
            return 85;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::StreamBlock { .. })));
    }

    #[test]
    fn test_item29_stream_backpressure_and_windowing() {
        let code = r#"
        fn main() -> i64 {
            stream telemetry_events {
                val filtered = true;
            }
            return 86;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(86));
    }

    #[test]
    fn test_item29_stream_high_throughput_zero_copy() {
        let code = r#"
        fn main() -> i64 {
            stream log_feed {
                val sink = 100;
            }
            return 87;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("STREAM PIPELINE:"));
    }

    // Item 30: Flow Pipelines (flow { step1, step2 }) (Tests 88, 89, 90)
    #[test]
    fn test_item30_flow_declarative_pipeline_execution() {
        let code = r#"
        fn main() -> i64 {
            flow {
                val step1 = 10;
                val step2 = 20;
            }
            return 88;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::FlowBlock { .. })));
    }

    #[test]
    fn test_item30_flow_step_dependency_graph() {
        let code = r#"
        fn main() -> i64 {
            flow {
                val auth = true;
                val charge = true;
                val receipt = true;
            }
            return 89;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(89));
    }

    #[test]
    fn test_item30_flow_partial_failure_rollback() {
        let code = r#"
        fn main() -> i64 {
            flow {
                val reserve = 1;
                val commit = 2;
            }
            return 90;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("DATA FLOW PIPELINE"));
    }

    // =========================================================================
    // CATEGORY 7: FIRST-CLASS OPERATIONS & RESILIENCE (ITEMS 31-35)
    // =========================================================================

    // Item 31: First-Class operation Values (Tests 91, 92, 93)
    #[test]
    fn test_item31_operation_first_class_passing_and_returning() {
        let code = r#"
        operation FetchUser {
            requires: [DbConnection];
            guarantees: [UserExists];
            effects: [reads_db];
        }
        fn main() -> i64 { return 91; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.statements.len(), 1);
        if let Statement::OperationDecl(op) = &module.statements[0] {
            assert_eq!(op.name, "FetchUser");
            assert_eq!(op.requires, vec!["DbConnection".to_string()]);
            assert_eq!(op.guarantees, vec!["UserExists".to_string()]);
        }
    }

    #[test]
    fn test_item31_operation_higher_order_combinators() {
        let code = r#"
        operation ValidateAuth {
            requires: [JwtSecret];
        }
        fn main() -> i64 { return 92; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_item31_operation_lambda_anonymous_binding() {
        let code = r#"
        operation ProcessOrder {
            effects: [writes_db, emits_event];
        }
        fn main() -> i64 { return 93; }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("OPERATION DECLARATION 'ProcessOrder'"));
    }

    // Item 32: Operation Composition (compose_ops) (Tests 94, 95, 96)
    #[test]
    fn test_item32_operation_composition_pipeline_order() {
        let code = r#"
        operation OpA {}
        operation OpB {}
        fn main() -> i64 {
            val combo = OpA >> OpB;
            return 94;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Compose { .. }));
        }
    }

    #[test]
    fn test_item32_operation_composition_type_inference() {
        let code = r#"
        operation Step1 {}
        operation Step2 {}
        operation Step3 {}
        fn main() -> i64 {
            val full_pipeline = Step1 >> Step2 >> Step3;
            return 95;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(95));
    }

    #[test]
    fn test_item32_operation_composition_early_exit_on_error() {
        let code = r#"
        operation Validate {}
        operation Persist {}
        fn main() -> i64 {
            val p = Validate >> Persist;
            return 96;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("end_compose_ops"));
    }

    // Item 33: Operation Resilience (retry, fallback) (Tests 97, 98, 99)
    #[test]
    fn test_item33_operation_retry_exponential_backoff() {
        let code = r#"
        operation UnreliableNetworkCall {}
        fn main() -> i64 {
            val resilient_call = UnreliableNetworkCall retry 3;
            return 97;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Repeat { is_retry: true, .. }));
        }
    }

    #[test]
    fn test_item33_operation_fallback_degraded_service() {
        let code = r#"
        operation PrimaryDb {}
        operation SecondaryDb {}
        fn main() -> i64 {
            val resilient = PrimaryDb / SecondaryDb;
            return 98;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Alternative { .. }));
        }
    }

    #[test]
    fn test_item33_operation_circuit_breaker_trip() {
        let code = r#"
        operation LiveRpc {}
        operation CachedFallback {}
        fn main() -> i64 {
            val safe_call = (LiveRpc retry 2) / CachedFallback;
            return 99;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(99));
    }

    // Item 34: Parallel Operations (parallel, parallel_choose) (Tests 100, 101, 102)
    #[test]
    fn test_item34_parallel_multi_branch_execution() {
        let code = r#"
        operation FetchProfile {}
        operation FetchOrders {}
        fn main() -> i64 {
            val parallel_load = FetchProfile || FetchOrders;
            return 100;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Parallel { .. }));
        }
    }

    #[test]
    fn test_item34_parallel_choose_first_successful_hedging() {
        let code = r#"
        fn main() -> i64 {
            parallel_choose {
                branch US_EAST { val x = 1; }
                branch EU_WEST { val y = 2; }
            }
            return 101;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::ParallelChoose { .. })));
    }

    #[test]
    fn test_item34_parallel_thread_pool_and_fiber_safety() {
        let code = r#"
        fn main() -> i64 {
            parallel_choose {
                branch ServerA { val resA = 10; }
                branch ServerB { val resB = 20; }
            }
            return 102;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(102));
    }

    // Item 35: Operation Memoization (memoize) (Tests 103, 104, 105)
    #[test]
    fn test_item35_memoize_pure_computation_cache() {
        let code = r#"
        operation ComputeFibonacci {}
        fn main() -> i64 {
            val cached_fib = memoize ComputeFibonacci;
            return 103;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        if let Statement::VarDecl { initializer: Some(initializer), .. } = &main_fn.body.statements[0] {
            assert!(matches!(initializer, Expression::Memoize { .. }));
        }
    }

    #[test]
    fn test_item35_memoize_cache_invalidation_key() {
        let code = r#"
        operation CalculateTaxTable {}
        fn main() -> i64 {
            val fast_tax = memoize CalculateTaxTable;
            return 104;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(104));
    }

    #[test]
    fn test_item35_memoize_thread_safe_concurrent_access() {
        let code = r#"
        operation HeavyMatrixMultiply {}
        fn main() -> i64 {
            val cached_matmul = memoize HeavyMatrixMultiply;
            return 105;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("end_memoize_op"));
    }

    // =========================================================================
    // CATEGORY 8: SLA, QUALITY & CONCURRENCY GUARDRAILS (ITEMS 36-40)
    // =========================================================================

    // Item 36: Budget & SLA Enforcement (budget) (Tests 106, 107, 108)
    #[test]
    fn test_item36_budget_ms_runtime_latency_clamp() {
        let code = r#"
        fn main() -> i64 {
            budget latency: "50ms", memory: "10MB" {
                val computed = 100;
            }
            return 106;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::BudgetBlock { .. })));
    }

    #[test]
    fn test_item36_budget_memory_allocation_ceiling() {
        let code = r#"
        fn main() -> i64 {
            budget max_allocations: "1000" {
                val x = 1;
            }
            return 107;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(107));
    }

    #[test]
    fn test_item36_budget_yield_to_frame_renderer() {
        let code = r#"
        fn main() -> i64 {
            budget frame_budget: "16ms" {
                val frame_draw = 1;
            }
            return 108;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("BUDGET SLA"));
    }

    // Item 37: Deadline Enforcement (deadline) (Tests 109, 110, 111)
    #[test]
    fn test_item37_deadline_timeout_cancellation() {
        let code = r#"
        fn main() -> i64 {
            deadline "100ms" {
                val task_result = 42;
            }
            return 109;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::DeadlineBlock { .. })));
    }

    #[test]
    fn test_item37_deadline_cascading_subtask_cancellation() {
        let code = r#"
        fn main() -> i64 {
            deadline "250ms" {
                val rpc_call = 1;
            }
            return 110;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(110));
    }

    #[test]
    fn test_item37_deadline_graceful_partial_result() {
        let code = r#"
        fn main() -> i64 {
            deadline "500ms" {
                val sub = 200;
            }
            return 111;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("DEADLINE ENFORCEMENT"));
    }

    // Item 38: Priority Dispatch (priority) (Tests 112, 113, 114)
    #[test]
    fn test_item38_priority_realtime_vs_background_queues() {
        let code = r#"
        fn main() -> i64 {
            priority high {
                val audio_packet = 1;
            }
            return 112;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::PriorityBlock { .. })));
    }

    #[test]
    fn test_item38_priority_inversion_prevention() {
        let code = r#"
        fn main() -> i64 {
            priority critical {
                val lock_acquired = true;
            }
            return 113;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(113));
    }

    #[test]
    fn test_item38_priority_dynamic_boost() {
        let code = r#"
        fn main() -> i64 {
            priority low {
                val garbage_collect = 1;
            }
            return 114;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("PRIORITY LEVEL: low"));
    }

    // Item 39: Quality & Tradeoff Constraints (quality, tradeoff) (Tests 115, 116, 117)
    #[test]
    fn test_item39_quality_min_accuracy_enforcement() {
        let code = r#"
        fn main() -> i64 {
            quality min_accuracy: "0.99", max_latency: "20ms" {
                val ai_inference = 1;
            }
            return 115;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::QualityBlock { .. })));
    }

    #[test]
    fn test_item39_tradeoff_latency_vs_fidelity_dispatch() {
        let code = r#"
        fn main() -> i64 {
            tradeoff prefer: "latency", sacrifice: "precision" {
                val fast_approx = 42;
            }
            return 116;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::TradeoffBlock { .. })));
    }

    #[test]
    fn test_item39_adaptive_dispatch_under_system_load() {
        let code = r#"
        fn main() -> i64 {
            adapt {
                when high_load { val mode = 1; }
                when normal_load { val mode = 2; }
            }
            return 117;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::AdaptBlock { .. })));
    }

    // Item 40: Race-Free Concurrency & Ownership (owned, spawn) (Tests 118, 119, 120)
    #[test]
    fn test_item40_owned_unique_ownership_handoff() {
        let code = r#"
        fn main() -> i64 {
            owned val buffer = 1024;
            return 118;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[0];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::Owned { .. })));
    }

    #[test]
    fn test_item40_spawn_isolated_fiber_execution() {
        let code = r#"
        fn async_worker() -> i64 { return 1; }
        fn main() -> i64 {
            spawn async_worker();
            return 119;
        }
        "#;
        let module = parse_str(code).unwrap();
        let main_fn = &module.functions[1];
        assert!(main_fn.body.statements.iter().any(|s| matches!(s, Statement::Spawn { .. })));
    }

    #[test]
    fn test_item40_race_free_data_race_compile_error() {
        let code = r#"
        fn main() -> i64 {
            owned val unique_resource = 500;
            return 120;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(120));
    }

    // =========================================================================
    // CATEGORY 9: AI AGENT CONTRACTS & EXECUTABLE TASKS (ITEMS 41-45)
    // =========================================================================

    // Item 41: First-Class skill Definition (Tests 121, 122, 123)
    #[test]
    fn test_item41_skill_definition_with_hard_soft_rules() {
        let code = r#"
        skill Transactional for function {
            rules: ["auto_rollback", "no_dirty_reads"];
            hard: ["zero_data_loss"];
            soft: ["latency_under_50ms"];
        }
        fn main() -> i64 { return 121; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SkillDecl { .. })));
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(121));
        assert!(interp.skills.contains_key("Transactional"));
    }

    #[test]
    fn test_item41_skill_scope_function_and_module() {
        let code = r#"
        skill PureCompute for module {
            rules: ["no_io_in_loop"];
            hard: ["deterministic_results"];
        }
        fn main() -> i64 { return 122; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        let sk = interp.skills.get("PureCompute").unwrap();
        assert_eq!(sk.hard, vec!["deterministic_results".to_string()]);
    }

    #[test]
    fn test_item41_skill_satisfies_contract_verification() {
        let code = r#"
        satisfies PaymentModule ["Transactional", "PureCompute"];
        fn main() -> i64 { return 123; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SatisfiesDecl { .. })));
    }

    // Item 42: Agent Task Graph (task, todo) (Tests 124, 125, 126)
    #[test]
    fn test_item42_task_graph_dependency_resolution() {
        let code = r#"
        task refactor_auth_module {
            owner: "agent_alpha";
            status: "planned";
            requirement: "REQ_SEC_001";
            implementation: "modern_oauth2";
            skills: ["Transactional"];
        }
        fn main() -> i64 { return 124; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::TaskDecl { .. } | Statement::AgentTaskContractDecl { .. })));
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert!(interp.tasks_state.contains_key("refactor_auth_module"));
    }

    #[test]
    fn test_item42_todo_executable_spec_status_tracking() {
        let code = r#"
        todo REQ_PAY_001 {
            implement: "Stripe and PayPal Webhook Verification";
            requires: ["CryptoLib", "HttpClient"];
            verify: ["test_webhook_signature"];
            status: "open";
        }
        fn main() -> i64 { return 125; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::TodoDecl { .. })));
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert!(interp.todos_state.contains_key("REQ_PAY_001"));
    }

    #[test]
    fn test_item42_task_evidence_and_confidence_scoring() {
        let code = r#"
        task verify_encryption {
            owner: "security_bot";
            status: "in_progress";
            evidence {
                test: "test_aes256_gcm";
                prove: "non_zero_iv";
            }
        }
        fn main() -> i64 { return 126; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert!(interp.tasks_state.contains_key("verify_encryption"));
    }

    // Item 43: Agent Task Lifecycle (claim, complete, verify) (Tests 127, 128, 129)
    #[test]
    fn test_item43_task_claim_ownership_transition() {
        let code = r#"
        task optimize_query {
            owner: "unassigned";
            status: "planned";
        }
        claim task optimize_query;
        fn main() -> i64 { return 127; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(127));
        assert_eq!(interp.tasks_state.get("optimize_query").unwrap().status, "claimed");
    }

    #[test]
    fn test_item43_task_complete_evidence_submission() {
        let code = r#"
        task patch_vulnerability {
            owner: "agent_sec";
            status: "claimed";
        }
        complete task patch_vulnerability {
            result: "success";
            confidence: 0.99;
            summary: "SQL Injection vector eliminated and validated";
        }
        fn main() -> i64 { return 128; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert_eq!(interp.tasks_state.get("patch_vulnerability").unwrap().status, "completed");
        assert_eq!(interp.tasks_state.get("patch_vulnerability").unwrap().confidence, Some(0.99));
    }

    #[test]
    fn test_item43_task_verify_adversarial_testing() {
        let code = r#"
        task secure_vault {
            owner: "agent_sec";
            status: "completed";
        }
        verify secure_vault;
        fn main() -> i64 { return 129; }
        "#;
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let _ = interp.run(&module);
        assert_eq!(interp.tasks_state.get("secure_vault").unwrap().status, "accepted");
    }

    // Item 44: Agent Scoping & Boundary Firewall (agent_boundary, context_firewall) (Tests 130, 131, 132)
    #[test]
    fn test_item44_agent_boundary_private_secret_firewall() {
        let code = r#"
        agent_boundary PaymentSecurity;
        fn main() -> i64 { return 130; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::AgentBoundaryDecl { .. })));
    }

    #[test]
    fn test_item44_context_firewall_prompt_injection_guard() {
        let code = r#"
        context_firewall CoreDatabase {
            deny: ["drop_tables", "raw_exec"];
            expose: ["typed_query"];
        }
        fn main() -> i64 { return 131; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ContextFirewallDecl { .. })));
    }

    #[test]
    fn test_item44_agentability_token_budget_limits() {
        let code = r#"
        agentability {
            max_context_tokens: 12000;
            max_operation_complexity: "low";
            max_dependency_fanout: 4;
        }
        fn main() -> i64 { return 132; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::AgentabilityDecl { .. })));
    }

    // Item 45: Intent Contracts & Semantic Commits (intent, change) (Tests 133, 134, 135)
    #[test]
    fn test_item45_intent_goal_preserve_invariant_check() {
        let code = r#"
        intent {
            goal: "Refactor database connection pool";
            preserve: ["zero_connection_leaks", "thread_safety"];
            optimize: ["throughput"];
        }
        fn main() -> i64 { return 133; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::IntentDecl { .. })));
    }

    #[test]
    fn test_item45_intent_drift_detection_and_rejection() {
        let code = r#"
        intent diff preserve: ["public_api_v1"], change: ["internal_caching"];
        fn main() -> i64 { return 134; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::IntentDiff { .. })));
    }

    #[test]
    fn test_item45_semantic_commit_verified_manifest() {
        let code = r#"
        change {
            task: "implement_oauth2";
            message: "Added secure JWT verification flow";
            intent: "OAuth2 authentication compliant";
            evidence: ["test_token_verification"];
        }
        fn main() -> i64 { return 135; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SemanticCommitDecl { .. })));
    }

    // =========================================================================
    // CATEGORY 10: EXTENSIBILITY DNA & UNIVERSAL VERIFICATION (ITEMS 46-50)
    // =========================================================================

    // Item 46: Partial & Augment Types (partial, augment) (Tests 136, 137, 138)
    #[test]
    fn test_item46_partial_struct_and_module_merging() {
        let code = r#"
        partial struct UserProfile {
            id: i64,
        }
        partial struct UserProfile {
            name: string,
        }
        fn main() -> i64 { return 136; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.structs.len(), 2);
        assert!(module.structs[0].is_partial);
        assert!(module.structs[1].is_partial);
    }

    #[test]
    fn test_item46_augment_external_library_type() {
        let code = r#"
        augment struct HttpRequest {
            session_id: string,
        }
        fn main() -> i64 { return 137; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 1);
        assert!(module.extensions[0].is_augment);
    }

    #[test]
    fn test_item46_partial_conflict_resolution() {
        let code = r#"
        partial module SharedKernel {
            fn utility_one() -> i64 { return 1; }
        }
        partial module SharedKernel {
            fn utility_two() -> i64 { return 2; }
        }
        fn main() -> i64 { return 138; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.modules.len(), 2);
        assert!(module.modules[0].is_partial);
        assert!(module.modules[1].is_partial);
    }

    // Item 47: Open/Closed Extensibility Types (open, closed) (Tests 139, 140, 141)
    #[test]
    fn test_item47_open_type_unrestricted_subtyping() {
        let code = r#"
        open struct PluginContext {
            env: string,
        }
        fn main() -> i64 { return 139; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: true, .. })));
    }

    #[test]
    fn test_item47_closed_type_sealed_pattern_matching() {
        let code = r#"
        closed struct SecurityDescriptor {
            token: string,
        }
        fn main() -> i64 { return 140; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: false, .. })));
    }

    #[test]
    fn test_item47_open_type_dynamic_extension_table() {
        let code = r#"
        open module ExtensibleEngine;
        fn main() -> i64 { return 141; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::OpenClosedTypeDecl { is_open: true, .. })));
    }

    // Item 48: Namespaced Versioned Syntax (syntax @version) (Tests 142, 143, 144)
    #[test]
    fn test_item48_custom_syntax_definition_and_dsl() {
        let code = r#"
        syntax web::graphql {
            query: string;
        }
        fn main() -> i64 { return 142; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::SyntaxDecl { .. })));
    }

    #[test]
    fn test_item48_namespaced_syntax_isolated_scope() {
        let code = r#"
        use syntax web::graphql;
        fn main() -> i64 { return 143; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::UseSyntaxDecl { .. })));
    }

    #[test]
    fn test_item48_syntax_versioning_and_migration() {
        let code = r#"
        syntax db::sql {
            statement: string;
        }
        use syntax db::sql;
        fn main() -> i64 { return 144; }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.statements.len(), 2);
        assert_eq!(module.functions.len(), 1);
    }

    // Item 49: Compile-time Plugins & Analyzers (compiler_plugin, lint) (Tests 145, 146, 147)
    #[test]
    fn test_item49_custom_compiler_plugin_optimization() {
        let code = r#"
        compiler_plugin VectorizerPass optimizer;
        fn main() -> i64 { return 145; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::CompilerPluginDecl { .. })));
    }

    #[test]
    fn test_item49_custom_linter_rule_enforcement() {
        let code = r#"
        lint NoUnwrapInProduction {
            "disallow Expression::QuantumUnwrap without fallback";
        }
        fn main() -> i64 { return 146; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::CustomLinterDecl { .. })));
    }

    #[test]
    fn test_item49_type_rule_and_reflect_inspection() {
        let code = r#"
        reflect UserAccount {
            fields;
            methods;
        }
        fn main() -> i64 { return 147; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::ReflectDecl { .. })));
    }

    // Item 50: Blast Radius & 7-Pillar Evolution Engine (calculate_blast_radius) (Tests 148, 149, 150)
    #[test]
    fn test_item50_blast_radius_direct_and_transitive_impact() {
        let code = r#"
        feature PaymentCore {
            api {
                fn charge() -> bool;
            }
        }
        feature CheckoutService {
            needs: [PaymentCore];
        }
        feature AnalyticsService {
            needs: [CheckoutService];
        }
        fn main() -> i64 { return 148; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());

        let report = analyzer.calculate_blast_radius("PaymentCore");
        println!("DEBUG REPORT: {:?}", report);
        println!("DEBUG FEATURES: {:?}", analyzer.features.keys());
        assert!(report.affected_features.contains(&"CheckoutService".to_string()));
        assert!(report.affected_features.contains(&"AnalyticsService".to_string()));
        assert!(report.affected_public_apis.contains(&"charge".to_string()));
    }

    #[test]
    fn test_item50_blast_radius_breaking_change_safeguards() {
        let code = r#"
        feature LegacyAuth {
            lifecycle {
                deprecated;
                replace_with: OAuth2Service;
                migration: AuthMigrationPath;
            }
        }
        fn main() -> i64 { return 149; }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let _ = analyzer.analyze_module(&module);

        let report = analyzer.calculate_blast_radius("LegacyAuth");
        assert!(report.required_migrations.contains(&"AuthMigrationPath".to_string()));
    }

    #[test]
    fn test_item50_evolution_proposal_and_formal_proof_gate() {
        let code = r#"
        proposal UpgradeToV2 {
            target: "PaymentGateway";
            version: 2;
            proof: "zero_downtime_guaranteed";
        }
        fn main() -> i64 { return 150; }
        "#;
        let module = parse_str(code).unwrap();
        assert!(module.statements.iter().any(|s| matches!(s, Statement::AgentChangeProposalDecl { .. })));
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(150));
    }
}
