// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! A TLS round trip through the real dialler. On Linux the platform verifier
//! reads its roots from `SSL_CERT_FILE` when that is set, so this binary
//! trusts a test authority made for it and nothing else; the other
//! platforms' verifiers read only the system store.
#![cfg(target_os = "linux")]

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

use lodgeit_calculator_adapter::dial::Net;
use lodgeit_calculator_adapter::transport::Dial;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

/// A server for 127.0.0.1 whose certificate the test authority issued.
fn server_config() -> rustls::ServerConfig {
    let cert = include_bytes!("../fixtures/tls/trusted-loopback.cert.der");
    let key = include_bytes!("../fixtures/tls/trusted-loopback.key.der");
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    rustls::ServerConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_single_cert(
            vec![CertificateDer::from(cert.to_vec())],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key.to_vec())),
        )
        .unwrap()
}

/// A DER certificate as the PEM text `SSL_CERT_FILE` holds.
fn pem(der: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut base64 = Vec::new();
    for chunk in der.chunks(3) {
        let bits = chunk.iter().enumerate().fold(0u32, |bits, (i, &byte)| {
            bits | u32::from(byte) << (16 - 8 * i)
        });
        for i in 0..4 {
            base64.push(if i <= chunk.len() {
                ALPHABET[(bits >> (18 - 6 * i) & 63) as usize]
            } else {
                b'='
            });
        }
    }
    let lines: Vec<&str> = base64
        .chunks(64)
        .map(|line| std::str::from_utf8(line).unwrap())
        .collect();
    format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        lines.join("\n")
    )
}

#[test]
fn pem_wraps_base64_with_its_padding() {
    assert_eq!(
        pem(b"Man\x00"),
        "-----BEGIN CERTIFICATE-----\nTWFuAA==\n-----END CERTIFICATE-----\n"
    );
}

#[test]
fn a_trusted_certificate_carries_a_request_and_its_reply() {
    let authority = include_bytes!("../fixtures/tls/loopback-authority.cert.der");
    let roots = std::env::temp_dir().join(format!("lodgeit-loopback-{}.pem", std::process::id()));
    std::fs::write(&roots, pem(authority)).unwrap();
    // SAFETY: set before this test starts a thread; the binary's other test
    // reads no environment.
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unsafe { std::env::set_var("SSL_CERT_FILE", &roots) };
    let config = Arc::new(server_config());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (peer, _) = listener.accept().unwrap();
        let session = rustls::ServerConnection::new(config).unwrap();
        let mut tls = rustls::StreamOwned::new(session, peer);
        let mut got = [0u8; 5];
        tls.read_exact(&mut got).unwrap();
        tls.write_all(b"pong").unwrap();
        tls.conn.send_close_notify();
        tls.flush().unwrap();
        got
    });
    let mut connection = Net.dial("127.0.0.1", port, true, 5.0).unwrap();
    assert_eq!(connection.write(b"ping").unwrap(), 4);
    connection.write_all(b"!").unwrap();
    connection.flush().unwrap();
    let mut reply = String::new();
    connection.read_to_string(&mut reply).unwrap();
    assert_eq!(reply, "pong");
    assert_eq!(&server.join().unwrap(), b"ping!");
    std::fs::remove_file(roots).unwrap();
}
