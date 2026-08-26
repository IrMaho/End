// End Language Cryptographic Transport Layer (TLS 1.2 & TLS 1.3)
// Powered by rustls, webpki-roots, rustls-pki-types, and rcgen

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName};
use rustls::{ClientConfig, ClientConnection, RootCertStore, ServerConfig, ServerConnection};
use sha2::{Digest, Sha256};

#[derive(Debug)]
pub enum TlsError {
    Io(io::Error),
    Rustls(rustls::Error),
    InvalidServerName(String),
    CertificateError(String),
    HandshakeFailed(String),
    DowngradeRefused(String),
    NotConnected,
}

impl std::fmt::Display for TlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsError::Io(e) => write!(f, "I/O error: {}", e),
            TlsError::Rustls(e) => write!(f, "TLS protocol error: {}", e),
            TlsError::InvalidServerName(s) => write!(f, "Invalid DNS/Server name: {}", s),
            TlsError::CertificateError(s) => write!(f, "Certificate error: {}", s),
            TlsError::HandshakeFailed(s) => write!(f, "Handshake failed: {}", s),
            TlsError::DowngradeRefused(s) => write!(f, "TLS Downgrade Refused: Client configured for TLS 1.3 only but server offered {}", s),
            TlsError::NotConnected => write!(f, "Session not connected"),
        }
    }
}

