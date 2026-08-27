use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::time::sleep;

pub struct TestHttpServer {
    pub addr: SocketAddr,
    shutdown_flag: Arc<AtomicBool>,
}

impl TestHttpServer {
    pub fn start() -> Result<Self, String> {
        let std_listener = std::net::TcpListener::bind("127.0.0.1:0")
            .map_err(|e| format!("Failed to bind test server: {}", e))?;
        std_listener
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set nonblocking: {}", e))?;
        let addr = std_listener
            .local_addr()
            .map_err(|e| format!("Failed to get local addr: {}", e))?;
        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let flag_clone = Arc::clone(&shutdown_flag);

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async move {
                let listener = TcpListener::from_std(std_listener).unwrap();
                while !flag_clone.load(Ordering::Relaxed) {
                    tokio::select! {
                        accept_res = listener.accept() => {
                            if let Ok((mut socket, _)) = accept_res {
                                tokio::spawn(async move {
                                    let mut buf = [0u8; 4096];
                                    loop {
                                        let n = match socket.read(&mut buf).await {
                                            Ok(0) => break,
                                            Ok(n) => n,
                                            Err(_) => break,
                                        };

                                        let req_str = String::from_utf8_lossy(&buf[..n]);
                                        let first_line = req_str.lines().next().unwrap_or("");
                                        let path = if let Some(p) = first_line.split_whitespace().nth(1) {
                                            p
                                        } else {
                                            "/"
                                        };

                                        let (status_line, body) = if path.starts_with("/slow") {
                                            sleep(Duration::from_millis(50)).await;
                                            ("HTTP/1.1 200 OK", "slow response")
                                        } else if path.starts_with("/error") {
                                            ("HTTP/1.1 500 Internal Server Error", "error response")
                                        } else if path.starts_with("/custom_delay") {
                                            let delay_ms = 30;
                                            sleep(Duration::from_millis(delay_ms)).await;
                                            ("HTTP/1.1 200 OK", "custom delay response")
                                        } else {
                                            ("HTTP/1.1 200 OK", "fast response")
                                        };

                                        let response = format!(
                                            "{}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: keep-alive\r\n\r\n{}",
                                            status_line,
                                            body.len(),
                                            body
                                        );

                                        if socket.write_all(response.as_bytes()).await.is_err() {
                                            break;
                                        }
                                    }
                                });
                            }
                        }
                        _ = tokio::time::sleep(Duration::from_millis(100)) => {
                            if flag_clone.load(Ordering::Relaxed) {
                                break;
                            }
                        }
                    }
                }
            });
        });

        std::thread::sleep(Duration::from_millis(20));

        Ok(Self {
            addr,
            shutdown_flag,
        })
    }

    pub fn url(&self, path: &str) -> String {
        let p = if path.starts_with('/') {
            path
        } else {
            &format!("/{}", path)
        };
        format!("http://{}:{}{}", self.addr.ip(), self.addr.port(), p)
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn shutdown(&self) {
        self.shutdown_flag.store(true, Ordering::SeqCst);
    }
}

impl Drop for TestHttpServer {
    fn drop(&mut self) {
        self.shutdown();
    }
}
