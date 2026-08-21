use crate::ast::*;
use std::fmt::Write;

pub struct LlvmBackend {
    output: String,
    target_triple: String,
    temp_var_id: usize,
}

impl LlvmBackend {
    pub fn new(target_triple: Option<&str>) -> Self {
        Self {
            output: String::new(),
            target_triple: target_triple.unwrap_or("x86_64-pc-windows-msvc").to_string(),
            temp_var_id: 0,
        }
    }

    fn next_temp(&mut self) -> String {
        let id = self.temp_var_id;
        self.temp_var_id += 1;
        format!("%t{}", id)
    }

    pub fn generate_llvm_ir(&mut self, module: &Module) -> String {
        self.output.clear();
        self.temp_var_id = 0;

        writeln!(self.output, "; ModuleID = '{}'", module.name).unwrap();
        writeln!(self.output, "target triple = \"{}\"", self.target_triple).unwrap();
        writeln!(self.output, "source_filename = \"{}.end\"\n", module.name).unwrap();

        // Standard LLVM Declarations
        writeln!(self.output, "declare i32 @printf(i8*, ...)\n").unwrap();

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
        let ret_ty = self.map_type_to_llvm(&func.return_type);
        write!(self.output, "define {} @{}(", ret_ty, func.name).unwrap();
        for (i, p) in func.params.iter().enumerate() {
            if i > 0 {
                write!(self.output, ", ").unwrap();
            }
            write!(self.output, "{} %{}", self.map_type_to_llvm(&p.param_type), p.name).unwrap();
        }
        writeln!(self.output, ") {{\nentry:").unwrap();

        let mut returned = false;
        for stmt in &func.body.statements {
            match stmt {
                Statement::Return { value, .. } => {
                    if let Some(val_expr) = value {
                        let res_var = self.generate_expression(val_expr);
                        writeln!(self.output, "  ret {} {}", ret_ty, res_var).unwrap();
                    } else {
                        writeln!(self.output, "  ret void").unwrap();
                    }
                    returned = true;
                    break;
                }
                Statement::VarDecl { name, initializer, .. } => {
                    if let Some(init) = initializer {
                        let res_var = self.generate_expression(init);
                        writeln!(self.output, "  %{} = add i64 {}, 0", name, res_var).unwrap();
                    }
                }
                _ => {}
            }
        }

        if !returned {
            if func.return_type == Type::Void {
                writeln!(self.output, "  ret void").unwrap();
            } else {
                writeln!(self.output, "  ret {} 0", ret_ty).unwrap();
            }
        }

        writeln!(self.output, "}}\n").unwrap();
    }

    fn generate_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Lit(Literal::Int(n), _) => n.to_string(),
            Expression::Lit(Literal::Float(f), _) => f.to_string(),
            Expression::Lit(Literal::Bool(b), _) => if *b { "1".to_string() } else { "0".to_string() },
            Expression::Ident(name, _) => format!("%{}", name),
            Expression::Binary { op, left, right, .. } => {
                let l = self.generate_expression(left);
                let r = self.generate_expression(right);
                let tmp = self.next_temp();
                let op_str = match op {
                    BinaryOp::Add => "add i64",
                    BinaryOp::Sub => "sub i64",
                    BinaryOp::Mul => "mul i64",
                    BinaryOp::Div => "sdiv i64",
                    BinaryOp::Mod => "srem i64",
                    _ => "add i64",
                };
                writeln!(self.output, "  {} = {} {}, {}", tmp, op_str, l, r).unwrap();
                tmp
            }
            Expression::Call { callee, args, .. } => {
                let func_name = match callee.as_ref() {
                    Expression::Ident(n, _) => n.clone(),
                    _ => "unknown_func".to_string(),
                };
                let mut arg_vars = Vec::new();
                for a in args {
                    let v = self.generate_expression(a);
                    arg_vars.push(format!("i64 {}", v));
                }
                let tmp = self.next_temp();
                writeln!(self.output, "  {} = call i64 @{}({})", tmp, func_name, arg_vars.join(", ")).unwrap();
                tmp
            }
            _ => "0".to_string(),
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
            Type::Custom(name) => format!("%struct.{}", name),
            _ => "i64".to_string(),
        }
    }
}
