use crate::ast::*;
use std::fmt::Write;
use std::collections::HashMap;

pub struct LlvmBackend {
    output: String,
    target_triple: String,
    temp_var_id: usize,
    block_id: usize,
    variables: HashMap<String, (String, String)>, // name -> (llvm_type, llvm_reg)
}

impl LlvmBackend {
    pub fn new(target_triple: Option<&str>) -> Self {
        Self {
            output: String::new(),
            target_triple: target_triple.unwrap_or("x86_64-pc-windows-msvc").to_string(),
            temp_var_id: 0,
            block_id: 0,
            variables: HashMap::new(),
        }
    }

    fn next_temp(&mut self) -> String {
        let id = self.temp_var_id;
        self.temp_var_id += 1;
        format!("%t{}", id)
    }

    fn next_label(&mut self, prefix: &str) -> String {
        let id = self.block_id;
        self.block_id += 1;
        format!("{}_{}", prefix, id)
    }

    pub fn generate_llvm_ir(&mut self, module: &Module) -> String {
        self.output.clear();
        self.temp_var_id = 0;
        self.block_id = 0;
        self.variables.clear();

        writeln!(self.output, "; ModuleID = '{}'", module.name).unwrap();
        writeln!(self.output, "target triple = \"{}\"", self.target_triple).unwrap();
        writeln!(self.output, "source_filename = \"{}.end\"\n", module.name).unwrap();

        // Standard LLVM Declarations
        writeln!(self.output, "declare i32 @printf(i8*, ...)").unwrap();
        writeln!(self.output, "declare i8* @malloc(i64)").unwrap();
        writeln!(self.output, "declare void @free(i8*)\n").unwrap();

        // Struct Definitions
        for st in &module.structs {
            write!(self.output, "%struct.{} = type {{ ", st.name).unwrap();
            for (i, field) in st.fields.iter().enumerate() {
                if i > 0 {
                    write!(self.output, ", ").unwrap();
                }
                write!(self.output, "{}", self.map_type_to_llvm(&field.field_type)).unwrap();
            }
            writeln!(self.output, " }}\n").unwrap();
        }

        // Functions
        for func in &module.functions {
            self.generate_function(func);
        }

        self.output.clone()
    }

