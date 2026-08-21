use crate::ast::*;
use crate::semantic::graph::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum OwnershipState {
    Uninitialized,
    Owned,
    Moved { to: String, at_line: usize },
    BorrowedShared(usize), // count
    BorrowedMut(usize),    // line where &mut was taken
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoanKind {
    Shared,
    Mutable,
}

#[derive(Debug, Clone)]
pub struct ActiveLoan {
    pub place: String,
    pub kind: LoanKind,
    pub borrowed_at: usize,
    pub holder: String,
}

pub struct SemanticAnalyzer {
    pub graph: SemanticGraph,
    pub source_lines: Vec<String>,
    pub errors: Vec<DiagnosticError>,
    pub enums: HashMap<String, EnumDef>,
    pub structs: HashMap<String, StructDef>,
    pub function_signatures: HashMap<String, (Vec<Type>, Type, bool)>, // params, ret, is_pure
    pub function_effects: HashMap<String, HashSet<String>>,
    pub strict_leaks: bool,
    current_function: Option<String>,
    region_depth: usize,
    region_allocations: Vec<HashSet<String>>, // track pointers allocated inside each region depth
    var_scopes: Vec<HashMap<String, (Type, usize, bool)>>, // name -> (Type, line_def, is_mut)
    ownership_scopes: Vec<HashMap<String, OwnershipState>>,
    active_loans: Vec<ActiveLoan>,
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
            structs: HashMap::new(),
            function_signatures: HashMap::new(),
            function_effects: HashMap::new(),
            strict_leaks: false,
            current_function: None,
            region_depth: 0,
            region_allocations: vec![HashSet::new()],
            var_scopes: vec![HashMap::new()],
            ownership_scopes: vec![HashMap::new()],
            active_loans: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.var_scopes.push(HashMap::new());
        self.ownership_scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.var_scopes.pop();
        self.ownership_scopes.pop();
    }

    fn declare_var(&mut self, name: &str, ty: Type, line: usize, is_mut: bool) {
        if let Some(scope) = self.var_scopes.last_mut() {
            scope.insert(name.to_string(), (ty, line, is_mut));
        }
        if let Some(o_scope) = self.ownership_scopes.last_mut() {
            o_scope.insert(name.to_string(), OwnershipState::Owned);
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

    fn get_ownership_state(&self, name: &str) -> Option<OwnershipState> {
        for scope in self.ownership_scopes.iter().rev() {
            if let Some(state) = scope.get(name) {
                return Some(state.clone());
            }
        }
        None
    }

    fn set_ownership_state(&mut self, name: &str, new_state: OwnershipState) {
        for scope in self.ownership_scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), new_state);
                return;
            }
        }
    }

    pub fn analyze_module(&mut self, module: &Module) -> Result<(), Vec<DiagnosticError>> {
        // 1. Register Enums
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
                capabilities: CapabilityContract::default(),
            };
            self.graph.symbols.insert(e.name.clone(), info);
        }

        // Register Extensions
        for ext in &module.extensions {
            let info = SymbolInfo {
                name: format!("extend_{}", ext.target),
                kind: "extension".to_string(),
                type_signature: format!("extend {}", ext.target),
                file: ext.span.file.clone(),
                defined_at_line: ext.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: true,
                memory_region: None,
                capabilities: CapabilityContract::default(),
            };
            self.graph.symbols.insert(format!("extend_{}", ext.target), info);
            for f in &ext.functions {
                let mangled = format!("{}_{}", ext.target, f.name);
                let param_types = f.params.iter().map(|p| p.param_type.clone()).collect();
                self.function_signatures.insert(mangled.clone(), (param_types, f.return_type.clone(), true));
            }
        }

        // Register Modules
        for m in &module.modules {
            let info = SymbolInfo {
                name: m.name.clone(),
                kind: "module".to_string(),
                type_signature: format!("mod {} derives {:?}", m.name, m.parent),
                file: m.span.file.clone(),
                defined_at_line: m.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: true,
                memory_region: None,
                capabilities: CapabilityContract::default(),
            };
            self.graph.symbols.insert(m.name.clone(), info);
            for f in &m.functions {
                let mangled = format!("{}_{}", m.name, f.name);
                let param_types = f.params.iter().map(|p| p.param_type.clone()).collect();
                self.function_signatures.insert(mangled, (param_types, f.return_type.clone(), true));
            }
            for ov in &m.overrides {
                let mangled = format!("{}_{}", m.name, ov.name);
                let param_types = ov.params.iter().map(|p| p.param_type.clone()).collect();
                self.function_signatures.insert(mangled, (param_types, ov.return_type.clone(), true));
            }
        }

        // 2. Register Structs
        for s in &module.structs {
            self.structs.insert(s.name.clone(), s.clone());
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
                capabilities: CapabilityContract::default(),
            };
            self.graph.symbols.insert(s.name.clone(), info);
        }

        // 3. Register Function Signatures
        for f in &module.functions {
            let mut effect_set = HashSet::new();
            let is_pure = f.directives.iter().any(|d| d.name == "@pure");
            if is_pure {
                effect_set.insert("pure".to_string());
            }

            let param_types = f.params.iter().map(|p| p.param_type.clone()).collect();
            self.function_signatures.insert(f.name.clone(), (param_types, f.return_type.clone(), is_pure));
            self.function_effects.insert(f.name.clone(), effect_set);

            let mut cap = CapabilityContract::default();
            for dir in &f.directives {
                if dir.name == "@pure" {
                    cap.is_pure = true;
                } else if dir.name == "@capability" {
                    for arg in &dir.args {
                        let parts: Vec<&str> = arg.split('=').collect();
                        if parts.len() == 2 {
                            match parts[0].trim() {
                                "net" => cap.net = parts[1].trim().trim_matches('"') == "true",
                                "disk" => cap.disk = parts[1].trim().trim_matches('"') == "true",
                                "io" => cap.io = parts[1].trim().trim_matches('"') == "true",
                                "memory" => cap.memory = parts[1].trim().trim_matches('"').to_string(),
                                _ => {}
                            }
                        }
                    }
                }
            }

            let info = SymbolInfo {
                name: f.name.clone(),
                kind: "function".to_string(),
                type_signature: format!(
                    "fn {}({}) -> {}",
                    f.name,
                    f.params.iter().map(|p| format!("{}: {}", p.name, p.param_type)).collect::<Vec<_>>().join(", "),
                    f.return_type
                ),
                file: f.span.file.clone(),
                defined_at_line: f.span.line,
                callers: Vec::new(),
                callees: Vec::new(),
                effects: Vec::new(),
                is_pure: cap.is_pure,
                memory_region: Some(cap.memory.clone()),
                capabilities: cap,
            };
            self.graph.symbols.insert(f.name.clone(), info);
        }

        // 4. Analyze Function Bodies
        for f in &module.functions {
            self.analyze_function(f);
        }

        // 5. Transitive Effect & Purity Verification
        self.verify_transitive_effects_and_purity();

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn analyze_function(&mut self, func: &FunctionDef) {
        self.current_function = Some(func.name.clone());
        self.active_loans.clear();
        self.push_scope();

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
            Statement::VarDecl { name, var_type, is_mut, is_lease: _, initializer, span } => {
                let inferred_ty = if let Some(init) = initializer {
                    self.analyze_expression(init)
                } else {
                    var_type.clone().unwrap_or(Type::Void)
                };

                let ty = var_type.clone().unwrap_or(inferred_ty);
                self.declare_var(name, ty.clone(), span.line, *is_mut);

                // Check borrow creation from initializer (e.g. val r = &x)
                if let Some(init) = initializer {
                    if let Expression::Unary { expr, op, .. } = init {
                        if *op == UnaryOp::AddressOf {
                            if let Expression::Ident(place_name, _) = expr.as_ref() {
                                // Check if already mutably borrowed
                                if let Some(existing_loan) = self.active_loans.iter().find(|l| l.place == *place_name && l.kind == LoanKind::Mutable) {
                                    self.errors.push(DiagnosticError {
                                        code: "E0907".to_string(),
                                        message: format!("BorrowConflict: cannot borrow '{}' at line {} because it is already mutably borrowed by '{}' at line {}", place_name, span.line, existing_loan.holder, existing_loan.borrowed_at),
                                        line: span.line,
                                        col: span.col,
                                        kind: "BorrowConflictError".to_string(),
                                        repair_suggestion: Some("release previous mutable reference before borrowing again".to_string()),
                                    });
                                } else {
                                    self.active_loans.push(ActiveLoan {
                                        place: place_name.clone(),
                                        kind: LoanKind::Shared,
                                        borrowed_at: span.line,
                                        holder: name.clone(),
                                    });
                                }
                            }
                        }
                    } else if let Expression::Ident(src_name, _) = init {
                        // Check Move State
                        if let Some(OwnershipState::Moved { to, at_line }) = self.get_ownership_state(src_name) {
                            self.errors.push(DiagnosticError {
                                code: "E0906".to_string(),
                                message: format!("UseAfterMove: use of moved value '{}' at line {} (previously moved to '{}' at line {})", src_name, span.line, to, at_line),
                                line: span.line,
                                col: span.col,
                                kind: "UseAfterMoveError".to_string(),
                                repair_suggestion: Some(format!("clone '{}' or reinitialize before transferring ownership", src_name)),
                            });
                        } else {
                            if !matches!(ty, Type::I8 | Type::I16 | Type::I32 | Type::I64 | Type::U8 | Type::U16 | Type::U32 | Type::U64 | Type::F32 | Type::F64 | Type::Bool) {
                                self.set_ownership_state(src_name, OwnershipState::Moved { to: name.clone(), at_line: span.line });
                            }
                        }
                    }

                    if self.expr_allocates(init) && self.region_depth > 0 {
                        if let Some(reg_set) = self.region_allocations.last_mut() {
                            reg_set.insert(name.clone());
                        }
                    }
                }

                // Strict leak check
                let memory_allocated = initializer.as_ref().map(|i| self.expr_allocates(i)).unwrap_or(false);
                if self.strict_leaks && (memory_allocated || matches!(ty, Type::Pointer(_))) && self.region_depth == 0 {
                    self.errors.push(DiagnosticError {
                        code: "E0901".to_string(),
                        message: format!("Memory leak detected: pointer allocated at line {} escapes without safe region boundary", span.line),
                        line: span.line,
                        col: span.col,
                        kind: "MemoryLeakError".to_string(),
                        repair_suggestion: Some("wrap in 'region arena { ... }' to guarantee zero memory leak".to_string()),
                    });
                }
            }
            Statement::Assignment { target, value, span } => {
                self.analyze_expression(target);
                self.analyze_expression(value);

                // Check mutation while actively borrowed
                if let Expression::Ident(target_name, _) = target {
                    if let Some(loan) = self.active_loans.iter().find(|l| l.place == *target_name && l.holder != *target_name) {
                        self.errors.push(DiagnosticError {
                            code: "E0907".to_string(),
                            message: format!("BorrowConflict: cannot mutate '{}' at line {} because it is currently borrowed by '{}' (borrowed at line {})", target_name, span.line, loan.holder, loan.borrowed_at),
                            line: span.line,
                            col: span.col,
                            kind: "BorrowConflictError".to_string(),
                            repair_suggestion: Some(format!("ensure borrow '{}' goes out of scope before modifying '{}'", loan.holder, target_name)),
                        });
                    }
                }
            }
            Statement::Return { value, span } => {
                if let Some(val) = value {
                    self.analyze_expression(val);

                    // Formal Escape Analysis: check if returning an inner-region allocated pointer
                    if let Expression::Ident(ret_var, _) = val {
                        if self.region_depth > 0 {
                            for reg_set in &self.region_allocations {
                                if reg_set.contains(ret_var) {
                                    self.errors.push(DiagnosticError {
                                        code: "E0903".to_string(),
                                        message: format!("EscapeViolation: reference to region-scoped memory '{}' escapes region boundary at line {}", ret_var, span.line),
                                        line: span.line,
                                        col: span.col,
                                        kind: "EscapeViolationError".to_string(),
                                        repair_suggestion: Some("copy data or allocate on parent region before returning".to_string()),
                                    });
                                }
                            }
                        }
                    }
                }
            }
            Statement::RegionBlock { name, body, span } => {
                self.region_depth += 1;
                self.region_allocations.push(HashSet::new());
                self.push_scope();
                self.declare_var(&format!("region_{}", name), Type::Region(name.clone()), span.line, false);
                self.analyze_block(body);
                self.pop_scope();
                self.region_allocations.pop();
                self.region_depth = self.region_depth.saturating_sub(1);
            }
            Statement::Spawn { call, span } => {
                self.analyze_expression(call);
                // In Thread Spawn: capture variables transfer ownership to spawn
                if let Expression::Call { args, .. } = call {
                    for arg in args {
                        if let Expression::Ident(var_name, _) = arg {
                            self.set_ownership_state(var_name, OwnershipState::Moved {
                                to: "thread_spawn".to_string(),
                                at_line: span.line,
                            });
                        }
                    }
                }
            }
            Statement::Expression(expr) => {
                self.analyze_expression(expr);
            }
            Statement::If { condition, then_block, else_block, .. } => {
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
            Statement::ForIn { item_name, iterable, body, span } => {
                self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(item_name, Type::I32, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::ParallelFor { item_name, iterable, body, span } => {
                self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(item_name, Type::I32, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::Match { expr, arms, .. } => {
                let match_type = self.analyze_expression(expr);
                for arm in arms {
                    self.push_scope();
                    if let Pattern::Ident(id) = &arm.pattern {
                        self.declare_var(id, match_type.clone(), arm.span.line, false);
                    }
                    if let Some(g) = &arm.guard {
                        self.analyze_expression(g);
                    }
                    self.analyze_block(&arm.body);
                    self.pop_scope();
                }
            }
            Statement::Defer { expr, .. } => {
                self.analyze_expression(expr);
            }
            Statement::LeaseBlock { name, var_type, initializer, condition, body, span } => {
                let inferred_ty = self.analyze_expression(initializer);
                if let Some(c) = condition {
                    self.analyze_expression(c);
                }
                self.push_scope();
                let ty = var_type.clone().unwrap_or(inferred_ty);
                self.declare_var(name, ty, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::LeaseCpu { cores, priority, body, span } => {
                self.analyze_expression(cores);
                if let Some(p) = priority {
                    self.analyze_expression(p);
                }
                self.push_scope();
                self.declare_var("__lease_cpu_cores", Type::I32, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::LeaseEvent { event_expr, condition, body, span } => {
                self.analyze_expression(event_expr);
                if let Some(c) = condition {
                    self.analyze_expression(c);
                }
                self.push_scope();
                self.declare_var("__lease_event_handle", Type::I64, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            Statement::LeaseLoop { budget, item_name, iterable, body, span } => {
                if let Some(b) = budget {
                    self.analyze_expression(b);
                }
                self.analyze_expression(iterable);
                self.push_scope();
                self.declare_var(item_name, Type::I64, span.line, false);
                self.analyze_block(body);
                self.pop_scope();
            }
            _ => {}
        }
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Type {
        match expr {
            Expression::Lit(Literal::Int(_), _) => Type::I64,
            Expression::Lit(Literal::Float(_), _) => Type::F64,
            Expression::Lit(Literal::String(_), _) => Type::Str,
            Expression::Lit(Literal::Bool(_), _) => Type::Bool,
            Expression::Lit(Literal::Null, _) => Type::Pointer(Box::new(Type::Void)),
            Expression::Ident(name, span) => {
                if let Some(OwnershipState::Moved { to, at_line }) = self.get_ownership_state(name) {
                    self.errors.push(DiagnosticError {
                        code: "E0906".to_string(),
                        message: format!("UseAfterMove: use of moved value '{}' at line {} (moved to '{}' at line {})", name, span.line, to, at_line),
                        line: span.line,
                        col: span.col,
                        kind: "UseAfterMoveError".to_string(),
                        repair_suggestion: Some(format!("clone '{}' or reinitialize before transferring ownership", name)),
                    });
                }
                if let Some((ty, _, _)) = self.lookup_var(name) {
                    ty
                } else {
                    Type::Custom(name.clone())
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let l_ty = self.analyze_expression(left);
                let r_ty = self.analyze_expression(right);
                match op {
                    BinaryOp::Equal | BinaryOp::NotEqual | BinaryOp::LessThan | BinaryOp::LessEqual | BinaryOp::GreaterThan | BinaryOp::GreaterEqual | BinaryOp::And | BinaryOp::Or => Type::Bool,
                    _ => {
                        if l_ty == Type::F64 || r_ty == Type::F64 {
                            Type::F64
                        } else if l_ty == Type::F32 || r_ty == Type::F32 {
                            Type::F32
                        } else if l_ty != Type::I64 && l_ty != Type::Void && l_ty != Type::Bool {
                            l_ty
                        } else {
                            r_ty
                        }
                    }
                }
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
                    UnaryOp::Negate | UnaryOp::BitNot => inner,
                }
            }
            Expression::Call { callee, args, .. } => {
                for arg in args {
                    self.analyze_expression(arg);
                }
                let mut ret_type = Type::Void;
                if let Expression::Ident(callee_name, _) = callee.as_ref() {
                    if let Some((_params, sig_ret, _)) = self.function_signatures.get(callee_name) {
                        ret_type = sig_ret.clone();
                    }
                    if let Some(curr_fn) = &self.current_function {
                        self.graph.add_call(curr_fn, callee_name);
                        if let Some(effects) = self.function_effects.get_mut(curr_fn) {
                            if callee_name.contains("socket") || callee_name.contains("send") || callee_name.contains("recv") || callee_name.contains("http") || callee_name.contains("net") {
                                effects.insert("network".to_string());
                            }
                            if callee_name.contains("print") || callee_name.contains("write") || callee_name.contains("read") {
                                effects.insert("io".to_string());
                            }
                        }
                    }
                }
                ret_type
            }
            Expression::FieldAccess { object, field, .. } => {
                let obj_ty = self.analyze_expression(object);
                match obj_ty {
                    Type::Custom(ref sname) => {
                        if let Some(st_def) = self.structs.get(sname) {
                            if let Some(f_def) = st_def.fields.iter().find(|f| &f.name == field) {
                                return f_def.field_type.clone();
                            }
                        }
                        Type::I64
                    }
                    _ => Type::I64,
                }
            }
            Expression::Index { array, index, .. } => {
                let arr_ty = self.analyze_expression(array);
                self.analyze_expression(index);
                match arr_ty {
                    Type::Pointer(inner) | Type::Slice(inner) | Type::Array(inner, _) => *inner,
                    Type::Str => Type::U8,
                    _ => Type::I64,
                }
            }
            Expression::Alloc { target_type, allocator, .. } => {
                self.analyze_expression(allocator);
                Type::Pointer(Box::new(target_type.clone()))
            }
            Expression::Cast { target_type, expr, .. } => {
                self.analyze_expression(expr);
                target_type.clone()
            }
            Expression::Await { expr, .. } => {
                self.analyze_expression(expr)
            }
            Expression::StructInit { name, fields, .. } => {
                for (_, f_expr) in fields {
                    self.analyze_expression(f_expr);
                }
                Type::Custom(name.clone())
            }
            Expression::Match { expr, arms, .. } => {
                self.analyze_expression(expr);
                for arm in arms {
                    for stmt in &arm.body.statements {
                        self.analyze_statement(stmt);
                    }
                }
                Type::I64
            }
            _ => Type::Void,
        }
    }

    fn verify_transitive_effects_and_purity(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            let current_effects = self.function_effects.clone();
            for (caller, callees) in self.graph.symbols.iter().map(|(k, v)| (k.clone(), v.callees.clone())) {
                for callee in callees {
                    if let Some(callee_effects) = current_effects.get(&callee) {
                        if let Some(caller_effects) = self.function_effects.get_mut(&caller) {
                            for eff in callee_effects {
                                if caller_effects.insert(eff.clone()) {
                                    changed = true;
                                }
                            }
                        }
                    }
                }
            }
        }

        for (func_name, (_params, _ret, is_pure)) in &self.function_signatures {
            if *is_pure {
                if let Some(effects) = self.function_effects.get(func_name) {
                    let impure_effects: Vec<&String> = effects.iter().filter(|e| *e == "network" || *e == "io" || *e == "database" || *e == "filesystem").collect();
                    if !impure_effects.is_empty() {
                        if let Some(sym) = self.graph.symbols.get(func_name) {
                            self.errors.push(DiagnosticError {
                                code: "E0904".to_string(),
                                message: format!("PurityViolation: function '{}' is marked @pure but transitively invokes impure operations: {:?}", func_name, impure_effects),
                                line: sym.defined_at_line,
                                col: 1,
                                kind: "PurityViolationError".to_string(),
                                repair_suggestion: Some("remove @pure directive or refactor to isolate side-effects".to_string()),
                            });
                        }
                    }
                }
            }
        }
    }

    fn expr_allocates(&self, expr: &Expression) -> bool {
        match expr {
            Expression::Alloc { .. } => true,
            Expression::Call { callee, .. } => {
                if let Expression::Ident(name, _) = callee.as_ref() {
                    name == "alloc" || name.contains("create")
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}



