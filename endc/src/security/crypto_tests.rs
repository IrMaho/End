// Comprehensive Cryptographic Test Suite for P14
// Covers NIST FIPS 180-4 SHA-256 vectors, RFC 4231 HMAC-SHA256 (all 7 cases),
// Argon2id deterministic hashing, constant-time verification, negative attack matrix,
// and bi-directional Python interoperability.

#[cfg(test)]
mod tests {
    use super::super::crypto::*;
    use std::process::Command;

    // =========================================================================
    // 1. NIST FIPS 180-4 SHA-256 Authoritative Test Vectors
    // =========================================================================

    #[test]
    fn test_nist_sha256_empty_string() {
        let digest = sha256_hex(b"");
        assert_eq!(
            digest,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            "NIST SHA-256 Vector 1 (Empty String) must match exactly"
        );
    }

    #[test]
    fn test_nist_sha256_abc() {
        let digest = sha256_hex(b"abc");
        assert_eq!(
            digest,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
            "NIST SHA-256 Vector 2 ('abc') must match exactly"
        );
    }

    #[test]
    fn test_nist_sha256_two_block_input() {
        let input = b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq";
        let digest = sha256_hex(input);
        assert_eq!(
            digest,
            "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1",
            "NIST SHA-256 Vector 3 (Two-block 56-byte string) must match exactly"
        );
    }

    #[test]
    fn test_nist_sha256_million_a_chars() {
        let input = vec![b'a'; 1_000_000];
        let digest = sha256_hex(&input);
        assert_eq!(
            digest,
            "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0",
            "NIST SHA-256 Vector 4 (1,000,000 'a' characters) must match exactly"
        );
    }

    // =========================================================================
    // 2. RFC 4231 HMAC-SHA256 Authoritative Test Vectors (Cases 1 to 7)
    // =========================================================================

    #[test]
    fn test_rfc4231_hmac_case1() {
        // Case 1: Key = 20 bytes 0x0b, Data = "Hi There"
        let key = vec![0x0bu8; 20];
        let data = b"Hi There";
        let tag_hex = hmac_sha256_hex(&key, data);
        assert_eq!(
            tag_hex,
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7",
            "RFC 4231 Case 1 failed"
        );
        assert!(hmac_sha256_verify_hex(&key, data, &tag_hex));
    }

    #[test]
    fn test_rfc4231_hmac_case2() {
        // Case 2: Key = "Jefe", Data = "what do ya want for nothing?"
        let key = b"Jefe";
        let data = b"what do ya want for nothing?";
        let tag_hex = hmac_sha256_hex(key, data);
        assert_eq!(
            tag_hex,
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843",
            "RFC 4231 Case 2 failed"
        );
        assert!(hmac_sha256_verify_hex(key, data, &tag_hex));
    }

    #[test]
    fn test_rfc4231_hmac_case3() {
        // Case 3: Key = 20 bytes 0xaa, Data = 50 bytes 0xdd
        let key = vec![0xaau8; 20];
        let data = vec![0xddu8; 50];
        let tag_hex = hmac_sha256_hex(&key, &data);
        assert_eq!(
            tag_hex,
            "773ea91e36800e46854db8ebd09181a72959098b3ef8c122d9635514ced565fe",
            "RFC 4231 Case 3 failed"
        );
        assert!(hmac_sha256_verify_hex(&key, &data, &tag_hex));
    }

    #[test]
    fn test_rfc4231_hmac_case4() {
        // Case 4: Key = 25 bytes 0x01..0x19, Data = 50 bytes 0xcd
        let key: Vec<u8> = (0x01..=0x19).collect();
        let data = vec![0xcdu8; 50];
        let tag_hex = hmac_sha256_hex(&key, &data);
        assert_eq!(
            tag_hex,
            "82558a389a443c0ea4cc819899f2083a85f0faa3e578f8077a2e3ff46729665b",
            "RFC 4231 Case 4 failed"
        );
        assert!(hmac_sha256_verify_hex(&key, &data, &tag_hex));
    }

    #[test]
    fn test_rfc4231_hmac_case5() {
        // Case 5: Key = 20 bytes 0x0c, Data = "Test With Truncation"
        let key = vec![0x0cu8; 20];
        let data = b"Test With Truncation";
        let tag_hex = hmac_sha256_hex(&key, data);
        assert_eq!(
            tag_hex,
            "a3b6167473100ee06e0c796c2955552bfa6f7c0a6a8aef8b93f860aab0cd20c5",
            "RFC 4231 Case 5 failed"
        );
        assert!(hmac_sha256_verify_hex(&key, data, &tag_hex));
    }

    #[test]
    fn test_rfc4231_hmac_case6() {
        // Case 6: Key = 131 bytes 0xaa (larger than block size 64), Data = "Test Using Larger Than Block-Size Key - Hash Key First"
        let key = vec![0xaau8; 131];
        let data = b"Test Using Larger Than Block-Size Key - Hash Key First";
        let tag_hex = hmac_sha256_hex(&key, data);
        assert_eq!(
            tag_hex,
            "60e431591ee0b67f0d8a26aacbf5b77f8e0bc6213728c5140546040f0ee37f54",
            "RFC 4231 Case 6 failed"
        );
        assert!(hmac_sha256_verify_hex(&key, data, &tag_hex));
    }

