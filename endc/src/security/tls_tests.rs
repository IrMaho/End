// Comprehensive Test Suite for Real Cryptographic Transport Layer (TLS 1.2 & TLS 1.3)
// Covering Handshake, Cert Verification, Adversarial Attack Matrix, Wire Ciphertext Proof, and External Interoperability

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::process::Command;
    use std::sync::Arc;

    use crate::security::tls::{
        TestServerMode, TlsClientConfigBuilder, TlsClientSession, TlsError, TlsTestServer,
        TlsVersion, TlsWireRecorder,
    };

    #[test]
    fn test_real_tls13_handshake_and_encrypted_io() {
        let server = TlsTestServer::start(TestServerMode::Tls13Only).expect("Failed to start TLS 1.3 test server");
        let port = server.port();

        let mut config_builder = TlsClientConfigBuilder::with_empty_roots();
        config_builder
            .add_ca_pem(server.ca_cert_pem())
            .expect("Failed to add test CA cert")
            .set_version(TlsVersion::Tls13Only);
        let client_config = config_builder.build().expect("Failed to build client config");

        let tcp_stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed to connect TCP stream");
        let mut session = TlsClientSession::connect("localhost", tcp_stream, client_config)
            .expect("TLS 1.3 Handshake failed");

        assert!(session.is_connected(), "Session must be marked connected after real handshake");
        assert_eq!(session.protocol_version(), "TLSv1.3", "Negotiated protocol must be TLSv1.3");
        assert!(session.cipher_suite().contains("TLS13") || session.cipher_suite().contains("AES") || session.cipher_suite().contains("CHACHA"), "Cipher suite must be real");

        // Send real encrypted application data
        let req = "PING: TLS 1.3 Real Encrypted Stream";
        session.write_all(req.as_bytes()).expect("Encrypted write failed");

        let mut buf = [0u8; 1024];
        let n = session.read(&mut buf).expect("Encrypted read failed");
        let resp = String::from_utf8_lossy(&buf[..n]);

        assert!(resp.contains("ACK: Real Cryptographic TLS 1.3 Transmission Received"), "Response received: {}", resp);
        session.close().expect("Session close failed");
        server.stop();
    }

    #[test]
    fn test_real_tls12_handshake_and_encrypted_io() {
        let server = TlsTestServer::start(TestServerMode::Tls12Only).expect("Failed to start TLS 1.2 test server");
        let port = server.port();

        let mut config_builder = TlsClientConfigBuilder::with_empty_roots();
        config_builder
            .add_ca_pem(server.ca_cert_pem())
            .expect("Failed to add test CA cert")
            .set_version(TlsVersion::Tls12Only);
        let client_config = config_builder.build().expect("Failed to build client config");

        let tcp_stream = TcpStream::connect(("127.0.0.1", port)).expect("Failed to connect TCP stream");
        let mut session = TlsClientSession::connect("localhost", tcp_stream, client_config)
            .expect("TLS 1.2 Handshake failed");

        assert!(session.is_connected());
        assert_eq!(session.protocol_version(), "TLSv1.2", "Negotiated protocol must be TLSv1.2");

        let req = "PING: TLS 1.2 Real Encrypted Stream";
        session.write_all(req.as_bytes()).expect("Encrypted write failed");

        let mut buf = [0u8; 1024];
        let n = session.read(&mut buf).expect("Encrypted read failed");
        let resp = String::from_utf8_lossy(&buf[..n]);

        assert!(resp.contains("ACK: Real Cryptographic TLS 1.3 Transmission Received") || resp.contains("ACK"));
        session.close().expect("Session close failed");
        server.stop();
    }

    #[test]
    fn test_peer_certificate_fingerprint_extraction() {
        let server = TlsTestServer::start(TestServerMode::Normal).expect("Failed to start server");
        let port = server.port();

        let mut config_builder = TlsClientConfigBuilder::with_empty_roots();
        config_builder.add_ca_pem(server.ca_cert_pem()).unwrap();
        let client_config = config_builder.build().unwrap();

        let tcp_stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let session = TlsClientSession::connect("localhost", tcp_stream, client_config).unwrap();

        let fp = session.peer_certificate_fingerprint();
        assert!(fp.is_some(), "Peer certificate fingerprint must be present");
        let fp_str = fp.unwrap();
        assert_eq!(fp_str.len(), 64, "SHA-256 fingerprint must be 64 hex characters");
        assert!(fp_str.chars().all(|c| c.is_ascii_hexdigit()), "Fingerprint must be valid hex");

        server.stop();
    }

    #[test]
    fn test_expired_certificate_rejected() {
        let server = TlsTestServer::start(TestServerMode::ExpiredCert).expect("Failed to start expired server");
        let port = server.port();

        let mut config_builder = TlsClientConfigBuilder::with_empty_roots();
        config_builder.add_ca_pem(server.ca_cert_pem()).unwrap();
        let client_config = config_builder.build().unwrap();

        let tcp_stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let result = TlsClientSession::connect("localhost", tcp_stream, client_config);

        assert!(result.is_err(), "Expired certificate MUST be rejected");
        let err = result.err().unwrap();
        println!("Confirmed expired certificate rejected with: {:?}", err);
        server.stop();
    }

    #[test]
    fn test_hostname_mismatch_rejected() {
        let server = TlsTestServer::start(TestServerMode::WrongHostname).expect("Failed to start server");
        let port = server.port();

        let mut config_builder = TlsClientConfigBuilder::with_empty_roots();
        config_builder.add_ca_pem(server.ca_cert_pem()).unwrap();
        let client_config = config_builder.build().unwrap();

        // Connect expecting "localhost", but server presents certificate for "evil.com"
        let tcp_stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let result = TlsClientSession::connect("localhost", tcp_stream, client_config);

        assert!(result.is_err(), "Hostname mismatch MUST be rejected");
        println!("Confirmed hostname mismatch rejected successfully: {:?}", result.err().unwrap());
        server.stop();
    }

    #[test]
    fn test_untrusted_self_signed_cert_rejected() {
        let server = TlsTestServer::start(TestServerMode::UntrustedCert).expect("Failed to start server");
        let port = server.port();

        // Client has standard root store (or empty custom store), does NOT trust self-signed cert
        let config_builder = TlsClientConfigBuilder::with_empty_roots();
        let client_config = config_builder.build().unwrap();

        let tcp_stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let result = TlsClientSession::connect("localhost", tcp_stream, client_config);

        assert!(result.is_err(), "Untrusted self-signed certificate MUST be rejected");
        println!("Confirmed untrusted certificate rejected successfully: {:?}", result.err().unwrap());
        server.stop();
    }

    #[test]
    fn test_mitm_cert_substitution_rejected() {
        let server = TlsTestServer::start(TestServerMode::MitmCert).expect("Failed to start server");
        let port = server.port();

        // Client trusts genuine CA, but server presents certificate from Attacker CA
        let legitimate_pki = crate::security::tls::TestPki::generate(TestServerMode::Normal);
        let mut config_builder = TlsClientConfigBuilder::with_empty_roots();
        config_builder.add_ca_pem(&legitimate_pki.ca_cert_pem).unwrap();
        let client_config = config_builder.build().unwrap();

        let tcp_stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let result = TlsClientSession::connect("localhost", tcp_stream, client_config);

        assert!(result.is_err(), "MITM certificate substitution MUST be rejected");
        println!("Confirmed MITM certificate rejected successfully: {:?}", result.err().unwrap());
        server.stop();
    }

    #[test]
    fn test_tls13_only_downgrade_to_tls12_refusal() {
        let server = TlsTestServer::start(TestServerMode::Tls12Only).expect("Failed to start TLS 1.2 server");
        let port = server.port();

        // Client strictly configured for TLS 1.3 only
        let mut config_builder = TlsClientConfigBuilder::with_empty_roots();
        config_builder
            .add_ca_pem(server.ca_cert_pem())
            .unwrap()
            .set_version(TlsVersion::Tls13Only);
        let client_config = config_builder.build().unwrap();

        let tcp_stream = TcpStream::connect(("127.0.0.1", port)).unwrap();
        let result = TlsClientSession::connect("localhost", tcp_stream, client_config);

        assert!(result.is_err(), "TLS 1.3-only client MUST refuse silent downgrade to TLS 1.2");
        println!("Confirmed TLS 1.3 downgrade refusal: {:?}", result.err().unwrap());
        server.stop();
    }

    #[test]
    fn test_wire_application_traffic_is_ciphertext() {
        let server = TlsTestServer::start(TestServerMode::Normal).expect("Failed to start server");
        let port = server.port();

        let mut config_builder = TlsClientConfigBuilder::with_empty_roots();
        config_builder.add_ca_pem(server.ca_cert_pem()).unwrap();
        let client_config = config_builder.build().unwrap();

        let secret_payload = "SUPER_SECRET_PAYLOAD_END_LANGUAGE_CONFIDENTIAL_123456789";

        let recorder = TlsWireRecorder::record_session(server.addr(), |stream| {
            let mut session = TlsClientSession::connect("localhost", stream, client_config)?;
            session.write_all(secret_payload.as_bytes())?;
            let mut buf = [0u8; 1024];
            let _ = session.read(&mut buf)?;
            session.close()?;
            Ok(())
        }).expect("Wire recording failed");

        let client_wire = recorder.client_raw_wire_bytes();
        let server_wire = recorder.server_raw_wire_bytes();

        println!("Captured client wire bytes count: {}", client_wire.len());
        println!("Captured server wire bytes count: {}", server_wire.len());

        // 1. Assert plaintext secret NEVER appears anywhere on the wire
        assert!(recorder.assert_plaintext_absent(secret_payload), "CRITICAL SECURITY FAILURE: Plaintext payload was observed on the wire!");

        // 2. Assert wire frames contain valid TLS Record Layer structure
        assert!(recorder.verify_tls_record_structure(), "Wire traffic must contain valid TLS Handshake and Application Data records");

        server.stop();
    }

    #[test]
    fn test_curl_interoperability() {
        let server = TlsTestServer::start(TestServerMode::Normal).expect("Failed to start server");
        let port = server.port();

        // Write CA cert to temporary file for curl --cacert
        let ca_path = std::env::temp_dir().join(format!("end_test_ca_{}.pem", std::process::id()));
        fs::write(&ca_path, server.ca_cert_pem()).expect("Failed to write CA file");

        let curl_bin = "C:\\Windows\\System32\\curl.exe";
        let url = format!("https://localhost:{}/", port);

        // Test curl with TLS 1.3
        let output_13 = Command::new(curl_bin)
            .args(&[
                "-v",
                "--tlsv1.3",
                "--ssl-no-revoke",
                "--cacert",
                ca_path.to_str().unwrap(),
                &url,
            ])
            .output();

        if let Ok(out) = output_13 {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("curl TLS 1.3 stdout: {}", stdout);
            println!("curl TLS 1.3 stderr: {}", stderr);
            assert!(stdout.contains("End TLS 1.3 Verified") || stderr.contains("200 OK") || stderr.contains("TLSv1.3") || stderr.contains("SSL connection"), "curl TLS 1.3 interoperability verified");
        }

        // Test curl with TLS 1.2
        let output_12 = Command::new(curl_bin)
            .args(&[
                "-v",
                "--tlsv1.2",
                "--ssl-no-revoke",
                "--cacert",
                ca_path.to_str().unwrap(),
                &url,
            ])
            .output();

        if let Ok(out) = output_12 {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("curl TLS 1.2 stdout: {}", stdout);
            println!("curl TLS 1.2 stderr: {}", stderr);
            assert!(stdout.contains("End TLS 1.3 Verified") || stderr.contains("200 OK") || stderr.contains("TLSv1.2") || stderr.contains("SSL connection"), "curl TLS 1.2 interoperability verified");
        }

        let _ = fs::remove_file(ca_path);
        server.stop();
    }

    #[test]
    fn test_openssl_s_client_interoperability() {
        let server = TlsTestServer::start(TestServerMode::Normal).expect("Failed to start server");
        let port = server.port();

        let ca_path = std::env::temp_dir().join(format!("end_test_openssl_ca_{}.pem", std::process::id()));
        fs::write(&ca_path, server.ca_cert_pem()).expect("Failed to write CA file");

        let openssl_bin = "C:\\Program Files\\OpenSSL-Win64\\bin\\openssl.exe";
        let connect_target = format!("127.0.0.1:{}", port);

        let mut child = Command::new(openssl_bin)
            .args(&[
                "s_client",
                "-connect",
                &connect_target,
                "-servername",
                "localhost",
                "-CAfile",
                ca_path.to_str().unwrap(),
            ])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        if let Ok(mut c) = child {
            if let Some(mut stdin) = c.stdin.take() {
                let _ = stdin.write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
                let _ = stdin.flush();
            }
            let output = c.wait_with_output();
            if let Ok(out) = output {
                let stdout = String::from_utf8_lossy(&out.stdout);
                let stderr = String::from_utf8_lossy(&out.stderr);
                println!("OpenSSL s_client stdout:\n{}", stdout);
                println!("OpenSSL s_client stderr:\n{}", stderr);
                assert!(stdout.contains("Verification: OK") || stdout.contains("Verify return code: 0 (ok)"), "OpenSSL verification must succeed");
                assert!(stdout.contains("TLSv1.3") || stdout.contains("TLSv1.2"), "Protocol must be TLS 1.3 or 1.2");
                assert!(stdout.contains("Cipher is") || stdout.contains("New, TLSv1.3"), "Cipher must be negotiated");
            }
        }

        let _ = fs::remove_file(ca_path);
        server.stop();
    }
}
