use super::passport::ProjectPassport;
use serde_json::Value;

pub struct HtmlUiGenerator;

impl HtmlUiGenerator {
    pub fn generate_dashboard_html(passport: &ProjectPassport, openapi_json: &Value) -> String {
        let passport_json_str = serde_json::to_string(passport).unwrap_or_default();
        let openapi_json_str = serde_json::to_string(openapi_json).unwrap_or_default();

        let total_symbols = passport.structs.len() + passport.functions.len() + passport.enums.len();

        let html_template = include_str!("template.html");
        html_template
            .replace("{title}", &passport.metadata.name)
            .replace("{version}", &passport.metadata.compiler_version)
            .replace("{entry_file}", &passport.metadata.entry_file)
            .replace("{total_lines}", &passport.metadata.total_lines.to_string())
            .replace("{total_symbols}", &total_symbols.to_string())
            .replace("{total_structs}", &passport.structs.len().to_string())
            .replace("{total_fns}", &passport.functions.len().to_string())
            .replace("{total_endpoints}", &passport.endpoints.len().to_string())
            .replace("{zero_overhead_pct}", &format!("{:.1}", passport.memory_safety_summary.zero_overhead_percentage))
            .replace("{concurrency_safe_pct}", &format!("{:.1}", passport.capability_summary.concurrency_safe_percentage))
            .replace("{passport_json_str}", &passport_json_str)
            .replace("{openapi_json_str}", &openapi_json_str)
    }
}
