#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::semantic::refactoring_analyzer::*;
    use crate::codegen::interpreter::Interpreter;

    fn parse_code(code: &str) -> Module {
        let mut lexer = Lexer::new("test.end", code);
        let tokens = lexer.tokenize_all().expect("Lexing failed");
        let mut parser = Parser::new("test.end", tokens);
        parser.parse_module("test_mod").expect("Parsing failed")
    }

    #[test]
    fn test_phase1_symbol_inventory_and_classification() {
        let code = r#"
            inventory ParserModule {
                classes: ["Parser", "TokenStream"],
                functions: ["parse_expr", "parse_stmt", "parse_type"],
                types: ["Span", "TokenKind"],
                public_exports: ["Parser", "parse_expr"],
                internal_symbols: ["advance_cursor", "match_internal"]
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 1);
        if let Statement::SymbolInventoryStmt(ref inv) = module.statements[0] {
            assert_eq!(inv.module_name, "ParserModule");
            assert_eq!(inv.classes.len(), 2);
            assert_eq!(inv.functions.len(), 3);
            assert_eq!(inv.types.len(), 2);
            assert_eq!(inv.public_exports.len(), 2);
            assert_eq!(inv.internal_symbols.len(), 2);
        } else {
            panic!("Expected SymbolInventoryStmt");
        }
    }

    #[test]
    fn test_phase2_responsibility_mapping_and_boundaries() {
        let code = r#"
            responsibility: "Orchestrate syntax parsing and diagnostic collection";
            boundary CoreParser {
                allow: ["TokenStream", "ASTNode"],
                deny: ["FileIO", "NetworkSocket"],
                sealed
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 2);
        if let Statement::BoundaryDecl { ref name, ref allows, ref denies, is_sealed, .. } = module.statements[1] {
            assert_eq!(name, "CoreParser");
            assert_eq!(allows.len(), 2);
            assert_eq!(denies.len(), 2);
            assert!(is_sealed);
        }
    }

    #[test]
    fn test_phase3_target_architecture_decomposition_planning() {
        let code = r#"
            decompose for "parser.ts" {
                source: "src/compiler/parser.ts",
                target_architecture: "ModularSubsystem",
                facade: "parser/mod.ts",
                submodules: [
                    { name: "state", role: "state_management", symbols: ["ParserState", "Position"], max_loc: 280 },
                    { name: "expressions", role: "expression_parsing", symbols: ["parse_binary", "parse_unary"], max_loc: 420 },
                    { name: "statements", role: "statement_parsing", symbols: ["parse_stmt", "parse_decl"], max_loc: 440 }
                ]
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 1);
        if let Statement::DecompositionPlanStmt(ref plan) = module.statements[0] {
            assert_eq!(plan.source, "src/compiler/parser.ts");
            assert_eq!(plan.target_architecture, "ModularSubsystem");
            assert_eq!(plan.facade_name, Some("parser/mod.ts".to_string()));
            assert_eq!(plan.submodules.len(), 3);
            assert_eq!(plan.submodules[0].max_loc, 280);
            assert_eq!(plan.submodules[1].max_loc, 420);
        } else {
            panic!("Expected DecompositionPlanStmt");
        }
    }

    #[test]
    fn test_phase4_hard_line_limit_constraint_verification() {
        let code = r#"
            decompose for "monolith.rs" {
                source: "src/monolith.rs",
                target_architecture: "ModularPackage",
                submodules: [
                    { name: "sub1", role: "core", symbols: ["SymA"], max_loc: 450 },
                    { name: "sub2", role: "aux", symbols: ["SymB"], max_loc: 520 }
                ]
            }
        "#;
        let module = parse_code(code);
        let mut analyzer = RefactoringAnalyzer::new();
        if let Statement::DecompositionPlanStmt(ref plan) = module.statements[0] {
            analyzer.register_plan(plan);
        }
        let report = analyzer.run_full_audit();
        assert!(!report.is_valid);
        assert_eq!(report.line_limit_violations.len(), 1);
        assert!(report.line_limit_violations[0].contains("exceeds hard limit 500"));
    }

    #[test]
    fn test_phase5_symbol_conservation_and_lossless_audit() {
        let code = r#"
            conservation {
                original_source: "src/compiler/parser.ts",
                original_loc: 10247,
                original_symbols: ["Parser", "TokenStream", "parse_expr", "parse_stmt", "Span"],
                new_loc: 10391,
                accounted_symbols: ["Parser", "TokenStream", "parse_expr", "parse_stmt", "Span"],
                unaccounted: 0,
                allow_deletion: false
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 1);
        if let Statement::ConservationAuditStmt(ref audit) = module.statements[0] {
            assert_eq!(audit.original_loc, 10247);
            assert_eq!(audit.new_loc, 10391);
            assert_eq!(audit.unaccounted_count, 0);
            assert!(!audit.allow_semantic_deletion);

            let mut analyzer = RefactoringAnalyzer::new();
            analyzer.register_conservation_audit(audit);
            let report = analyzer.run_full_audit();
            assert!(report.is_valid);
            assert_eq!(report.original_symbols_count, 5);
            assert_eq!(report.accounted_symbols_count, 5);
            assert_eq!(report.unaccounted_symbols_count, 0);

            let (diff, pct) = analyzer.compute_line_differential(audit.original_loc, audit.new_loc);
            assert_eq!(diff, 144);
            assert!(pct > 1.0 && pct < 2.0);
        }
    }

    #[test]
    fn test_phase6_unaccounted_symbol_detection_failure() {
        let code = r#"
            conservation {
                original_source: "src/compiler/legacy.ts",
                original_loc: 5000,
                original_symbols: ["A", "B", "C"],
                new_loc: 4800,
                accounted_symbols: ["A", "B"],
                unaccounted: 1,
                allow_deletion: false
            }
        "#;
        let module = parse_code(code);
        let mut analyzer = RefactoringAnalyzer::new();
        if let Statement::ConservationAuditStmt(ref audit) = module.statements[0] {
            analyzer.register_conservation_audit(audit);
        }
        let report = analyzer.run_full_audit();
        assert!(!report.is_valid);
        assert_eq!(report.warnings.len(), 1);
        assert!(report.warnings[0].contains("Lossless Violation"));
    }

    #[test]
    fn test_phase7_solid_compliance_audit() {
        let code = r#"
            solid ParserState {
                srp: true,
                ocp: true,
                lsp: true,
                isp: true,
                dip: true,
                max_responsibilities: 1
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 1);
        if let Statement::SolidAuditStmt(ref solid) = module.statements[0] {
            assert_eq!(solid.module_name, "ParserState");
            assert!(solid.verify_srp);
            assert_eq!(solid.max_responsibilities, 1);

            let mut analyzer = RefactoringAnalyzer::new();
            analyzer.register_solid_audit(solid);
            let report = analyzer.run_full_audit();
            assert!(report.is_valid);
            assert_eq!(report.solid_violations.len(), 0);
        }
    }

    #[test]
    fn test_phase8_refactoring_transactions_and_rollbacks() {
        let code = r#"
            refactor transaction SurgicalModularization {
                checkpoint: "pre_refactor_baseline",
                steps: [
                    "create_directory_structure",
                    "extract_state_module",
                    "extract_expression_module",
                    "generate_facade_exports"
                ],
                auto_rollback: true,
                test_gate: true,
                build_gate: true,
                max_lines: 500
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 1);
        if let Statement::RefactoringTxStmt(ref tx) = module.statements[0] {
            assert_eq!(tx.tx_name, "SurgicalModularization");
            assert_eq!(tx.checkpoint, "pre_refactor_baseline");
            assert_eq!(tx.steps.len(), 4);
            assert!(tx.auto_rollback);
            assert!(tx.run_test_gate);
            assert_eq!(tx.max_lines_limit, 500);
        }
    }

    #[test]
    fn test_phase9_traceable_destination_mapping() {
        let code = r#"
            traceable destination for "parser.ts" {
                ParserState -> state_module,
                parse_binary -> expression_module,
                parse_stmt -> statement_module,
                DiagnosticEngine -> diagnostics_module
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 1);
        if let Statement::TraceableMapStmt(ref map) = module.statements[0] {
            assert_eq!(map.source_module, "parser.ts");
            assert_eq!(map.mappings.len(), 4);
            assert_eq!(map.mappings[0], ("ParserState".to_string(), "state_module".to_string()));
            assert_eq!(map.mappings[3], ("DiagnosticEngine".to_string(), "diagnostics_module".to_string()));

            let mut analyzer = RefactoringAnalyzer::new();
            analyzer.register_traceable_map(map);
            let report = analyzer.run_full_audit();
            assert!(report.is_valid);
        }
    }

    #[test]
    fn test_phase10_agent_session_orchestration_and_vm_execution() {
        let code = r#"
            refactor session RefactoringAgent {
                target: "src/monolith.rs",
                scope: ["ast", "parser", "codegen"],
                forbid: ["security_core"],
                goals: ["eliminate_monolith", "enforce_500_lines", "zero_data_loss"]
            }
        "#;
        let module = parse_code(code);
        assert_eq!(module.statements.len(), 1);
        if let Statement::RefactorSessionStmt(ref session) = module.statements[0] {
            assert_eq!(session.agent_name, "RefactoringAgent");
            assert_eq!(session.target, "src/monolith.rs");
            assert_eq!(session.scope.len(), 3);
            assert_eq!(session.forbid.len(), 1);
            assert_eq!(session.goals.len(), 3);
        }

        let mut interp = Interpreter::new();
        for stmt in &module.statements {
            let res = interp.eval_statement(stmt);
            assert!(res.is_ok());
        }
    }
}
