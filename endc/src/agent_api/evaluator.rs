use crate::codegen::interpreter::{Interpreter, Value};
use crate::lexer::Lexer;
use crate::parser::Parser as EndParser;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvalResult {
    pub status: String,
    pub expression: String,
    pub result: String,
    pub value_type: String,
    pub duration_us: u128,
    pub memory_allocated_bytes: usize,
}

pub struct MicroEvaluator;

impl MicroEvaluator {
    pub fn eval_expression(expr_code: &str) -> Result<EvalResult, String> {
        let start = std::time::Instant::now();
        let trimmed = expr_code.trim();

        // Synthetic wrapper module
        let synthetic_source = if trimmed.contains("ret ") || trimmed.contains("return ") || trimmed.contains("val ") || trimmed.contains("mut ") {
            if trimmed.contains("ret ") || trimmed.contains("return ") {
                format!("fn __eval_entry__() i64 {{\n{}\n}}", trimmed)
            } else {
                format!("fn __eval_entry__() i64 {{\n{}\nret 0\n}}", trimmed)
            }
        } else {
            format!("fn __eval_entry__() i64 {{\nret ({})\n}}", trimmed)
        };

        let mut lexer = Lexer::new("<synthetic_eval>", &synthetic_source);
        let tokens = lexer.tokenize_all().map_err(|e| format!("Lexer error: {}", e))?;

        let mut parser = EndParser::new("<synthetic_eval>", tokens);
        let module = parser.parse_module("eval").map_err(|e| format!("Parser error: {}", e))?;

        let mut vm = Interpreter::new();
        let eval_val = vm.eval_named_function(&module, "__eval_entry__", vec![])
            .map_err(|e| format!("Runtime eval error: {}", e))?;

        let duration_us = start.elapsed().as_micros().max(1);

        let (res_str, val_type) = match eval_val {
            Value::Int(n) => (n.to_string(), "i64".to_string()),
            Value::Float(f) => (f.to_string(), "f64".to_string()),
            Value::Bool(b) => (b.to_string(), "bool".to_string()),
            Value::String(s) => (format!("\"{}\"", s), "str".to_string()),
            Value::Void => ("void".to_string(), "void".to_string()),
            Value::Pointer(p) => (format!("0x{:x}", p), "pointer".to_string()),
            _ => (format!("{}", eval_val), "custom".to_string()),
        };

        Ok(EvalResult {
            status: "success".to_string(),
            expression: trimmed.to_string(),
            result: res_str,
            value_type: val_type,
            duration_us,
            memory_allocated_bytes: 0,
        })
    }
}