impl std::error::Error for TlsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            TlsError::Io(e) => Some(e),
            TlsError::Rustls(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for TlsError {
    fn from(err: io::Error) -> Self {
        TlsError::Io(err)
    }
}

impl From<rustls::Error> for TlsError {
    fn from(err: rustls::Error) -> Self {
        TlsError::Rustls(err)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TlsVersion {
    All,
    Tls12Only,
    Tls13Only,
}

/// Real TLS Client Configuration Builder backed by webpki-roots and rustls
#[derive(Clone)]
pub struct TlsClientConfigBuilder {
    root_store: RootCertStore,
    version: TlsVersion,
    alpn_protocols: Vec<Vec<u8>>,
    custom_ca_added: bool,
}

impl Default for TlsClientConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsClientConfigBuilder {
    pub fn new() -> Self {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        Self {
            root_store,
            version: TlsVersion::All,
            alpn_protocols: vec![b"http/1.1".to_vec()],
            custom_ca_added: false,
        }
    }

    pub fn with_empty_roots() -> Self {
        Self {
            root_store: RootCertStore::empty(),
            version: TlsVersion::All,
            alpn_protocols: vec![b"http/1.1".to_vec()],
            custom_ca_added: false,
        }
    }

    pub fn add_ca_pem(&mut self, pem_str: &str) -> Result<&mut Self, TlsError> {
        let mut reader = io::Cursor::new(pem_str.as_bytes());
        let certs = rustls_pemfile::certs(&mut reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsError::CertificateError(format!("Failed to parse CA PEM: {}", e)))?;
        for cert in certs {
            self.root_store
                .add(cert)
                .map_err(|e| TlsError::CertificateError(format!("Failed to add root cert: {}", e)))?;
        }
        self.custom_ca_added = true;
        Ok(self)
    }

    pub fn set_version(&mut self, version: TlsVersion) -> &mut Self {
        self.version = version;
        self
    }

    pub fn set_alpn(&mut self, protocols: &[&str]) -> &mut Self {
        self.alpn_protocols = protocols.iter().map(|p| p.as_bytes().to_vec()).collect();
        self
    }

    pub fn build(self) -> Result<Arc<ClientConfig>, TlsError> {
        let builder = match self.version {
            TlsVersion::All => {
                ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                    .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
                    .map_err(|e| TlsError::Rustls(e))?
            }
            TlsVersion::Tls13Only => {
                ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                    .with_protocol_versions(&[&rustls::version::TLS13])
                    .map_err(|e| TlsError::Rustls(e))?
            }
            TlsVersion::Tls12Only => {
                ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                    .with_protocol_versions(&[&rustls::version::TLS12])
                    .map_err(|e| TlsError::Rustls(e))?
            }
        };

        let mut config = builder
            .with_root_certificates(self.root_store)
            .with_no_client_auth();

        config.alpn_protocols = self.alpn_protocols;
        Ok(Arc::new(config))
    }
}

/// Real TLS Server Configuration Builder
pub struct TlsServerConfigBuilder {
    certs: Vec<CertificateDer<'static>>,
    key: Option<PrivateKeyDer<'static>>,
    version: TlsVersion,
    alpn_protocols: Vec<Vec<u8>>,
}

impl Clone for TlsServerConfigBuilder {
    fn clone(&self) -> Self {
        Self {
            certs: self.certs.clone(),
            key: self.key.as_ref().map(|k| k.clone_key()),
            version: self.version,
            alpn_protocols: self.alpn_protocols.clone(),
        }
    }
}

impl TlsServerConfigBuilder {
    pub fn new() -> Self {
        Self {
            certs: Vec::new(),
            key: None,
            version: TlsVersion::All,
            alpn_protocols: vec![b"http/1.1".to_vec()],
        }
    }

    pub fn set_cert_and_key_pem(
        &mut self,
        cert_pem: &str,
        key_pem: &str,
    ) -> Result<&mut Self, TlsError> {
        let mut cert_reader = io::Cursor::new(cert_pem.as_bytes());
        let certs = rustls_pemfile::certs(&mut cert_reader)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| TlsError::CertificateError(format!("Failed to parse server cert PEM: {}", e)))?;
        if certs.is_empty() {
            return Err(TlsError::CertificateError("No certificates found in PEM".to_string()));
        }
        self.certs = certs;

        let mut key_reader = io::Cursor::new(key_pem.as_bytes());
        let key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| TlsError::CertificateError(format!("Failed to parse server key PEM: {}", e)))?
            .ok_or_else(|| TlsError::CertificateError("No private key found in PEM".to_string()))?;
        self.key = Some(key);

        Ok(self)
    }

    pub fn set_version(&mut self, version: TlsVersion) -> &mut Self {
        self.version = version;
        self
    }

    pub fn set_alpn(&mut self, protocols: &[&str]) -> &mut Self {
        self.alpn_protocols = protocols.iter().map(|p| p.as_bytes().to_vec()).collect();
        self
    }

    pub fn build(self) -> Result<Arc<ServerConfig>, TlsError> {
        let key = self
            .key
            .ok_or_else(|| TlsError::CertificateError("Server private key not provided".to_string()))?;

        let builder = match self.version {
            TlsVersion::All => {
                ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                    .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])
                    .map_err(|e| TlsError::Rustls(e))?
            }
            TlsVersion::Tls13Only => {
                ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                    .with_protocol_versions(&[&rustls::version::TLS13])
                    .map_err(|e| TlsError::Rustls(e))?
            }
            TlsVersion::Tls12Only => {
                ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                    .with_protocol_versions(&[&rustls::version::TLS12])
                    .map_err(|e| TlsError::Rustls(e))?
            }
        };

        let mut config = builder
            .with_no_client_auth()
            .with_single_cert(self.certs, key)
            .map_err(|e| TlsError::Rustls(e))?;

        config.alpn_protocols = self.alpn_protocols;
        Ok(Arc::new(config))
    }
}

/// Real Active Client TLS Session
pub struct TlsClientSession {
    stream: TcpStream,
    conn: ClientConnection,
    server_name: String,
    handshake_completed: bool,
    peer_cert_fingerprint: Option<String>,
    negotiated_protocol: Option<String>,
    negotiated_cipher: Option<String>,
    negotiated_alpn: Option<String>,
    bytes_sent: u64,
    bytes_received: u64,
}

