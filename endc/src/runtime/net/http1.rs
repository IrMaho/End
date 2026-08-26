use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// HTTP/1.x Methods (RFC 7231 §4)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Http1Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Options,
    Trace,
    Patch,
    Custom(String),
}

impl Http1Method {
    pub fn from_str(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "GET" => Http1Method::Get,
            "HEAD" => Http1Method::Head,
            "POST" => Http1Method::Post,
            "PUT" => Http1Method::Put,
            "DELETE" => Http1Method::Delete,
            "OPTIONS" => Http1Method::Options,
            "TRACE" => Http1Method::Trace,
            "PATCH" => Http1Method::Patch,
            _ => Http1Method::Custom(s.to_string()),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Http1Method::Get => "GET",
            Http1Method::Head => "HEAD",
            Http1Method::Post => "POST",
            Http1Method::Put => "PUT",
            Http1Method::Delete => "DELETE",
            Http1Method::Options => "OPTIONS",
            Http1Method::Trace => "TRACE",
            Http1Method::Patch => "PATCH",
            Http1Method::Custom(s) => s.as_str(),
        }
    }
}

/// HTTP/1.x Error representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Http1Error {
    IncompleteRequest,
    BadRequest(String),
    MalformedHeader(String),
    MissingHostHeader,
    UnsupportedVersion(String),
    InvalidContentLength,
    InvalidChunk(String),
    ConnectionClosed,
    Io(String),
}

impl std::fmt::Display for Http1Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Http1Error::IncompleteRequest => write!(f, "Incomplete HTTP request received"),
            Http1Error::BadRequest(msg) => write!(f, "400 Bad Request: {}", msg),
            Http1Error::MalformedHeader(msg) => write!(f, "Malformed Header: {}", msg),
            Http1Error::MissingHostHeader => write!(f, "400 Bad Request: Missing Host header in HTTP/1.1 request (RFC 7230 §5.4)"),
            Http1Error::UnsupportedVersion(v) => write!(f, "505 HTTP Version Not Supported: {}", v),
            Http1Error::InvalidContentLength => write!(f, "Invalid Content-Length header"),
            Http1Error::InvalidChunk(msg) => write!(f, "Invalid Chunked encoding: {}", msg),
            Http1Error::ConnectionClosed => write!(f, "Connection closed by peer"),
            Http1Error::Io(err) => write!(f, "I/O Error: {}", err),
        }
    }
}

/// HTTP/1.x Request (RFC 7230 §3)
#[derive(Debug, Clone)]
pub struct Http1Request {
    pub method: Http1Method,
    pub uri: String,
    pub path: String,
    pub query: Option<String>,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Http1Request {
    /// Retrieves a header with case-insensitive key lookup (RFC 7230 §3.2)
    pub fn header(&self, name: &str) -> Option<&str> {
        let needle = name.to_ascii_lowercase();
        for (k, v) in &self.headers {
            if k.to_ascii_lowercase() == needle {
                return Some(v.as_str());
            }
        }
        None
    }

    /// Determines if the connection should persist (RFC 7230 §6.3)
    pub fn should_keep_alive(&self) -> bool {
        if let Some(conn) = self.header("connection") {
            if conn.to_ascii_lowercase().contains("close") {
                return false;
            }
            if conn.to_ascii_lowercase().contains("keep-alive") {
                return true;
            }
        }
        // In HTTP/1.1, connections are persistent by default unless "close" is specified.
        // In HTTP/1.0, connections are closed by default unless "keep-alive" is specified.
        self.version == "HTTP/1.1"
    }

    /// Parses raw bytes into an Http1Request, returning (Request, bytes_consumed)
    pub fn parse(buffer: &[u8]) -> Result<(Http1Request, usize), Http1Error> {
        // Find CRLF CRLF delimiter
        let header_end = if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
            pos
        } else if let Some(pos) = buffer.windows(2).position(|w| w == b"\n\n") {
            pos
        } else {
            return Err(Http1Error::IncompleteRequest);
        };

        let delimiter_len = if buffer.windows(4).position(|w| w == b"\r\n\r\n").is_some() { 4 } else { 2 };
        let header_bytes = &buffer[..header_end];
        let header_str = std::str::from_utf8(header_bytes)
            .map_err(|_| Http1Error::BadRequest("Headers contain non-UTF8 bytes".to_string()))?;

        let mut lines = header_str.split("\r\n");
        let first_line = lines.next().unwrap_or("");
        let first_line = if first_line.is_empty() {
            header_str.split('\n').next().unwrap_or("")
        } else {
            first_line
        };

        // Parse Request-Line: Method SP Request-URI SP HTTP-Version
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 3 {
            return Err(Http1Error::BadRequest(format!(
                "Malformed request line '{}': expected 'METHOD URI HTTP-VERSION'",
                first_line
            )));
        }

        let method = Http1Method::from_str(parts[0]);
        let uri = parts[1].to_string();
        let version = parts[2].to_ascii_uppercase();

        if !version.starts_with("HTTP/1.") {
            return Err(Http1Error::UnsupportedVersion(version));
        }

        // Split URI into path and query
        let (path, query) = if let Some(q_idx) = uri.find('?') {
            (uri[..q_idx].to_string(), Some(uri[q_idx + 1..].to_string()))
        } else {
            (uri.clone(), None)
        };

        // Parse headers
        let mut headers = HashMap::new();
        let header_lines = header_str.lines().skip(1);
        for line in header_lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Some(colon_idx) = trimmed.find(':') {
                let key = trimmed[..colon_idx].trim().to_string();
                let val = trimmed[colon_idx + 1..].trim().to_string();
                headers.insert(key, val);
            } else {
                return Err(Http1Error::MalformedHeader(format!("Invalid header line: '{}'", trimmed)));
            }
        }

