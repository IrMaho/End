#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::codegen::llvm_backend::LlvmBackend;
    use inkwell::context::Context;
    use std::fs;
    use std::process::Command;

    #[test]
    fn test_valid_llvm_module_verification() {
        let mut module = Module::empty("test_arithmetic");
        let func = FunctionDef {
            name: "main".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![],
            return_type: Type::I32,
            directives: vec![],
            morphic_param: None,
            body: crate::ast::pattern::Block {
                statements: vec![
                    Statement::VarDecl {
                        name: "a".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Lit(Literal::Int(40), Span::default())),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::VarDecl {
                        name: "b".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Lit(Literal::Int(2), Span::default())),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::VarDecl {
                        name: "sum".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Binary {
                            left: Box::new(Expression::Ident("a".to_string(), Span::default())),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Ident("b".to_string(), Span::default())),
                            span: Span::default(),
                        }),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::Return {
                        value: Some(Expression::Lit(Literal::Int(0), Span::default())),
                        span: Span::default(),
                    },
                ],
                span: Span::default(),
            },
            span: Span::default(),
        };
        module.functions.push(func);

        let backend = LlvmBackend::new(None);
        let ir_res = backend.generate_llvm_ir(&module);
        assert!(ir_res.is_ok(), "Valid module must generate LLVM IR: {:?}", ir_res.err());
        let ir = ir_res.unwrap();
        assert!(ir.contains("define i32 @main()"), "Generated IR must contain main: {}", ir);
        assert!(ir.contains("add i64"), "Generated IR must contain add instruction: {}", ir);
    }

    #[test]
    fn test_invalid_module_rejection_by_verification() {
        let context = Context::create();
        let module = context.create_module("deliberately_broken_module");
        let i32_type = context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let func = module.add_function("broken_fn", fn_type, None);

        // Create a basic block without terminator and leave it open
        let _bb = context.append_basic_block(func, "unterminated_entry");
        // Do NOT add any terminator (no ret, no br)

        // module.verify() MUST detect the unterminated basic block and fail
        let verify_res = module.verify();
        assert!(
            verify_res.is_err(),
            "LLVM module.verify() MUST reject invalid unterminated function module!"
        );

        let err_msg = verify_res.unwrap_err().to_string();
        assert!(
            !err_msg.is_empty(),
            "Verification error message must be captured: {}",
            err_msg
        );
    }

    #[test]
    fn test_llvm_compile_and_execute_arithmetic() {
        let mut module = Module::empty("test_exec_arithmetic");
        let func = FunctionDef {
            name: "main".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![],
            return_type: Type::I32,
            directives: vec![],
            morphic_param: None,
            body: crate::ast::pattern::Block {
                statements: vec![
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::Lit(Literal::Int(42), Span::default())],
                        span: Span::default(),
                    }),
                    Statement::Return {
                        value: Some(Expression::Lit(Literal::Int(0), Span::default())),
                        span: Span::default(),
                    },
                ],
                span: Span::default(),
            },
            span: Span::default(),
        };
        module.functions.push(func);

        let backend = LlvmBackend::new(None);
        let temp_dir = std::env::temp_dir().join("endc_llvm_test");
        let _ = fs::create_dir_all(&temp_dir);
        let out_exe = temp_dir.join("test_exec.exe");

        let artifacts_res = backend.compile_to_executable(&module, &out_exe);
        assert!(artifacts_res.is_ok(), "compile_to_executable must succeed: {:?}", artifacts_res.err());
        let artifacts = artifacts_res.unwrap();
        assert!(artifacts.executable_path.exists(), "Executable file must exist");
        assert!(!artifacts.ir_sha256.is_empty(), "IR sha256 must be computed");
        assert!(!artifacts.executable_sha256.is_empty(), "Executable sha256 must be computed");

        let run_res = Command::new(&artifacts.executable_path).output();
        assert!(run_res.is_ok(), "Executable must run");
        let out = run_res.unwrap();
        assert!(out.status.success(), "Execution must return exit code 0");
        let stdout_str = String::from_utf8_lossy(&out.stdout);
        assert_eq!(stdout_str.trim(), "42", "Stdout must match expected output 42");

        // Clean up
        let _ = fs::remove_file(&out_exe);
        let _ = fs::remove_file(out_exe.with_extension("ll"));
        let _ = fs::remove_file(out_exe.with_extension("obj"));
    }
}