impl TlsClientSession {
    pub fn connect(
        host: &str,
        mut stream: TcpStream,
        config: Arc<ClientConfig>,
    ) -> Result<Self, TlsError> {
        let server_name = ServerName::try_from(host.to_string())
            .map_err(|_| TlsError::InvalidServerName(host.to_string()))?;

        let mut conn = ClientConnection::new(config, server_name)
            .map_err(|e| TlsError::Rustls(e))?;

        // Perform real TLS handshake over stream
        let mut handshake_done = false;
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(10);

        while !handshake_done {
            if start.elapsed() > timeout {
                return Err(TlsError::HandshakeFailed("TLS handshake timed out".to_string()));
            }

            while conn.wants_write() {
                conn.write_tls(&mut stream)?;
            }

            if !conn.is_handshaking() {
                handshake_done = true;
                break;
            }

            if conn.wants_read() {
                let n = conn.read_tls(&mut stream)?;
                if n == 0 {
                    return Err(TlsError::HandshakeFailed("Peer closed connection during handshake".to_string()));
                }
                conn.process_new_packets()
                    .map_err(|e| TlsError::HandshakeFailed(format!("TLS handshake verification error: {}", e)))?;
            }

            if !conn.is_handshaking() {
                handshake_done = true;
                break;
            }
        }

        // Flush any remaining handshake bytes
        while conn.wants_write() {
            conn.write_tls(&mut stream)?;
        }

        // Extract real peer certificate fingerprint
        let peer_cert_fingerprint = conn.peer_certificates().and_then(|certs| {
            certs.first().map(|cert| {
                let mut hasher = Sha256::new();
                hasher.update(cert.as_ref());
                let result = hasher.finalize();
                result.iter().map(|b| format!("{:02x}", b)).collect::<String>()
            })
        });

        // Extract real negotiated protocol version
        let negotiated_protocol = conn.protocol_version().map(|v| match v {
            rustls::ProtocolVersion::TLSv1_3 => "TLSv1.3".to_string(),
            rustls::ProtocolVersion::TLSv1_2 => "TLSv1.2".to_string(),
            _ => format!("{:?}", v),
        });

        // Extract real negotiated cipher suite
        let negotiated_cipher = conn.negotiated_cipher_suite().map(|cs| {
            format!("{:?}", cs.suite())
        });

        // Extract real negotiated ALPN
        let negotiated_alpn = conn.alpn_protocol().map(|p| String::from_utf8_lossy(p).to_string());

        Ok(Self {
            stream,
            conn,
            server_name: host.to_string(),
            handshake_completed: true,
            peer_cert_fingerprint,
            negotiated_protocol,
            negotiated_cipher,
            negotiated_alpn,
            bytes_sent: 0,
            bytes_received: 0,
        })
    }

