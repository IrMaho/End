use crate::ast::*;
use crate::semantic::graph::*;
use std::collections::HashMap;

pub struct SemanticAnalyzer {
    pub graph: SemanticGraph,
    pub source_lines: Vec<String>,
    pub errors: Vec<DiagnosticError>,
    pub enums: HashMap<String, EnumDef>,
    current_function: Option<String>,
    var_scopes: Vec<HashMap<String, (Type, usize, bool)>>, // name -> (Type, line_def, is_mut)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DiagnosticError {
    pub code: String,
    pub message: String,
    pub line: usize,
    pub col: usize,
    pub kind: String,
    pub repair_suggestion: Option<String>,
}

impl SemanticAnalyzer {
    pub fn new(filename: &str, source: &str) -> Self {
        Self {
            graph: SemanticGraph::new(filename),
            source_lines: source.lines().map(|s| s.to_string()).collect(),
            errors: Vec::new(),
            enums: HashMap::new(),
            current_function: None,
            var_scopes: vec![HashMap::new()],
        }
    }

    fn push_scope(&mut self) {
        self.var_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.var_scopes.pop();
    }

    fn declare_var(&mut self, name: &str, ty: Type, line: usize, is_mut: bool) {
        if let Some(scope) = self.var_scopes.last_mut() {
            scope.insert(name.to_string(), (ty, line, is_mut));
        }
    }

