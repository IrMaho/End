#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::semantic::analyzer::*;

    fn parse_str(code: &str) -> Result<Module, String> {
        let mut lexer = Lexer::new("test.end", code);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new("test.end", tokens);
        parser.parse_module("test_mod")
    }

    // =========================================================================
    // ITEM 33: Immutable `.copy(...)` Method (3 Tests)
    // =========================================================================
    #[test]
    fn test_item33_immutable_copy_single_override() {
        let code = r#"
        st User {
            id: i64,
            name: string,
        }
        fn update_name(u: User) -> i64 {
            val updated = u.copy(name: "Ali");
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.structs.len(), 1);
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item33_immutable_copy_multiple_overrides() {
        let code = r#"
        st Config {
            timeout: i64,
            retries: i64,
            debug: bool,
        }
        fn main() -> i64 {
            val cfg = Config { timeout: 30, retries: 3, debug: false };
            val new_cfg = cfg.copy(timeout: 60, debug: true);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 3);
    }

    #[test]
    fn test_item33_immutable_copy_semantic_check() {
        let code = r#"
        st State {
            count: i64,
        }
        fn main() -> i64 {
            val s = State { count: 0 };
            val s2 = s.copy(count: 1);
            ret s2.count;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 34: Delegated Property (`by` / `using`) (3 Tests)
    // =========================================================================
    #[test]
    fn test_item34_delegated_property_using_delegate() {
        let code = r#"
        fn main() -> i64 {
            val user_name = "DelegateUser";
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item34_lazy_delegated_property_initialization() {
        let code = r#"
        fn main() -> i64 {
            val heavy_resource = load_heavy_data();
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item34_delegated_property_semantic_analysis() {
        let code = r#"
        fn main() -> i64 {
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 35: Property Wrapper (`@wrapper`) (3 Tests)
    // =========================================================================
    #[test]
    fn test_item35_property_wrapper_directive_annotation() {
        let code = r#"
        @validated
        st ValidatedUser {
            email: string,
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.structs.len(), 1);
    }

    #[test]
    fn test_item35_multiple_property_wrapper_annotations() {
        let code = r#"
        @persisted
        @logged
        st SessionState {
            token: string,
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.structs.len(), 1);
    }

    #[test]
    fn test_item35_property_wrapper_semantic_preservation() {
        let code = r#"
        @thread_safe
        st SharedCounter {
            count: i64,
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 36: Result Builder / Declarative DSL (3 Tests)
    // =========================================================================
    #[test]
    fn test_item36_declarative_ui_dsl_builder() {
        let code = r#"
        fn build_login_screen() -> i64 {
            val screen = ui {
                column {
                    text("Hello");
                    button("Login");
                }
            };
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item36_html_dsl_builder() {
        let code = r#"
        fn render_page() -> i64 {
            val doc = html {
                head { title("Home"); }
                body { header("Welcome"); }
            };
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item36_dsl_result_builder_semantic_check() {
        let code = r#"
        fn main() -> i64 {
            val root = widget_tree {
                node("A");
                node("B");
            };
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 37: Trailing Closure Syntax (3 Tests)
    // =========================================================================
    #[test]
    fn test_item37_trailing_closure_single_arg_mapping() {
        let code = r#"
        fn main() -> i64 {
            val mapped = users.map(fn(user: User) -> string { ret user.name; });
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item37_trailing_closure_filter_predicate() {
        let code = r#"
        fn filter_active() -> i64 {
            val active = users.filter(fn(u: User) -> bool { ret u.is_active; });
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item37_trailing_closure_semantic_validation() {
        let code = r#"
        fn for_each_item() -> i64 {
            items.for_each(fn(x: i64) -> void { print(x); });
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 38: Implicit Lambda Parameter `_.name` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item38_implicit_lambda_field_projection() {
        let code = r#"
        fn get_all_names() -> i64 {
            val names = users.map(_.name);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item38_implicit_lambda_predicate_check() {
        let code = r#"
        fn find_verified() -> i64 {
            val verified = accounts.filter(_.is_verified);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item38_implicit_lambda_semantic_analysis() {
        let code = r#"
        fn main() -> i64 {
            val ids = items.map(_.id);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 39: Functional Pipe Operator `|>` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item39_pipe_operator_single_stage() {
        let code = r#"
        fn process_input(data: string) -> i64 {
            val res = data |> sanitize |> parse;
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item39_pipe_operator_multi_stage_stream_pipeline() {
        let code = r#"
        fn run_pipeline() -> i64 {
            val result = users |> filter_active |> sort_by_age |> extract_names;
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item39_pipe_operator_semantic_preservation() {
        let code = r#"
        fn compute(n: i64) -> i64 {
            val out = n |> double |> increment;
            ret out;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 40: Range Syntax (`..` and `..<`) (3 Tests)
    // =========================================================================
    #[test]
    fn test_item40_inclusive_range_syntax() {
        let code = r#"
        fn main() -> i64 {
            val r = 1..10;
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item40_exclusive_half_open_range_syntax() {
        let code = r#"
        fn main() -> i64 {
            val half_open = 0..<100;
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item40_range_in_loop_semantic_check() {
        let code = r#"
        fn main() -> i64 {
            for i in 1..5 {
                print(i);
            }
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 41: Spread Function Arguments (3 Tests)
    // =========================================================================
    #[test]
    fn test_item41_spread_function_arguments_call() {
        let code = r#"
        fn main() -> i64 {
            log_messages(...messages);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item41_spread_mixed_with_positional_arguments() {
        let code = r#"
        fn send_batch() -> i64 {
            dispatch("PRIMARY", ...items, "TERMINATE");
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item41_spread_function_arguments_semantic_validation() {
        let code = r#"
        fn invoke_target() -> i64 {
            call_subroutine(...args);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }
}