        // Enforce RFC 7230 §5.4: HTTP/1.1 requires Host header
        if version == "HTTP/1.1" {
            let has_host = headers.keys().any(|k| k.eq_ignore_ascii_case("host"));
            if !has_host {
                return Err(Http1Error::MissingHostHeader);
            }
        }

        let body_start = header_end + delimiter_len;
        let mut body = Vec::new();
        let mut bytes_consumed = body_start;

        // Check for Transfer-Encoding: chunked (RFC 7230 §4.1)
        let is_chunked = headers.keys().any(|k| {
            k.eq_ignore_ascii_case("transfer-encoding")
                && headers[k].to_ascii_lowercase().contains("chunked")
        });

        if is_chunked {
            let mut cursor = body_start;
            loop {
                let remaining = &buffer[cursor..];
                let crlf_pos = match remaining.windows(2).position(|w| w == b"\r\n") {
                    Some(p) => p,
                    None => return Err(Http1Error::IncompleteRequest),
                };
                let size_str = std::str::from_utf8(&remaining[..crlf_pos])
                    .map_err(|_| Http1Error::InvalidChunk("Invalid UTF-8 in chunk size".to_string()))?;
                let chunk_size = usize::from_str_radix(size_str.trim().split(';').next().unwrap_or(""), 16)
                    .map_err(|e| Http1Error::InvalidChunk(format!("Failed to parse chunk hex size '{}': {}", size_str, e)))?;

                let chunk_data_start = cursor + crlf_pos + 2;
                if chunk_size == 0 {
                    // Last chunk (0\r\n\r\n)
                    let final_remaining = &buffer[chunk_data_start..];
                    if final_remaining.len() < 2 {
                        return Err(Http1Error::IncompleteRequest);
                    }
                    bytes_consumed = chunk_data_start + 2; // skip final \r\n
                    break;
                }

                let chunk_data_end = chunk_data_start + chunk_size;
                if buffer.len() < chunk_data_end + 2 {
                    return Err(Http1Error::IncompleteRequest);
                }

                body.extend_from_slice(&buffer[chunk_data_start..chunk_data_end]);
                cursor = chunk_data_end + 2; // skip chunk data \r\n
            }
        } else if let Some(cl_val) = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("content-length")).map(|(_, v)| v) {
            let content_len: usize = cl_val
                .trim()
                .parse()
                .map_err(|_| Http1Error::InvalidContentLength)?;

            if buffer.len() < body_start + content_len {
                return Err(Http1Error::IncompleteRequest);
            }
            body.extend_from_slice(&buffer[body_start..body_start + content_len]);
            bytes_consumed = body_start + content_len;
        }

        Ok((
            Http1Request {
                method,
                uri,
                path,
                query,
                version,
                headers,
                body,
            },
            bytes_consumed,
        ))
    }
}

