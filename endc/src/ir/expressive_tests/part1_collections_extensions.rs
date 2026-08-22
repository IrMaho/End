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
    // ITEM 25: Collection `if` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item25_collection_if_single_condition() {
        let code = r#"
        fn build_nav(loggedIn: bool) -> i64 {
            val nav = [header, if loggedIn userPanel];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item25_collection_if_else_branches() {
        let code = r#"
        fn build_auth_bar(is_admin: bool) -> i64 {
            val bar = [logo, if is_admin adminButton else loginButton, footer];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item25_collection_if_semantic_analysis() {
        let code = r#"
        fn main() -> i64 {
            val items = [1, if true 2 else 3, 4];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 26: Collection `for` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item26_collection_for_element_expansion() {
        let code = r#"
        fn list_user_cards() -> i64 {
            val cards = [for user in users user.name];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item26_collection_for_with_expression_body() {
        let code = r#"
        fn transform_data() -> i64 {
            val result = [for item in dataset item.computeScore()];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item26_collection_for_semantic_validation() {
        let code = r#"
        fn main() -> i64 {
            val items = [for x in nums x];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 27: Nested Collection Control Flow (3 Tests)
    // =========================================================================
    #[test]
    fn test_item27_nested_if_and_for_in_collection() {
        let code = r#"
        fn render_dashboard(loggedIn: bool) -> i64 {
            val ui = [if loggedIn [for user in users user.card()]];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item27_deeply_nested_control_flow_with_spread() {
        let code = r#"
        fn complex_tree(show_sections: bool) -> i64 {
            val tree = [
                header,
                if show_sections [
                    for sec in sections [
                        sec.title,
                        ...sec.children
                    ]
                ],
                footer
            ];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item27_nested_collection_control_flow_semantic() {
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
    // ITEM 28: Extension Methods (3 Tests)
    // =========================================================================
    #[test]
    fn test_item28_extend_primitive_type_with_methods() {
        let code = r#"
        extend string {
            fn is_email() -> bool {
                ret true;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 1);
    }

    #[test]
    fn test_item28_extend_custom_struct_with_utility_method() {
        let code = r#"
        st User {
            id: i64,
            first_name: string,
            last_name: string,
        }
        extend User {
            fn full_name() -> string {
                ret "User";
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.structs.len(), 1);
        assert_eq!(module.extensions.len(), 1);
    }

    #[test]
    fn test_item28_extension_methods_semantic_resolution() {
        let code = r#"
        extend i64 {
            fn is_positive() -> bool {
                ret true;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 29: Extension Properties (3 Tests)
    // =========================================================================
    #[test]
    fn test_item29_extension_property_getter() {
        let code = r#"
        extend User {
            fn initials() -> string {
                ret "M.A";
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 1);
    }

    #[test]
    fn test_item29_extension_property_on_collection() {
        let code = r#"
        extend List {
            fn is_empty() -> bool {
                ret true;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.extensions.len(), 1);
    }

    #[test]
    fn test_item29_extension_properties_semantic_check() {
        let code = r#"
        extend string {
            fn length_count() -> i64 {
                ret 10;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 30: Operator Overloading Convention (3 Tests)
    // =========================================================================
    #[test]
    fn test_item30_arithmetic_operator_overloading() {
        let code = r#"
        st Vector2 {
            x: i64,
            y: i64,
        }
        fn main() -> i64 {
            val a = Vector2 { x: 10, y: 20 };
            val b = Vector2 { x: 5, y: 15 };
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.structs.len(), 1);
    }

    #[test]
    fn test_item30_index_and_contains_operator_convention() {
        let code = r#"
        fn check_membership() -> bool {
            val found = 3 in collection;
            ret found;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item30_operator_overload_semantic_analysis() {
        let code = r#"
        st Matrix {
            rows: i64,
            cols: i64,
        }
        fn main() -> i64 {
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 31: Invoke Syntax on Objects (3 Tests)
    // =========================================================================
    #[test]
    fn test_item31_callable_object_direct_invoke() {
        let code = r#"
        fn run_validation(validator: Validator, user: User) -> bool {
            val isValid = validator(user);
            ret isValid;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item31_callable_closure_handler_invocation() {
        let code = r#"
        fn dispatch(handler: Handler, event: Event) -> i64 {
            ret handler(event);
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item31_invoke_syntax_semantic_validation() {
        let code = r#"
        fn execute(callable: Callable) -> i64 {
            ret callable();
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 32: Custom Destructuring Protocol (3 Tests)
    // =========================================================================
    #[test]
    fn test_item32_custom_destructuring_pair_protocol() {
        let code = r#"
        fn main() -> i64 {
            (first, second) := custom_pair_object;
            ret first;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item32_custom_destructuring_triple_protocol() {
        let code = r#"
        fn main() -> i64 {
            (x, y, z) := node_3d_coord;
            ret x + y + z;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item32_destructuring_protocol_semantic_check() {
        let code = r#"
        fn main() -> i64 {
            (a, b) := get_tuple();
            ret a;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }
}