    #[test]
    fn test_rfc4231_hmac_case7() {
        // Case 7: Key = 131 bytes 0xaa, Data = "This is a test using a larger than block-size key and a larger than block-size data. The key needs to be hashed before being used by the HMAC algorithm."
        let key = vec![0xaau8; 131];
        let data = b"This is a test using a larger than block-size key and a larger than block-size data. The key needs to be hashed before being used by the HMAC algorithm.";
        let tag_hex = hmac_sha256_hex(&key, data);
        assert_eq!(
            tag_hex,
            "9b09ffa71b942fcb27635fbcd5b0e944bfdc63644f0713938a7f51535c3a35e2",
            "RFC 4231 Case 7 failed"
        );
        assert!(hmac_sha256_verify_hex(&key, data, &tag_hex));
    }

    // =========================================================================
    // 3. Argon2id Positive & Deterministic Test Matrix
    // =========================================================================

    #[test]
    fn test_argon2id_deterministic_hashing() {
        let password = b"SuperSecretPassword2026!";
        let salt = "somesalt16bytesX";
        let cfg = Argon2Config {
            memory_cost_kib: 65536,
            time_cost: 3,
            parallelism: 4,
            output_len: 32,
        };

        let hash1 = argon2id_hash_with_config(password, Some(salt), cfg).unwrap();
        let hash2 = argon2id_hash_with_config(password, Some(salt), cfg).unwrap();

        assert_eq!(hash1, hash2, "Same password + same salt + same params must be 100% deterministic");
        assert!(hash1.starts_with("$argon2id$v=19$m=65536,t=3,p=4$"), "Must contain standard PHC prefix");

        // Verify password against generated hash
        let verified = argon2id_verify(password, &hash1).unwrap();
        assert!(verified, "Password verification must succeed for correct password");

        // Verify wrong password fails
        let wrong_verified = argon2id_verify(b"WrongPassword123", &hash1).unwrap();
        assert!(!wrong_verified, "Password verification must fail for wrong password");
    }

    #[test]
    fn test_argon2id_different_salts_produce_different_hashes() {
        let password = b"ConsistentPassword";
        let salt1 = "saltOne16bytesXX";
        let salt2 = "saltTwo16bytesYY";
        let cfg = Argon2Config::default();

        let hash1 = argon2id_hash_with_config(password, Some(salt1), cfg).unwrap();
        let hash2 = argon2id_hash_with_config(password, Some(salt2), cfg).unwrap();

        assert_ne!(hash1, hash2, "Different salts must produce different hashes");
        assert!(argon2id_verify(password, &hash1).unwrap());
        assert!(argon2id_verify(password, &hash2).unwrap());
    }

    #[test]
    fn test_argon2id_default_config_parameters() {
        let cfg = Argon2Config::default();
        assert_eq!(cfg.memory_cost_kib, 65536, "Default memory must be 64 MiB (65536 KiB)");
        assert_eq!(cfg.time_cost, 3, "Default time cost must be 3");
        assert_eq!(cfg.parallelism, 4, "Default parallelism must be 4");
        assert_eq!(cfg.output_len, 32, "Default output length must be 32 bytes");

        let hash = argon2id_hash(b"test_default_password").unwrap();
        assert!(hash.contains("m=65536,t=3,p=4"), "Generated hash must encode default params");
    }

    // =========================================================================
    // 4. Constant-Time Verification & Password Boundaries
    // =========================================================================

    #[test]
    fn test_constant_time_equality_primitives() {
        let secret1 = "CorrectSecretTokenValue_1234567890";
        let secret2 = "CorrectSecretTokenValue_1234567890";
        let secret3 = "WrongSecretTokenValue_1234567890X";

        assert!(constant_time_eq_str(secret1, secret2));
        assert!(!constant_time_eq_str(secret1, secret3));
        assert!(!constant_time_eq_str(secret1, "Short"));

        assert!(constant_time_eq_bytes(secret1.as_bytes(), secret2.as_bytes()));
        assert!(!constant_time_eq_bytes(secret1.as_bytes(), secret3.as_bytes()));
    }

    #[test]
    fn test_argon2id_empty_password() {
        let empty_pw = b"";
        let hash = argon2id_hash(empty_pw).unwrap();
        assert!(argon2id_verify(empty_pw, &hash).unwrap(), "Empty password should be hashable and verifiable");
        assert!(!argon2id_verify(b"non_empty", &hash).unwrap(), "Non-empty password must not verify against empty password hash");
    }

    #[test]
    fn test_argon2id_large_password() {
        // Test password with 2048 bytes
        let large_pw = vec![b'K'; 2048];
        let hash = argon2id_hash(&large_pw).unwrap();
        assert!(argon2id_verify(&large_pw, &hash).unwrap(), "2048-byte large password must verify successfully");
        assert!(!argon2id_verify(b"short_pw", &hash).unwrap());
    }

    // =========================================================================
    // 5. Adversarial Malformed / Error-Path Attack Matrix
    // =========================================================================

