#[cfg(test)]
mod tests {
    use super::super::http1::*;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::sync::Arc;

    // =========================================================================
    // RFC 7230: MESSAGE FORMAT & HOST HEADER CONFORMANCE
    // =========================================================================

    #[test]
    fn test_rfc7230_valid_request_parsing() {
        let raw = b"GET /index.html HTTP/1.1\r\nHost: example.com\r\nUser-Agent: curl/8.0\r\nAccept: */*\r\n\r\n";
        let (req, consumed) = Http1Request::parse(raw).expect("Failed to parse valid HTTP/1.1 request");
        assert_eq!(consumed, raw.len());
        assert_eq!(req.method, Http1Method::Get);
        assert_eq!(req.uri, "/index.html");
        assert_eq!(req.path, "/index.html");
        assert_eq!(req.version, "HTTP/1.1");
        assert_eq!(req.header("host"), Some("example.com"));
        assert_eq!(req.header("user-agent"), Some("curl/8.0"));
        assert!(req.body.is_empty());
    }

    #[test]
    fn test_rfc7230_content_length_body_parsing() {
        let body = b"{\"user\":\"alice\",\"role\":\"admin\"}";
        let raw = format!(
            "POST /api/users HTTP/1.1\r\nHost: api.example.com\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
            body.len(),
            std::str::from_utf8(body).unwrap()
        );
        let (req, consumed) = Http1Request::parse(raw.as_bytes()).expect("Failed to parse POST body");
        assert_eq!(consumed, raw.len());
        assert_eq!(req.method, Http1Method::Post);
        assert_eq!(req.path, "/api/users");
        assert_eq!(req.body, body);
    }

    #[test]
    fn test_rfc7230_chunked_transfer_encoding_decoding() {
        let raw = b"POST /upload HTTP/1.1\r\nHost: upload.com\r\nTransfer-Encoding: chunked\r\n\r\n4\r\nWiki\r\n5\r\npedia\r\nF\r\n in \r\n\r\nchunks.\r\n0\r\n\r\n";
        let (req, consumed) = Http1Request::parse(raw).expect("Failed to parse chunked request");
        assert_eq!(consumed, raw.len());
        assert_eq!(req.method, Http1Method::Post);
        assert_eq!(req.body, b"Wikipedia in \r\n\r\nchunks.");
    }

    #[test]
    fn test_rfc7230_missing_host_header_rejected() {
        // RFC 7230 §5.4: A client MUST send a Host header field in all HTTP/1.1 request messages.
        // A server MUST respond with a 400 (Bad Request) status code to any HTTP/1.1 request message that lacks a Host header field.
        let raw = b"GET /resource HTTP/1.1\r\nUser-Agent: bad-client\r\n\r\n";
        let err = Http1Request::parse(raw).unwrap_err();
        assert_eq!(err, Http1Error::MissingHostHeader);
    }

    #[test]
    fn test_rfc7230_malformed_request_line_rejected() {
        let raw = b"INVALID_REQUEST\r\nHost: example.com\r\n\r\n";
        let err = Http1Request::parse(raw).unwrap_err();
        assert!(matches!(err, Http1Error::BadRequest(_)));
    }

    // =========================================================================
    // RFC 7231: HTTP METHOD SEMANTICS CONFORMANCE (SERVER INTEGRATION)
    // =========================================================================

    #[test]
    fn test_rfc7231_get_and_head_semantics() {
        let server = Http1Server::start(0).expect("Failed to start server");
        let addr = SocketAddr::from(([127, 0, 0, 1], server.port()));

        server.add_route("GET", "/article", |_| {
            Http1Response::new(200, "The full article body text content.").header("Content-Type", "text/plain")
        });

        // 1. GET Request: must return headers and body
        let get_req = b"GET /article HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        let get_resp = Http1Client::send_raw(addr, get_req).expect("GET request failed");
        let get_str = String::from_utf8_lossy(&get_resp);
        assert!(get_str.starts_with("HTTP/1.1 200 OK"));
        assert!(get_str.contains("The full article body text content."));
        assert!(get_str.contains("Content-Length: 35"));

        // 2. HEAD Request (RFC 7231 §4.3.2): must return identical headers but NO body
        let head_req = b"HEAD /article HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        let head_resp = Http1Client::send_raw(addr, head_req).expect("HEAD request failed");
        let head_str = String::from_utf8_lossy(&head_resp);
        assert!(head_str.starts_with("HTTP/1.1 200 OK"));
        assert!(head_str.contains("Content-Length: 35"));
        assert!(head_str.ends_with("\r\n\r\n"), "HEAD response must end immediately after headers with no body");
        assert!(!head_str.contains("The full article body text content."));

        server.stop();
    }