    pub fn write_all(&mut self, payload: &[u8]) -> Result<usize, TlsError> {
        if !self.handshake_completed {
            return Err(TlsError::NotConnected);
        }

        let mut writer = self.conn.writer();
        writer.write_all(payload)?;

        while self.conn.wants_write() {
            let written = self.conn.write_tls(&mut self.stream)?;
            self.bytes_sent += written as u64;
        }
        self.stream.flush()?;

        Ok(payload.len())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, TlsError> {
        if !self.handshake_completed {
            return Err(TlsError::NotConnected);
        }

        loop {
            // First check if there is already plaintext in the reader
            let mut reader = self.conn.reader();
            match reader.read(buf) {
                Ok(n) if n > 0 => {
                    self.bytes_received += n as u64;
                    return Ok(n);
                }
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(TlsError::Io(e)),
            }

            // Need to read more ciphertext from socket
            if self.conn.wants_read() {
                let n = self.conn.read_tls(&mut self.stream)?;
                if n == 0 {
                    return Ok(0); // EOF
                }
                self.conn.process_new_packets()
                    .map_err(|e| TlsError::Rustls(e))?;
            } else {
                return Ok(0);
            }
        }
    }

    pub fn close(&mut self) -> Result<(), TlsError> {
        self.conn.send_close_notify();
        while self.conn.wants_write() {
            let _ = self.conn.write_tls(&mut self.stream);
        }
        let _ = self.stream.flush();
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        self.handshake_completed = false;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.handshake_completed
    }

    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    pub fn protocol_version(&self) -> &str {
        self.negotiated_protocol.as_deref().unwrap_or("NONE")
    }

    pub fn cipher_suite(&self) -> &str {
        self.negotiated_cipher.as_deref().unwrap_or("NONE")
    }

    pub fn alpn_protocol(&self) -> &str {
        self.negotiated_alpn.as_deref().unwrap_or("")
    }

    pub fn peer_certificate_fingerprint(&self) -> Option<&str> {
        self.peer_cert_fingerprint.as_deref()
    }
}

/// Real Active Server TLS Session
pub struct TlsServerSession {
    stream: TcpStream,
    conn: ServerConnection,
    handshake_completed: bool,
    negotiated_protocol: Option<String>,
    negotiated_cipher: Option<String>,
    negotiated_alpn: Option<String>,
}

impl TlsServerSession {
    pub fn accept(mut stream: TcpStream, config: Arc<ServerConfig>) -> Result<Self, TlsError> {
        let mut conn = ServerConnection::new(config)
            .map_err(|e| TlsError::Rustls(e))?;

        let mut handshake_done = false;
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(10);

        while !handshake_done {
            if start.elapsed() > timeout {
                return Err(TlsError::HandshakeFailed("TLS server handshake timed out".to_string()));
            }

            if conn.wants_read() {
                let n = conn.read_tls(&mut stream)?;
                if n == 0 {
                    return Err(TlsError::HandshakeFailed("Peer closed connection".to_string()));
                }
                conn.process_new_packets()
                    .map_err(|e| TlsError::HandshakeFailed(format!("TLS packet error: {}", e)))?;
            }

            while conn.wants_write() {
                conn.write_tls(&mut stream)?;
            }

            if !conn.is_handshaking() {
                handshake_done = true;
                break;
            }
        }

        let negotiated_protocol = conn.protocol_version().map(|v| match v {
            rustls::ProtocolVersion::TLSv1_3 => "TLSv1.3".to_string(),
            rustls::ProtocolVersion::TLSv1_2 => "TLSv1.2".to_string(),
            _ => format!("{:?}", v),
        });

        let negotiated_cipher = conn.negotiated_cipher_suite().map(|cs| {
            format!("{:?}", cs.suite())
        });

        let negotiated_alpn = conn.alpn_protocol().map(|p| String::from_utf8_lossy(p).to_string());

        Ok(Self {
            stream,
            conn,
            handshake_completed: true,
            negotiated_protocol,
            negotiated_cipher,
            negotiated_alpn,
        })
    }

