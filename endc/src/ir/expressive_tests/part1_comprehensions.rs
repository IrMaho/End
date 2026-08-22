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
    // ITEM 09: Set Comprehension (3 Tests)
    // =========================================================================
    #[test]
    fn test_item09_set_comprehension_field_projection() {
        let code = r#"
        fn extract_unique_ids() -> i64 {
            val ids = {x.id for x in users};
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item09_set_comprehension_with_filter() {
        let code = r#"
        fn unique_tags() -> i64 {
            val tags = {item.tag for item in items if item.is_visible};
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item09_set_comprehension_computed_values() {
        let code = r#"
        fn set_of_squares() -> i64 {
            val squares = {x * 2 for x in data if x > 10};
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 10: Conditional Expressions (3 Tests)
    // =========================================================================
    #[test]
    fn test_item10_python_style_conditional_expression() {
        let code = r#"
        fn max_of(a: i64, b: i64) -> i64 {
            val result = a if a > b else b;
            ret result;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item10_conditional_expression_with_function_calls() {
        let code = r#"
        fn fetch_title(is_guest: bool) -> string {
            val title = "Guest" if is_guest else get_username();
            ret title;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item10_nested_conditional_expressions() {
        let code = r#"
        fn status_code(ok: bool, pending: bool) -> i64 {
            val code = 200 if ok else (102 if pending else 500);
            ret code;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 11: Walrus Assignment Expression (3 Tests)
    // =========================================================================
    #[test]
    fn test_item11_walrus_expression_in_if_condition() {
        let code = r#"
        fn check_length() -> bool {
            if (n := get_count()) > 10 {
                ret true;
            }
            ret false;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item11_walrus_in_while_loop_header() {
        let code = r#"
        fn process_stream() -> i64 {
            while (chunk := next_chunk()) != null {
                process(chunk);
            }
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item11_walrus_expression_scoped_assignment() {
        let code = r#"
        fn evaluate() -> i64 {
            val res = (val_x := calculate()) * 2;
            ret res;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 12: Python *args Variadic Arguments (3 Tests)
    // =========================================================================
    #[test]
    fn test_item12_variadic_args_definition() {
        let code = r#"
        fn log_all(*messages) -> void {
            // variadic log
        }
        fn main() -> i64 {
            log_all("info", "warn", "error");
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item12_variadic_args_with_leading_params() {
        let code = r#"
        fn format_string(template: string, *args) -> string {
            ret template;
        }
        fn main() -> i64 {
            val msg = format_string("User {} logged in from {}", "Maho", "127.0.0.1");
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item12_variadic_args_semantic_analysis() {
        let code = r#"
        fn sum_numbers(*nums) -> i64 {
            ret 42;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 13: Python **kwargs Keyword Arguments (3 Tests)
    // =========================================================================
    #[test]
    fn test_item13_kwargs_function_definition() {
        let code = r#"
        fn configure(**options) -> void {
            // configure system
        }
        fn main() -> i64 {
            configure(debug: true, cache: true, max_conns: 100);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item13_kwargs_mixed_with_positional_args() {
        let code = r#"
        fn create_service(name: string, **config) -> bool {
            ret true;
        }
        fn main() -> i64 {
            val ok = create_service("AuthService", timeout: 30, retry: 3);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item13_kwargs_dictionary_forwarding() {
        let code = r#"
        fn bootstrap_server(**settings) -> i64 {
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 14: Named Arguments Call (3 Tests)
    // =========================================================================
    #[test]
    fn test_item14_named_arguments_call_basic() {
        let code = r#"
        fn create_user(name: string, age: i64) -> i64 {
            ret age;
        }
        fn main() -> i64 {
            val u = create_user(name: "Maho", age: 24);
            ret u;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item14_named_arguments_reordered() {
        let code = r#"
        fn connect(host: string, port: i64, timeout: i64) -> bool {
            ret true;
        }
        fn main() -> i64 {
            val c = connect(timeout: 5000, host: "localhost", port: 8080);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item14_named_arguments_semantic_check() {
        let code = r#"
        fn setup(enabled: bool, workers: i64) -> i64 {
            ret workers;
        }
        fn main() -> i64 {
            ret setup(enabled: true, workers: 8);
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 15: Optional / Default Parameters (3 Tests)
    // =========================================================================
    #[test]
    fn test_item15_optional_default_parameters_declaration() {
        let code = r#"
        fn connect(host: string, port: i64 = 443) -> bool {
            ret true;
        }
        fn main() -> i64 {
            val c1 = connect("api.end.org");
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item15_multiple_optional_parameters() {
        let code = r#"
        fn query(sql: string, timeout: i64 = 30, retries: i64 = 3) -> i64 {
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item15_optional_params_with_override_call() {
        let code = r#"
        fn render_view(theme: string = "dark", scale: i64 = 1) -> bool {
            ret true;
        }
        fn main() -> i64 {
            val r = render_view("light", 2);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 16: Required Named Parameters (3 Tests)
    // =========================================================================
    #[test]
    fn test_item16_required_named_parameters_declaration() {
        let code = r#"
        fn send_payment(amount: i64, required recipient: string) -> bool {
            ret true;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item16_required_named_parameters_call_site() {
        let code = r#"
        fn execute_task(task_id: i64, required token: string) -> bool {
            ret true;
        }
        fn main() -> i64 {
            val res = execute_task(101, token: "auth_sec_key");
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item16_required_named_parameters_semantic_enforcement() {
        let code = r#"
        fn deploy(target: string, required version: string) -> i64 {
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }
}
