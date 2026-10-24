//! The dialler `transport::request` uses outside the differential:
//! `socket.create_connection((host, port), timeout)` and, for https,
//! `SSLContext.wrap_socket(sock, server_hostname=host)` with the default
//! context, as `http.client` runs them under the adapter's handlers.
//!
//! - The timeout bounds the connect to each address and every later read
//!   and write, as Python's socket timeout does, the TLS handshake included.
//! - TLS is rustls with the operating system's certificate verifier, where
//!   Python uses OpenSSL with the system's roots: both refuse an untrusted
//!   certificate or a name that does not match, and both offer HTTP/1.1 by
//!   ALPN. Their failure wording differs, which `request` never shows.
//! - A TLS connection closed without `close_notify` is a read error, not
//!   the end of the body, as the adapter's `suppress_ragged_eofs = False`.
//! - Plain HTTP is dialled only where `config.check_url` allowed it, which
//!   is loopback with `allow_loopback` set.

use std::io;
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::pyops::{Exc, R};
use crate::transport::{Connection, Dial, Raised};

/// The dialler for real connections.
pub struct Net;

impl Dial for Net {
    fn dial(
        &mut self,
        host: &str,
        port: u16,
        tls: bool,
        timeout: f64,
    ) -> io::Result<Box<dyn Connection>> {
        let addresses: Vec<SocketAddr> = (host, port).to_socket_addrs()?.collect();
        if addresses.is_empty() {
            return Err(io::Error::other("getaddrinfo returns an empty list"));
        }
        // `sock.settimeout(timeout)` runs once an address is known.
        let timeout = socket_timeout(timeout)?;
        let mut last = None;
        let mut socket = None;
        for address in addresses {
            match TcpStream::connect_timeout(&address, timeout) {
                Ok(s) => {
                    socket = Some(s);
                    break;
                }
                Err(e) => last = Some(e),
            }
        }
        let socket = match (socket, last) {
            (Some(socket), _) => socket,
            (None, error) => return Err(error.unwrap_or_else(|| io::Error::other("no address"))),
        };
        let _ = socket.set_nodelay(true);
        socket.set_read_timeout(Some(timeout))?;
        socket.set_write_timeout(Some(timeout))?;
        if !tls {
            return Ok(Box::new(socket));
        }
        let name = rustls::pki_types::ServerName::try_from(host)
            .map_err(io::Error::other)?
            .to_owned();
        let config = crate::https::tls_config().map_err(io::Error::other)?;
        let mut session = rustls::ClientConnection::new(config, name).map_err(io::Error::other)?;
        let mut socket = socket;
        // `do_handshake()`: each read and write under the socket timeout.
        while session.is_handshaking() {
            session.complete_io(&mut socket)?;
        }
        Ok(Box::new(rustls::StreamOwned::new(session, socket)))
    }
}

/// `sock.settimeout(seconds)`: CPython rounds away from zero to whole
/// nanoseconds and refuses what its clock type, or Windows' `timeval`,
/// cannot hold. NaN and a negative timeout are `ValueError`s, which
/// `_attempt` catches as a failure to open, as it does the connect a zero
/// timeout makes non-blocking; an overflow is an `OverflowError`, which
/// nothing catches.
pub(crate) fn socket_timeout(seconds: f64) -> io::Result<Duration> {
    let overflow = |message: &str| io::Error::other(Raised(Exc::msg("OverflowError", message)));
    let nanoseconds = pytime(seconds).map_err(|exc| match exc.class {
        "OverflowError" => io::Error::other(Raised(exc)),
        _ => io::Error::other(crate::pyvalue::points_lossy(&exc.text)),
    })?;
    if nanoseconds < 0.0 {
        return Err(io::Error::other("Timeout value out of range"));
    }
    if cfg!(windows) && seconds > 2_147_483.647 {
        return Err(overflow("timeout doesn't fit into C timeval"));
    }
    if nanoseconds == 0.0 {
        return Err(io::ErrorKind::WouldBlock.into());
    }
    Ok(Duration::from_nanos(nanoseconds as u64))
}

/// CPython's seconds-to-`PyTime_t` conversion with `_PyTime_ROUND_TIMEOUT`:
/// whole nanoseconds rounded away from zero, or its `ValueError` for NaN and
/// `OverflowError` past the clock type.
fn pytime(seconds: f64) -> R<f64> {
    if seconds.is_nan() {
        return Err(Exc::msg("ValueError", "Invalid value NaN (not a number)"));
    }
    let scaled = seconds * 1e9;
    let nanoseconds = if scaled < 0.0 {
        scaled.floor()
    } else {
        scaled.ceil()
    };
    if !(-9_223_372_036_854_775_808.0..9_223_372_036_854_775_808.0).contains(&nanoseconds) {
        return Err(Exc::msg(
            "OverflowError",
            "timestamp out of range for C PyTime_t",
        ));
    }
    Ok(nanoseconds)
}

