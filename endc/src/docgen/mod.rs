pub mod passport;
pub mod passport_types;
pub mod openapi;
pub mod html_ui;
pub mod markdown;
pub mod server;

use crate::ast::Module;
use crate::semantic::analyzer::SemanticAnalyzer;
use passport::PassportBuilder;
use openapi::OpenApiGenerator;
use html_ui::HtmlUiGenerator;
use markdown::MarkdownDocGenerator;
use std::fs;
use std::path::{Path, PathBuf};

pub struct DocOrchestrator;

impl DocOrchestrator {
    pub fn generate_all(
        module: &Module,
        analyzer: &SemanticAnalyzer,
        source: &str,
        output_dir: &Path,
    ) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|e| format!("Failed to create docs directory {:?}: {}", output_dir, e))?;

        // 1. Build Project Passport
        let passport = PassportBuilder::build(module, analyzer, source);
        let passport_json = serde_json::to_string_pretty(&passport).map_err(|e| e.to_string())?;
        fs::write(output_dir.join("project_passport.json"), passport_json)
            .map_err(|e| format!("Failed to write project_passport.json: {}", e))?;

        // 2. Build OpenAPI v3.1 Specification
        let openapi_val = OpenApiGenerator::generate_openapi_v3(&passport);
        let openapi_json = serde_json::to_string_pretty(&openapi_val).map_err(|e| e.to_string())?;
        fs::write(output_dir.join("openapi.json"), openapi_json)
            .map_err(|e| format!("Failed to write openapi.json: {}", e))?;

        // 3. Build Markdown Reference & Passport
        let md_reference = MarkdownDocGenerator::generate_api_reference(&passport);
        fs::write(output_dir.join("API_REFERENCE.md"), md_reference)
            .map_err(|e| format!("Failed to write API_REFERENCE.md: {}", e))?;

        // 4. Build Standalone Interactive Dashboard HTML
        let html_dashboard = HtmlUiGenerator::generate_dashboard_html(&passport, &openapi_val);
        let index_html_path = output_dir.join("index.html");
        fs::write(&index_html_path, html_dashboard)
            .map_err(|e| format!("Failed to write index.html: {}", e))?;

        Ok(index_html_path)
    }
}
