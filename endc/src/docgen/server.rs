use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::Path;
use colored::*;

pub struct DocServer;

impl DocServer {
    pub fn serve(docs_dir: &Path, port: u16) -> Result<(), String> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = TcpListener::bind(&addr).map_err(|e| format!("Failed to bind to {}: {}", addr, e))?;

        println!("⚡ {} Documentation Server running live at {}", "End Doc:".green().bold(), format!("http://{}", addr).cyan().underline().bold());
        println!("ℹ Press Ctrl+C to terminate documentation server.\n");

        for stream in listener.incoming() {
            match stream {
                Ok(mut s) => {
                    let mut buffer = [0u8; 2048];
                    let _ = s.read(&mut buffer);
                    let req_str = String::from_utf8_lossy(&buffer);

                    let path = if let Some(first_line) = req_str.lines().next() {
                        let parts: Vec<&str> = first_line.split_whitespace().collect();
                        if parts.len() >= 2 { parts[1] } else { "/" }
                    } else {
                        "/"
                    };

                    let (file_name, mime_type) = if path == "/" || path == "/index.html" {
                        ("index.html", "text/html; charset=utf-8")
                    } else if path == "/openapi.json" {
                        ("openapi.json", "application/json")
                    } else if path == "/project_passport.json" {
                        ("project_passport.json", "application/json")
                    } else if path == "/API_REFERENCE.md" {
                        ("API_REFERENCE.md", "text/markdown; charset=utf-8")
                    } else if path == "/PASSPORT.md" {
                        ("PASSPORT.md", "text/markdown; charset=utf-8")
                    } else {
                        ("index.html", "text/html; charset=utf-8")
                    };

                    let file_path = docs_dir.join(file_name);
                    if let Ok(content) = fs::read_to_string(&file_path) {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}",
                            mime_type,
                            content.len(),
                            content
                        );
                        let _ = s.write_all(response.as_bytes());
                    } else {
                        let not_found = "HTTP/1.1 404 NOT FOUND\r\nContent-Length: 9\r\n\r\nNot Found";
                        let _ = s.write_all(not_found.as_bytes());
                    }
                    let _ = s.flush();
                }
                Err(e) => {
                    eprintln!("Connection error: {}", e);
                }
            }
        }

        Ok(())
    }
}