/// HTTP/1.x Response (RFC 7230 §3.1.2)
#[derive(Debug, Clone)]
pub struct Http1Response {
    pub status: u16,
    pub reason: String,
    pub headers: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Http1Response {
    pub fn new(status: u16, body: impl Into<Vec<u8>>) -> Self {
        let reason = Self::default_reason_phrase(status).to_string();
        let mut headers = HashMap::new();
        headers.insert("Server".to_string(), "End-Native-HTTP/1.1".to_string());
        headers.insert("Content-Type".to_string(), "text/plain; charset=utf-8".to_string());
        Self {
            status,
            reason,
            headers,
            body: body.into(),
        }
    }

    pub fn ok_json(json: &str) -> Self {
        let mut res = Self::new(200, json.as_bytes());
        res.headers.insert("Content-Type".to_string(), "application/json".to_string());
        res
    }

    pub fn not_found() -> Self {
        Self::new(404, "404 Not Found")
    }

    pub fn bad_request(msg: &str) -> Self {
        Self::new(400, format!("400 Bad Request: {}", msg))
    }

    pub fn method_not_allowed(allowed_methods: &str) -> Self {
        let mut res = Self::new(405, "405 Method Not Allowed");
        res.headers.insert("Allow".to_string(), allowed_methods.to_string());
        res
    }

    pub fn options(allowed_methods: &str) -> Self {
        let mut res = Self::new(200, "");
        res.headers.insert("Allow".to_string(), allowed_methods.to_string());
        res.headers.insert("Content-Length".to_string(), "0".to_string());
        res
    }

    pub fn header(mut self, key: &str, val: &str) -> Self {
        self.headers.insert(key.to_string(), val.to_string());
        self
    }

    /// Evaluates RFC 7232 Conditional Request headers against this resource's metadata
    pub fn evaluate_conditions(
        &self,
        req: &Http1Request,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Option<Http1Response> {
        // 1. If-Match (RFC 7232 §3.1)
        if let Some(if_match) = req.header("if-match") {
            if let Some(curr_etag) = etag {
                if if_match != "*" && !if_match.split(',').any(|t| t.trim() == curr_etag) {
                    return Some(Http1Response::new(412, "412 Precondition Failed: If-Match mismatch"));
                }
            } else {
                return Some(Http1Response::new(412, "412 Precondition Failed: Resource has no ETag"));
            }
        }

        // 2. If-None-Match (RFC 7232 §3.2)
        if let Some(if_none_match) = req.header("if-none-match") {
            if let Some(curr_etag) = etag {
                if if_none_match == "*" || if_none_match.split(',').any(|t| t.trim() == curr_etag) {
                    let mut not_mod = Http1Response::new(304, "");
                    not_mod.headers.insert("ETag".to_string(), curr_etag.to_string());
                    return Some(not_mod);
                }
            }
        }

        // 3. If-Modified-Since (RFC 7232 §3.3)
        if req.header("if-none-match").is_none() {
            if let Some(if_mod_since) = req.header("if-modified-since") {
                if let Some(curr_lm) = last_modified {
                    if if_mod_since.trim() == curr_lm.trim() {
                        let mut not_mod = Http1Response::new(304, "");
                        not_mod.headers.insert("Last-Modified".to_string(), curr_lm.to_string());
                        return Some(not_mod);
                    }
                }
            }
        }

        None
    }

    /// Evaluates RFC 7233 Range requests
    pub fn apply_range(&mut self, range_header: &str) {
        let trimmed = range_header.trim();
        if !trimmed.starts_with("bytes=") {
            return;
        }

        let range_spec = &trimmed["bytes=".len()..];
        let total_len = self.body.len();

        if let Some((start_str, end_str)) = range_spec.split_once('-') {
            let start_opt: Option<usize> = if start_str.is_empty() { None } else { start_str.parse().ok() };
            let end_opt: Option<usize> = if end_str.is_empty() { None } else { end_str.parse().ok() };

            match (start_opt, end_opt) {
                // bytes=start-end (e.g. bytes=0-499)
                (Some(start), Some(end)) => {
                    if start >= total_len || start > end {
                        self.status = 416;
                        self.reason = "Range Not Satisfiable".to_string();
                        self.headers.insert("Content-Range".to_string(), format!("bytes */{}", total_len));
                        self.body.clear();
                        return;
                    }
                    let actual_end = (end + 1).min(total_len);
                    self.body = self.body[start..actual_end].to_vec();
                    self.status = 206;
                    self.reason = "Partial Content".to_string();
                    self.headers.insert(
                        "Content-Range".to_string(),
                        format!("bytes {}-{}/{}", start, actual_end - 1, total_len),
                    );
                }
                // bytes=start- (e.g. bytes=500-)
                (Some(start), None) => {
                    if start >= total_len {
                        self.status = 416;
                        self.reason = "Range Not Satisfiable".to_string();
                        self.headers.insert("Content-Range".to_string(), format!("bytes */{}", total_len));
                        self.body.clear();
                        return;
                    }
                    self.body = self.body[start..].to_vec();
                    self.status = 206;
                    self.reason = "Partial Content".to_string();
                    self.headers.insert(
                        "Content-Range".to_string(),
                        format!("bytes {}-{}/{}", start, total_len - 1, total_len),
                    );
                }
                // bytes=-suffix (e.g. bytes=-500)
                (None, Some(suffix_len)) => {
                    if suffix_len == 0 {
                        self.status = 416;
                        self.reason = "Range Not Satisfiable".to_string();
                        self.headers.insert("Content-Range".to_string(), format!("bytes */{}", total_len));
                        self.body.clear();
                        return;
                    }
                    let start = total_len.saturating_sub(suffix_len);
                    self.body = self.body[start..].to_vec();
                    self.status = 206;
                    self.reason = "Partial Content".to_string();
                    self.headers.insert(
                        "Content-Range".to_string(),
                        format!("bytes {}-{}/{}", start, total_len - 1, total_len),
                    );
                }
                _ => {}
            }
        }
    }

    /// Serializes response to HTTP/1.1 wire bytes.
    /// If is_head is true, omits the body from output per RFC 7231 §4.3.2.
    pub fn serialize(&self, is_head: bool) -> Vec<u8> {
        let mut out = Vec::new();
        // Status line
        let status_line = format!("HTTP/1.1 {} {}\r\n", self.status, self.reason);
        out.extend_from_slice(status_line.as_bytes());

        // Ensure Content-Length is present
        let mut headers = self.headers.clone();
        if !headers.keys().any(|k| k.eq_ignore_ascii_case("content-length")) {
            headers.insert("Content-Length".to_string(), self.body.len().to_string());
        }

        // Headers
        for (k, v) in &headers {
            let line = format!("{}: {}\r\n", k, v);
            out.extend_from_slice(line.as_bytes());
        }
        out.extend_from_slice(b"\r\n");

        // Body (omitted for HEAD requests)
        if !is_head {
            out.extend_from_slice(&self.body);
        }

        out
    }

    fn default_reason_phrase(status: u16) -> &'static str {
        match status {
            200 => "OK",
            201 => "Created",
            202 => "Accepted",
            204 => "No Content",
            206 => "Partial Content",
            301 => "Moved Permanently",
            302 => "Found",
            304 => "Not Modified",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            405 => "Method Not Allowed",
            408 => "Request Timeout",
            412 => "Precondition Failed",
            413 => "Payload Too Large",
            415 => "Unsupported Media Type",
            416 => "Range Not Satisfiable",
            500 => "Internal Server Error",
            501 => "Not Implemented",
            502 => "Bad Gateway",
            503 => "Service Unavailable",
            505 => "HTTP Version Not Supported",
            _ => "Unknown",
        }
    }
}

pub type Http1Handler = Arc<dyn Fn(&Http1Request) -> Http1Response + Send + Sync>;

/// Real RFC 7230/7231/7232/7233 HTTP/1.x Server
pub struct Http1Server {
    port: u16,
    is_running: Arc<AtomicBool>,
    routes: Arc<Mutex<HashMap<(String, String), Http1Handler>>>,
    custom_handler: Arc<Mutex<Option<Http1Handler>>>,
}

impl Http1Server {
    pub fn start(port: u16) -> Result<Arc<Self>, String> {
        let listener = std::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .map_err(|e| format!("Failed to bind HTTP/1.x listener on port {}: {}", port, e))?;
        listener
            .set_nonblocking(true)
            .map_err(|e| format!("Failed to set nonblocking on listener: {}", e))?;

        let bound_port = listener
            .local_addr()
            .map_err(|e| format!("Failed to get local addr: {}", e))?
            .port();

        let is_running = Arc::new(AtomicBool::new(true));
        let routes = Arc::new(Mutex::new(HashMap::new()));
        let custom_handler: Arc<Mutex<Option<Http1Handler>>> = Arc::new(Mutex::new(None));

        let server = Arc::new(Self {
            port: bound_port,
            is_running: is_running.clone(),
            routes: routes.clone(),
            custom_handler: custom_handler.clone(),
        });

        // Spawn dedicated background OS thread with its own independent Tokio runtime
        let is_running_clone = is_running.clone();
        let routes_clone = routes.clone();
        let custom_handler_clone = custom_handler.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to build Tokio runtime for Http1Server");

            rt.block_on(async move {
                let async_listener = match TcpListener::from_std(listener) {
                    Ok(l) => l,
                    Err(e) => {
                        eprintln!("Error converting std listener to tokio: {}", e);
                        return;
                    }
                };

                while is_running_clone.load(Ordering::SeqCst) {
                    tokio::select! {
                        accept_res = async_listener.accept() => {
                            if let Ok((stream, _)) = accept_res {
                                let routes = routes_clone.clone();
                                let custom = custom_handler_clone.clone();
                                let running = is_running_clone.clone();
                                tokio::spawn(async move {
                                    Self::handle_connection(stream, routes, custom, running).await;
                                });
                            }
                        }
                        _ = sleep(Duration::from_millis(50)) => {
                            if !is_running_clone.load(Ordering::SeqCst) {
                                break;
                            }
                        }
                    }
                }
            });
        });

