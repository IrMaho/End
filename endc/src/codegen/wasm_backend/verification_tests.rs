#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::codegen::c_backend::CBackend;
    use crate::codegen::wasm_backend::{WasmBackend, WasmValidator};
    use std::fs;
    use std::process::Command;

    // 1. Module Verification: Valid WAT and WASM generation
    #[test]
    fn test_valid_wat_and_wasm_generation() {
        let mut module = Module::empty("test_wasm_valid");
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
                        name: "x".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Lit(Literal::Int(40), Span::default())),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::VarDecl {
                        name: "y".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Lit(Literal::Int(2), Span::default())),
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

        let mut backend = WasmBackend::new(None);
        let wat_res = backend.generate_wat(&module);
        assert!(wat_res.is_ok(), "WAT generation must succeed: {:?}", wat_res.err());
        let wat = wat_res.unwrap();
        assert!(wat.contains("(func $main"), "WAT must define $main");

        let wasm_res = backend.compile_to_wasm(&module);
        assert!(wasm_res.is_ok(), "WASM compilation must succeed: {:?}", wasm_res.err());
        let wasm_bytes = wasm_res.unwrap();
        assert!(wasm_bytes.starts_with(b"\0asm"), "Binary must start with magic header");
    }

    // 2. Adversarial Gate: Malformed WAT rejection
    #[test]
    fn test_invalid_wat_rejection_by_validator() {
        let invalid_wat = "(module (func $broken (param i32) ;; unclosed";
        let res = WasmValidator::validate_wat(invalid_wat);
        assert!(res.is_err(), "Validator MUST reject unclosed parenthesis in WAT");
    }

    // 3. Adversarial Gate: Malformed WASM binary rejection
    #[test]
    fn test_invalid_wasm_rejection_by_validator() {
        let corrupt_bytes = vec![0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
        let res = WasmValidator::validate_wasm(&corrupt_bytes);
        assert!(res.is_err(), "Validator MUST reject invalid magic header in WASM binary");
    }

    // 4. Arithmetic Operations (add, sub, mul, div, mod)
    #[test]
    fn test_wasm_compile_and_execute_arithmetic() {
        let mut module = Module::empty("test_wasm_arith");
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
                        args: vec![Expression::Binary {
                            left: Box::new(Expression::Lit(Literal::Int(100), Span::default())),
                            op: BinaryOp::Sub,
                            right: Box::new(Expression::Lit(Literal::Int(58), Span::default())),
                            span: Span::default(),
                        }],
                        span: Span::default(),
                    }),
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::Binary {
                            left: Box::new(Expression::Lit(Literal::Int(6), Span::default())),
                            op: BinaryOp::Mul,
                            right: Box::new(Expression::Lit(Literal::Int(7), Span::default())),
                            span: Span::default(),
                        }],
                        span: Span::default(),
                    }),
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::Binary {
                            left: Box::new(Expression::Lit(Literal::Int(144), Span::default())),
                            op: BinaryOp::Div,
                            right: Box::new(Expression::Lit(Literal::Int(12), Span::default())),
                            span: Span::default(),
                        }],
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

        let mut backend = WasmBackend::new(None);
        let run_res = backend.compile_and_run(&module);
        assert!(run_res.is_ok(), "Execution failed: {:?}", run_res.err());
        let result = run_res.unwrap();
        assert!(result.executed);
        assert_eq!(result.exit_code, 0);
        let lines: Vec<&str> = result.stdout.trim().lines().map(|s| s.trim()).collect();
        assert_eq!(lines, vec!["42", "42", "12"]);
    }

    // 5. Control Flow (if/else, while, for in)
    #[test]
    fn test_wasm_compile_and_execute_control_flow() {
        let mut module = Module::empty("test_wasm_ctrl");
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
                        name: "count".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Lit(Literal::Int(15), Span::default())),
                        is_mut: true,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::If {
                        condition: Expression::Binary {
                            left: Box::new(Expression::Ident("count".to_string(), Span::default())),
                            op: BinaryOp::GreaterThan,
                            right: Box::new(Expression::Lit(Literal::Int(10), Span::default())),
                            span: Span::default(),
                        },
                        then_block: crate::ast::pattern::Block {
                            statements: vec![Statement::Expression(Expression::Call {
                                callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                                args: vec![Expression::Lit(Literal::String("greater".to_string()), Span::default())],
                                span: Span::default(),
                            })],
                            span: Span::default(),
                        },
                        else_block: Some(crate::ast::pattern::Block {
                            statements: vec![Statement::Expression(Expression::Call {
                                callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                                args: vec![Expression::Lit(Literal::String("lesser".to_string()), Span::default())],
                                span: Span::default(),
                            })],
                            span: Span::default(),
                        }),
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

        let mut backend = WasmBackend::new(None);
        let run_res = backend.compile_and_run(&module);
        assert!(run_res.is_ok(), "Execution failed: {:?}", run_res.err());
        let result = run_res.unwrap();
        assert_eq!(result.stdout.trim(), "greater");
    }

    // 6. Strings and String Printing
    #[test]
    fn test_wasm_compile_and_execute_strings() {
        let mut module = Module::empty("test_wasm_strings");
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
                        args: vec![Expression::Lit(Literal::String("Hello, WebAssembly!".to_string()), Span::default())],
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

        let mut backend = WasmBackend::new(None);
        let run_res = backend.compile_and_run(&module);
        assert!(run_res.is_ok(), "Execution failed: {:?}", run_res.err());
        let result = run_res.unwrap();
        assert_eq!(result.stdout.trim(), "Hello, WebAssembly!");
    }

    // 7. Functions and Factorial Recursion
    #[test]
    fn test_wasm_compile_and_execute_functions_and_recursion() {
        let mut module = Module::empty("test_wasm_recursion");
        let fact_func = FunctionDef {
            name: "factorial".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![FunctionParam { name: "n".to_string(), param_type: Type::I64, is_mut: false, span: Span::default() }],
            return_type: Type::I64,
            directives: vec![],
            morphic_param: None,
            body: crate::ast::pattern::Block {
                statements: vec![
                    Statement::If {
                        condition: Expression::Binary {
                            left: Box::new(Expression::Ident("n".to_string(), Span::default())),
                            op: BinaryOp::LessEqual,
                            right: Box::new(Expression::Lit(Literal::Int(1), Span::default())),
                            span: Span::default(),
                        },
                        then_block: crate::ast::pattern::Block {
                            statements: vec![Statement::Return {
                                value: Some(Expression::Lit(Literal::Int(1), Span::default())),
                                span: Span::default(),
                            }],
                            span: Span::default(),
                        },
                        else_block: None,
                        span: Span::default(),
                    },
                    Statement::Return {
                        value: Some(Expression::Binary {
                            left: Box::new(Expression::Ident("n".to_string(), Span::default())),
                            op: BinaryOp::Mul,
                            right: Box::new(Expression::Call {
                                callee: Box::new(Expression::Ident("factorial".to_string(), Span::default())),
                                args: vec![Expression::Binary {
                                    left: Box::new(Expression::Ident("n".to_string(), Span::default())),
                                    op: BinaryOp::Sub,
                                    right: Box::new(Expression::Lit(Literal::Int(1), Span::default())),
                                    span: Span::default(),
                                }],
                                span: Span::default(),
                            }),
                            span: Span::default(),
                        }),
                        span: Span::default(),
                    },
                ],
                span: Span::default(),
            },
            span: Span::default(),
        };
        module.functions.push(fact_func);

        let main_func = FunctionDef {
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
                        args: vec![Expression::Call {
                            callee: Box::new(Expression::Ident("factorial".to_string(), Span::default())),
                            args: vec![Expression::Lit(Literal::Int(6), Span::default())],
                            span: Span::default(),
                        }],
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
        module.functions.push(main_func);

        let mut backend = WasmBackend::new(None);
        let run_res = backend.compile_and_run(&module);
        assert!(run_res.is_ok(), "Execution failed: {:?}", run_res.err());
        let result = run_res.unwrap();
        assert_eq!(result.stdout.trim(), "720");
    }

    // 8. Fibonacci Recursion
    #[test]
    fn test_wasm_compile_and_execute_fibonacci_recursion() {
        let mut module = Module::empty("test_wasm_fib");
        let fib_func = FunctionDef {
            name: "fib".to_string(),
            generic_params: vec![],
            is_pub: true,
            params: vec![FunctionParam { name: "n".to_string(), param_type: Type::I64, is_mut: false, span: Span::default() }],
            return_type: Type::I64,
            directives: vec![],
            morphic_param: None,
            body: crate::ast::pattern::Block {
                statements: vec![
                    Statement::If {
                        condition: Expression::Binary {
                            left: Box::new(Expression::Ident("n".to_string(), Span::default())),
                            op: BinaryOp::LessEqual,
                            right: Box::new(Expression::Lit(Literal::Int(0), Span::default())),
                            span: Span::default(),
                        },
                        then_block: crate::ast::pattern::Block {
                            statements: vec![Statement::Return {
                                value: Some(Expression::Lit(Literal::Int(0), Span::default())),
                                span: Span::default(),
                            }],
                            span: Span::default(),
                        },
                        else_block: None,
                        span: Span::default(),
                    },
                    Statement::If {
                        condition: Expression::Binary {
                            left: Box::new(Expression::Ident("n".to_string(), Span::default())),
                            op: BinaryOp::Equal,
                            right: Box::new(Expression::Lit(Literal::Int(1), Span::default())),
                            span: Span::default(),
                        },
                        then_block: crate::ast::pattern::Block {
                            statements: vec![Statement::Return {
                                value: Some(Expression::Lit(Literal::Int(1), Span::default())),
                                span: Span::default(),
                            }],
                            span: Span::default(),
                        },
                        else_block: None,
                        span: Span::default(),
                    },
                    Statement::Return {
                        value: Some(Expression::Binary {
                            left: Box::new(Expression::Call {
                                callee: Box::new(Expression::Ident("fib".to_string(), Span::default())),
                                args: vec![Expression::Binary {
                                    left: Box::new(Expression::Ident("n".to_string(), Span::default())),
                                    op: BinaryOp::Sub,
                                    right: Box::new(Expression::Lit(Literal::Int(1), Span::default())),
                                    span: Span::default(),
                                }],
                                span: Span::default(),
                            }),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Call {
                                callee: Box::new(Expression::Ident("fib".to_string(), Span::default())),
                                args: vec![Expression::Binary {
                                    left: Box::new(Expression::Ident("n".to_string(), Span::default())),
                                    op: BinaryOp::Sub,
                                    right: Box::new(Expression::Lit(Literal::Int(2), Span::default())),
                                    span: Span::default(),
                                }],
                                span: Span::default(),
                            }),
                            span: Span::default(),
                        }),
                        span: Span::default(),
                    },
                ],
                span: Span::default(),
            },
            span: Span::default(),
        };
        module.functions.push(fib_func);

        let main_func = FunctionDef {
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
                        args: vec![Expression::Call {
                            callee: Box::new(Expression::Ident("fib".to_string(), Span::default())),
                            args: vec![Expression::Lit(Literal::Int(8), Span::default())],
                            span: Span::default(),
                        }],
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
        module.functions.push(main_func);

        let mut backend = WasmBackend::new(None);
        let run_res = backend.compile_and_run(&module);
        assert!(run_res.is_ok(), "Execution failed: {:?}", run_res.err());
        let result = run_res.unwrap();
        assert_eq!(result.stdout.trim(), "21");
    }

    // 9. Bitwise Operations & Shifts
    #[test]
    fn test_wasm_compile_and_execute_bitwise_and_shifts() {
        let mut module = Module::empty("test_wasm_bitwise");
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
                        args: vec![Expression::Binary {
                            left: Box::new(Expression::Lit(Literal::Int(12), Span::default())),
                            op: BinaryOp::BitAnd,
                            right: Box::new(Expression::Lit(Literal::Int(10), Span::default())),
                            span: Span::default(),
                        }],
                        span: Span::default(),
                    }),
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::Binary {
                            left: Box::new(Expression::Lit(Literal::Int(12), Span::default())),
                            op: BinaryOp::BitOr,
                            right: Box::new(Expression::Lit(Literal::Int(10), Span::default())),
                            span: Span::default(),
                        }],
                        span: Span::default(),
                    }),
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::Binary {
                            left: Box::new(Expression::Lit(Literal::Int(12), Span::default())),
                            op: BinaryOp::BitXor,
                            right: Box::new(Expression::Lit(Literal::Int(10), Span::default())),
                            span: Span::default(),
                        }],
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

        let mut backend = WasmBackend::new(None);
        let run_res = backend.compile_and_run(&module);
        assert!(run_res.is_ok(), "Execution failed: {:?}", run_res.err());
        let result = run_res.unwrap();
        let lines: Vec<&str> = result.stdout.trim().lines().map(|s| s.trim()).collect();
        assert_eq!(lines, vec!["8", "14", "6"]);
    }

    // 10. Floating Point Operations
    #[test]
    fn test_wasm_compile_and_execute_floats() {
        let mut module = Module::empty("test_wasm_floats");
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
                        args: vec![Expression::Binary {
                            left: Box::new(Expression::Lit(Literal::Float(1.5), Span::default())),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Lit(Literal::Float(2.5), Span::default())),
                            span: Span::default(),
                        }],
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

        let mut backend = WasmBackend::new(None);
        let run_res = backend.compile_and_run(&module);
        assert!(run_res.is_ok(), "Execution failed: {:?}", run_res.err());
        let result = run_res.unwrap();
        assert_eq!(result.stdout.trim(), "4");
    }

    // 11. Differential Equivalence: C Backend vs WebAssembly Backend
    #[test]
    fn test_wasm_differential_against_c_backend() {
        let mut module = Module::empty("test_wasm_differential");
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
                        name: "sum".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Lit(Literal::Int(0), Span::default())),
                        is_mut: true,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::VarDecl {
                        name: "i".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Lit(Literal::Int(1), Span::default())),
                        is_mut: true,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::While {
                        condition: Expression::Binary {
                            left: Box::new(Expression::Ident("i".to_string(), Span::default())),
                            op: BinaryOp::LessEqual,
                            right: Box::new(Expression::Lit(Literal::Int(10), Span::default())),
                            span: Span::default(),
                        },
                        body: crate::ast::pattern::Block {
                            statements: vec![
                                Statement::Assignment {
                                    target: Expression::Ident("sum".to_string(), Span::default()),
                                    value: Expression::Binary {
                                        left: Box::new(Expression::Ident("sum".to_string(), Span::default())),
                                        op: BinaryOp::Add,
                                        right: Box::new(Expression::Ident("i".to_string(), Span::default())),
                                        span: Span::default(),
                                    },
                                    span: Span::default(),
                                },
                                Statement::Assignment {
                                    target: Expression::Ident("i".to_string(), Span::default()),
                                    value: Expression::Binary {
                                        left: Box::new(Expression::Ident("i".to_string(), Span::default())),
                                        op: BinaryOp::Add,
                                        right: Box::new(Expression::Lit(Literal::Int(1), Span::default())),
                                        span: Span::default(),
                                    },
                                    span: Span::default(),
                                },
                            ],
                            span: Span::default(),
                        },
                        span: Span::default(),
                    },
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::Ident("sum".to_string(), Span::default())],
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

        // 1. Run WASM Backend
        let mut wasm_be = WasmBackend::new(None);
        let wasm_run = wasm_be.compile_and_run(&module).expect("WASM run failed");

        // 2. Run C Backend
        let mut c_backend = CBackend::new();
        let (c_code, _) = c_backend.generate_with_options(&module, false);
        let temp_dir = std::env::temp_dir().join("end_diff_test");
        let _ = fs::create_dir_all(&temp_dir);
        let c_src_path = temp_dir.join(format!("diff_{}.c", std::process::id()));
        let c_exe_path = temp_dir.join(format!("diff_{}.exe", std::process::id()));
        fs::write(&c_src_path, &c_code).expect("Write C source failed");

        let gcc_status = Command::new("gcc")
            .arg(&c_src_path)
            .arg("-o")
            .arg(&c_exe_path)
            .status()
            .expect("GCC invoke failed");
        assert!(gcc_status.success());
        let c_run = Command::new(&c_exe_path).output().expect("C run failed");

        // Cleanup
        let _ = fs::remove_file(&c_src_path);
        let _ = fs::remove_file(&c_exe_path);

        // 3. Differential assertions: exact stdout, stderr, exit code match
        assert_eq!(
            wasm_run.exit_code,
            c_run.status.code().unwrap_or(0),
            "Exit codes must match between C and WASM backends"
        );
        assert_eq!(
            wasm_run.stdout.trim(),
            String::from_utf8_lossy(&c_run.stdout).trim(),
            "Stdout must match exactly between C and WASM backends"
        );
        assert_eq!(wasm_run.stdout.trim(), "55");
    }
}
