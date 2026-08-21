use super::passport::ProjectPassport;

pub struct MarkdownDocGenerator;

impl MarkdownDocGenerator {
    pub fn generate_api_reference(passport: &ProjectPassport) -> String {
        let mut md = String::new();
        md.push_str(&format!("# 📖 {} API & Technical Reference\n\n", passport.metadata.name));
        md.push_str(&format!("> **Compiler Engine:** End Language `{}`  \n", passport.metadata.compiler_version));
        md.push_str(&format!("> **Source Entrypoint:** `{}`  \n", passport.metadata.entry_file));
        md.push_str(&format!("> **Total Lines:** `{}` lines  \n\n", passport.metadata.total_lines));

        md.push_str("## ⚡ HTTP REST Endpoints (OpenAPI 3.1 Compatible)\n\n");
        if passport.endpoints.is_empty() {
            md.push_str("_No HTTP endpoints registered via directives._\n\n");
        } else {
            md.push_str("| Method | Path | Summary | Handler | Response Type |\n");
            md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
            for ep in &passport.endpoints {
                md.push_str(&format!("| **{}** | `{}` | {} | `{}` | `{}` |\n", ep.http_method, ep.path, ep.summary, ep.handler_name, ep.response_type));
            }
            md.push_str("\n");
        }

        md.push_str("## 📦 Struct Definitions & Memory Layout\n\n");
        for s in &passport.structs {
            md.push_str(&format!("### `st {}`\n", s.name));
            if !s.doc.is_empty() {
                md.push_str(&format!("*{}*\n\n", s.doc));
            }
            md.push_str(&format!("- **Total Memory Size:** `{}` Bytes\n", s.total_size_bytes));
            md.push_str(&format!("- **Hardware Alignment:** `{}` Bytes\n\n", s.alignment_bytes));
            md.push_str("| Offset | Field | Type | Size | Alignment |\n");
            md.push_str("| :--- | :--- | :--- | :--- | :--- |\n");
            for f in &s.fields {
                md.push_str(&format!("| `+{}B` | `{}` | `{}` | `{}B` | `{}B` |\n", f.byte_offset, f.name, f.field_type, f.byte_size, f.alignment));
            }
            md.push_str("\n");
        }

        md.push_str("## ⚡ Functions & Invariants\n\n");
        for f in &passport.functions {
            md.push_str(&format!("### `{}`\n", f.signature));
            md.push_str(&format!("- **Memory Safety Tier:** `{}`\n", f.memory_tier));
            md.push_str(&format!("- **Purity:** `{}`\n", f.purity));
            md.push_str(&format!("- **Capabilities:** `{}`\n", f.capabilities.join(", ")));
            if !f.invariants.is_empty() {
                md.push_str("- **Invariants:**\n");
                for inv in &f.invariants {
                    md.push_str(&format!("  - {}\n", inv));
                }
            }
            md.push_str("\n");
        }

        md
    }
}
