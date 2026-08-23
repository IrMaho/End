use clap::Args;
use std::path::PathBuf;

    /// Flutter / Dart Native FFI Bridge and Widget Bindings Generator
        /// Action: bindgen or bridge
        /// Path to .end entrypoint file
        /// Output directory for generated Dart files (default: ./lib)
#[derive(Args, Debug, Clone)]
pub struct FlutterArgs {
        pub action: String,
        pub file: PathBuf,
        #[arg(short, long)]
        pub output: Option<PathBuf>,
}

        /// Path to .end entrypoint file or project directory
        /// Output documentation directory (default: ./docs)
        /// Documentation format: all, html, openapi, passport, markdown
        /// Start built-in local live documentation preview server
        /// Port for local preview server (default: 8080)
        /// Automatically open generated documentation in default web browser
#[derive(Args, Debug, Clone)]
pub struct DocArgs {
        pub file: PathBuf,
        #[arg(short, long)]
        pub output: Option<PathBuf>,
        #[arg(short, long, default_value = "all")]
        pub format: String,
        #[arg(long, default_value_t = false)]
        pub serve: bool,
        #[arg(long, default_value_t = 8080)]
        pub port: u16,
        #[arg(long, default_value_t = false)]
        pub open: bool,
}

    /// Create a new End language project with end.toml manifest and scaffold
        /// Project directory name
#[derive(Args, Debug, Clone)]
pub struct NewArgs {
        pub name: String,
}

    /// Add a dependency to the current project's end.toml
        /// Package name
#[derive(Args, Debug, Clone)]
pub struct AddArgs {
        pub package: String,
}

    /// Publish the current package distribution
        /// Validate package without uploading
        /// Store package in local repository (~/.end/local-registry)
#[derive(Args, Debug, Clone)]
pub struct PublishArgs {
        #[arg(long, default_value_t = false)]
        pub dry_run: bool,
        #[arg(long, default_value_t = false)]
        pub local: bool,
}

    /// Generate idiomatic FFI bindings for Python, TypeScript, Dart/Flutter, and C#/Unity
        /// Path to .end source file
        /// Output directory for generated bindings
        /// Target languages (comma-separated: python,typescript,dart,csharp)
        /// Name of the native library file (default: file basename)
#[derive(Args, Debug, Clone)]
pub struct BindgenArgs {
        pub file: PathBuf,
        #[arg(short, long, default_value = "bindings")]
        pub out_dir: PathBuf,
        #[arg(short, long, default_value = "python,typescript,dart,csharp")]
        pub target: String,
        #[arg(long)]
        pub lib_name: Option<String>,
}

    /// Parse C/C++ Header (.h/.hpp) and auto-generate typed End Language bindings
        /// Path to C header file (.h or .hpp)
        /// Output file for generated End bindings (default: <header_name>.end)
#[derive(Args, Debug, Clone)]
pub struct CBindgenArgs {
        pub header: PathBuf,
        #[arg(short, long)]
        pub out: Option<PathBuf>,
}

    /// Package End source files into native Android (.aar) or iOS (.xcframework) archives
        /// Platform target (android or ios)
        /// Path to .end source file
        /// Output package archive path (e.g. --out dist/app.aar or dist/App.xcframework)
#[derive(Args, Debug, Clone)]
pub struct MobileArgs {
        pub platform: String,
        pub file: PathBuf,
        #[arg(short, long)]
        pub out: Option<PathBuf>,
}

    /// Intelligent Architectural Scaffolder & Auto-Template Generator (Clean Architecture)
        /// Entity/Feature type (e.g. feature, entity, service)
        /// Name of the feature/module (e.g. auth, billing, payment)
        /// Architecture preset (clean_hexagonal, game_ecs, event_driven_microservice, layered_api)
#[derive(Args, Debug, Clone)]
pub struct GenArgs {
        pub generator_type: String,
        pub name: String,
        #[arg(short, long, default_value = "clean_hexagonal")]
        pub preset: String,
}

