use crate::ast::*;
use crate::codegen::backend_trait::{BackendError, CodeGenBackend};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmBuildReport {
    pub status: String,
    pub target: String,
    pub functions_exported: usize,
    pub memory_pages: usize,
    pub wat_size_bytes: usize,
    pub js_glue_generated: bool,
    pub wasm_binary_preview: String,
}

pub struct WasmBackend {
    output_wat: String,
    target: String,
    var_map: HashMap<String, usize>,
    local_id: usize,
}

impl WasmBackend {
    pub fn new(target: Option<&str>) -> Self {
        Self {
            output_wat: String::new(),
            target: target.unwrap_or("wasm32-wasi").to_string(),
            var_map: HashMap::new(),
            local_id: 0,
        }
    }

    pub fn generate_wat(&mut self, module: &Module) -> Result<String, BackendError> {
        self.output_wat.clear();
        self.var_map.clear();
        self.local_id = 0;

        writeln!(self.output_wat, "(module").unwrap();
        writeln!(self.output_wat, "  ;; End Language WebAssembly Module: {}", module.name).unwrap();
        writeln!(self.output_wat, "  ;; Target: {}", self.target).unwrap();
        writeln!(self.output_wat, "  (memory (export \"memory\") 2)").unwrap();
        writeln!(self.output_wat, "  (import \"env\" \"println\" (func $println (param i64)))").unwrap();
        writeln!(self.output_wat, "  (import \"env\" \"print_str\" (func $print_str (param i32 i32)))\n").unwrap();

        for func in &module.functions {
            self.generate_function_wat(func)?;
        }

        writeln!(self.output_wat, ")").unwrap();
        Ok(self.output_wat.clone())
    }

    fn generate_function_wat(&mut self, func: &FunctionDef) -> Result<(), BackendError> {
        let mut params_str = String::new();
        for p in &func.params {
            let p_ty = self.map_wasm_type(&p.param_type);
            write!(params_str, " (param ${} {})", p.name, p_ty).unwrap();
        }

        let ret_str = if func.return_type == Type::Void {
            "".to_string()
        } else {
            format!(" (result {})", self.map_wasm_type(&func.return_type))
        };

        writeln!(self.output_wat, "  (func ${} (export \"{}\"){}{}", func.name, func.name, params_str, ret_str).unwrap();
        
        for stmt in &func.body.statements {
            self.generate_statement_wat(stmt, "    ")?;
        }

        writeln!(self.output_wat, "  )\n").unwrap();
        Ok(())
    }

