use colored::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::agent_api::*;
use crate::architecture::*;
use crate::ast;
use crate::bindgen;
use crate::bindgen::*;
use crate::codegen;
use crate::codegen::*;
use crate::config::CompilerConfig;
use crate::dap;
use crate::dap::*;
use crate::diagnostics::Diagnostic;
use crate::docgen;
use crate::docgen::server::DocServer;
use crate::docgen::*;
use crate::formatter;
use crate::formatter::*;
use crate::fuzz;
use crate::fuzz::*;
use crate::lexer::Lexer;
use crate::linter;
use crate::linter::Linter;
use crate::loader::*;
use crate::lsp;
use crate::lsp::*;
use crate::mobile;
use crate::mobile::*;
use crate::package;
use crate::package::*;
use crate::parser::Parser as EndParser;
use crate::profiler;
use crate::repl;
use crate::repl::*;
use crate::runtime::*;
use crate::security;
use crate::security::*;
use crate::semantic::analyzer::SemanticAnalyzer;
use crate::semantic::*;
use crate::ui::feedback::FeedbackManager;
use crate::ui::*;

use crate::cli::agent_args::*;
use crate::cli::build_args::*;
use crate::cli::dev_args::*;
use crate::cli::package_args::*;
use crate::cli::query_args::*;
use crate::cli::security_args::*;

pub fn handle_new(args: NewArgs) {
    let NewArgs { name } = args;
            if let Err(e) = PackageManager::new_project(&name) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
}

