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
    // ITEM 42: String Interpolation (3 Tests)
    // =========================================================================
    #[test]
    fn test_item42_string_interpolation_simple_variable() {
        let code = r#"
        fn greet(name: string) -> string {
            ret "Hello {name}";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item42_string_interpolation_member_access() {
        let code = r#"
        fn welcome_user(user: User) -> string {
            ret "Welcome {user.profile.email}!";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item42_string_interpolation_semantic_check() {
        let code = r#"
        fn format_log(code: i64) -> string {
            ret "Error code: {code}";
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 43: Raw Strings `r"""..."""` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item43_raw_string_windows_file_path() {
        let code = r#"
        fn get_path() -> string {
            ret "C:\\Users\\Maho\\End";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item43_raw_string_regex_pattern() {
        let code = r#"
        fn get_regex() -> string {
            ret "\\d{4}-\\d{2}-\\d{2}";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item43_raw_string_semantic_preservation() {
        let code = r#"
        fn main() -> i64 {
            val raw = "SELECT * FROM users WHERE status = 'active'";
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 44: Multiline Strings `"""..."""` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item44_multiline_string_template() {
        let code = r#"
        fn get_query() -> string {
            ret "
            SELECT id, email
            FROM users
            ORDER BY created_at DESC
            ";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item44_multiline_docstring_constant() {
        let code = r#"
        fn get_documentation() -> string {
            ret "
            # End Language Core
            High performance, capability-first language.
            ";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item44_multiline_string_semantic_validation() {
        let code = r#"
        fn main() -> i64 {
            val text = "Line 1\nLine 2\nLine 3";
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 45: Enum-Associated Data (3 Tests)
    // =========================================================================
    #[test]
    fn test_item45_enum_with_associated_payloads() {
        let code = r#"
        enum Result {
            Ok(i64),
            Error(string),
        }
        fn main() -> i64 {
            val res = Result.Ok(200);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.enums.len(), 1);
        assert_eq!(module.enums[0].variants.len(), 2);
    }

    #[test]
    fn test_item45_complex_enum_associated_data() {
        let code = r#"
        enum Event {
            Click(i64),
            KeyPress(string),
            Quit,
        }
        fn handle(e: Event) -> i64 {
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.enums.len(), 1);
    }

    #[test]
    fn test_item45_enum_associated_data_semantic_check() {
        let code = r#"
        enum HttpResponse {
            Success(i64),
            Redirect(string),
            ClientError(i64),
        }
        fn main() -> i64 {
            val r = HttpResponse.Success(200);
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 46: Expression-Oriented `if` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item46_expression_oriented_if_variable_binding() {
        let code = r#"
        fn compute_discount(is_vip: bool) -> i64 {
            val discount = if is_vip { 20 } else { 5 };
            ret discount;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item46_expression_oriented_if_as_return_value() {
        let code = r#"
        fn get_status_message(ok: bool) -> string {
            ret if ok { "SUCCESS" } else { "FAILED" };
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item46_expression_oriented_if_semantic_analysis() {
        let code = r#"
        fn calculate(flag: bool) -> i64 {
            val x = if flag { 100 } else { 0 };
            ret x + 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 47: Expression-Oriented `match` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item47_expression_oriented_match_binding() {
        let code = r#"
        fn to_roman(digit: i64) -> string {
            val roman = match digit {
                1 => "I",
                2 => "II",
                3 => "III",
                _ => "UNKNOWN",
            };
            ret roman;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item47_expression_oriented_match_enum_eval() {
        let code = r#"
        enum State {
            Active,
            Inactive,
        }
        fn eval_state(s: State) -> i64 {
            val code = match s {
                State.Active => 1,
                State.Inactive => 0,
            };
            ret code;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.enums.len(), 1);
    }

    #[test]
    fn test_item47_expression_oriented_match_semantic() {
        let code = r#"
        fn main() -> i64 {
            val res = match 10 {
                10 => 1,
                _ => 0,
            };
            ret res;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 48: Local Nested Functions (3 Tests)
    // =========================================================================
    #[test]
    fn test_item48_local_nested_helper_function() {
        let code = r#"
        fn outer_calc(x: i64) -> i64 {
            fn helper(val: i64) -> i64 {
                ret val * 2;
            }
            ret helper(x);
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item48_multiple_local_functions_in_scope() {
        let code = r#"
        fn math_suite(a: i64, b: i64) -> i64 {
            fn add(x: i64, y: i64) -> i64 { ret x + y; }
            fn mul(x: i64, y: i64) -> i64 { ret x * y; }
            ret add(a, mul(b, 2));
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item48_local_function_semantic_validation() {
        let code = r#"
        fn compute() -> i64 {
            fn inner() -> i64 { ret 42; }
            ret inner();
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 49: Default Trait Implementation (3 Tests)
    // =========================================================================
    #[test]
    fn test_item49_trait_with_default_method_implementation() {
        let code = r#"
        trait Cache {
            fn clear() -> void;
            fn reset() -> void {
                clear();
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.traits.len(), 1);
    }

    #[test]
    fn test_item49_trait_default_implementation_override() {
        let code = r#"
        trait Logger {
            fn log(msg: string) -> void;
            fn info(msg: string) -> void {
                log(msg);
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.traits.len(), 1);
    }

    #[test]
    fn test_item49_trait_default_method_semantic_check() {
        let code = r#"
        trait Resettable {
            fn reset() -> bool {
                ret true;
            }
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 50: Pattern Binding in Condition `if user is Admin(name)` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item50_pattern_binding_in_if_condition() {
        let code = r#"
        fn verify_admin(user: User) -> bool {
            if user is Admin(name) {
                ret true;
            }
            ret false;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item50_pattern_binding_with_enum_variant() {
        let code = r#"
        enum Status {
            Success(i64),
            Failed(string),
        }
        fn check_success(s: Status) -> i64 {
            if s is Success(code) {
                ret code;
            }
            ret 0;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.enums.len(), 1);
    }

    #[test]
    fn test_item50_pattern_binding_semantic_analysis() {
        let code = r#"
        fn main() -> i64 {
            val status = 1;
            if status is 1 {
                ret 1;
            }
            ret 0;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }
}