    #[test]
    fn test_malformed_phc_rejected() {
        let valid_pw = b"password";

        // 1. Completely corrupted string
        let err1 = argon2id_verify(valid_pw, "not_a_phc_hash");
        assert!(err1.is_err(), "Non-PHC string must be rejected");

        // 2. Empty string
        let err2 = argon2id_verify(valid_pw, "");
        assert!(err2.is_err(), "Empty string must be rejected");

        // 3. Truncated PHC
        let err3 = argon2id_verify(valid_pw, "$argon2id$v=19$m=65536,t=3,p=4$short");
        assert!(err3.is_err(), "Truncated PHC must be rejected");

        // 4. Algorithm mismatch ($argon2i$)
        let err4 = argon2id_verify(valid_pw, "$argon2i$v=19$m=65536,t=3,p=4$c29tZXNhbHQ$eD/vO1ZJg9L4sU");
        assert!(err4.is_err(), "Argon2i must be rejected when expecting Argon2id");

        // 5. Unsupported version
        let err5 = argon2id_verify(valid_pw, "$argon2id$v=99$m=65536,t=3,p=4$c29tZXNhbHQ$eD/vO1ZJg9L4sU");
        assert!(err5.is_err(), "Invalid version must be rejected");

        // 6. Invalid parameters (m=0)
        let err6 = argon2id_verify(valid_pw, "$argon2id$v=19$m=0,t=3,p=4$c29tZXNhbHQ$eD/vO1ZJg9L4sU");
        assert!(err6.is_err(), "Zero memory parameter must be rejected");
    }

    #[test]
    fn test_invalid_argon2_config_parameters_rejected() {
        let invalid_cfg = Argon2Config {
            memory_cost_kib: 0, // Invalid: memory must be at least 8 * p
            time_cost: 0,       // Invalid: time must be at least 1
            parallelism: 0,    // Invalid: parallelism must be at least 1
            output_len: 0,
        };
        let res = argon2id_hash_with_config(b"pass", Some("salt16bytes"), invalid_cfg);
        assert!(res.is_err(), "Invalid parameters must fail closed");
    }

    // =========================================================================
    // 6. Bi-Directional Python Argon2 Interoperability
    // =========================================================================

    #[test]
    fn test_python_to_end_interoperability() {
        // Execute Python to generate an authentic Argon2id hash using argon2-cffi
        let py_script = r#"
from argon2 import PasswordHasher, Type
ph = PasswordHasher(time_cost=3, memory_cost=65536, parallelism=4, hash_len=32, type=Type.ID)
h = ph.hash("CrossPlatformPassword2026!")
print(h)
"#;
        let output = Command::new("python")
            .args(&["-c", py_script])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let py_hash = String::from_utf8_lossy(&out.stdout).trim().to_string();
                println!("Python generated Argon2id hash: {}", py_hash);
                assert!(py_hash.starts_with("$argon2id$v=19$m=65536,t=3,p=4$"));

                // End verifies the Python-generated hash
                let verify_result = argon2id_verify(b"CrossPlatformPassword2026!", &py_hash);
                assert!(verify_result.is_ok(), "End must parse and verify Python Argon2id hash");
                assert!(verify_result.unwrap(), "Password must match successfully");

                // Wrong password verification fails
                let wrong_result = argon2id_verify(b"WrongCrossPlatformPassword", &py_hash);
                assert!(wrong_result.is_ok());
                assert!(!wrong_result.unwrap(), "Wrong password must not match Python hash");
            }
            Ok(out) => {
                let err_msg = String::from_utf8_lossy(&out.stderr);
                panic!("Python command failed with stderr: {}", err_msg);
            }
            Err(e) => {
                panic!("Failed to invoke Python for interoperability test: {}", e);
            }
        }
    }

    #[test]
    fn test_end_to_python_interoperability() {
        // End generates an authentic Argon2id hash
        let password = "EndGeneratedSecret@2026!";
        let end_hash = argon2id_hash(password.as_bytes()).unwrap();
        println!("End generated Argon2id hash: {}", end_hash);
        assert!(end_hash.starts_with("$argon2id$v=19$m=65536,t=3,p=4$"));

        // Execute Python to verify the End-generated hash
        let py_script = format!(
            r#"
from argon2 import PasswordHasher
ph = PasswordHasher()
try:
    verified = ph.verify("{}", "{}")
    print("MATCH:True")
except Exception as e:
    print(f"MATCH:False ({{e}})")
"#,
            end_hash, password
        );

        let output = Command::new("python")
            .args(&["-c", &py_script])
            .output();

        match output {
            Ok(out) if out.status.success() => {
                let py_out = String::from_utf8_lossy(&out.stdout);
                println!("Python verification output: {}", py_out.trim());
                assert!(py_out.contains("MATCH:True"), "Python must successfully verify End Argon2id hash");
            }
            Ok(out) => {
                let err_msg = String::from_utf8_lossy(&out.stderr);
                panic!("Python verification failed with stderr: {}", err_msg);
            }
            Err(e) => {
                panic!("Failed to invoke Python for interoperability test: {}", e);
            }
        }
    }
}