    fn generate_function(&mut self, func: &FunctionDef) {
        self.variables.clear();
        let ret_ty = self.map_type_to_llvm(&func.return_type);
        write!(self.output, "define {} @{}(", ret_ty, func.name).unwrap();
        for (i, p) in func.params.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ").unwrap();
            }
            let p_ty = self.map_type_to_llvm(&p.param_type);
            write!(self.output, "{} %arg_{}", p_ty, p.name).unwrap();
        }
        writeln!(self.output, ") {{").unwrap();
        writeln!(self.output, "entry:").unwrap();

        // Allocate stack slots for parameters
        for p in &func.params {
            let p_ty = self.map_type_to_llvm(&p.param_type);
            let ptr_reg = format!("%{}", p.name);
            writeln!(self.output, "  {} = alloca {}", ptr_reg, p_ty).unwrap();
            writeln!(self.output, "  store {} %arg_{}, {}* {}", p_ty, p.name, p_ty, ptr_reg).unwrap();
            self.variables.insert(p.name.clone(), (p_ty, ptr_reg));
        }

        let mut returned = false;
        for stmt in &func.body.statements {
            if self.generate_statement(stmt) {
                returned = true;
            }
        }

        if !returned {
            if func.return_type == Type::Void {
                writeln!(self.output, "  ret void").unwrap();
            } else {
                let default_val = if ret_ty.starts_with('i') { "0" } else { "null" };
                writeln!(self.output, "  ret {} {}", ret_ty, default_val).unwrap();
            }
        }

        writeln!(self.output, "}}\n").unwrap();
    }

    fn generate_statement(&mut self, stmt: &Statement) -> bool {
        match stmt {
            Statement::VarDecl { name, var_type, initializer, .. } => {
                let ty = var_type.as_ref().map(|t| self.map_type_to_llvm(t)).unwrap_or_else(|| "i64".to_string());
                let ptr_reg = format!("%{}", name);
                writeln!(self.output, "  {} = alloca {}", ptr_reg, ty).unwrap();
                self.variables.insert(name.clone(), (ty.clone(), ptr_reg.clone()));

                if let Some(init) = initializer {
                    let (init_val, _) = self.generate_expression(init);
                    writeln!(self.output, "  store {} {}, {}* {}", ty, init_val, ty, ptr_reg).unwrap();
                }
                false
            }
            Statement::Assignment { target, value, .. } => {
                let (val, val_ty) = self.generate_expression(value);
                if let Expression::Ident(name, _) = target {
                    if let Some((ty, ptr_reg)) = self.variables.get(name) {
                        writeln!(self.output, "  store {} {}, {}* {}", ty, val, ty, ptr_reg).unwrap();
                    }
                }
                false
            }
            Statement::Return { value, .. } => {
                if let Some(val_expr) = value {
                    let (val, val_ty) = self.generate_expression(val_expr);
                    writeln!(self.output, "  ret {} {}", val_ty, val).unwrap();
                } else {
                    writeln!(self.output, "  ret void").unwrap();
                }
                true
            }
            Statement::Expression(expr) => {
                self.generate_expression(expr);
                false
            }
            Statement::If { condition, then_block, else_block, .. } => {
                let (cond_val, _) = self.generate_expression(condition);
                let then_lbl = self.next_label("then");
                let else_lbl = self.next_label("else");
                let merge_lbl = self.next_label("if_merge");

                writeln!(self.output, "  br i1 {}, label %{}, label %{}", cond_val, then_lbl, if else_block.is_some() { &else_lbl } else { &merge_lbl }).unwrap();

                writeln!(self.output, "{}:", then_lbl).unwrap();
                for s in &then_block.statements {
                    self.generate_statement(s);
                }
                writeln!(self.output, "  br label %{}", merge_lbl).unwrap();

                if let Some(eb) = else_block {
                    writeln!(self.output, "{}:", else_lbl).unwrap();
                    for s in &eb.statements {
                        self.generate_statement(s);
                    }
                    writeln!(self.output, "  br label %{}", merge_lbl).unwrap();
                }

                writeln!(self.output, "{}:", merge_lbl).unwrap();
                false
            }
            Statement::While { condition, body, .. } => {
                let cond_lbl = self.next_label("while_cond");
                let body_lbl = self.next_label("while_body");
                let end_lbl = self.next_label("while_end");

                writeln!(self.output, "  br label %{}", cond_lbl).unwrap();
                writeln!(self.output, "{}:", cond_lbl).unwrap();
                let (cond_val, _) = self.generate_expression(condition);
                writeln!(self.output, "  br i1 {}, label %{}, label %{}", cond_val, body_lbl, end_lbl).unwrap();

                writeln!(self.output, "{}:", body_lbl).unwrap();
                for s in &body.statements {
                    self.generate_statement(s);
                }
                writeln!(self.output, "  br label %{}", cond_lbl).unwrap();
                writeln!(self.output, "{}:", end_lbl).unwrap();
                false
            }
            Statement::RegionBlock { body, .. } => {
                for s in &body.statements {
                    self.generate_statement(s);
                }
                false
            }
            _ => false,
        }
    }

    fn generate_expression(&mut self, expr: &Expression) -> (String, String) {
        match expr {
            Expression::Lit(Literal::Int(n), _) => (n.to_string(), "i64".to_string()),
            Expression::Lit(Literal::Float(f), _) => (format!("{:.6}", f), "double".to_string()),
            Expression::Lit(Literal::Bool(b), _) => ((if *b { "1" } else { "0" }).to_string(), "i1".to_string()),
            Expression::Lit(Literal::String(s), _) => (format!("\"{}\"", s), "i8*".to_string()),
            Expression::Lit(Literal::Null, _) => ("null".to_string(), "i8*".to_string()),
            Expression::Ident(name, _) => {
                let var_info = self.variables.get(name).cloned();
                if let Some((ty, ptr_reg)) = var_info {
                    let tmp = self.next_temp();
                    writeln!(self.output, "  {} = load {}, {}* {}", tmp, ty, ty, ptr_reg).unwrap();
                    (tmp, ty)
                } else {
                    (name.clone(), "i64".to_string())
                }
            }
            Expression::Binary { left, op, right, .. } => {
                let (l_val, l_ty) = self.generate_expression(left);
                let (r_val, _) = self.generate_expression(right);
                let tmp = self.next_temp();
                let op_instr = match op {
                    BinaryOp::Add => "add",
                    BinaryOp::Sub => "sub",
                    BinaryOp::Mul => "mul",
                    BinaryOp::Div => "sdiv",
                    BinaryOp::Mod => "srem",
                    BinaryOp::BitAnd => "and",
                    BinaryOp::BitOr => "or",
                    BinaryOp::BitXor => "xor",
                    BinaryOp::Shl => "shl",
                    BinaryOp::Shr => "ashr",
                    BinaryOp::Equal => "icmp eq",
                    BinaryOp::NotEqual => "icmp ne",
                    BinaryOp::LessThan => "icmp slt",
                    BinaryOp::LessEqual => "icmp sle",
                    BinaryOp::GreaterThan => "icmp sgt",
                    BinaryOp::GreaterEqual => "icmp sge",
                    _ => "add",
                };
                writeln!(self.output, "  {} = {} {} {}, {}", tmp, op_instr, l_ty, l_val, r_val).unwrap();
                let res_ty = if op_instr.starts_with("icmp") { "i1".to_string() } else { l_ty };
                (tmp, res_ty)
            }
            Expression::Unary { expr, op, .. } => {
                let (val, ty) = self.generate_expression(expr);
                let tmp = self.next_temp();
                match op {
                    UnaryOp::Negate => {
                        writeln!(self.output, "  {} = sub {} 0, {}", tmp, ty, val).unwrap();
                        (tmp, ty)
                    }
                    UnaryOp::Not => {
                        writeln!(self.output, "  {} = xor i1 {}, 1", tmp, val).unwrap();
                        (tmp, "i1".to_string())
                    }
                    _ => (val, ty),
                }
            }
            Expression::Call { callee, args, .. } => {
                let mut evaluated_args = Vec::new();
                for a in args {
                    evaluated_args.push(self.generate_expression(a));
                }
                let callee_name = match callee.as_ref() {
                    Expression::Ident(n, _) => n.clone(),
                    _ => "callee".to_string(),
                };
                let tmp = self.next_temp();
                let args_str = evaluated_args.iter().map(|(v, t)| format!("{} {}", t, v)).collect::<Vec<_>>().join(", ");
                writeln!(self.output, "  {} = call i64 @{}({})", tmp, callee_name, args_str).unwrap();
                (tmp, "i64".to_string())
            }
            Expression::NameOf { target, .. } => (format!("\"{}\"", target), "i8*".to_string()),
            Expression::TypeOf { expr, .. } => ("\"i64\"".to_string(), "i8*".to_string()),
            _ => ("0".to_string(), "i64".to_string()),
        }
    }

    fn map_type_to_llvm(&self, ty: &Type) -> String {
        match ty {
            Type::Void => "void".to_string(),
            Type::Bool => "i1".to_string(),
            Type::I8 | Type::U8 => "i8".to_string(),
            Type::I16 | Type::U16 => "i16".to_string(),
            Type::I32 | Type::U32 => "i32".to_string(),
            Type::I64 | Type::U64 => "i64".to_string(),
            Type::F32 => "float".to_string(),
            Type::F64 => "double".to_string(),
            Type::Str => "i8*".to_string(),
            Type::Pointer(inner) => format!("{}*", self.map_type_to_llvm(inner)),
            Type::Region(_) => "i8*".to_string(),
            Type::Custom(name) => format!("%struct.{}", name),
            _ => "i64".to_string(),
        }
    }
}