        // Ensure server has started
        std::thread::sleep(Duration::from_millis(30));

        Ok(server)
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn url(&self, path: &str) -> String {
        let clean_path = if path.starts_with('/') { path } else { &format!("/{}", path) };
        format!("http://127.0.0.1:{}{}", self.port, clean_path)
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::SeqCst)
    }

    pub fn stop(&self) {
        self.is_running.store(false, Ordering::SeqCst);
    }

    pub fn add_route<F>(&self, method: &str, path: &str, handler: F)
    where
        F: Fn(&Http1Request) -> Http1Response + Send + Sync + 'static,
    {
        if let Ok(mut lock) = self.routes.try_lock() {
            lock.insert((method.to_ascii_uppercase(), path.to_string()), Arc::new(handler));
        } else {
            let routes_clone = self.routes.clone();
            let method = method.to_ascii_uppercase();
            let path = path.to_string();
            let handler_arc: Http1Handler = Arc::new(handler);
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                rt.block_on(async {
                    let mut lock = routes_clone.lock().await;
                    lock.insert((method, path), handler_arc);
                });
            }).join().unwrap();
        }
    }

    pub fn set_handler<F>(&self, handler: F)
    where
        F: Fn(&Http1Request) -> Http1Response + Send + Sync + 'static,
    {
        if let Ok(mut lock) = self.custom_handler.try_lock() {
            *lock = Some(Arc::new(handler));
        } else {
            let custom_clone = self.custom_handler.clone();
            let handler_arc: Http1Handler = Arc::new(handler);
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
                rt.block_on(async {
                    let mut lock = custom_clone.lock().await;
                    *lock = Some(handler_arc);
                });
            }).join().unwrap();
        }
    }

    async fn handle_connection(
        mut stream: TcpStream,
        routes: Arc<Mutex<HashMap<(String, String), Http1Handler>>>,
        custom_handler: Arc<Mutex<Option<Http1Handler>>>,
        is_running: Arc<AtomicBool>,
    ) {
        let mut buffer = vec![0u8; 65536];
        let mut buf_len = 0;

        while is_running.load(Ordering::SeqCst) {
            // Read from socket
            let bytes_read = match stream.read(&mut buffer[buf_len..]).await {
                Ok(0) => break, // EOF
                Ok(n) => n,
                Err(_) => break,
            };
            buf_len += bytes_read;

            // Attempt to parse request
            loop {
                if buf_len == 0 {
                    break;
                }

                match Http1Request::parse(&buffer[..buf_len]) {
                    Ok((req, consumed)) => {
                        let is_head = req.method == Http1Method::Head;
                        let keep_alive = req.should_keep_alive();

                        // Route dispatch
                        let response = Self::dispatch_request(&req, &routes, &custom_handler).await;

                        // Serialize and send
                        let response_bytes = response.serialize(is_head);
                        if stream.write_all(&response_bytes).await.is_err() {
                            return;
                        }
                        let _ = stream.flush().await;

                        // Shift buffer
                        buffer.copy_within(consumed..buf_len, 0);
                        buf_len -= consumed;

                        if !keep_alive {
                            return;
                        }
                    }
                    Err(Http1Error::IncompleteRequest) => {
                        // Need more data
                        break;
                    }
                    Err(Http1Error::MissingHostHeader) => {
                        let res = Http1Response::bad_request("Missing Host header in HTTP/1.1 request (RFC 7230 §5.4)")
                            .header("Connection", "close");
                        let _ = stream.write_all(&res.serialize(false)).await;
                        let _ = stream.flush().await;
                        return;
                    }
                    Err(Http1Error::BadRequest(msg)) => {
                        let res = Http1Response::bad_request(&msg).header("Connection", "close");
                        let _ = stream.write_all(&res.serialize(false)).await;
                        let _ = stream.flush().await;
                        return;
                    }
                    Err(Http1Error::UnsupportedVersion(v)) => {
                        let res = Http1Response::new(505, format!("505 HTTP Version Not Supported: {}", v))
                            .header("Connection", "close");
                        let _ = stream.write_all(&res.serialize(false)).await;
                        let _ = stream.flush().await;
                        return;
                    }
                    Err(e) => {
                        let res = Http1Response::bad_request(&format!("{}", e)).header("Connection", "close");
                        let _ = stream.write_all(&res.serialize(false)).await;
                        let _ = stream.flush().await;
                        return;
                    }
                }
            }
        }
    }

    async fn dispatch_request(
        req: &Http1Request,
        routes: &Arc<Mutex<HashMap<(String, String), Http1Handler>>>,
        custom_handler: &Arc<Mutex<Option<Http1Handler>>>,
    ) -> Http1Response {
        // 1. Check custom handler
        {
            let custom_lock = custom_handler.lock().await;
            if let Some(h) = &*custom_lock {
                return h(req);
            }
        }

        // 2. Check registered routes
        let method_str = match req.method {
            Http1Method::Head => "GET", // HEAD shares GET routing
            _ => req.method.as_str(),
        };

        let route_lock = routes.lock().await;
        if let Some(handler) = route_lock.get(&(method_str.to_string(), req.path.clone())) {
            let mut res = handler(req);
            // Check Range header
            if let Some(range) = req.header("range") {
                res.apply_range(range);
            }
            return res;
        }

        // 3. Check OPTIONS
        if req.method == Http1Method::Options {
            let mut allowed = vec!["OPTIONS".to_string()];
            for ((m, p), _) in route_lock.iter() {
                if p == &req.path && !allowed.contains(m) {
                    allowed.push(m.clone());
                }
            }
            if !allowed.contains(&"GET".to_string()) && allowed.len() == 1 {
                allowed.push("GET".to_string());
                allowed.push("HEAD".to_string());
            }
            return Http1Response::options(&allowed.join(", "));
        }

        // 4. Check Method Not Allowed (405)
        let other_methods: Vec<String> = route_lock
            .keys()
            .filter(|(_, p)| p == &req.path)
            .map(|(m, _)| m.clone())
            .collect();

        if !other_methods.is_empty() {
            return Http1Response::method_not_allowed(&other_methods.join(", "));
        }

        // 5. Default routes if empty
        if req.path == "/" || req.path == "/health" {
            return Http1Response::new(200, "OK").header("Content-Type", "text/plain");
        }

        Http1Response::not_found()
    }
}