    fn generate_statement_wat(&mut self, stmt: &Statement, indent: &str) -> Result<(), BackendError> {
        match stmt {
            Statement::VarDecl { name, initializer, .. } => {
                if let Some(init) = initializer {
                    self.generate_expression_wat(init, indent)?;
                    writeln!(self.output_wat, "{}local.set ${}", indent, name).unwrap();
                }
            }
            Statement::Assignment { target, value, .. } => {
                if let Expression::Ident(name, _) = target {
                    self.generate_expression_wat(value, indent)?;
                    writeln!(self.output_wat, "{}local.set ${}", indent, name).unwrap();
                }
            }
            Statement::Return { value, .. } => {
                if let Some(v) = value {
                    self.generate_expression_wat(v, indent)?;
                }
                writeln!(self.output_wat, "{}return", indent).unwrap();
            }
            Statement::If { condition, then_block, else_block, .. } => {
                self.generate_expression_wat(condition, indent)?;
                writeln!(self.output_wat, "{}(if", indent).unwrap();
                writeln!(self.output_wat, "{}  (then", indent).unwrap();
                for s in &then_block.statements {
                    self.generate_statement_wat(s, &format!("{}    ", indent))?;
                }
                writeln!(self.output_wat, "{}  )", indent).unwrap();
                if let Some(eb) = else_block {
                    writeln!(self.output_wat, "{}  (else", indent).unwrap();
                    for s in &eb.statements {
                        self.generate_statement_wat(s, &format!("{}    ", indent))?;
                    }
                    writeln!(self.output_wat, "{}  )", indent).unwrap();
                }
                writeln!(self.output_wat, "{})", indent).unwrap();
            }
            Statement::While { condition, body, .. } => {
                writeln!(self.output_wat, "{}(block $while_break", indent).unwrap();
                writeln!(self.output_wat, "{}  (loop $while_loop", indent).unwrap();
                self.generate_expression_wat(condition, &format!("{}    ", indent))?;
                writeln!(self.output_wat, "{}    i32.eqz", indent).unwrap();
                writeln!(self.output_wat, "{}    br_if $while_break", indent).unwrap();
                for s in &body.statements {
                    self.generate_statement_wat(s, &format!("{}    ", indent))?;
                }
                writeln!(self.output_wat, "{}    br $while_loop", indent).unwrap();
                writeln!(self.output_wat, "{}  )", indent).unwrap();
                writeln!(self.output_wat, "{})", indent).unwrap();
            }
            Statement::Expression(expr) => {
                self.generate_expression_wat(expr, indent)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn generate_expression_wat(&mut self, expr: &Expression, indent: &str) -> Result<(), BackendError> {
        match expr {
            Expression::Lit(Literal::Int(n), _) => {
                writeln!(self.output_wat, "{}i64.const {}", indent, n).unwrap();
            }
            Expression::Lit(Literal::Float(f), _) => {
                writeln!(self.output_wat, "{}f64.const {}", indent, f).unwrap();
            }
            Expression::Lit(Literal::Bool(b), _) => {
                writeln!(self.output_wat, "{}i32.const {}", indent, if *b { 1 } else { 0 }).unwrap();
            }
            Expression::Ident(name, _) => {
                writeln!(self.output_wat, "{}local.get ${}", indent, name).unwrap();
            }
            Expression::Binary { left, op, right, .. } => {
                self.generate_expression_wat(left, indent)?;
                self.generate_expression_wat(right, indent)?;
                let op_wat = match op {
                    BinaryOp::Add => "i64.add",
                    BinaryOp::Sub => "i64.sub",
                    BinaryOp::Mul => "i64.mul",
                    BinaryOp::Div => "i64.div_s",
                    BinaryOp::Mod => "i64.rem_s",
                    BinaryOp::Equal => "i64.eq",
                    BinaryOp::NotEqual => "i64.ne",
                    BinaryOp::LessThan => "i64.lt_s",
                    BinaryOp::LessEqual => "i64.le_s",
                    BinaryOp::GreaterThan => "i64.gt_s",
                    BinaryOp::GreaterEqual => "i64.ge_s",
                    BinaryOp::BitAnd => "i64.and",
                    BinaryOp::BitOr => "i64.or",
                    BinaryOp::BitXor => "i64.xor",
                    BinaryOp::Shl => "i64.shl",
                    BinaryOp::Shr => "i64.shr_s",
                    _ => "i64.add",
                };
                writeln!(self.output_wat, "{}{}", indent, op_wat).unwrap();
            }
            Expression::Call { callee, args, .. } => {
                for a in args {
                    self.generate_expression_wat(a, indent)?;
                }
                let callee_name = if let Expression::Ident(n, _) = callee.as_ref() { n.as_str() } else { "fn" };
                writeln!(self.output_wat, "{}call ${}", indent, callee_name).unwrap();
            }
            _ => {
                writeln!(self.output_wat, "{}i64.const 0", indent).unwrap();
            }
        }
        Ok(())
    }

    pub fn generate_js_glue(&self, module: &Module) -> String {
        format!(
            r#"// End Language WebAssembly JavaScript/TypeScript Runtime Glue
export async function loadEndWasm(wasmBytesOrUrl) {{
    const imports = {{
        env: {{
            println: (val) => console.log(val.toString()),
            print_str: (ptr, len) => {{ /* String decode */ }}
        }}
    }};
    const {{ instance }} = await WebAssembly.instantiate(wasmBytesOrUrl, imports);
    return instance.exports;
}}
"#
        )
    }

    fn map_wasm_type(&self, ty: &Type) -> &'static str {
        match ty {
            Type::Bool | Type::I8 | Type::U8 | Type::I16 | Type::U16 | Type::I32 | Type::U32 => "i32",
            Type::I64 | Type::U64 => "i64",
            Type::F32 => "f32",
            Type::F64 => "f64",
            _ => "i32",
        }
    }
}

impl CodeGenBackend for WasmBackend {
    type Output = String;

    fn compile_module(&mut self, module: &Module) -> Result<Self::Output, BackendError> {
        self.generate_wat(module)
    }

    fn name(&self) -> &'static str {
        "wasm"
    }

    fn supports_jit(&self) -> bool {
        false
    }

    fn supports_aot(&self) -> bool {
        true
    }
}
