//! `transport::request` behind a `RequestConfig` whose `check_url` accepts
//! every URL: how urllib and `http.client` split the URL, where they connect
//! and what they send, or that they refuse before connecting. The trait is
//! public, so the transport's answer for such a URL is its own behaviour,
//! not `AdapterConfig::check_url`'s. Expected values are the retired Python
//! adapter's `transport._attempt` under CPython 3.15.0 (win32), with
//! `socket.create_connection` replaced by a socketpair.
use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::rc::Rc;

use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::errors::TRANSPORT;
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::transport::{Call, Connection, Dial, RequestConfig, request};
use num_bigint::BigInt;

const OK: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";

/// The adapter's configuration with the URL check left out.
struct AnyUrl(AdapterConfig);

impl RequestConfig for AnyUrl {
    fn require_enabled(&self) -> Result<(), Exc> {
        Ok(())
    }
    fn check_url(&self, _: &str) -> Result<(), Exc> {
        Ok(())
    }
    fn read_timeout(&self) -> f64 {
        self.0.read_timeout
    }
    fn max_response_bytes(&self) -> &BigInt {
        &self.0.max_response_bytes
    }
    fn max_attempts(&self) -> u32 {
        2
    }
    fn retry_backoff_seconds(&self) -> f64 {
        0.0
    }
    fn user_agent(&self) -> &str {
        "fixture-ua/1.0"
    }
    fn extra_headers(&self) -> &[(String, String)] {
        &self.0.extra_headers
    }
}

struct Canned {
    reply: io::Cursor<Vec<u8>>,
    sent: Rc<RefCell<Vec<u8>>>,
}

impl Read for Canned {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reply.read(buf)
    }
}

