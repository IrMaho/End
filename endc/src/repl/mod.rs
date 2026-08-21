use crate::agent_api::MicroEvaluator;
use crate::codegen::Interpreter;
use colored::*;
use std::io::{self, Write};

pub struct ReplEngine {
    history: Vec<String>,
    interpreter: Interpreter,
    environment_code: String,
}

impl ReplEngine {
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            interpreter: Interpreter::new(),
            environment_code: String::new(),
        }
    }

    pub fn start(&mut self) {
        println!("================================================================================");
        println!("👑 {} (Interactive Read-Eval-Print Loop v2.0)", "End Language REPL".green().bold());
        println!("  Type End expressions, function definitions, or commands.");
        println!("  Commands: {} | {} | {} | {} | {}", 
            ":help".cyan(), ":types".cyan(), ":load <file>".cyan(), ":bench <expr>".cyan(), ":exit".cyan()
        );
        println!("================================================================================");

        loop {
            print!("{} ", "end>".yellow().bold());
            let _ = io::stdout().flush();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            let trimmed = input.trim();
            if trimmed.is_empty() {
                continue;
            }

            if trimmed == ":exit" || trimmed == ":quit" || trimmed == "exit" {
                println!("👋 {}", "Goodbye from End REPL!".green());
                break;
            }

            if trimmed == ":help" {
                println!("👑 {} Commands:", "REPL".cyan().bold());
                println!("  :help          - Show this help message");
                println!("  :types         - List active types and symbols in REPL environment");
                println!("  :load <file>   - Load and evaluate an .end source file into REPL");
                println!("  :bench <expr>  - Benchmark an expression execution time in nanoseconds");
                println!("  :clear         - Reset the interactive REPL session memory");
                println!("  :exit          - Terminate REPL session");
                continue;
            }

            if trimmed == ":types" {
                println!("Active Symbols: [i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, str, bool, Result, Option, List, HashMap]");
                continue;
            }

            if trimmed == ":clear" {
                self.environment_code.clear();
                self.interpreter = Interpreter::new();
                println!("{} REPL session environment reset cleanly.", "✔".green());
                continue;
            }

            if trimmed.starts_with(":load ") {
                let file_path = trimmed[6..].trim();
                match std::fs::read_to_string(file_path) {
                    Ok(content) => {
                        self.environment_code.push_str(&content);
                        self.environment_code.push('\n');
                        println!("{} Loaded file `{}` into REPL environment.", "✔".green(), file_path.cyan());
                    }
                    Err(e) => {
                        println!("{} Failed to load `{}`: {}", "✖".red(), file_path, e);
                    }
                }
                continue;
            }

            if trimmed.starts_with(":bench ") {
                let expr = trimmed[7..].trim();
                let start = std::time::Instant::now();
                match MicroEvaluator::eval_expression(expr) {
                    Ok(report) => {
                        let duration = start.elapsed();
                        println!("⚡ {} Duration: {:?} | Result: {} ({})", "Benchmark:".green().bold(), duration, report.result.yellow(), report.value_type.cyan());
                    }
                    Err(e) => {
                        println!("{} {}", "Benchmark Error:".red().bold(), e);
                    }
                }
                continue;
            }

            // Normal Expression or Statement Evaluation
            self.history.push(trimmed.to_string());
            match MicroEvaluator::eval_expression(trimmed) {
                Ok(report) => {
                    println!("{} {} ({})", "=>".green().bold(), report.result.yellow().bold(), report.value_type.cyan());
                }
                Err(_) => {
                    // If it's a statement/function definition, append to environment
                    if trimmed.starts_with("fn ") || trimmed.starts_with("st ") || trimmed.starts_with("enum ") || trimmed.starts_with("val ") || trimmed.starts_with("mut ") {
                        self.environment_code.push_str(trimmed);
                        self.environment_code.push('\n');
                        println!("{} Defined in REPL session", "✔".green());
                    } else {
                        println!("{} Could not evaluate expression", "Error:".red().bold());
                    }
                }
            }
        }
    }
}
