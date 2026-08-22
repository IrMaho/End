// End Compiler: src/config.rs
// Project Configuration Parser — reads end.config.toml and provides validated CompilerConfig

use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct CompilerConfig {
    /// Architecture configuration — deserialized from `end.config.toml`.
    /// Currently loaded and validated but enforcement is gated behind `end arch check`.
    #[serde(default)]
    #[allow(dead_code)]
    pub architecture: ArchitectureConfig,
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default)]
    pub comments: CommentsConfig,
    #[serde(default)]
    pub naming: NamingConfig,
    #[serde(default)]
    pub quality: QualityConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ArchitectureConfig {
    #[serde(default = "default_pattern")]
    #[allow(dead_code)]
    pub pattern: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub enforce_layers: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub layers: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilesConfig {
    #[serde(default = "default_max_lines")]
    pub max_lines: usize,
    #[serde(default = "default_max_functions")]
    pub max_functions_per_file: usize,
    #[serde(default = "default_max_fn_lines")]
    pub max_function_lines: usize,
    #[serde(default = "default_max_params")]
    pub max_params: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommentsConfig {
    #[serde(default = "default_true")]
    pub allowed: bool,
    #[serde(default = "default_lang_any")]
    pub language: String,
    #[serde(default)]
    pub require_doc_comments: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NamingConfig {
    #[serde(default = "default_pascal")]
    pub struct_style: String,
    #[serde(default = "default_snake")]
    pub function_style: String,
    #[serde(default = "default_snake")]
    #[allow(dead_code)]
    pub variable_style: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QualityConfig {
    #[serde(default = "default_complexity")]
    pub max_cyclomatic_complexity: usize,
    #[serde(default)]
    #[allow(dead_code)]
    pub no_dead_code: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub no_unused_imports: bool,
}

// Default value functions for serde
fn default_pattern() -> String { "none".to_string() }
fn default_max_lines() -> usize { 500 }
fn default_max_functions() -> usize { 20 }
fn default_max_fn_lines() -> usize { 100 }
fn default_max_params() -> usize { 8 }
fn default_true() -> bool { true }
fn default_lang_any() -> String { "any".to_string() }
fn default_pascal() -> String { "PascalCase".to_string() }
fn default_snake() -> String { "snake_case".to_string() }
fn default_complexity() -> usize { 15 }

impl Default for CompilerConfig {
    fn default() -> Self {
        CompilerConfig {
            architecture: ArchitectureConfig::default(),
            files: FilesConfig::default(),
            comments: CommentsConfig::default(),
            naming: NamingConfig::default(),
            quality: QualityConfig::default(),
        }
    }
}

impl Default for ArchitectureConfig {
    fn default() -> Self {
        ArchitectureConfig {
            pattern: default_pattern(),
            enforce_layers: false,
            layers: Vec::new(),
        }
    }
}

impl Default for FilesConfig {
    fn default() -> Self {
        FilesConfig {
            max_lines: default_max_lines(),
            max_functions_per_file: default_max_functions(),
            max_function_lines: default_max_fn_lines(),
            max_params: default_max_params(),
        }
    }
}

impl Default for CommentsConfig {
    fn default() -> Self {
        CommentsConfig {
            allowed: true,
            language: default_lang_any(),
            require_doc_comments: false,
        }
    }
}

impl Default for NamingConfig {
    fn default() -> Self {
        NamingConfig {
            struct_style: default_pascal(),
            function_style: default_snake(),
            variable_style: default_snake(),
        }
    }
}

impl Default for QualityConfig {
    fn default() -> Self {
        QualityConfig {
            max_cyclomatic_complexity: default_complexity(),
            no_dead_code: false,
            no_unused_imports: false,
        }
    }
}

impl CompilerConfig {
    pub fn load_from_project(project_dir: &Path) -> Self {
        let config_path = project_dir.join("end.config.toml");
        if config_path.exists() {
            match fs::read_to_string(&config_path) {
                Ok(content) => {
                    match toml::from_str::<CompilerConfig>(&content) {
                        Ok(config) => config,
                        Err(e) => {
                            eprintln!("⚠ Warning: Failed to parse end.config.toml: {}", e);
                            CompilerConfig::default()
                        }
                    }
                }
                Err(e) => {
                    eprintln!("⚠ Warning: Failed to read end.config.toml: {}", e);
                    CompilerConfig::default()
                }
            }
        } else {
            CompilerConfig::default()
        }
    }
}