/// Simple Synchronous HTTP/1.1 Raw TCP Client for independent conformance verification
pub struct Http1Client;

impl Http1Client {
    pub fn send_raw(addr: SocketAddr, raw_request: &[u8]) -> Result<Vec<u8>, String> {
        use std::io::{Read, Write};
        let mut stream = std::net::TcpStream::connect_timeout(&addr, Duration::from_secs(3))
            .map_err(|e| format!("Failed to connect to {}: {}", addr, e))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(3)))
            .map_err(|e| format!("Set timeout failed: {}", e))?;

        stream
            .write_all(raw_request)
            .map_err(|e| format!("Write failed: {}", e))?;
        stream.flush().map_err(|e| format!("Flush failed: {}", e))?;

        let mut resp = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            match stream.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    resp.extend_from_slice(&buf[..n]);
                    // Check if complete response has been received
                    if let Some(pos) = resp.windows(4).position(|w| w == b"\r\n\r\n") {
                        let header_str = String::from_utf8_lossy(&resp[..pos]);
                        if let Some(cl_line) = header_str.lines().find(|l| l.to_ascii_lowercase().starts_with("content-length:")) {
                            if let Some(val) = cl_line.split(':').nth(1) {
                                if let Ok(cl) = val.trim().parse::<usize>() {
                                    if resp.len() >= pos + 4 + cl {
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    if !resp.is_empty() {
                        break;
                    }
                    return Err(format!("Read error: {}", e));
                }
            }
        }
        Ok(resp)
    }
}