/// `time.sleep(seconds)`, the transport's backoff. Measured with CPython
/// 3.15: Linux also refuses, with `EINVAL`, a sleep whose deadline overflows
/// the monotonic clock (about 292 years); this sleeps instead.
pub fn sleep(seconds: f64) -> R<()> {
    let nanoseconds = pytime(seconds)?;
    if nanoseconds < 0.0 {
        return Err(Exc::msg("ValueError", "sleep length must be non-negative"));
    }
    std::thread::sleep(Duration::from_nanos(nanoseconds as u64));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    fn raised(seconds: f64) -> Option<String> {
        let error = socket_timeout(seconds).err()?;
        let raised = error.into_inner()?.downcast::<Raised>().ok()?;
        Some(
            raised
                .0
                .text
                .iter()
                .filter_map(|&cp| char::from_u32(cp))
                .collect(),
        )
    }

    #[test]
    fn sleep_refuses_what_time_sleep_refuses() {
        // Measured with CPython 3.15's time.sleep on Windows and Linux.
        let refused = |seconds: f64| {
            sleep(seconds)
                .err()
                .map(|e| (e.class, crate::pyvalue::points_lossy(&e.text)))
        };
        let pytime = Some((
            "OverflowError",
            "timestamp out of range for C PyTime_t".to_owned(),
        ));
        assert_eq!(refused(f64::INFINITY), pytime);
        assert_eq!(refused(-9.3e9), pytime);
        assert_eq!(
            refused(f64::NAN),
            Some(("ValueError", "Invalid value NaN (not a number)".to_owned()))
        );
        let negative = Some(("ValueError", "sleep length must be non-negative".to_owned()));
        assert_eq!(refused(-1e-300), negative);
        assert_eq!(refused(-1.0), negative);
        for slept in [0.0, -0.0, 5e-324] {
            assert_eq!(refused(slept), None);
        }
    }

    #[test]
    fn timeouts_are_refused_where_cpython_refuses_them() {
        // Measured with CPython 3.15's socket.settimeout on Windows and Linux.
        assert_eq!(socket_timeout(1e-300).unwrap(), Duration::from_nanos(1));
        assert_eq!(socket_timeout(0.001).unwrap(), Duration::from_millis(1));
        assert!(socket_timeout(2_147_483.647).is_ok());
        let timeval = raised(2_147_483.647_000_000_3);
        if cfg!(windows) {
            assert_eq!(
                timeval.as_deref(),
                Some("timeout doesn't fit into C timeval")
            );
        } else {
            assert_eq!(timeval, None);
            assert!(socket_timeout(9_223_372_036.854_774).is_ok());
        }
        let pytime = Some("timestamp out of range for C PyTime_t");
        assert_eq!(raised(9_223_372_036.854_776).as_deref(), pytime);
        assert_eq!(raised(f64::INFINITY).as_deref(), pytime);
        assert_eq!(raised(-9.3e9).as_deref(), pytime);
        assert_eq!(raised(f64::NEG_INFINITY).as_deref(), pytime);
        let nan = socket_timeout(f64::NAN).unwrap_err();
        assert_eq!(nan.to_string(), "Invalid value NaN (not a number)");
        assert!(nan.into_inner().unwrap().downcast::<Raised>().is_err());
        for negative in [-1e-300, -1.0] {
            let error = socket_timeout(negative).unwrap_err();
            assert_eq!(error.to_string(), "Timeout value out of range");
        }
        for zero in [0.0, -0.0] {
            assert_eq!(
                socket_timeout(zero).unwrap_err().kind(),
                io::ErrorKind::WouldBlock
            );
        }
    }

    #[test]
    fn plain_http_reaches_a_loopback_listener() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut peer, _) = listener.accept().unwrap();
            let mut got = [0u8; 5];
            peer.read_exact(&mut got).unwrap();
            peer.write_all(b"pong").unwrap();
            got
        });
        let mut connection = Net.dial("127.0.0.1", port, false, 5.0).unwrap();
        connection.write_all(b"ping!").unwrap();
        let mut reply = String::new();
        connection.read_to_string(&mut reply).unwrap();
        assert_eq!(reply, "pong");
        assert_eq!(&server.join().unwrap(), b"ping!");
    }

    #[test]
    fn a_silent_peer_times_out_and_tls_to_a_plain_peer_fails() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut peer, _) = listener.accept().unwrap();
            let mut sink = Vec::new();
            let _ = peer.read_to_end(&mut sink); // silent until the client gives up
            let (mut peer, _) = listener.accept().unwrap();
            let _ = peer.write_all(b"HTTP/1.1 200 OK\r\n\r\nnot TLS");
        });
        let mut connection = Net.dial("127.0.0.1", port, false, 0.2).unwrap();
        let mut byte = [0u8; 1];
        let error = connection.read(&mut byte).unwrap_err();
        assert!(matches!(
            error.kind(),
            io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
        ));
        drop(connection);
        assert!(Net.dial("127.0.0.1", port, true, 5.0).is_err());
        server.join().unwrap();
    }

    /// Certificate checks need a real server: `cargo test -- --ignored`.
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
}
