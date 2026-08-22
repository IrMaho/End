pub mod ir;
pub mod dap;
pub mod docgen;
pub mod formatter;
pub mod profiler;
pub mod runtime;
pub mod ui;
pub mod security;

pub(crate) mod agent_api;
pub(crate) mod architecture;
pub(crate) mod ast;
pub(crate) mod bindgen;
pub(crate) mod codegen;
pub(crate) mod config;
pub(crate) mod diagnostics;
pub(crate) mod fuzz;
pub(crate) mod lexer;
pub(crate) mod linter;
pub(crate) mod lsp;
pub(crate) mod mobile;
pub(crate) mod package;
pub(crate) mod parser;
pub(crate) mod repl;
pub(crate) mod semantic;

pub mod cli;
pub mod driver;
pub mod loader;

use clap::Parser;
use cli::Cli;

fn main() {
    let builder = std::thread::Builder::new().stack_size(16 * 1024 * 1024);
    let handler = builder
        .spawn(|| {
            run_app();
        })
        .unwrap();
    if let Err(e) = handler.join() {
        eprintln!("Fatal error in End compiler execution: {:?}", e);
        std::process::exit(1);
    }
}

fn run_app() {
    let cli = Cli::parse();
    driver::dispatch_command(cli.command);
}