    fn lookup_var(&self, name: &str) -> Option<(Type, usize, bool)> {
        for scope in self.var_scopes.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Some(val.clone());
            }
        }
        None
    }

    pub fn analyze_module(&mut self, module: &Module) -> Result<(), Vec<DiagnosticError>> {
        // Register Enums
        for e in &module.enums {
            self.enums.insert(e.name.clone(), e.clone());
            let variant_names = e.variants.iter().map(|v| v.name.clone()).collect::<Vec<_>>();
            let info = SymbolInfo {
                name: e.name.clone(),
                kind: "enum".to_string(),
                type_signature: format!("enum {} {{ {} }}", e.name, variant_names.join(", ")),
                file: e.span.file.clone(),
                defined_at_line: e.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: true,
                memory_region: None,
            };
            self.graph.symbols.insert(e.name.clone(), info);
        }

        // Register Structs
        for s in &module.structs {
            let info = SymbolInfo {
                name: s.name.clone(),
                kind: "struct".to_string(),
                type_signature: format!("struct {}", s.name),
                file: s.span.file.clone(),
                defined_at_line: s.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: true,
                memory_region: None,
            };
            self.graph.symbols.insert(s.name.clone(), info);
        }

        // Register Functions
        for f in &module.functions {
            let mut effect_list = Vec::new();
            for dir in &f.directives {
                if dir.name == "@alloc_bound" {
                    effect_list.push(format!("alloc_bound({})", dir.args.join(", ")));
                }
            }

            let info = SymbolInfo {
                name: f.name.clone(),
                kind: "function".to_string(),
                type_signature: format!(
                    "fn {}({}) -> {}",
                    f.name,
                    f.params
                        .iter()
                        .map(|p| format!("{}: {}", p.name, p.param_type))
                        .collect::<Vec<_>>()
                        .join(", "),
                    f.return_type
                ),
                file: f.span.file.clone(),
                defined_at_line: f.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: effect_list,
                is_pure: false,
                memory_region: None,
            };
            self.graph.symbols.insert(f.name.clone(), info);
        }

        // Analyze Function Bodies
        for f in &module.functions {
            self.analyze_function(f);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn analyze_function(&mut self, func: &FunctionDef) {
        self.current_function = Some(func.name.clone());
        self.push_scope();

        // Register parameters
        for p in &func.params {
            self.declare_var(&p.name, p.param_type.clone(), p.span.line, p.is_mut);
        }

        self.analyze_block(&func.body);

        self.pop_scope();
        self.current_function = None;
    }

    fn analyze_block(&mut self, block: &Block) {
        self.push_scope();
        for stmt in &block.statements {
            self.analyze_statement(stmt);
        }
        self.pop_scope();
    }

    fn analyze_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl {
                name,
                var_type,
                is_mut,
                initializer,
                span,
            } => {
                let inferred_ty = if let Some(init) = initializer {
                    self.analyze_expression(init)
                } else {
                    var_type.clone().unwrap_or(Type::Void)
                };

                let ty = var_type.clone().unwrap_or(inferred_ty);
                self.declare_var(name, ty.clone(), span.line, *is_mut);

                let raw_code = if span.line <= self.source_lines.len() && span.line > 0 {
                    self.source_lines[span.line - 1].trim().to_string()
                } else {
                    format!("{} {} = ...", if *is_mut { "mut" } else { "val" }, name)
                };

                let mut from_symbols = Vec::new();
                if let Some(init) = initializer {
                    self.extract_symbols_from_expr(init, &mut from_symbols);
                }

                let memory_allocated = if let Some(init) = initializer {
                    self.expr_allocates(init)
                } else {
                    false
                };

                let line_sem = LineSemantics {
                    line: span.line,
                    code: raw_code,
                    flow: DataFlow {
                        from: from_symbols,
                        to: vec![SymbolDestination {
                            symbol: name.clone(),
                            symbol_type: ty.to_string(),
                            lifetime: "scope_end".to_string(),
                            destination: self.current_function.clone().unwrap_or("module".into()),
                        }],
                    },
                    side_effects: SideEffects {
                        memory_allocated,
                        allocator_used: if memory_allocated { Some("alloc".into()) } else { None },
                        io_performed: false,
                        can_panic: false,
                        possible_errors: Vec::new(),
                        effects: if memory_allocated { vec!["alloc".into()] } else { Vec::new() },
                    },
                };

                self.graph.add_line(span.line, line_sem);
            }
            Statement::Return { value, span } => {
                let mut from_symbols = Vec::new();
                if let Some(val) = value {
                    self.analyze_expression(val);
                    self.extract_symbols_from_expr(val, &mut from_symbols);
                }

                let raw_code = if span.line <= self.source_lines.len() && span.line > 0 {
                    self.source_lines[span.line - 1].trim().to_string()
                } else {
                    "return".to_string()
                };

                let line_sem = LineSemantics {
                    line: span.line,
                    code: raw_code,
                    flow: DataFlow {
                        from: from_symbols,
                        to: vec![SymbolDestination {
                            symbol: "return_value".to_string(),
                            symbol_type: "inferred".to_string(),
                            lifetime: "caller".to_string(),
                            destination: "caller".to_string(),
                        }],
                    },
                    side_effects: SideEffects {
                        memory_allocated: false,
                        allocator_used: None,
                        io_performed: false,
                        can_panic: false,
                        possible_errors: Vec::new(),
                        effects: Vec::new(),
                    },
                };
                self.graph.add_line(span.line, line_sem);
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr);
                let span = expr.span();
                let raw_code = if span.line <= self.source_lines.len() && span.line > 0 {
                    self.source_lines[span.line - 1].trim().to_string()
                } else {
                    "expression".to_string()
                };

                let mut from_symbols = Vec::new();
                self.extract_symbols_from_expr(expr, &mut from_symbols);

                let is_io = raw_code.contains("print") || raw_code.contains("log") || raw_code.contains("http");
                let line_sem = LineSemantics {
                    line: span.line,
                    code: raw_code,
                    flow: DataFlow {
                        from: from_symbols,
                        to: Vec::new(),
                    },
                    side_effects: SideEffects {
                        memory_allocated: self.expr_allocates(expr),
                        allocator_used: None,
                        io_performed: is_io,
                        can_panic: false,
                        possible_errors: Vec::new(),
                        effects: if is_io { vec!["io".into()] } else { Vec::new() },
                    },
                };
                self.graph.add_line(span.line, line_sem);
            }
            Statement::Assignment { target, value, span } => {
                self.analyze_expression(target);
                self.analyze_expression(value);

                let raw_code = if span.line <= self.source_lines.len() && span.line > 0 {
                    self.source_lines[span.line - 1].trim().to_string()
                } else {
                    "assignment".to_string()
                };

                let mut from_symbols = Vec::new();
                self.extract_symbols_from_expr(value, &mut from_symbols);

                let target_name = match target {
                    Expression::Ident(n, _) => n.clone(),
                    Expression::FieldAccess { field, .. } => field.clone(),
                    _ => "target".to_string(),
                };

                let line_sem = LineSemantics {
                    line: span.line,
                    code: raw_code,
                    flow: DataFlow {
                        from: from_symbols,
                        to: vec![SymbolDestination {
                            symbol: target_name,
                            symbol_type: "inferred".to_string(),
                            lifetime: "scope".to_string(),
                            destination: self.current_function.clone().unwrap_or("module".into()),
                        }],
                    },
                    side_effects: SideEffects {
                        memory_allocated: false,
                        allocator_used: None,
                        io_performed: false,
                        can_panic: false,
                        possible_errors: Vec::new(),
                        effects: Vec::new(),
                    },
                };
                self.graph.add_line(span.line, line_sem);
            }
            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                self.analyze_expression(condition);
                self.analyze_block(then_block);
                if let Some(eb) = else_block {
                    self.analyze_block(eb);
                }
            }
            Statement::While { condition, body, .. } => {
                self.analyze_expression(condition);
                self.analyze_block(body);
            }
            Statement::ForIn {
                item_name,
                iterable,
                body,
                span,
            } => {
                self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(item_name, Type::I32, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::Match { expr, arms, span } => {
                let match_type = self.analyze_expression(expr);
                let raw_code = if span.line <= self.source_lines.len() && span.line > 0 {
                    self.source_lines[span.line - 1].trim().to_string()
                } else {
                    "match ...".to_string()
                };

                let mut from_symbols = Vec::new();
                self.extract_symbols_from_expr(expr, &mut from_symbols);

                // Analyze each match arm
                for arm in arms {
                    self.push_scope();
                    if let Pattern::Variant { binding: Some(b), .. } = &arm.pattern {
                        self.declare_var(b, Type::Custom("VariantPayload".into()), arm.span.line, false);
                    } else if let Pattern::Ident(id) = &arm.pattern {
                        self.declare_var(id, match_type.clone(), arm.span.line, false);
                    }

                    if let Some(g) = &arm.guard {
                        self.analyze_expression(g);
                    }
                    self.analyze_block(&arm.body);
                    self.pop_scope();
                }

                let line_sem = LineSemantics {
                    line: span.line,
                    code: raw_code,
                    flow: DataFlow {
                        from: from_symbols,
                        to: Vec::new(),
                    },
                    side_effects: SideEffects {
                        memory_allocated: false,
                        allocator_used: None,
                        io_performed: false,
                        can_panic: false,
                        possible_errors: Vec::new(),
                        effects: vec!["match".into()],
                    },
                };
                self.graph.add_line(span.line, line_sem);
            }
            Statement::RegionBlock { name, body, span } => {
                self.push_scope();
                self.declare_var(
                    &format!("region_{}", name),
                    Type::Region(name.clone()),
                    span.line,
                    false,
                );
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::Defer { expr, .. } => {
                self.analyze_expression(expr);
            }
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Type {
        match expr {
            Expression::Lit(lit, _) => match lit {
                Literal::Int(_) => Type::I32,
                Literal::Float(_) => Type::F32,
                Literal::String(_) => Type::Str,
                Literal::Bool(_) => Type::Bool,
                Literal::Null => Type::Pointer(Box::new(Type::Void)),
            },
            Expression::Ident(name, _) => {
                if let Some((ty, _, _)) = self.lookup_var(name) {
                    ty
                } else if self.graph.symbols.contains_key(name) {
                    Type::Custom(name.clone())
                } else {
                    Type::Custom(name.clone())
                }
            }
            Expression::Binary { left, right, .. } => {
                self.analyze_expression(left);
                self.analyze_expression(right);
                Type::I32
            }
            Expression::Unary { expr, op, .. } => {
                let inner = self.analyze_expression(expr);
                match op {
                    UnaryOp::AddressOf => Type::Pointer(Box::new(inner)),
                    UnaryOp::Deref => match inner {
                        Type::Pointer(t) => *t,
                        _ => inner,
                    },
                    UnaryOp::Not => Type::Bool,
                    UnaryOp::Negate => inner,
                }
            }
            Expression::Call { callee, args, .. } => {
                for arg in args {
                    self.analyze_expression(arg);
                }
                if let Expression::Ident(name, _) = callee.as_ref() {
                    if let Some(curr_fn) = &self.current_function {
                        self.graph.add_call(curr_fn, name);
                    }
                }
                Type::Void
            }
            Expression::FieldAccess { object, .. } => {
                self.analyze_expression(object);
                Type::I32
            }
            Expression::StructInit { name, fields, .. } => {
                for (_, fval) in fields {
                    self.analyze_expression(fval);
                }
                Type::Custom(name.clone())
            }
            Expression::EnumInit { enum_name, variant_name, payload, .. } => {
                if let Some(p) = payload {
                    self.analyze_expression(p);
                }
                Type::Custom(enum_name.clone().unwrap_or_else(|| variant_name.clone()))
            }
            Expression::Alloc { target_type, .. } => Type::Pointer(Box::new(target_type.clone())),
            Expression::Catch { expr, .. } => self.analyze_expression(expr),
            Expression::Match { expr, arms, .. } => {
                let match_type = self.analyze_expression(expr);
                for arm in arms {
                    self.push_scope();
                    if let Pattern::Variant { binding: Some(b), .. } = &arm.pattern {
                        self.declare_var(b, Type::Custom("VariantPayload".into()), arm.span.line, false);
                    } else if let Pattern::Ident(id) = &arm.pattern {
                        self.declare_var(id, match_type.clone(), arm.span.line, false);
                    }
                    self.analyze_block(&arm.body);
                    self.pop_scope();
                }
                Type::Void
            }
            Expression::Block(b) => {
                self.analyze_block(b);
                Type::Void
            }
        }
    }

    fn extract_symbols_from_expr(&self, expr: &Expression, out: &mut Vec<SymbolSource>) {
        match expr {
            Expression::Ident(name, span) => {
                if let Some((ty, line_def, _)) = self.lookup_var(name) {
                    out.push(SymbolSource {
                        symbol: name.clone(),
                        symbol_type: ty.to_string(),
                        source: format!("{}:{} (param/var)", span.file, line_def),
                    });
                } else {
                    out.push(SymbolSource {
                        symbol: name.clone(),
                        symbol_type: "unknown".to_string(),
                        source: format!("{}:{}", span.file, span.line),
                    });
                }
            }
            Expression::Binary { left, right, .. } => {
                self.extract_symbols_from_expr(left, out);
                self.extract_symbols_from_expr(right, out);
            }
            Expression::Unary { expr, .. } => {
                self.extract_symbols_from_expr(expr, out);
            }
            Expression::Call { callee, args, .. } => {
                self.extract_symbols_from_expr(callee, out);
                for arg in args {
                    self.extract_symbols_from_expr(arg, out);
                }
            }
            Expression::FieldAccess { object, .. } => {
                self.extract_symbols_from_expr(object, out);
            }
            Expression::StructInit { fields, .. } => {
                for (_, fval) in fields {
                    self.extract_symbols_from_expr(fval, out);
                }
            }
            Expression::EnumInit { payload, .. } => {
                if let Some(p) = payload {
                    self.extract_symbols_from_expr(p, out);
                }
            }
            _ => {}
        }
    }

    fn expr_allocates(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Alloc { .. } => true,
            Expression::Call { callee, .. } => {
                if let Expression::Ident(name, _) = callee.as_ref() {
                    name.contains("alloc") || name.contains("init")
                } else if let Expression::FieldAccess { field, .. } = callee.as_ref() {
                    field.contains("alloc") || field.contains("init")
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
