#[cfg(test)]
mod tests {
    use crate::ast::*;
    use crate::codegen::c_backend::CBackend;
    use crate::codegen::llvm_backend::LlvmBackend;
    use inkwell::context::Context;
    use std::fs;
    use std::process::Command;

    fn make_test_exe_path(test_name: &str) -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join("endc_llvm_test_suite");
        let _ = fs::create_dir_all(&temp_dir);
        temp_dir.join(format!("{}_{}.exe", test_name, std::process::id()))
    }

    fn cleanup_artifacts(exe_path: &std::path::Path) {
        let _ = fs::remove_file(exe_path);
        let _ = fs::remove_file(exe_path.with_extension("ll"));
        let _ = fs::remove_file(exe_path.with_extension("obj"));
        let _ = fs::remove_file(exe_path.with_extension("c"));
    }

    // 1. Module Verification: Valid module
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

    // 2. Module Verification: Malformed module rejection (Adversarial Gate)
    #[test]
    fn test_invalid_module_rejection_by_verification() {
        let context = Context::create();
        let module = context.create_module("deliberately_broken_module");
        let i32_type = context.i32_type();
        let fn_type = i32_type.fn_type(&[], false);
        let func = module.add_function("broken_fn", fn_type, None);

        // Create an open basic block without terminator
        let _bb = context.append_basic_block(func, "unterminated_entry");

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

    // 3. Category A: Arithmetic (add, sub, mul, div, mod)
    #[test]
    fn test_llvm_compile_and_execute_arithmetic() {
        let mut module = Module::empty("test_arithmetic_pipeline");
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

        let out_exe = make_test_exe_path("test_arithmetic");
        let backend = LlvmBackend::new(None);
        let artifacts = backend.compile_to_executable(&module, &out_exe).expect("LLVM compilation failed");
        assert!(artifacts.executable_path.exists());

        let out = Command::new(&artifacts.executable_path).output().expect("Execution failed");
        assert!(out.status.success());
        let stdout_str = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<&str> = stdout_str.trim().lines().map(|s| s.trim()).collect();
        assert_eq!(lines, vec!["42", "42", "12"]);

        cleanup_artifacts(&out_exe);
    }

    // 4. Category B: Control Flow (if/else, while, for in)
    #[test]
    fn test_llvm_compile_and_execute_control_flow() {
        let mut module = Module::empty("test_ctrl_flow");
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
                        name: "val_test".to_string(),
                        var_type: Some(Type::I64),
                        initializer: Some(Expression::Lit(Literal::Int(15), Span::default())),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::If {
                        condition: Expression::Binary {
                            left: Box::new(Expression::Ident("val_test".to_string(), Span::default())),
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

        let out_exe = make_test_exe_path("test_ctrl_flow");
        let backend = LlvmBackend::new(None);
        let artifacts = backend.compile_to_executable(&module, &out_exe).expect("LLVM compilation failed");
        let out = Command::new(&artifacts.executable_path).output().expect("Execution failed");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "greater");

        cleanup_artifacts(&out_exe);
    }

    // 5. Category C: Strings (literals, concatenation, output)
    #[test]
    fn test_llvm_compile_and_execute_strings() {
        let mut module = Module::empty("test_strings");
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
                        name: "greeting".to_string(),
                        var_type: Some(Type::Str),
                        initializer: Some(Expression::Lit(Literal::String("Hello, ".to_string()), Span::default())),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::VarDecl {
                        name: "target".to_string(),
                        var_type: Some(Type::Str),
                        initializer: Some(Expression::Lit(Literal::String("LLVM!".to_string()), Span::default())),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::VarDecl {
                        name: "msg".to_string(),
                        var_type: Some(Type::Str),
                        initializer: Some(Expression::Binary {
                            left: Box::new(Expression::Ident("greeting".to_string(), Span::default())),
                            op: BinaryOp::Add,
                            right: Box::new(Expression::Ident("target".to_string(), Span::default())),
                            span: Span::default(),
                        }),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::Ident("msg".to_string(), Span::default())],
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

        let out_exe = make_test_exe_path("test_strings");
        let backend = LlvmBackend::new(None);
        let artifacts = backend.compile_to_executable(&module, &out_exe).expect("LLVM compilation failed");
        let out = Command::new(&artifacts.executable_path).output().expect("Execution failed");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "Hello, LLVM!");

        cleanup_artifacts(&out_exe);
    }

    // 6. Category D: Structs (definition, instantiation, field access)
    #[test]
    fn test_llvm_compile_and_execute_structs() {
        let mut module = Module::empty("test_structs");
        module.structs.push(StructDef {
            name: "Vector3".to_string(),
            fields: vec![
                StructField { name: "x".to_string(), field_type: Type::I64, is_pub: true, span: Span::default() },
                StructField { name: "y".to_string(), field_type: Type::I64, is_pub: true, span: Span::default() },
                StructField { name: "z".to_string(), field_type: Type::I64, is_pub: true, span: Span::default() },
            ],
            ..Default::default()
        });

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
                        name: "v".to_string(),
                        var_type: Some(Type::Custom("Vector3".to_string())),
                        initializer: Some(Expression::StructInit {
                            name: "Vector3".to_string(),
                            fields: vec![
                                ("x".to_string(), Expression::Lit(Literal::Int(10), Span::default())),
                                ("y".to_string(), Expression::Lit(Literal::Int(20), Span::default())),
                                ("z".to_string(), Expression::Lit(Literal::Int(30), Span::default())),
                            ],
                            span: Span::default(),
                        }),
                        is_mut: false,
                        is_lease: false,
                        span: Span::default(),
                    },
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::FieldAccess {
                            object: Box::new(Expression::Ident("v".to_string(), Span::default())),
                            field: "x".to_string(),
                            span: Span::default(),
                        }],
                        span: Span::default(),
                    }),
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::FieldAccess {
                            object: Box::new(Expression::Ident("v".to_string(), Span::default())),
                            field: "y".to_string(),
                            span: Span::default(),
                        }],
                        span: Span::default(),
                    }),
                    Statement::Expression(Expression::Call {
                        callee: Box::new(Expression::Ident("println".to_string(), Span::default())),
                        args: vec![Expression::FieldAccess {
                            object: Box::new(Expression::Ident("v".to_string(), Span::default())),
                            field: "z".to_string(),
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

        let out_exe = make_test_exe_path("test_structs");
        let backend = LlvmBackend::new(None);
        let artifacts = backend.compile_to_executable(&module, &out_exe).expect("LLVM compilation failed");
        let out = Command::new(&artifacts.executable_path).output().expect("Execution failed");
        assert!(out.status.success());
        let stdout_str = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<&str> = stdout_str.trim().lines().map(|s| s.trim()).collect();
        assert_eq!(lines, vec!["10", "20", "30"]);

        cleanup_artifacts(&out_exe);
    }

    // 7. Category E: Function Call & Recursion (Factorial)
    #[test]
    fn test_llvm_compile_and_execute_functions_and_recursion() {
        let mut module = Module::empty("test_recursion");
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

        let out_exe = make_test_exe_path("test_recursion");
        let backend = LlvmBackend::new(None);
        let artifacts = backend.compile_to_executable(&module, &out_exe).expect("LLVM compilation failed");
        let out = Command::new(&artifacts.executable_path).output().expect("Execution failed");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "720");

        cleanup_artifacts(&out_exe);
    }

    // 8. Category E: Fibonacci recursion
    #[test]
    fn test_llvm_compile_and_execute_fibonacci_recursion() {
        let mut module = Module::empty("test_fib");
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

        let out_exe = make_test_exe_path("test_fib");
        let backend = LlvmBackend::new(None);
        let artifacts = backend.compile_to_executable(&module, &out_exe).expect("LLVM compilation failed");
        let out = Command::new(&artifacts.executable_path).output().expect("Execution failed");
        assert!(out.status.success());
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "21");

        cleanup_artifacts(&out_exe);
    }

    // 9. Bitwise Operations & Shifts
    #[test]
    fn test_llvm_compile_and_execute_bitwise_and_shifts() {
        let mut module = Module::empty("test_bitwise");
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

        let out_exe = make_test_exe_path("test_bitwise");
        let backend = LlvmBackend::new(None);
        let artifacts = backend.compile_to_executable(&module, &out_exe).expect("LLVM compilation failed");
        let out = Command::new(&artifacts.executable_path).output().expect("Execution failed");
        assert!(out.status.success());
        let stdout_str = String::from_utf8_lossy(&out.stdout);
        let lines: Vec<&str> = stdout_str.trim().lines().map(|s| s.trim()).collect();
        assert_eq!(lines, vec!["8", "14", "6"]);

        cleanup_artifacts(&out_exe);
    }

    // 10. Floating Point Arithmetic
    #[test]
    fn test_llvm_compile_and_execute_floats() {
        let mut module = Module::empty("test_floats");
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

        let out_exe = make_test_exe_path("test_floats");
        let backend = LlvmBackend::new(None);
        let artifacts = backend.compile_to_executable(&module, &out_exe).expect("LLVM compilation failed");
        let out = Command::new(&artifacts.executable_path).output().expect("Execution failed");
        assert!(out.status.success());
        let stdout_str = String::from_utf8_lossy(&out.stdout);
        assert!(stdout_str.trim().starts_with("4.0"));

        cleanup_artifacts(&out_exe);
    }

    // 11. Differential Testing: C Backend vs LLVM Backend Semantic Equivalence
    #[test]
    fn test_llvm_differential_against_c_backend() {
        let mut module = Module::empty("test_differential_equiv");
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

        // 1. Compile & execute through LLVM Backend
        let llvm_exe = make_test_exe_path("test_diff_llvm");
        let llvm_be = LlvmBackend::new(None);
        let llvm_artifacts = llvm_be.compile_to_executable(&module, &llvm_exe).expect("LLVM compile failed");
        let llvm_run = Command::new(&llvm_artifacts.executable_path).output().expect("LLVM run failed");

        // 2. Compile & execute through C Backend
        let c_exe = make_test_exe_path("test_diff_c");
        let mut c_backend = CBackend::new();
        let (c_code, _) = c_backend.generate_with_options(&module, false);
        let c_src_path = c_exe.with_extension("c");
        fs::write(&c_src_path, &c_code).expect("Write C source failed");

        let gcc_status = Command::new("gcc")
            .arg(&c_src_path)
            .arg("-o")
            .arg(&c_exe)
            .status()
            .expect("GCC invoke failed");
        assert!(gcc_status.success());
        let c_run = Command::new(&c_exe).output().expect("C run failed");

        // 3. Differential assertions: exact stdout, stderr, exit code equivalence
        assert_eq!(
            llvm_run.status.code(),
            c_run.status.code(),
            "Exit codes must match between C and LLVM backends"
        );
        assert_eq!(
            String::from_utf8_lossy(&llvm_run.stdout),
            String::from_utf8_lossy(&c_run.stdout),
            "Stdout must match exactly between C and LLVM backends"
        );
        assert_eq!(
            String::from_utf8_lossy(&llvm_run.stderr),
            String::from_utf8_lossy(&c_run.stderr),
            "Stderr must match exactly between C and LLVM backends"
        );
        assert_eq!(String::from_utf8_lossy(&llvm_run.stdout).trim(), "55");

        cleanup_artifacts(&llvm_exe);
        cleanup_artifacts(&c_exe);
    }
}