    pub fn write_all(&mut self, payload: &[u8]) -> Result<usize, TlsError> {
        let mut writer = self.conn.writer();
        writer.write_all(payload)?;

        while self.conn.wants_write() {
            self.conn.write_tls(&mut self.stream)?;
        }
        self.stream.flush()?;
        Ok(payload.len())
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize, TlsError> {
        loop {
            let mut reader = self.conn.reader();
            match reader.read(buf) {
                Ok(n) if n > 0 => return Ok(n),
                Ok(_) => {}
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {}
                Err(e) => return Err(TlsError::Io(e)),
            }

            if self.conn.wants_read() {
                let n = self.conn.read_tls(&mut self.stream)?;
                if n == 0 {
                    return Ok(0);
                }
                self.conn.process_new_packets()
                    .map_err(|e| TlsError::Rustls(e))?;
            } else {
                return Ok(0);
            }
        }
    }

    pub fn close(&mut self) -> Result<(), TlsError> {
        self.conn.send_close_notify();
        while self.conn.wants_write() {
            let _ = self.conn.write_tls(&mut self.stream);
        }
        let _ = self.stream.flush();
        let _ = self.stream.shutdown(std::net::Shutdown::Both);
        self.handshake_completed = false;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.handshake_completed
    }

    pub fn protocol_version(&self) -> &str {
        self.negotiated_protocol.as_deref().unwrap_or("NONE")
    }

    pub fn cipher_suite(&self) -> &str {
        self.negotiated_cipher.as_deref().unwrap_or("NONE")
    }

    pub fn alpn_protocol(&self) -> &str {
        self.negotiated_alpn.as_deref().unwrap_or("")
    }
}

/// Mode configuration for Deterministic TLS Test Server
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestServerMode {
    Normal,         // Valid cert for localhost (TLS 1.3 & 1.2)
    Tls13Only,      // TLS 1.3 only
    Tls12Only,      // TLS 1.2 only
    ExpiredCert,    // Expired certificate
    WrongHostname,  // Cert issued for evil.com
    UntrustedCert,  // Self-signed cert not in CA root store
    MitmCert,       // Attacker-generated certificate
}

/// Generated Test PKI Material
pub struct TestPki {
    pub ca_cert_pem: String,
    pub server_cert_pem: String,
    pub server_key_pem: String,
}

impl TestPki {
    pub fn generate(mode: TestServerMode) -> Self {
        match mode {
            TestServerMode::Normal | TestServerMode::Tls13Only | TestServerMode::Tls12Only => {
                // 1. Generate Root CA
                let mut ca_params = rcgen::CertificateParams::default();
                ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
                ca_params.distinguished_name.push(
                    rcgen::DnType::CommonName,
                    "End Language Test Root CA",
                );
                let ca_key = rcgen::KeyPair::generate().unwrap();
                let ca_cert = ca_params.self_signed(&ca_key).unwrap();
                let ca_cert_pem = ca_cert.pem();

                // 2. Generate Server Cert signed by CA for localhost
                let mut srv_params = rcgen::CertificateParams::new(vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                ]).unwrap();
                srv_params.distinguished_name.push(
                    rcgen::DnType::CommonName,
                    "localhost",
                );
                let srv_key = rcgen::KeyPair::generate().unwrap();
                let srv_cert = srv_params.signed_by(&srv_key, &ca_cert, &ca_key).unwrap();
                let srv_cert_pem = srv_cert.pem();
                let srv_key_pem = srv_key.serialize_pem();

                Self {
                    ca_cert_pem,
                    server_cert_pem: srv_cert_pem,
                    server_key_pem: srv_key_pem,
                }
            }
            TestServerMode::ExpiredCert => {
                // Generate expired certificate (not_after in the past)
                let mut ca_params = rcgen::CertificateParams::default();
                ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
                let ca_key = rcgen::KeyPair::generate().unwrap();
                let ca_cert = ca_params.self_signed(&ca_key).unwrap();
                let ca_cert_pem = ca_cert.pem();

                let mut srv_params = rcgen::CertificateParams::new(vec!["localhost".to_string()]).unwrap();
                // Set validity to past: 2020-01-01 to 2020-01-02
                srv_params.not_before = rcgen::date_time_ymd(2020, 1, 1);
                srv_params.not_after = rcgen::date_time_ymd(2020, 1, 2);
                let srv_key = rcgen::KeyPair::generate().unwrap();
                let srv_cert = srv_params.signed_by(&srv_key, &ca_cert, &ca_key).unwrap();
                let srv_cert_pem = srv_cert.pem();
                let srv_key_pem = srv_key.serialize_pem();

                Self {
                    ca_cert_pem,
                    server_cert_pem: srv_cert_pem,
                    server_key_pem: srv_key_pem,
                }
            }
            TestServerMode::WrongHostname => {
                // Generate certificate for evil.com signed by valid CA
                let mut ca_params = rcgen::CertificateParams::default();
                ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
                let ca_key = rcgen::KeyPair::generate().unwrap();
                let ca_cert = ca_params.self_signed(&ca_key).unwrap();
                let ca_cert_pem = ca_cert.pem();

                let mut srv_params = rcgen::CertificateParams::new(vec!["evil.com".to_string(), "attacker.net".to_string()]).unwrap();
                srv_params.distinguished_name.push(rcgen::DnType::CommonName, "evil.com");
                let srv_key = rcgen::KeyPair::generate().unwrap();
                let srv_cert = srv_params.signed_by(&srv_key, &ca_cert, &ca_key).unwrap();
                let srv_cert_pem = srv_cert.pem();
                let srv_key_pem = srv_key.serialize_pem();

                Self {
                    ca_cert_pem,
                    server_cert_pem: srv_cert_pem,
                    server_key_pem: srv_key_pem,
                }
            }
            TestServerMode::UntrustedCert => {
                // Generate standalone self-signed certificate (not signed by any CA)
                let srv_params = rcgen::CertificateParams::new(vec!["localhost".to_string()]).unwrap();
                let srv_key = rcgen::KeyPair::generate().unwrap();
                let srv_cert = srv_params.self_signed(&srv_key).unwrap();
                let srv_cert_pem = srv_cert.pem();
                let srv_key_pem = srv_key.serialize_pem();

                Self {
                    ca_cert_pem: String::new(),
                    server_cert_pem: srv_cert_pem,
                    server_key_pem: srv_key_pem,
                }
            }
            TestServerMode::MitmCert => {
                // Generate certificate signed by an Attacker CA
                let mut attacker_ca_params = rcgen::CertificateParams::default();
                attacker_ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
                attacker_ca_params.distinguished_name.push(
                    rcgen::DnType::CommonName,
                    "Untrusted Attacker MITM CA",
                );
                let attacker_ca_key = rcgen::KeyPair::generate().unwrap();
                let attacker_ca = attacker_ca_params.self_signed(&attacker_ca_key).unwrap();
                let attacker_ca_pem = attacker_ca.pem();

                let mut srv_params = rcgen::CertificateParams::new(vec!["localhost".to_string()]).unwrap();
                let srv_key = rcgen::KeyPair::generate().unwrap();
                let srv_cert = srv_params.signed_by(&srv_key, &attacker_ca, &attacker_ca_key).unwrap();
                let srv_cert_pem = srv_cert.pem();
                let srv_key_pem = srv_key.serialize_pem();

                Self {
                    ca_cert_pem: attacker_ca_pem,
                    server_cert_pem: srv_cert_pem,
                    server_key_pem: srv_key_pem,
                }
            }
        }
    }
}

/// Deterministic in-process TLS Test Server
pub struct TlsTestServer {
    addr: SocketAddr,
    pki: TestPki,
    shutdown: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<()>>,
}

impl TlsTestServer {
    pub fn start(mode: TestServerMode) -> Result<Self, TlsError> {
        let pki = TestPki::generate(mode);
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        listener.set_nonblocking(true)?;

        let version = match mode {
            TestServerMode::Tls13Only => TlsVersion::Tls13Only,
            TestServerMode::Tls12Only => TlsVersion::Tls12Only,
            _ => TlsVersion::All,
        };

        let mut config_builder = TlsServerConfigBuilder::new();
        config_builder
            .set_cert_and_key_pem(&pki.server_cert_pem, &pki.server_key_pem)?
            .set_version(version);
        let server_config = config_builder.build()?;

        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);