// =========================================================================
// GLOBAL HANDLE REGISTRY FOR INTERPRETER & RUNTIME BUILTINS
// =========================================================================

use std::sync::atomic::AtomicI64;
static NEXT_HTTP1_HANDLE: AtomicI64 = AtomicI64::new(100);
static HTTP1_SERVERS: std::sync::OnceLock<std::sync::Mutex<HashMap<i64, Arc<Http1Server>>>> = std::sync::OnceLock::new();

fn get_http1_servers() -> &'static std::sync::Mutex<HashMap<i64, Arc<Http1Server>>> {
    HTTP1_SERVERS.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
}

pub fn end_runtime_http1_server_start(port: i64) -> i64 {
    let target_port = if port <= 0 { 0 } else { port as u16 };
    match Http1Server::start(target_port) {
        Ok(server) => {
            let handle = NEXT_HTTP1_HANDLE.fetch_add(1, Ordering::SeqCst);
            if let Ok(mut lock) = get_http1_servers().lock() {
                lock.insert(handle, server);
            }
            handle
        }
        Err(_) => -1,
    }
}

pub fn end_runtime_http1_server_port(handle: i64) -> i64 {
    if let Ok(lock) = get_http1_servers().lock() {
        if let Some(s) = lock.get(&handle) {
            return s.port() as i64;
        }
    }
    -1
}

