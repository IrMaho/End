use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::Instant;

fn xorshift_compute(iterations: u64) -> u64 {
    let mut state: u64 = 0x853c49e6748fea9b;
    for _ in 0..iterations {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    }
    state
}

fn send_response(stream: &mut std::net::TcpStream, status: &str, body: &str) {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: {}\r\n\r\n{}",
        status,
        body.len(),
        body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
    let _ = stream.shutdown(std::net::Shutdown::Both);
}

fn handle_health(stream: &mut std::net::TcpStream) {
    send_response(stream, "200 OK", r#"{"status":"ok","lang":"Rust 1.89.0"}"#);
}

fn handle_compute(stream: &mut std::net::TcpStream, path: &str) {
    let mut n: u64 = 1_000_000;
    if let Some(pos) = path.find("n=") {
        if let Ok(parsed) = path[pos + 2..].split('&').next().unwrap_or("").parse::<u64>() {
            n = parsed;
        }
    }
    let t0 = Instant::now();
    let hash = xorshift_compute(n);
    let time_us = t0.elapsed().as_micros();
    let body = format!(r#"{{"hash":{},"time_us":{},"lang":"Rust 1.89.0"}}"#, hash, time_us);
    send_response(stream, "200 OK", &body);
}

fn handle_json(stream: &mut std::net::TcpStream) {
    let body = r#"{"server":"Rust HTTP Backend","version":"1.89.0","users":[{"id":1,"name":"Alice","score":9850,"active":true},{"id":2,"name":"Bob","score":8720,"active":true},{"id":3,"name":"Charlie","score":7630,"active":false},{"id":4,"name":"Diana","score":9210,"active":true},{"id":5,"name":"Eve","score":8890,"active":true}],"metadata":{"total_users":5,"avg_score":8860,"active_count":4,"server_uptime":99.97}}"#;
    send_response(stream, "200 OK", body);
}

fn main() {
    let listener = TcpListener::bind("0.0.0.0:9003").expect("Failed to bind port 9003");
    println!("[Rust] HTTP Server listening on :9003");

    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            let mut buf = [0u8; 4096];
            if let Ok(n) = stream.read(&mut buf) {
                if n == 0 { continue; }
                let request = String::from_utf8_lossy(&buf[..n]);
                let first_line: &str = request.lines().next().unwrap_or("");
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                let path = if parts.len() >= 2 { parts[1] } else { "/" };

                if path.contains("/health") {
                    handle_health(&mut stream);
                } else if path.contains("/compute") {
                    handle_compute(&mut stream, path);
                } else if path.contains("/json") {
                    handle_json(&mut stream);
                } else {
                    send_response(&mut stream, "404 Not Found", r#"{"error":"not found"}"#);
                }
            }
        }
    }
}