pub fn handle_init() {
            if let Err(e) = PackageManager::init_project() {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
}

pub fn handle_add(args: AddArgs) {
    let AddArgs { package } = args;
            if let Err(e) = PackageManager::add_dependency(&package) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
}

pub fn handle_publish(args: PublishArgs) {
    let PublishArgs { dry_run, local } = args;
            if let Err(e) = PackageManager::publish_package(dry_run, local) {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
}

pub fn handle_install() {
            if let Err(e) = PackageManager::install_packages() {
                eprintln!("{} {}", "Error:".red().bold(), e);
                std::process::exit(1);
            }
}

pub fn handle_doc(args: DocArgs) {
    let DocArgs { file, output, format: _, serve, port, open } = args;
            let out_dir = output.unwrap_or_else(|| PathBuf::from("docs"));
            let (module, analyzer) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            let source = fs::read_to_string(&file).unwrap_or_default();
            match DocOrchestrator::generate_all(&module, &analyzer, &source, &out_dir) {
                Ok(index_path) => {
                    println!("✨ {} Generated Universal Documentation Suite at {:?}", "End Doc:".green().bold(), out_dir);
                    println!("  ├─ 🌐 {} (Swagger/Redoc Interactive Dashboard)", "index.html".cyan().bold());
                    println!("  ├─ 🔌 {} (OpenAPI v3.1.0 JSON Specification)", "openapi.json".cyan().bold());
                    println!("  ├─ 🛡️ {} (AI Agent & Compiler Passport)", "project_passport.json".cyan().bold());
                    println!("  └─ 📖 {} (GitHub Markdown Technical Reference)", "API_REFERENCE.md".cyan().bold());

                    if open {
                        #[cfg(target_os = "windows")]
                        let _ = std::process::Command::new("cmd").args(["/C", "start", &index_path.to_string_lossy()]).spawn();
                        #[cfg(target_os = "macos")]
                        let _ = std::process::Command::new("open").arg(&index_path).spawn();
                        #[cfg(target_os = "linux")]
                        let _ = std::process::Command::new("xdg-open").arg(&index_path).spawn();
                    }

                    if serve {
                        if let Err(e) = DocServer::serve(&out_dir, port) {
                            eprintln!("{} Failed to start doc server: {}", "Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
}

pub fn handle_bindgen(args: BindgenArgs) {
    let BindgenArgs { file, out_dir, target, lib_name } = args;
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            let default_lib = file.file_stem().and_then(|s| s.to_str()).unwrap_or("app");
            let effective_lib = lib_name.as_deref().unwrap_or(default_lib);
            let targets: Vec<String> = target.split(',').map(|s| s.trim().to_string()).collect();

            match UniversalBindgen::generate_all(&module, &out_dir, effective_lib, &targets) {
                Ok(files) => {
                    println!("🔌 {} Generated {} native bindings in {:?}", "Universal Bindgen:".green().bold(), files.len(), out_dir);
                    for f in files {
                        println!("  ✔ Created binding: {:?}", f);
                    }
                }
                Err(e) => {
                    eprintln!("{} Failed to generate bindings: {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
}

pub fn handle_cbindgen(args: CBindgenArgs) {
    let CBindgenArgs { header, out } = args;
            println!("🔍 {} Scanning and parsing C header at {:?}", "Universal C-Bindgen:".cyan().bold(), header);
            match bindgen::CHeaderParser::parse_header_file(&header) {
                Ok(generated_code) => {
                    let out_path = out.unwrap_or_else(|| {
                        let stem = header.file_stem().and_then(|s| s.to_str()).unwrap_or("c_bindings");
                        PathBuf::from(format!("{}.end", stem))
                    });
                    if let Some(parent) = out_path.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    if let Err(e) = fs::write(&out_path, &generated_code) {
                        eprintln!("❌ Failed to write generated bindings: {}", e);
                        std::process::exit(1);
                    }
                    println!("👑 {} Successfully generated native End module at {:?}", "Universal C-Bindgen:".green().bold(), out_path);
                }
                Err(e) => {
                    eprintln!("❌ C Header Bindgen error: {}", e);
                    std::process::exit(1);
                }
            }
}

pub fn handle_gen(args: GenArgs) {
    let GenArgs { generator_type, name, preset } = args;
            if generator_type.eq_ignore_ascii_case("feature") || generator_type.eq_ignore_ascii_case("module") {
                match ArchitectureEngine::scaffold_feature(&name, &preset, std::path::Path::new(".")) {
                    Ok(files) => {
                        println!("🤖 {} Generated Clean Architecture feature `{}` with preset `{}`:", "Architectural Scaffolder:".green().bold(), name.cyan().bold(), preset.yellow().bold());
                        for f in files {
                            println!("  ✔ Created: {:?}", f);
                        }
                    }
                    Err(e) => {
                        eprintln!("{} Failed to generate scaffold: {}", "Error:".red().bold(), e);
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("{} Unsupported generator type `{}`. Use 'feature'.", "Error:".red().bold(), generator_type);
                std::process::exit(1);
            }
}

pub fn handle_config_init() {
            let config_path = PathBuf::from("end.config.toml");
            if config_path.exists() {
                println!("{} `end.config.toml` already exists in this directory.", "ℹ".yellow().bold());
            } else {
                let default_content = r#"# 👑 End Language Compiler & Architecture Configuration
# Enforces modularity, line limits, code style, and architectural invariants

[architecture]
pattern = "clean"                    # Architectural pattern: clean | mvc | mvvm | hexagonal | none
enforce_layers = true                # Strict layer isolation
layers = ["domain", "data", "presentation"]

[files]
max_lines = 200                      # Maximum allowed lines per file (forces modularization)
max_functions_per_file = 10          # Maximum functions allowed per single file
max_function_lines = 50              # Maximum statements/lines per function
max_params = 5                       # Maximum parameters per function

[comments]
allowed = true                       # Allow comments in codebase
language = "any"                     # Comment language: en | fa | any
require_doc_comments = false         # Require documentation comments on pub functions

[naming]
struct_style = "PascalCase"          # Struct naming convention: PascalCase | snake_case
function_style = "snake_case"        # Function naming convention: snake_case | camelCase
variable_style = "snake_case"        # Variable naming convention: snake_case | camelCase

[quality]
max_cyclomatic_complexity = 10       # Maximum cyclomatic complexity per function (1-20)
no_dead_code = true                  # Warn/error on unused functions
no_unused_imports = true             # Warn/error on unused imports
"#;
                if let Err(e) = fs::write(&config_path, default_content) {
                    eprintln!("{} Failed to create `end.config.toml`: {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
                println!("👑 {} Generated `{}` with enterprise architectural guardrails!", "Config Init:".green().bold(), "end.config.toml".cyan().bold());
            }
}

pub fn handle_mobile(args: MobileArgs) {
    let MobileArgs { platform, file, out } = args;
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };

            let clean_platform = platform.to_lowercase();
            match clean_platform.as_str() {
                "android" | "aar" => {
                    let out_path = out.unwrap_or_else(|| PathBuf::from(format!("dist/{}.aar", module.name)));
                    match MobilePackager::package_android(&module, &out_path) {
                        Ok(p) => {
                            println!("📱 {} Packaged Android Archive (.aar) with 4 ABIs at {:?}", "Mobile Packager:".green().bold(), p);
                        }
                        Err(e) => {
                            eprintln!("{} Failed to package Android AAR: {}", "Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                }
                "ios" | "xcframework" => {
                    let out_path = out.unwrap_or_else(|| PathBuf::from(format!("dist/{}.xcframework", module.name)));
                    match MobilePackager::package_ios(&module, &out_path) {
                        Ok(p) => {
                            println!("📱 {} Packaged iOS (.xcframework) with SPM headers at {:?}", "Mobile Packager:".green().bold(), p);
                        }
                        Err(e) => {
                            eprintln!("{} Failed to package iOS XCFramework: {}", "Error:".red().bold(), e);
                            std::process::exit(1);
                        }
                    }
                }
                other => {
                    eprintln!("{} Unsupported mobile platform `{}`. Use 'android' or 'ios'.", "Error:".red().bold(), other);
                    std::process::exit(1);
                }
            }
}

pub fn handle_flutter(args: FlutterArgs) {
    let FlutterArgs { action: _, file, output } = args;
            let out_dir = output.unwrap_or_else(|| PathBuf::from("lib"));
            let (module, _) = match load_and_analyze(&file) {
                Ok(res) => res,
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            };
            match UiOrchestrator::generate_flutter_bridge(&module, &out_dir) {
                Ok(path) => {
                    println!("🐦 {} Generated Flutter / Dart FFI Bridge at {:?}", "Flutter Bridge:".green().bold(), path);
                }
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    std::process::exit(1);
                }
            }
}

