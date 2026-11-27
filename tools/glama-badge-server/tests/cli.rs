// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use std::net::TcpListener;
use std::process::{Command, Output};
use std::sync::Mutex;

// Tests that bind a fresh loopback port hold this, so one test's port, freed for
// the server to take, is never handed to another test's listener in between.
static PORTS: Mutex<()> = Mutex::new(());

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_glama-badge-server"))
        .args(args)
        .output()
        .unwrap()
}

fn refused(args: &[&str]) -> String {
    let output = run(args);
    assert_eq!(output.status.code(), Some(1), "{args:?}");
    assert!(output.stdout.is_empty());
    String::from_utf8(output.stderr).unwrap()
}

fn ports() -> std::sync::MutexGuard<'static, ()> {
    PORTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[test]
fn unknown_arguments_print_the_usage() {
    for args in [&["--bogus"][..], &["--listen"], &["--port", "127.0.0.1:1"]] {
        assert_eq!(
            refused(args),
            "Error: \"Usage: glama-badge-server [--listen IP:PORT]\"\n"
        );
    }
}

#[test]
fn an_address_that_is_not_a_socket_address_is_refused() {
    assert_eq!(
        refused(&["--listen", "localhost"]),
        "Error: AddrParseError(Socket)\n"
    );
}

#[test]
fn a_taken_address_is_refused_before_serving() {
    let _ports = ports();
    let taken = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = taken.local_addr().unwrap().to_string();
    assert!(refused(&["--listen", &address]).starts_with("Error: Os {"));
}

#[cfg(unix)]
mod interrupt {
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::process::{Child, Command};
    use std::time::{Duration, Instant};

    fn connect(address: &str, child: &mut Child) -> TcpStream {
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            if let Ok(stream) = TcpStream::connect(address) {
                return stream;
            }
            assert!(child.try_wait().unwrap().is_none(), "server exited early");
            assert!(Instant::now() < deadline, "server never listened");
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    // SIGINT is the shutdown signal tokio::signal::ctrl_c waits for; Windows has no
    // portable way to send a child its console control event.
    fn serves_then_stops_on_interrupt(args: &[&str], address: &str) {
        let mut child = Command::new(env!("CARGO_BIN_EXE_glama-badge-server"))
            .args(args)
            .spawn()
            .unwrap();
        let mut stream = connect(address, &mut child);
        stream
            .write_all(
                b"GET /glama/unknown.svg HTTP/1.1\r\nHost: badge\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
        assert!(
            response.starts_with("HTTP/1.1 404 Not Found\r\n"),
            "{response}"
        );
        assert!(response.ends_with("Unknown badge"), "{response}");
        let status = Command::new("kill")
            .args(["-INT", &child.id().to_string()])
            .status()
            .unwrap();
        assert!(status.success());
        assert!(child.wait().unwrap().success());
    }

    #[test]
    fn an_explicit_address_serves_and_stops_cleanly() {
        let _ports = super::ports();
        let free = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = free.local_addr().unwrap().to_string();
        drop(free);
        serves_then_stops_on_interrupt(&["--listen", &address], &address);
    }

    #[test]
    fn no_arguments_serve_on_the_loopback_default() {
        serves_then_stops_on_interrupt(&[], "127.0.0.1:8080");
    }
}
