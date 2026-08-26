#[cfg(test)]
pub mod tests {
    use crate::runtime::net::http2::{HpackCodec, Http2Client, Http2RequestPayload, Http2Server};
    use std::collections::HashMap;
    use std::time::Duration;

    #[test]
    fn test_http2_cleartext_connection_and_handshake() {
        let server = Http2Server::start(0, false).expect("Start HTTP/2 cleartext server");
        assert!(server.port > 0);

        let url = format!("http://127.0.0.1:{}", server.port);
        let mut client = Http2Client::connect(&url, None).expect("Connect HTTP/2 client");
        assert!(client.is_connected);

        let resp = client.request("GET", "/ping", &[], &[]).expect("Ping request");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.version, "HTTP/2.0");
        assert_eq!(resp.body, "pong");
        assert_eq!(resp.headers.get("x-end-protocol").map(|s| s.as_str()), Some("HTTP/2.0-HPACK"));

        client.close();
        server.stop();
    }

    #[test]
    fn test_http2_request_response_echo_payload() {
        let server = Http2Server::start(0, false).expect("Start server");
        let url = format!("http://127.0.0.1:{}", server.port);
        let mut client = Http2Client::connect(&url, None).expect("Connect client");

        let echo_payload = "Hello, End HTTP/2 Native Wire Protocol! 🚀 ⚡ 🦀";
        let resp = client.request(
            "POST",
            "/echo",
            &[("content-type", "text/plain; charset=utf-8"), ("x-custom-end", "42")],
            echo_payload.as_bytes(),
        ).expect("Echo POST request");

        assert_eq!(resp.status, 200);
        assert_eq!(resp.body, echo_payload);
        assert_eq!(resp.headers.get("content-type").map(|s| s.as_str()), Some("text/plain"));

        client.close();
        server.stop();
    }

    #[test]
    fn test_http2_concurrent_10_stream_multiplexing_over_single_connection() {
        let server = Http2Server::start(0, false).expect("Start server");
        let url = format!("http://127.0.0.1:{}", server.port);
        let mut client = Http2Client::connect(&url, None).expect("Connect client");

        let num_streams = 12; // Exceeds minimum 10 streams requirement
        let mut requests = Vec::new();

        for i in 1..=num_streams {
            let mut headers = HashMap::new();
            headers.insert("x-stream-id".to_string(), i.to_string());
            headers.insert("x-request-origin".to_string(), "EndMultiplexerTest".to_string());

            // Vary delays to test out-of-order response arrivals
            let delay_ms = if i % 2 == 0 { 20 } else { 0 };

            requests.push(Http2RequestPayload {
                method: "POST".to_string(),
                path: format!("/api/stream/{}", i),
                headers,
                body: format!("Payload data for stream #{}", i),
                delay_ms,
            });
        }

        // Execute all 12 streams simultaneously over ONE connection
        let responses = client.request_multiplexed(&requests).expect("Multiplexed request batch");
        assert_eq!(responses.len(), num_streams);

        for (idx, resp) in responses.iter().enumerate() {
            let stream_idx = idx + 1;
            assert_eq!(resp.status, 200, "Stream #{} must return 200 OK", stream_idx);
            assert_eq!(resp.version, "HTTP/2.0");
            assert!(resp.body.contains(&format!("\"stream\": \"{}\"", stream_idx)), "Response body must match stream index");
            assert!(resp.body.contains("\"status\": \"multiplexed\""));
        }

        client.close();
        server.stop();
    }

    #[test]
    fn test_hpack_header_compression_and_decompression() {
        let raw_headers = [
            (":method", "GET"),
            (":path", "/v2/api/query"),
            (":authority", "end-lang.org"),
            (":scheme", "https"),
            ("content-type", "application/json"),
            ("user-agent", "EndHyper/2.0 (Native LLVM/Rust Runtime)"),
            ("authorization", "Bearer token_secret_12345_end_native"),
            ("accept", "application/json, text/plain, */*"),
            ("x-end-feature", "hpack-dynamic-table-compression"),
        ];

        // 1. Encode headers to HPACK byte stream
        let encoded_bytes = HpackCodec::encode(&raw_headers).expect("HPACK encode");
        assert!(!encoded_bytes.is_empty(), "Encoded bytes must not be empty");

        // 2. Decode HPACK bytes back to logical headers
        let decoded_headers = HpackCodec::decode(&encoded_bytes).expect("HPACK decode");
        assert_eq!(decoded_headers.len(), raw_headers.len(), "Decoded header count must match original");

        let decoded_map: HashMap<String, String> = decoded_headers.into_iter().collect();
        for (k, v) in &raw_headers {
            assert_eq!(
                decoded_map.get(*k).map(|s| s.as_str()),
                Some(*v),
                "Header '{}' must match exact value '{}'",
                k,
                v
            );
        }
    }

    #[test]
    fn test_http2_tls13_alpn_negotiation() {
        // Start server with TLS enabled (generates self-signed test cert)
        let server = Http2Server::start(0, true).expect("Start TLS HTTP/2 server");
        assert!(server.is_tls);
        assert!(server.server_cert_pem.is_some());

        let url = format!("https://localhost:{}", server.port);
        let mut client = Http2Client::connect(&url, server.ca_cert_pem.as_deref()).expect("Connect HTTPS HTTP/2 client");
        assert!(client.is_connected);

        let resp = client.request("GET", "/ping", &[], &[]).expect("HTTPS HTTP/2 ping");
        assert_eq!(resp.status, 200);
        assert_eq!(resp.version, "HTTP/2.0");
        assert_eq!(resp.body, "pong");

        client.close();
        server.stop();
    }

    #[test]
    fn test_http2_rich_headers_and_hpack_roundtrip() {
        let server = Http2Server::start(0, false).expect("Start server");
        let url = format!("http://127.0.0.1:{}", server.port);
        let mut client = Http2Client::connect(&url, None).expect("Connect client");

        let custom_headers = [
            ("x-developer", "Sina Maho"),
            ("x-language", "End 2.0"),
            ("x-runtime", "Zero-GC Native LLVM"),
            ("x-spec", "RFC 7540 + RFC 7541"),
            ("x-unicode-test", "سلام دنیا — زبان اند"),
        ];

        let resp = client.request("GET", "/headers", &custom_headers, &[]).expect("Headers request");
        assert_eq!(resp.status, 200);

        let json_parsed: serde_json::Value = serde_json::from_str(&resp.body).expect("Parse headers JSON");
        for (k, v) in &custom_headers {
            assert_eq!(
                json_parsed.get(*k).and_then(|val| val.as_str()),
                Some(*v),
                "Server must observe header '{}' with value '{}'",
                k,
                v
            );
        }

        client.close();
        server.stop();
    }

    #[test]
    fn test_http2_negative_connection_failure() {
        // Attempting connection to a closed port must produce Err(ConnectionFailed)
        let invalid_url = "http://127.0.0.1:59999";
        let res = Http2Client::connect(invalid_url, None);
        assert!(res.is_err(), "Connection to closed port must fail cleanly");
    }
}