impl Write for Canned {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sent.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Records each dial's host and port and replies 200 `ok`.
#[derive(Default)]
struct Recorder {
    dials: Vec<(String, u16)>,
    sent: Rc<RefCell<Vec<u8>>>,
}

impl Dial for Recorder {
    fn dial(&mut self, host: &str, port: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
        self.dials.push((host.to_owned(), port));
        Ok(Box::new(Canned {
            reply: io::Cursor::new(OK.to_vec()),
            sent: Rc::clone(&self.sent),
        }))
    }
}

struct Sent {
    result: Result<String, (&'static str, String)>,
    dials: Vec<(String, u16)>,
    wire: String,
}

fn send(method: &str, url: &str, body: Option<&[u8]>, extra: &[(&str, &str)]) -> Sent {
    let config = AnyUrl(AdapterConfig {
        extra_headers: extra
            .iter()
            .map(|&(name, value)| (name.to_owned(), value.to_owned()))
            .collect(),
        ..AdapterConfig::default()
    });
    let mut dial = Recorder::default();
    let call = Call {
        method,
        url,
        body,
        content_type: None,
        retry_safe: true,
        bearer_token: None,
    };
    let result = request(&config, &mut dial, &call, &mut |_| Ok(()))
        .map(|raw| String::from_utf8(raw.body).expect("an ASCII body"))
        .map_err(|error: Exc| {
            let text = error
                .text
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect();
            (error.class, text)
        });
    Sent {
        result,
        dials: dial.dials,
        wire: String::from_utf8(dial.sent.take()).expect("an ASCII request"),
    }
}

fn refused_before_connecting(sent: &Sent, case: &str) {
    assert_eq!(
        sent.result,
        Err((TRANSPORT, "no response after 2 attempt(s)".into())),
        "{case}"
    );
    assert!(sent.dials.is_empty(), "{case}");
    assert!(sent.wire.is_empty(), "{case}");
}

fn wire(request_line: &str, host: &str, extra: &str) -> String {
    format!(
        "{request_line}\r\nAccept-Encoding: identity\r\n{extra}Host: {host}\r\n\
         Accept: application/json\r\nUser-Agent: fixture-ua/1.0\r\nConnection: close\r\n\r\n"
    )
}

#[test]
fn inputs_http_client_refuses_fail_every_attempt_without_a_dial() {
    // urllib: no host given (no `//`), InvalidURL (non-numeric port, a
    // space in the unquoted host or the path), ValueError (a control
    // character in the method, an empty header name) and
    // UnicodeEncodeError (a non-ASCII request line), all before connecting.
    for (case, method, url, extra) in [
        (
            "no // after the scheme",
            "GET",
            "http:127.0.0.1/v1/rates",
            &[][..],
        ),
        (
            "non-numeric port",
            "GET",
            "http://127.0.0.1:abc/v1/rates",
            &[],
        ),
        ("encoded space in host", "GET", "http://a%20b/v1/rates", &[]),
        (
            "control character in method",
            "G\x01T",
            "http://127.0.0.1/v1/rates",
            &[],
        ),
        ("space in path", "GET", "http://127.0.0.1/v1/ra tes", &[]),
        ("non-ASCII path", "GET", "http://127.0.0.1/v1/r\u{e9}", &[]),
        (
            "empty header name",
            "GET",
            "http://127.0.0.1/v1/rates",
            &[("", "1")],
        ),
    ] {
        refused_before_connecting(&send(method, url, None, extra), case);
    }
}

#[test]
fn a_query_straight_after_the_host_gets_a_leading_slash() {
    let sent = send("GET", "http://127.0.0.1?q=1", None, &[]);
    assert_eq!(sent.result, Ok("ok".into()));
    assert_eq!(sent.dials, [("127.0.0.1".to_owned(), 80)]);
    assert_eq!(sent.wire, wire("GET /?q=1 HTTP/1.1", "127.0.0.1", ""));
}

#[test]
fn an_empty_port_dials_the_default_and_keeps_the_host_header() {
    let sent = send("GET", "http://127.0.0.1:/v1/rates", None, &[]);
    assert_eq!(sent.result, Ok("ok".into()));
    assert_eq!(sent.dials, [("127.0.0.1".to_owned(), 80)]);
    assert_eq!(sent.wire, wire("GET /v1/rates HTTP/1.1", "127.0.0.1:", ""));
}

#[test]
fn a_bracketed_host_dials_the_address_inside_the_brackets() {
    let sent = send("GET", "http://[::1]/v1/rates", None, &[]);
    assert_eq!(sent.result, Ok("ok".into()));
    assert_eq!(sent.dials, [("::1".to_owned(), 80)]);
    assert_eq!(sent.wire, wire("GET /v1/rates HTTP/1.1", "[::1]", ""));
}

#[test]
fn a_transfer_encoding_header_sends_the_body_chunked() {
    let form = "Content-Type: application/x-www-form-urlencoded\r\n";
    let head = |tail: &str| {
        wire("POST /v1/rates HTTP/1.1", "127.0.0.1", form).replace(
            "Connection: close\r\n\r\n",
            &format!("Transfer-Encoding: chunked\r\nConnection: close\r\n\r\n{tail}"),
        )
    };
    let chunked = [("Transfer-Encoding", "chunked")];
    for (case, body, tail) in [
        ("3 bytes", &b"abc"[..], "3\r\nabc\r\n0\r\n\r\n"),
        ("empty: the zero-length chunk is skipped", b"", "0\r\n\r\n"),
    ] {
        let sent = send("POST", "http://127.0.0.1/v1/rates", Some(body), &chunked);
        assert_eq!(sent.result, Ok("ok".into()), "{case}");
        assert_eq!(sent.wire, head(tail), "{case}");
    }
}

#[test]
fn a_bodiless_post_sends_a_zero_content_length() {
    let sent = send("POST", "http://127.0.0.1/v1/rates", None, &[]);
    assert_eq!(sent.result, Ok("ok".into()));
    assert_eq!(
        sent.wire,
        wire(
            "POST /v1/rates HTTP/1.1",
            "127.0.0.1",
            "Content-Length: 0\r\n"
        )
    );
}