    #[test]
    fn test_rfc7231_options_method_and_405_method_not_allowed() {
        let server = Http1Server::start(0).expect("Failed to start server");
        let addr = SocketAddr::from(([127, 0, 0, 1], server.port()));

        server.add_route("POST", "/data", |req| {
            Http1Response::new(201, format!("Created: {} bytes", req.body.len()))
        });

        // 1. OPTIONS Request (RFC 7231 §4.3.7): must return Allow header
        let opt_req = b"OPTIONS /data HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        let opt_resp = Http1Client::send_raw(addr, opt_req).expect("OPTIONS request failed");
        let opt_str = String::from_utf8_lossy(&opt_resp);
        assert!(opt_str.starts_with("HTTP/1.1 200 OK"));
        assert!(opt_str.contains("Allow:"));
        assert!(opt_str.contains("POST"));

        // 2. Method Not Allowed (405): GET on POST-only route
        let get_req = b"GET /data HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        let get_resp = Http1Client::send_raw(addr, get_req).expect("GET on POST route failed");
        let get_str = String::from_utf8_lossy(&get_resp);
        assert!(get_str.starts_with("HTTP/1.1 405 Method Not Allowed"));
        assert!(get_str.contains("Allow: POST"));

        server.stop();
    }

    // =========================================================================
    // RFC 7232: CONDITIONAL REQUESTS CONFORMANCE
    // =========================================================================

    #[test]
    fn test_rfc7232_conditional_etag_and_if_none_match() {
        let server = Http1Server::start(0).expect("Failed to start server");
        let addr = SocketAddr::from(([127, 0, 0, 1], server.port()));

        server.add_route("GET", "/cached-resource", |req| {
            let resource = Http1Response::new(200, "V1-RESOURCE-PAYLOAD")
                .header("ETag", "\"v1-hash-abc\"");

            if let Some(cond) = resource.evaluate_conditions(req, Some("\"v1-hash-abc\""), None) {
                return cond;
            }
            resource
        });

        // 1. Request with matching If-None-Match (RFC 7232 §3.2) -> 304 Not Modified
        let if_none_match_req = b"GET /cached-resource HTTP/1.1\r\nHost: localhost\r\nIf-None-Match: \"v1-hash-abc\"\r\nConnection: close\r\n\r\n";
        let resp = Http1Client::send_raw(addr, if_none_match_req).expect("Conditional request failed");
        let resp_str = String::from_utf8_lossy(&resp);
        assert!(resp_str.starts_with("HTTP/1.1 304 Not Modified"));
        assert!(resp_str.contains("ETag: \"v1-hash-abc\""));

        // 2. Request with non-matching If-None-Match -> 200 OK with full body
        let nomatch_req = b"GET /cached-resource HTTP/1.1\r\nHost: localhost\r\nIf-None-Match: \"v0-old-hash\"\r\nConnection: close\r\n\r\n";
        let resp2 = Http1Client::send_raw(addr, nomatch_req).expect("Non-matching conditional request failed");
        let resp2_str = String::from_utf8_lossy(&resp2);
        assert!(resp2_str.starts_with("HTTP/1.1 200 OK"));
        assert!(resp2_str.contains("V1-RESOURCE-PAYLOAD"));

        server.stop();
    }

    #[test]
    fn test_rfc7232_if_match_precondition_failed() {
        let server = Http1Server::start(0).expect("Failed to start server");
        let addr = SocketAddr::from(([127, 0, 0, 1], server.port()));

        server.add_route("PUT", "/document", |req| {
            let doc = Http1Response::new(200, "Updated Document")
                .header("ETag", "\"etag-current\"");

            if let Some(cond) = doc.evaluate_conditions(req, Some("\"etag-current\""), None) {
                return cond;
            }
            doc
        });

        // If-Match mismatch -> 412 Precondition Failed (RFC 7232 §3.1)
        let put_req = b"PUT /document HTTP/1.1\r\nHost: localhost\r\nIf-Match: \"stale-etag-999\"\r\nContent-Length: 5\r\nConnection: close\r\n\r\nhello";
        let resp = Http1Client::send_raw(addr, put_req).expect("PUT with stale If-Match failed");
        let resp_str = String::from_utf8_lossy(&resp);
        assert!(resp_str.starts_with("HTTP/1.1 412 Precondition Failed"));

        server.stop();
    }

