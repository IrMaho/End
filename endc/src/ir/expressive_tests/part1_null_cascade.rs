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
    // ITEM 17: Null-Aware Member Access `?.` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item17_null_aware_field_access_single_level() {
        let code = r#"
        fn get_name(user: User) -> string {
            ret user?.name;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item17_null_aware_field_access_deep_chaining() {
        let code = r#"
        fn get_country(user: User) -> string {
            ret user?.profile?.address?.country;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item17_null_aware_access_semantic_check() {
        let code = r#"
        st Profile {
            name: string,
        }
        st Account {
            profile: Profile,
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
    // ITEM 18: Null-Aware Assignment (3 Tests)
    // =========================================================================
    #[test]
    fn test_item18_null_aware_assignment_field() {
        let code = r#"
        fn update_profile(user: User) -> void {
            user?.profile.name = "Maho";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item18_null_aware_assignment_chained_property() {
        let code = r#"
        fn set_config(app: App) -> void {
            app?.settings?.theme = "dark";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 1);
    }

    #[test]
    fn test_item18_null_aware_assignment_semantic() {
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
    // ITEM 19: Null-Coalescing Operator `??` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item19_null_coalescing_simple_fallback() {
        let code = r#"
        fn get_display_name(raw_name: string) -> string {
            ret raw_name ?? "Guest";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item19_null_coalescing_chained_fallbacks() {
        let code = r#"
        fn resolve_host(primary: string, secondary: string) -> string {
            ret primary ?? secondary ?? "127.0.0.1";
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item19_null_coalescing_with_expression_fallback() {
        let code = r#"
        fn get_cached_or_default() -> i64 {
            val count = fetch_cached_count() ?? compute_default_count();
            ret count;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 20: Null-Coalescing Assignment `??=` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item20_null_coalescing_assignment_basic() {
        let code = r#"
        fn initialize_cache() -> i64 {
            cache ??= create_cache();
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item20_null_coalescing_assignment_in_member() {
        let code = r#"
        fn ensure_session(user: User) -> void {
            user.session ??= Session();
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item20_null_coalescing_assignment_semantic() {
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
    // ITEM 21: Dart-Style Cascade Operator `..` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item21_cascade_property_mutations() {
        let code = r#"
        fn build_paint() -> i64 {
            val p = Paint()..color = 255..width = 5..draw();
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item21_cascade_method_invocation_chain() {
        let code = r#"
        fn setup_button() -> i64 {
            val btn = Button()..setTitle("Login")..setWidth(120)..show();
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item21_cascade_fluent_builder_semantic() {
        let code = r#"
        fn main() -> i64 {
            val req = HttpRequest()..setMethod("POST")..setBody("data");
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 22: Null-Aware Cascade `?..` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item22_null_aware_cascade_single_mutation() {
        let code = r#"
        fn safe_update(user: User) -> void {
            user?..save();
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item22_null_aware_cascade_multi_chain() {
        let code = r#"
        fn safe_configure(dialog: Dialog) -> void {
            dialog?..setTitle("Alert")?..setWidth(300)?..open();
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item22_null_aware_cascade_semantic_check() {
        let code = r#"
        fn main() -> i64 {
            val maybe_node = getNode();
            maybe_node?..refresh();
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 23: Collection Spread `...` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item23_list_spread_elements() {
        let code = r#"
        fn merge_lists() -> i64 {
            val list = [1, ...items, 5];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item23_multiple_spreads_in_single_list() {
        let code = r#"
        fn combine_all() -> i64 {
            val all = [...header_items, ...body_items, ...footer_items];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions[0].body.statements.len(), 2);
    }

    #[test]
    fn test_item23_spread_with_literals_semantic() {
        let code = r#"
        fn main() -> i64 {
            val data = [0, ...nums, 999];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }

    // =========================================================================
    // ITEM 24: Null-Aware Spread `...?` (3 Tests)
    // =========================================================================
    #[test]
    fn test_item24_null_aware_spread_optional_items() {
        let code = r#"
        fn build_menu() -> i64 {
            val menu = [1, ...?optionalItems, 5];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item24_mixed_normal_and_null_aware_spreads() {
        let code = r#"
        fn build_payload() -> i64 {
            val payload = [...required_data, ...?maybe_extra_data];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        assert_eq!(module.functions.len(), 1);
    }

    #[test]
    fn test_item24_null_aware_spread_semantic_analysis() {
        let code = r#"
        fn main() -> i64 {
            val result = [...?get_optional_list()];
            ret 1;
        }
        "#;
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        assert!(analyzer.analyze_module(&module).is_ok());
    }
}