        let thread_handle = thread::spawn(move || {
            while !shutdown_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let _ = stream.set_nonblocking(false);
                        let config = Arc::clone(&server_config);
                        thread::spawn(move || {
                            if let Ok(mut session) = TlsServerSession::accept(stream, config) {
                                let mut buf = [0u8; 4096];
                                if let Ok(n) = session.read(&mut buf) {
                                    if n > 0 {
                                        let req = String::from_utf8_lossy(&buf[..n]);
                                        let resp = if req.starts_with("GET") {
                                            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 19\r\n\r\nEnd TLS 1.3 Verified"
                                        } else {
                                            "ACK: Real Cryptographic TLS 1.3 Transmission Received"
                                        };
                                        let _ = session.write_all(resp.as_bytes());
                                        let _ = session.close();
                                    }
                                }
                            }
                        });
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            addr,
            pki,
            shutdown,
            thread_handle: Some(thread_handle),
        })
    }

    pub fn port(&self) -> u16 {
        self.addr.port()
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn ca_cert_pem(&self) -> &str {
        &self.pki.ca_cert_pem
    }

    pub fn server_cert_pem(&self) -> &str {
        &self.pki.server_cert_pem
    }

    pub fn server_key_pem(&self) -> &str {
        &self.pki.server_key_pem
    }

    pub fn stop(mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for TlsTestServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
    }
}

/// Real Wire Traffic Capture & Ciphertext Verifier
pub struct TlsWireRecorder {
    client_to_server: Vec<u8>,
    server_to_client: Vec<u8>,
}

impl TlsWireRecorder {
    pub fn record_session(
        server_addr: SocketAddr,
        client_action: impl FnOnce(TcpStream) -> Result<(), TlsError>,
    ) -> Result<Self, TlsError> {
        // Intercept connection via a recording proxy
        let proxy_listener = TcpListener::bind("127.0.0.1:0")?;
        let proxy_addr = proxy_listener.local_addr()?;

        let client_bytes = Arc::new(std::sync::Mutex::new(Vec::new()));
        let server_bytes = Arc::new(std::sync::Mutex::new(Vec::new()));

        let c_bytes_clone = Arc::clone(&client_bytes);
        let s_bytes_clone = Arc::clone(&server_bytes);

        let proxy_thread = thread::spawn(move || -> io::Result<()> {
            let (mut client_stream, _) = proxy_listener.accept()?;
            let mut server_stream = TcpStream::connect(server_addr)?;

            client_stream.set_nonblocking(true)?;
            server_stream.set_nonblocking(true)?;

            let mut c_buf = [0u8; 8192];
            let mut s_buf = [0u8; 8192];
            let start = std::time::Instant::now();

            while start.elapsed() < Duration::from_secs(5) {
                let mut did_work = false;
                match client_stream.read(&mut c_buf) {
                    Ok(n) if n > 0 => {
                        c_bytes_clone.lock().unwrap().extend_from_slice(&c_buf[..n]);
                        server_stream.write_all(&c_buf[..n])?;
                        did_work = true;
                    }
                    Ok(_) => break,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => break,
                }

                match server_stream.read(&mut s_buf) {
                    Ok(n) if n > 0 => {
                        s_bytes_clone.lock().unwrap().extend_from_slice(&s_buf[..n]);
                        client_stream.write_all(&s_buf[..n])?;
                        did_work = true;
                    }
                    Ok(_) => break,
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {}
                    Err(_) => break,
                }

                if !did_work {
                    thread::sleep(Duration::from_millis(2));
                }
            }
            Ok(())
        });

        // Connect client to proxy
        let client_stream = TcpStream::connect(proxy_addr)?;
        client_action(client_stream)?;

        let _ = proxy_thread.join();

        let client_to_server = client_bytes.lock().unwrap().clone();
        let server_to_client = server_bytes.lock().unwrap().clone();

        Ok(Self {
            client_to_server,
            server_to_client,
        })
    }

    pub fn client_raw_wire_bytes(&self) -> &[u8] {
        &self.client_to_server
    }

    pub fn server_raw_wire_bytes(&self) -> &[u8] {
        &self.server_to_client
    }

    /// Verifies that plaintext string does NOT appear as plaintext on the wire
    pub fn assert_plaintext_absent(&self, plaintext: &str) -> bool {
        let pattern = plaintext.as_bytes();
        let found_in_client = self.client_to_server.windows(pattern.len()).any(|w| w == pattern);
        let found_in_server = self.server_to_client.windows(pattern.len()).any(|w| w == pattern);
        !found_in_client && !found_in_server
    }

    /// Validates TLS 1.3/1.2 Record Layer encapsulation
    pub fn verify_tls_record_structure(&self) -> bool {
        // Must contain Handshake (0x16, 0x03) and Application Data (0x17, 0x03)
        let has_handshake = self.client_to_server.windows(2).any(|w| w == [0x16, 0x03]);
        let has_app_data = self.client_to_server.windows(2).any(|w| w == [0x17, 0x03])
            || self.server_to_client.windows(2).any(|w| w == [0x17, 0x03]);

        has_handshake && has_app_data
    }
}
