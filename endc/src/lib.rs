pub mod ir;
pub mod dap;
pub mod docgen;
pub mod formatter;
pub mod profiler;
pub mod runtime;
pub mod ui;
pub mod security;

pub mod agent_api;
pub mod agent;
pub mod architecture;
pub mod ast;
pub mod bindgen;
pub mod codegen;
pub mod config;
pub mod diagnostics;
pub mod fuzz;
pub mod lexer;
pub mod linter;
pub mod lsp;
pub mod mobile;
pub mod package;
pub mod parser;
pub mod repl;
pub mod semantic;

pub mod cli;
pub mod driver;
pub mod loader;

pub fn run_app() {
    use clap::Parser;
    let cli = cli::Cli::parse();
    driver::dispatch_command(cli.command, cli.release);
}
