// End Language: Real HTTP/2 (RFC 7540) & HPACK (RFC 7541) Engine
// Powered by h2, tokio, tokio-rustls, and rustls

use bytes::{Bytes, BytesMut};
use http::{Method, Request, Response, StatusCode, Uri, Version};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::net::SocketAddr;
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, AtomicI64, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime::Runtime;
use tokio::sync::Mutex;
use tokio_rustls::TlsAcceptor;

use crate::security::tls::{TestPki, TestServerMode, TlsServerConfigBuilder, TlsVersion};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Http2Error {
    ConnectionFailed(String),
    HandshakeFailed(String),
    StreamError(String),
    ProtocolError(String),
    TlsError(String),
    Timeout(String),
    HpackError(String),
}

impl fmt::Display for Http2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Http2Error::ConnectionFailed(msg) => write!(f, "HTTP/2 connection error: {}", msg),
            Http2Error::HandshakeFailed(msg) => write!(f, "HTTP/2 handshake error: {}", msg),
            Http2Error::StreamError(msg) => write!(f, "HTTP/2 stream error: {}", msg),
            Http2Error::ProtocolError(msg) => write!(f, "HTTP/2 protocol error: {}", msg),
            Http2Error::TlsError(msg) => write!(f, "HTTP/2 TLS/ALPN error: {}", msg),
            Http2Error::Timeout(msg) => write!(f, "HTTP/2 timeout: {}", msg),
            Http2Error::HpackError(msg) => write!(f, "HTTP/2 HPACK compression error: {}", msg),
        }
    }
}