    // =========================================================================
    // RFC 7233: RANGE REQUESTS CONFORMANCE
    // =========================================================================

    #[test]
    fn test_rfc7233_range_requests_partial_content() {
        let server = Http1Server::start(0).expect("Failed to start server");
        let addr = SocketAddr::from(([127, 0, 0, 1], server.port()));

        let text_100_bytes = "0123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789"; // exactly 100 bytes
        assert_eq!(text_100_bytes.len(), 100);

        server.add_route("GET", "/largefile.dat", move |_| {
            Http1Response::new(200, text_100_bytes).header("Accept-Ranges", "bytes")
        });

        // 1. Range: bytes=0-9 (first 10 bytes) -> 206 Partial Content
        let range1 = b"GET /largefile.dat HTTP/1.1\r\nHost: localhost\r\nRange: bytes=0-9\r\nConnection: close\r\n\r\n";
        let resp1 = Http1Client::send_raw(addr, range1).expect("Range 0-9 failed");
        let resp1_str = String::from_utf8_lossy(&resp1);
        assert!(resp1_str.starts_with("HTTP/1.1 206 Partial Content"));
        assert!(resp1_str.contains("Content-Range: bytes 0-9/100"));
        assert!(resp1_str.contains("Content-Length: 10"));
        assert!(resp1_str.ends_with("0123456789"));

        // 2. Range: bytes=90- (last 10 bytes) -> 206 Partial Content
        let range2 = b"GET /largefile.dat HTTP/1.1\r\nHost: localhost\r\nRange: bytes=90-\r\nConnection: close\r\n\r\n";
        let resp2 = Http1Client::send_raw(addr, range2).expect("Range 90- failed");
        let resp2_str = String::from_utf8_lossy(&resp2);
        assert!(resp2_str.starts_with("HTTP/1.1 206 Partial Content"));
        assert!(resp2_str.contains("Content-Range: bytes 90-99/100"));
        assert!(resp2_str.contains("Content-Length: 10"));

        // 3. Range: bytes=200-300 (unsatisfiable) -> 416 Range Not Satisfiable
        let range_bad = b"GET /largefile.dat HTTP/1.1\r\nHost: localhost\r\nRange: bytes=200-300\r\nConnection: close\r\n\r\n";
        let resp_bad = Http1Client::send_raw(addr, range_bad).expect("Bad range failed");
        let bad_str = String::from_utf8_lossy(&resp_bad);
        assert!(bad_str.starts_with("HTTP/1.1 416 Range Not Satisfiable"));
        assert!(bad_str.contains("Content-Range: bytes */100"));

        server.stop();
    }

    // =========================================================================
    // RFC 7230 §6.3: PERSISTENT CONNECTIONS (KEEP-ALIVE)
    // =========================================================================

    #[test]
    fn test_rfc7230_keep_alive_connection_reuse() {
        use std::io::{Read, Write};

        let server = Http1Server::start(0).expect("Failed to start server");
        let addr = SocketAddr::from(([127, 0, 0, 1], server.port()));

        server.add_route("GET", "/ping", |_| {
            Http1Response::new(200, "pong")
        });

        let mut stream = std::net::TcpStream::connect(addr).expect("Connect failed");
        stream.set_read_timeout(Some(std::time::Duration::from_secs(3))).unwrap();

        // Send 3 requests sequentially over the single persistent TCP connection
        for i in 1..=3 {
            let req = format!("GET /ping HTTP/1.1\r\nHost: localhost\r\nX-Seq: {}\r\n\r\n", i);
            stream.write_all(req.as_bytes()).expect("Write failed");
            stream.flush().unwrap();

            let mut buf = [0u8; 1024];
            let n = stream.read(&mut buf).expect("Read failed");
            let resp_str = String::from_utf8_lossy(&buf[..n]);
            assert!(resp_str.starts_with("HTTP/1.1 200 OK"), "Request {} failed", i);
            assert!(resp_str.contains("pong"));
        }

        server.stop();
    }
}
