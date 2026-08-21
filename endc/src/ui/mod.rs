pub mod widget;
pub mod feedback;
pub mod dev_overlay;
pub mod html_renderer;
pub mod flutter_bridge;

use crate::ast::Module;
use widget::WidgetTreeExtractor;
use html_renderer::HtmlUiRenderer;
use feedback::{FeedbackManager, FeedbackItem};
use flutter_bridge::FlutterBridgeGenerator;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use colored::*;

pub struct UiOrchestrator;

impl UiOrchestrator {
    pub fn build_ui(
        module: &Module,
        output_dir: &Path,
        is_dev_mode: bool,
    ) -> Result<PathBuf, String> {
        fs::create_dir_all(output_dir).map_err(|e| e.to_string())?;

        let base_project_dir = Path::new(".");
        let feedback_items = FeedbackManager::list_all(base_project_dir);
        let feedback_json_str = serde_json::to_string(&feedback_items).unwrap_or_else(|_| "[]".to_string());

        let root_widget = WidgetTreeExtractor::extract_from_module(module);
        let html_content = HtmlUiRenderer::render_to_html(&root_widget, is_dev_mode, &feedback_json_str);

        let index_html_path = output_dir.join("index.html");
        fs::write(&index_html_path, html_content).map_err(|e| e.to_string())?;

        Ok(index_html_path)
    }

    pub fn serve_ui(output_dir: &Path, port: u16) -> Result<(), String> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).map_err(|e| format!("Failed to bind port {}: {}", port, e))?;

        println!("⚡ {} EndUI Interactive DevServer running at {}", "EndUI:".green().bold(), format!("http://{}", addr).cyan().underline().bold());
        println!("🤖 AI Agent DevMode Canvas Overlay: {} | Press Ctrl+C to stop.\n", "ACTIVE".green().bold());

        for stream in listener.incoming() {
            if let Ok(mut s) = stream {
                let mut buf = [0u8; 8192];
                let _ = s.read(&mut buf);
                let req_str = String::from_utf8_lossy(&buf);

                if req_str.starts_with("POST /api/feedback") {
                    // Extract body
                    if let Some(body_start) = req_str.find("\r\n\r\n") {
                        let body = &req_str[body_start + 4..];
                        if let Ok(item) = serde_json::from_str::<FeedbackItem>(body.trim_end_matches('\0')) {
                            let _ = FeedbackManager::save_item(Path::new("."), &item);
                            let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\"}";
                            let _ = s.write_all(resp.as_bytes());
                            continue;
                        }
                    }
                }

                let html_path = output_dir.join("index.html");
                if let Ok(content) = fs::read_to_string(&html_path) {
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
                        content.len(),
                        content
                    );
                    let _ = s.write_all(resp.as_bytes());
                }
                let _ = s.flush();
            }
        }

        Ok(())
    }

    pub fn generate_flutter_bridge(module: &Module, out_dir: &Path) -> Result<PathBuf, String> {
        fs::create_dir_all(out_dir).map_err(|e| e.to_string())?;
        let dart_code = FlutterBridgeGenerator::generate_dart_bridge(module);
        let out_file = out_dir.join("end_flutter_bridge.dart");
        fs::write(&out_file, dart_code).map_err(|e| e.to_string())?;
        Ok(out_file)
    }
}