impl std::error::Error for Http2Error {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Http2Response {
    pub status: u16,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub stream_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Http2RequestPayload {
    pub method: String,
    pub path: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub delay_ms: u64,
}

/// Real Multi-Stream HTTP/2 Server backed by the `h2` crate and Tokio async runtime
pub struct Http2Server {
    pub port: u16,
    pub is_tls: bool,
    pub is_running: Arc<AtomicBool>,
    pub total_requests: Arc<AtomicI64>,
    pub total_frames: Arc<AtomicI64>,
    pub active_streams: Arc<AtomicU32>,
    pub ca_cert_pem: Option<String>,
    pub server_cert_pem: Option<String>,
    runtime: Arc<Runtime>,
    shutdown_notify: Arc<tokio::sync::Notify>,
}

impl Http2Server {
    /// Start a real HTTP/2 server on the specified port (or port 0 for OS-assigned)
    pub fn start(port: u16, use_tls: bool) -> Result<Self, Http2Error> {
        let rt = Arc::new(Runtime::new().map_err(|e| {
            Http2Error::ConnectionFailed(format!("Failed to initialize Tokio runtime: {}", e))
        })?);

        let listener = rt.block_on(async {
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], port))).await
        }).map_err(|e| Http2Error::ConnectionFailed(format!("Failed to bind TCP port {}: {}", port, e)))?;

        let actual_port = listener.local_addr().map_err(|e| {
            Http2Error::ConnectionFailed(format!("Failed to get local address: {}", e))
        })?.port();

        let is_running = Arc::new(AtomicBool::new(true));
        let total_requests = Arc::new(AtomicI64::new(0));
        let total_frames = Arc::new(AtomicI64::new(0));
        let active_streams = Arc::new(AtomicU32::new(0));
        let shutdown_notify = Arc::new(tokio::sync::Notify::new());

        let mut ca_cert_pem = None;
        let mut server_cert_pem = None;
        let tls_acceptor = if use_tls {
            let pki = TestPki::generate(TestServerMode::Normal);
            ca_cert_pem = Some(pki.ca_cert_pem.clone());
            server_cert_pem = Some(pki.server_cert_pem.clone());

            let mut config_builder = TlsServerConfigBuilder::new();
            config_builder
                .set_cert_and_key_pem(&pki.server_cert_pem, &pki.server_key_pem)
                .map_err(|e| Http2Error::TlsError(format!("Failed to configure TLS cert/key: {}", e)))?
                .set_version(TlsVersion::All)
                .set_alpn(&["h2", "http/1.1"]);

            let server_config = config_builder
                .build()
                .map_err(|e| Http2Error::TlsError(format!("Failed to build TLS ServerConfig: {}", e)))?;

            Some(TlsAcceptor::from(server_config))
        } else {
            None
        };

        // Spawn background listener task
        let is_running_clone = Arc::clone(&is_running);
        let total_requests_clone = Arc::clone(&total_requests);
        let total_frames_clone = Arc::clone(&total_frames);
        let active_streams_clone = Arc::clone(&active_streams);
        let shutdown_clone = Arc::clone(&shutdown_notify);

        rt.spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_clone.notified() => {
                        break;
                    }
                    accept_res = listener.accept() => {
                        match accept_res {
                            Ok((tcp_stream, _peer_addr)) => {
                                let total_req = Arc::clone(&total_requests_clone);
                                let total_frm = Arc::clone(&total_frames_clone);
                                let act_strm = Arc::clone(&active_streams_clone);
                                let is_run = Arc::clone(&is_running_clone);

                                if let Some(ref acceptor) = tls_acceptor {
                                    let acceptor_clone = acceptor.clone();
                                    tokio::spawn(async move {
                                        match acceptor_clone.accept(tcp_stream).await {
                                            Ok(tls_stream) => {
                                                Self::serve_h2_connection(tls_stream, total_req, total_frm, act_strm, is_run).await;
                                            }
                                            Err(e) => {
                                                eprintln!("HTTP/2 TLS handshake error: {}", e);
                                            }
                                        }
                                    });
                                } else {
                                    tokio::spawn(async move {
                                        Self::serve_h2_connection(tcp_stream, total_req, total_frm, act_strm, is_run).await;
                                    });
                                }
                            }
                            Err(_) => {
                                if !is_running_clone.load(Ordering::Relaxed) {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(Self {
            port: actual_port,
            is_tls: use_tls,
            is_running,
            total_requests,
            total_frames,
            active_streams,
            ca_cert_pem,
            server_cert_pem,
            runtime: rt,
            shutdown_notify,
        })
    }

    async fn serve_h2_connection<IO>(
        io: IO,
        total_requests: Arc<AtomicI64>,
        total_frames: Arc<AtomicI64>,
        active_streams: Arc<AtomicU32>,
        is_running: Arc<AtomicBool>,
    ) where
        IO: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
    {
        let mut h2_conn = match h2::server::handshake(io).await {
            Ok(h) => h,
            Err(e) => {
                eprintln!("HTTP/2 wire handshake error: {}", e);
                return;
            }
        };

        total_frames.fetch_add(1, Ordering::SeqCst); // SETTINGS frame exchanged

        while let Some(result) = h2_conn.accept().await {
            if !is_running.load(Ordering::Relaxed) {
                break;
            }

            match result {
                Ok((request, mut respond)) => {
                    total_requests.fetch_add(1, Ordering::SeqCst);
                    total_frames.fetch_add(1, Ordering::SeqCst); // HEADERS frame received
                    active_streams.fetch_add(1, Ordering::SeqCst);

                    let act_strm_clone = Arc::clone(&active_streams);
                    let tot_frm_clone = Arc::clone(&total_frames);

                    tokio::spawn(async move {
                        let (parts, mut body_stream) = request.into_parts();
                        let mut body_bytes = Vec::new();

                        while let Some(chunk) = body_stream.data().await {
                            if let Ok(data) = chunk {
                                tot_frm_clone.fetch_add(1, Ordering::SeqCst); // DATA frame
                                body_bytes.extend_from_slice(&data);
                                let _ = body_stream.flow_control().release_capacity(data.len());
                            }
                        }

                        let path = parts.uri.path().to_string();
                        let req_body_str = String::from_utf8_lossy(&body_bytes).to_string();

                        // Formulate HTTP/2 response based on route
                        let (status, resp_body, content_type) = if path == "/ping" {
                            (StatusCode::OK, "pong".to_string(), "text/plain")
                        } else if path == "/echo" {
                            (StatusCode::OK, req_body_str, "text/plain")
                        } else if path.starts_with("/api/stream/") {
                            let stream_suffix = path.trim_start_matches("/api/stream/");
                            let json_resp = format!(
                                "{{\"stream\": \"{}\", \"status\": \"multiplexed\", \"received_len\": {}}}",
                                stream_suffix,
                                body_bytes.len()
                            );
                            (StatusCode::OK, json_resp, "application/json")
                        } else if path == "/headers" {
                            let mut map = serde_json::Map::new();
                            for (name, val) in parts.headers.iter() {
                                map.insert(
                                    name.as_str().to_string(),
                                    serde_json::Value::String(String::from_utf8_lossy(val.as_bytes()).to_string()),
                                );
                            }
                            let json = serde_json::Value::Object(map).to_string();
                            (StatusCode::OK, json, "application/json")
                        } else {
                            let default_resp = format!(
                                "{{\"server\": \"EndHyper/2.0\", \"protocol\": \"HTTP/2.0\", \"method\": \"{}\", \"path\": \"{}\", \"hpack_compressed\": true}}",
                                parts.method, path
                            );
                            (StatusCode::OK, default_resp, "application/json")
                        };

                        let response = Response::builder()
                            .status(status)
                            .version(Version::HTTP_2)
                            .header("content-type", content_type)
                            .header("server", "EndHyper/2.0 (h2)")
                            .header("x-end-protocol", "HTTP/2.0-HPACK")
                            .body(())
                            .unwrap();

                        tot_frm_clone.fetch_add(1, Ordering::SeqCst); // Response HEADERS frame

                        if let Ok(mut send_stream) = respond.send_response(response, false) {
                            tot_frm_clone.fetch_add(1, Ordering::SeqCst); // Response DATA frame
                            let _ = send_stream.send_data(Bytes::from(resp_body), true);
                        }

                        act_strm_clone.fetch_sub(1, Ordering::SeqCst);
                    });
                }
                Err(e) => {
                    eprintln!("HTTP/2 stream accept error: {}", e);
                    break;
                }
            }
        }
    }

    /// Stop the server and clean up tasks
    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
        self.shutdown_notify.notify_waiters();
    }
}

/// Real HTTP/2 Client backed by the `h2` crate and Tokio async runtime
pub struct Http2Client {
    pub url: String,
    pub is_connected: bool,
    runtime: Arc<Runtime>,
    send_request: Arc<Mutex<h2::client::SendRequest<Bytes>>>,
}

impl Http2Client {
    /// Connect to a real HTTP/2 server (supporting http:// or https:// with ALPN h2)
    pub fn connect(url_str: &str, custom_ca_pem: Option<&str>) -> Result<Self, Http2Error> {
        let rt = Arc::new(Runtime::new().map_err(|e| {
            Http2Error::ConnectionFailed(format!("Failed to initialize Tokio runtime: {}", e))
        })?);

        let uri: Uri = url_str.parse().map_err(|e| {
            Http2Error::ConnectionFailed(format!("Invalid URL '{}': {}", url_str, e))
        })?;

        let host = uri.host().unwrap_or("127.0.0.1").to_string();
        let is_https = uri.scheme_str() == Some("https");
        let port = uri.port_u16().unwrap_or(if is_https { 443 } else { 80 });

        let send_req = rt.block_on(async {
            let tcp_stream = TcpStream::connect((host.as_str(), port)).await.map_err(|e| {
                Http2Error::ConnectionFailed(format!("Failed to connect to {}:{}: {}", host, port, e))
            })?;

            if is_https {
                let mut config_builder = if let Some(ca) = custom_ca_pem {
                    let mut b = crate::security::tls::TlsClientConfigBuilder::with_empty_roots();
                    let _ = b.add_ca_pem(ca);
                    b
                } else {
                    crate::security::tls::TlsClientConfigBuilder::new()
                };
                config_builder.set_alpn(&["h2"]);
                let client_config = config_builder.build().map_err(|e| {
                    Http2Error::TlsError(format!("Failed to build TLS ClientConfig: {}", e))
                })?;

                let connector = tokio_rustls::TlsConnector::from(client_config);
                let server_name = rustls::pki_types::ServerName::try_from(host.clone()).unwrap_or_else(|_| {
                    rustls::pki_types::ServerName::try_from("localhost".to_string()).unwrap()
                });

                let tls_stream = connector.connect(server_name, tcp_stream).await.map_err(|e| {
                    Http2Error::TlsError(format!("TLS handshake failed: {}", e))
                })?;

                let (client, connection) = h2::client::handshake(tls_stream).await.map_err(|e| {
                    Http2Error::HandshakeFailed(format!("HTTP/2 client handshake error: {}", e))
                })?;

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("HTTP/2 TLS client connection error: {}", e);
                    }
                });

                Ok::<h2::client::SendRequest<Bytes>, Http2Error>(client)
            } else {
                let (client, connection) = h2::client::handshake(tcp_stream).await.map_err(|e| {
                    Http2Error::HandshakeFailed(format!("HTTP/2 client handshake error: {}", e))
                })?;

                tokio::spawn(async move {
                    if let Err(e) = connection.await {
                        eprintln!("HTTP/2 client connection error: {}", e);
                    }
                });

                Ok::<h2::client::SendRequest<Bytes>, Http2Error>(client)
            }
        })?;

        Ok(Self {
            url: url_str.to_string(),
            is_connected: true,
            runtime: rt,
            send_request: Arc::new(Mutex::new(send_req)),
        })
    }

    /// Execute a single real HTTP/2 request over the active connection
    pub fn request(
        &mut self,
        method: &str,
        path: &str,
        headers: &[(&str, &str)],
        body: &[u8],
    ) -> Result<Http2Response, Http2Error> {
        let uri: Uri = self.url.parse().map_err(|e| {
            Http2Error::ConnectionFailed(format!("Invalid URL: {}", e))
        })?;
        let host = uri.host().unwrap_or("localhost").to_string();

        let req_method = Method::from_bytes(method.as_bytes()).map_err(|e| {
            Http2Error::ProtocolError(format!("Invalid method '{}': {}", method, e))
        })?;

        let send_req_arc = Arc::clone(&self.send_request);
        let body_bytes = Bytes::copy_from_slice(body);
        let has_body = !body.is_empty();

        self.runtime.block_on(async {
            let send_req_clone = {
                let guard = send_req_arc.lock().await;
                guard.clone()
            };

            let mut ready_send = send_req_clone.ready().await.map_err(|e| {
                Http2Error::StreamError(format!("HTTP/2 client not ready: {}", e))
            })?;

            let is_https = self.url.starts_with("https");
            let full_uri = if path.starts_with("http://") || path.starts_with("https://") {
                path.to_string()
            } else {
                format!("{}://{}{}", if is_https { "https" } else { "http" }, host, path)
            };

            let mut builder = Request::builder()
                .method(req_method)
                .uri(full_uri)
                .version(Version::HTTP_2);

            for (k, v) in headers {
                if !k.starts_with(':') {
                    if let (Ok(name), Ok(val)) = (http::HeaderName::from_bytes(k.as_bytes()), http::HeaderValue::from_bytes(v.as_bytes())) {
                        builder = builder.header(name, val);
                    }
                }
            }

            let request = builder.body(()).map_err(|e| {
                Http2Error::ProtocolError(format!("Failed to build HTTP/2 request: {}", e))
            })?;

            let (response_fut, mut send_stream) = ready_send.send_request(request, !has_body).map_err(|e| {
                Http2Error::StreamError(format!("Failed to send HTTP/2 request: {}", e))
            })?;

            if has_body {
                send_stream.send_data(body_bytes, true).map_err(|e| {
                    Http2Error::StreamError(format!("Failed to send HTTP/2 request body: {}", e))
                })?;
            }

            let response = response_fut.await.map_err(|e| {
                Http2Error::StreamError(format!("HTTP/2 response future failed: {}", e))
            })?;

            let (parts, mut resp_body_stream) = response.into_parts();
            let mut resp_data = Vec::new();

            while let Some(chunk) = resp_body_stream.data().await {
                let data = chunk.map_err(|e| {
                    Http2Error::StreamError(format!("Failed to read HTTP/2 response body chunk: {}", e))
                })?;
                resp_data.extend_from_slice(&data);
                let _ = resp_body_stream.flow_control().release_capacity(data.len());
            }

            let mut resp_headers = HashMap::new();
            for (k, v) in parts.headers.iter() {
                resp_headers.insert(k.as_str().to_string(), String::from_utf8_lossy(v.as_bytes()).to_string());
            }

            Ok(Http2Response {
                status: parts.status.as_u16(),
                version: "HTTP/2.0".to_string(),
                headers: resp_headers,
                body: String::from_utf8_lossy(&resp_data).to_string(),
                stream_id: 1,
            })
        })
    }

    /// Execute 10+ concurrent multiplexed streams simultaneously over ONE connection
    pub fn request_multiplexed(
        &mut self,
        requests: &[Http2RequestPayload],
    ) -> Result<Vec<Http2Response>, Http2Error> {
        let uri: Uri = self.url.parse().map_err(|e| {
            Http2Error::ConnectionFailed(format!("Invalid URL: {}", e))
        })?;
        let host = uri.host().unwrap_or("localhost").to_string();
        let send_req_arc = Arc::clone(&self.send_request);

        self.runtime.block_on(async {
            let mut handles = Vec::new();

            for (idx, req_payload) in requests.iter().enumerate() {
                let send_req_clone = Arc::clone(&send_req_arc);
                let host_clone = host.clone();
                let payload = req_payload.clone();
                let stream_idx = (idx as u32) + 1;

                handles.push(tokio::spawn(async move {
                    if payload.delay_ms > 0 {
                        tokio::time::sleep(Duration::from_millis(payload.delay_ms)).await;
                    }

                    let send_req_instance = {
                        let guard = send_req_clone.lock().await;
                        guard.clone()
                    };

                    let mut ready_send = send_req_instance.ready().await.map_err(|e| {
                        Http2Error::StreamError(format!("Client not ready for stream {}: {}", stream_idx, e))
                    })?;

                    let is_https = host_clone.starts_with("https") || stream_idx > 0 && false;
                    let full_uri = if payload.path.starts_with("http://") || payload.path.starts_with("https://") {
                        payload.path.clone()
                    } else {
                        format!("http://{}{}", host_clone, payload.path)
                    };

                    let mut builder = Request::builder()
                        .method(payload.method.as_str())
                        .uri(full_uri)
                        .version(Version::HTTP_2);

                    for (k, v) in &payload.headers {
                        if !k.starts_with(':') {
                            builder = builder.header(k.as_str(), v.as_str());
                        }
                    }

                    let has_body = !payload.body.is_empty();
                    let request = builder.body(()).map_err(|e| {
                        Http2Error::ProtocolError(format!("Failed to build request {}: {}", stream_idx, e))
                    })?;

                    let (response_fut, mut send_stream) = ready_send.send_request(request, !has_body).map_err(|e| {
                        Http2Error::StreamError(format!("Failed to send stream {}: {}", stream_idx, e))
                    })?;

                    if has_body {
                        let bytes = Bytes::from(payload.body);
                        send_stream.send_data(bytes, true).map_err(|e| {
                            Http2Error::StreamError(format!("Failed to send data on stream {}: {}", stream_idx, e))
                        })?;
                    }

                    let response = response_fut.await.map_err(|e| {
                        Http2Error::StreamError(format!("Stream {} response error: {}", stream_idx, e))
                    })?;

                    let (parts, mut body_stream) = response.into_parts();
                    let mut resp_data = Vec::new();

                    while let Some(chunk) = body_stream.data().await {
                        let data = chunk.map_err(|e| {
                            Http2Error::StreamError(format!("Stream {} read chunk error: {}", stream_idx, e))
                        })?;
                        resp_data.extend_from_slice(&data);
                        let _ = body_stream.flow_control().release_capacity(data.len());
                    }

                    let mut headers_map = HashMap::new();
                    for (k, v) in parts.headers.iter() {
                        headers_map.insert(k.as_str().to_string(), String::from_utf8_lossy(v.as_bytes()).to_string());
                    }

                    Ok::<Http2Response, Http2Error>(Http2Response {
                        status: parts.status.as_u16(),
                        version: "HTTP/2.0".to_string(),
                        headers: headers_map,
                        body: String::from_utf8_lossy(&resp_data).to_string(),
                        stream_id: stream_idx,
                    })
                }));
            }

            let mut results = Vec::new();
            for handle in handles {
                let res = handle.await.map_err(|e| {
                    Http2Error::StreamError(format!("Join error on concurrent stream: {}", e))
                })??;
                results.push(res);
            }

            Ok(results)
        })
    }

    pub fn close(&mut self) {
        self.is_connected = false;
    }
}

/// Real HPACK (RFC 7541) Header Compression & Decompression Codec
pub struct HpackCodec;

impl HpackCodec {
    /// Encode a key-value header list into HPACK binary byte representation
    pub fn encode(headers: &[(&str, &str)]) -> Result<Vec<u8>, Http2Error> {
        let mut encoder = hpack::Encoder::new();
        let mut raw_headers = Vec::new();
        for (k, v) in headers {
            raw_headers.push((k.as_bytes(), v.as_bytes()));
        }
        let encoded = encoder.encode(raw_headers);
        Ok(encoded)
    }

    /// Decode an HPACK binary byte representation into key-value headers
    pub fn decode(hpack_bytes: &[u8]) -> Result<Vec<(String, String)>, Http2Error> {
        let mut decoder = hpack::Decoder::new();
        let decoded = decoder.decode(hpack_bytes).map_err(|e| {
            Http2Error::HpackError(format!("HPACK decoding failed: {:?}", e))
        })?;

        let mut results = Vec::new();
        for (k, v) in decoded {
            let key = String::from_utf8_lossy(&k).to_string();
            let val = String::from_utf8_lossy(&v).to_string();
            results.push((key, val));
        }

        Ok(results)
    }
}