pub fn end_runtime_http1_server_is_running(handle: i64) -> bool {
    if let Ok(lock) = get_http1_servers().lock() {
        if let Some(s) = lock.get(&handle) {
            return s.is_running();
        }
    }
    false
}

pub fn end_runtime_http1_server_stop(handle: i64) {
    if let Ok(mut lock) = get_http1_servers().lock() {
        if let Some(s) = lock.remove(&handle) {
            s.stop();
        }
    }
}

pub fn end_runtime_http1_server_add_route(handle: i64, method: &str, path: &str, response_body: &str) -> i64 {
    if let Ok(lock) = get_http1_servers().lock() {
        if let Some(s) = lock.get(&handle) {
            let body_owned = response_body.to_string();
            s.add_route(method, path, move |_| {
                Http1Response::new(200, body_owned.clone())
            });
            return 1;
        }
    }
    0
}

pub fn end_runtime_http1_request_sync(url: &str, method: &str, body: &str) -> String {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(5))
        .build();
    let Ok(client) = client else {
        return "{\"status\": 500, \"body\": \"Client Init Error\"}".to_string();
    };

    let req_builder = match method.to_ascii_uppercase().as_str() {
        "GET" => client.get(url),
        "POST" => client.post(url).body(body.to_string()),
        "PUT" => client.put(url).body(body.to_string()),
        "DELETE" => client.delete(url),
        "HEAD" => client.head(url),
        _ => client.get(url),
    };

    match req_builder.send() {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.text().unwrap_or_default();
            serde_json::json!({
                "status": status,
                "body": text,
            }).to_string()
        }
        Err(e) => {
            serde_json::json!({
                "status": 502,
                "error": e.to_string(),
            }).to_string()
        }
    }
}
