#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::ir::hir::*;
    use crate::ir::lowering::*;
    use crate::ir::mir_lowering::*;
    use crate::semantic::analyzer::*;
    use crate::codegen::interpreter::{Interpreter, Value};

    fn parse_str(code: &str) -> Result<Module, String> {
        let mut lexer = Lexer::new("test.end", code);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new("test.end", tokens);
        parser.parse_module("test_mod")
    }

    #[test]
    fn test_hir_lowering_and_region_preservation() {
        let code = "pub fn foo() void {\nregion arena {\nval x = 100\nval y = 200\n}\n}";
        let module = parse_str(code).unwrap();
        let hir = AstLowering::lower_module(&module);
        assert_eq!(hir.functions.len(), 1);
        assert_eq!(hir.functions[0].body.len(), 1);

        if let HirStatement::RegionBlock { name, body, .. } = &hir.functions[0].body[0] {
            assert_eq!(name, "arena");
            assert_eq!(body.len(), 2); // Both statements preserved!
        } else {
            panic!("Expected HirStatement::RegionBlock");
        }
    }

    #[test]
    fn test_hir_to_mir_lowering_pipeline() {
        let code = "pub fn calc(a: i64) i64 {\nif a > 0 {\nret a * 2\n} else {\nret 0\n}\n}";
        let module = parse_str(code).unwrap();
        let hir = AstLowering::lower_module(&module);
        let mir = HirToMirLowering::lower_module(&hir);

        assert_eq!(mir.functions.len(), 1);
        assert!(mir.functions[0].blocks.len() >= 3); // entry, then, else, merge blocks
    }

    #[test]
    fn test_borrow_conflict_mutation_during_borrow() {
        let code = "pub fn test_borrow() void {\nval x = 42\nval r = &x\nx = 100\n}";
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0907"));
    }

    #[test]
    fn test_spawn_thread_ownership_transfer() {
        let code = "pub fn worker(data: i64) void {}\npub fn run_spawn() void {\nval res = 99\nspawn worker(res)\nval leak = res\n}";
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0906"));
    }

    #[test]
    fn test_transitive_purity_violation_error() {
        let code = "pub fn net_call() void { socket(1, 2, 3) }\n@pure\npub fn pure_func() void { net_call() }";
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_err());
        let errors = res.unwrap_err();
        assert!(errors.iter().any(|e| e.code == "E0904"));
    }

    #[test]
    fn test_bitwise_not_and_cast_soundness() {
        let code = "pub fn test_ops() void {\nval x: i64 = ~10\nval y: u64 = x as u64\n}";
        let module = parse_str(code).unwrap();
        let mut analyzer = SemanticAnalyzer::new("test.end", code);
        let res = analyzer.analyze_module(&module);
        assert!(res.is_ok());
    }

    #[test]
    fn test_fuzz_malformed_syntax_graceful_recovery() {
        let malformed_inputs = [
            "fn unclosed( {",
            "val string = \"unterminated string",
            "struct @#$ {",
            "fn foo() { val x = 10 / / 2 }",
            "import \"broken.end",
        ];

        for input in malformed_inputs {
            let res = parse_str(input);
            // Parser must return Err without any unhandled panic!
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_interpreter_constant_folding() {
        let code = "pub fn main() i64 {\nret 10 * 5 + 42 - 2\n}";
        let module = parse_str(code).unwrap();
        let mut interp = Interpreter::new();
        let res = interp.run(&module).unwrap();
        assert_eq!(res, Value::Int(90));
    }
}

