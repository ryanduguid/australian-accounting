// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Certificate checks against real servers: `cargo test -- --ignored`.

use std::io::{Read, Write};

use lodgeit_calculator_adapter::dial::Net;
use lodgeit_calculator_adapter::transport::Dial;

#[test]
#[ignore = "reaches the internet"]
fn tls_verifies_the_certificate_and_its_name() {
    let mut connection = Net.dial("example.com", 443, true, 10.0).unwrap();
    connection
        .write_all(b"HEAD / HTTP/1.1\r\nHost: example.com\r\nConnection: close\r\n\r\n")
        .unwrap();
    connection.flush().unwrap();
    let mut status = [0u8; 9];
    connection.read_exact(&mut status).unwrap();
    assert_eq!(&status, b"HTTP/1.1 ");
    assert!(Net.dial("wrong.host.badssl.com", 443, true, 10.0).is_err());
    assert!(Net.dial("self-signed.badssl.com", 443, true, 10.0).is_err());
}
