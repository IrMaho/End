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
    // ITEM 01: Multi-value Return & Unpacking Assignment (3 Tests)
    // =========================================================================
    #[test]
    fn test_item01_multi_return_simple_tuple_unpack() {
        let code = r#"
        fn user_credentials() -> (i64, string, string) {
            ret (101, "Maho", "admin@end.org");
        }
        fn main() -> i64 {
            (id, name, email) := user_credentials();
            ret id;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item01_multi_return_type_preservation_and_semantic() {
        let code = r#"
        fn compute_stats() -> (i64, i64, i64) {
            ret (10, 20, 30);
        }
        fn main() -> i64 {
            (min, avg, max) := compute_stats();
            ret max;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    #[test]
    fn test_item01_multi_return_complex_nested_unpacking() {
        let code = r#"
        fn get_pair() -> (i64, i64) {
            ret (42, 84);
        }
        fn main() -> i64 {
            (first, second) := get_pair();
            (x, y) := (first, second);
            ret x + y;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[1].body.statements.len(), 3);
    }

    // =========================================================================
    // ITEM 02: Rest Unpacking with `*` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item02_rest_unpacking_head_and_tail() {
        let code = r#"
        fn main() -> i64 {
            (head, *rest) := (1, 2, 3, 4, 5);
            ret head;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item02_rest_unpacking_middle_wildcard() {
        let code = r#"
        fn main() -> i64 {
            (first, *middle, last) := (10, 20, 30, 40, 50);
            ret first + last;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item02_rest_unpacking_typed_vector() {
        let code = r#"
        fn main() -> i64 {
            (first_item, *remaining) := (100, 200, 300);
            ret first_item;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 03: `_` Wildcard Ignore in Destructuring (3 Tests)
    // =========================================================================
    #[test]
    fn test_item03_wildcard_single_ignore() {
        let code = r#"
        fn get_data() -> (i64, string, i64) {
            ret (1, "Test", 100);
        }
        fn main() -> i64 {
            (_, name, _) := get_data();
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 2);
    }

    #[test]
    fn test_item03_wildcard_multiple_positions() {
        let code = r#"
        fn main() -> i64 {
            (a, _, c, _) := (10, 20, 30, 40);
            ret a + c;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item03_wildcard_in_expression_context() {
        let code = r#"
        fn main() -> i64 {
            (_, target_val) := (0, 999);
            ret target_val;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 04: Named Destructuring (3 Tests)
    // =========================================================================
    #[test]
    fn test_item04_named_struct_destructuring_basic() {
        let code = r#"
        st UserProfile {
            id: i64,
            email: string,
        }
        fn main() -> i64 {
            UserProfile { id, email } := UserProfile { id: 42, email: "dev@end.org" };
            ret id;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.structs.len(), 1);
    }

    #[test]
    fn test_item04_named_destructuring_with_field_shorthand() {
        let code = r#"
        st Order {
            order_id: i64,
            amount: i64,
        }
        fn main() -> i64 {
            Order { order_id, amount } := Order { order_id: 1001, amount: 500 };
            ret order_id;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item04_named_destructuring_nested_records() {
        let code = r#"
        st Point {
            x: i64,
            y: i64,
        }
        fn main() -> i64 {
            Point { x, y } := Point { x: 12, y: 24 };
            ret x + y;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 05: Pattern Matching with `match` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item05_pattern_matching_primitive_literals() {
        let code = r#"
        fn eval_status(code: i64) -> i64 {
            ret match code {
                200 => 1,
                404 => 0,
                _ => -1,
            };
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item05_pattern_matching_enum_variants_with_payload() {
        let code = r#"
        enum Result {
            Ok(i64),
            Err(i64),
        }
        fn unwrap_or(res: Result) -> i64 {
            ret match res {
                Result.Ok(val) => val,
                Result.Err(code) => 0,
            };
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.enums.len(), 1);
    }

    #[test]
    fn test_item05_pattern_matching_nested_block_arms() {
        let code = r#"
        fn process(val: i64) -> i64 {
            ret match val {
                1 => {
                    val doubled = val * 2;
                    doubled
                },
                _ => 0,
            };
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    // =========================================================================
    // ITEM 06: Guard in Pattern (3 Tests)
    // =========================================================================
    #[test]
    fn test_item06_pattern_guard_with_if_condition() {
        let code = r#"
        fn check_age(age: i64) -> bool {
            ret match age {
                val if val >= 18 => true,
                _ => false,
            };
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item06_pattern_guard_complex_boolean_predicate() {
        let code = r#"
        fn access_gate(level: i64, is_admin: bool) -> i64 {
            ret match level {
                lvl if lvl > 5 && is_admin => 1,
                lvl if lvl > 0 => 2,
                _ => 0,
            };
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    #[test]
    fn test_item06_pattern_guard_enum_payload_filter() {
        let code = r#"
        enum Packet {
            Data(i64),
            Control(i64),
        }
        fn filter_packet(p: Packet) -> i64 {
            ret match p {
                Packet.Data(size) if size > 1024 => 1,
                Packet.Data(size) => 0,
                Packet.Control(cmd) => 2,
            };
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.enums.len(), 1);
    }

    // =========================================================================
    // ITEM 07: List Comprehension (3 Tests)
    // =========================================================================
    #[test]
    fn test_item07_list_comprehension_basic_map() {
        let code = r#"
        fn main() -> i64 {
            val doubled = [x * 2 for x in xs];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item07_list_comprehension_with_filter_predicate() {
        let code = r#"
        fn filter_positives() -> i64 {
            val positives = [x for x in numbers if x > 0];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item07_list_comprehension_nested_computation() {
        let code = r#"
        fn compute_squares() -> i64 {
            val squares = [x * x for x in items if x % 2 == 0];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 08: Dictionary Comprehension (3 Tests)
    // =========================================================================
    #[test]
    fn test_item08_dict_comprehension_key_value_mapping() {
        let code = r#"
        fn main() -> i64 {
            val user_map = {k: v for k, v in users};
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item08_dict_comprehension_filtered_entries() {
        let code = r#"
        fn active_users_map() -> i64 {
            val active = {k: v for k, v in user_dict if v.active};
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item08_dict_comprehension_computed_keys_and_values() {
        let code = r#"
        fn index_by_id() -> i64 {
            val index = {item.id: item.name for item in items if item.valid};
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }
}
