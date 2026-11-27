//! `transport::request` under a real `AdapterConfig` (enabled, loopback
//! allowed, two attempts): URLs that pass `check_url` but that urllib and
//! `http.client` split, connect to and send in their own way, or refuse
//! before connecting. Expected values are the retired Python adapter's
//! `transport.request` with the same configuration under CPython 3.15.0
//! (win32; no platform-specific code runs in these paths), with
//! `socket.create_connection` replaced by a socketpair.
use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::rc::Rc;

use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::errors::{DISALLOWED_TARGET, TRANSPORT};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::transport::{Call, Connection, Dial, request};

const OK: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";

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

/// One request: `hosts` is the allowlist, `extra` the configured headers.
fn send(
    method: &str,
    url: &str,
    body: Option<&[u8]>,
    extra: &[(&str, &str)],
    hosts: &[&str],
) -> Sent {
    let config = AdapterConfig {
        enabled: true,
        base_url: Some("http://127.0.0.1".into()),
        allowed_hosts: hosts.iter().map(|&host| host.to_owned()).collect(),
        allow_loopback: true,
        user_agent: "fixture-ua/1.0".into(),
        extra_headers: extra
            .iter()
            .map(|&(name, value)| (name.to_owned(), value.to_owned()))
            .collect(),
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a valid configuration");
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

fn get(url: &str) -> Sent {
    send("GET", url, None, &[], &[])
}

fn nothing_sent(sent: &Sent, case: &str) {
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
fn inputs_check_url_passes_but_http_client_refuses_fail_every_attempt_without_a_dial() {
    // urllib finds no host (urlsplit drops the tab, urllib does not),
    // InvalidURL (a non-numeric port or a space once the host is decoded,
    // a space in the path), ValueError (a control character in the method,
    // an empty header name) and UnicodeEncodeError (a non-ASCII request
    // line), all before connecting.
    for (case, method, url, extra, hosts) in [
        (
            "tab between the slashes",
            "GET",
            "http:/\t/127.0.0.1/v1/rates",
            &[][..],
            &[][..],
        ),
        (
            "encoded colon: non-numeric port",
            "GET",
            "https://h%3Aabc/v1/rates",
            &[],
            &["h%3aabc"],
        ),
        (
            "encoded space in host",
            "GET",
            "https://a%20b/v1/rates",
            &[],
            &["a%20b"],
        ),
        (
            "control character in method",
            "G\x01T",
            "http://127.0.0.1/v1/rates",
            &[],
            &[],
        ),
        (
            "space in path",
            "GET",
            "http://127.0.0.1/v1/rates/a b",
            &[],
            &[],
        ),
        (
            "non-ASCII path",
            "GET",
            "http://127.0.0.1/v1/rates/r\u{e9}",
            &[],
            &[],
        ),
        (
            "empty header name",
            "GET",
            "http://127.0.0.1/v1/rates",
            &[("", "1")],
            &[],
        ),
    ] {
        let sent = send(method, url, None, extra, hosts);
        assert_eq!(
            sent.result,
            Err((TRANSPORT, "no response after 2 attempt(s)".into())),
            "{case}"
        );
        nothing_sent(&sent, case);
    }
}

#[test]
fn check_url_refuses_every_url_whose_path_does_not_follow_the_host_with_a_slash() {
    // Why urllib's `_splithost` never needs to add a leading `/`: a query,
    // a fragment or an empty path after the host is refused first.
    let swallow = "a query string or fragment on a base URL would swallow the route built from it";
    let no_route = "path / is not one of the routes this adapter calls. A URL found in a response body is \
         data, not a destination.";
    for (url, reason) in [
        ("http://127.0.0.1?q=1", swallow),
        ("http://127.0.0.1?/v1/rates", swallow),
        ("http://127.0.0.1#/v1/rates", swallow),
        ("http://127.0.0.1#a#/v1/rates", swallow),
        ("http://127.0.0.1?", no_route),
        ("http://127.0.0.1", no_route),
    ] {
        let sent = get(url);
        assert_eq!(
            sent.result,
            Err((DISALLOWED_TARGET, format!("{url}: {reason}"))),
            "{url}"
        );
        nothing_sent(&sent, url);
    }
}

#[test]
fn an_empty_port_dials_the_default_and_keeps_the_host_header() {
    let sent = get("http://127.0.0.1:/v1/rates");
    assert_eq!(sent.result, Ok("ok".into()));
    assert_eq!(sent.dials, [("127.0.0.1".to_owned(), 80)]);
    assert_eq!(sent.wire, wire("GET /v1/rates HTTP/1.1", "127.0.0.1:", ""));
}

#[test]
fn a_bracketed_host_dials_the_address_inside_the_brackets() {
    let sent = get("http://[::1]/v1/rates");
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
        let sent = send(
            "POST",
            "http://127.0.0.1/v1/rates",
            Some(body),
            &chunked,
            &[],
        );
        assert_eq!(sent.result, Ok("ok".into()), "{case}");
        assert_eq!(sent.wire, head(tail), "{case}");
    }
}

#[test]
fn a_bodiless_post_sends_a_zero_content_length() {
    let sent = send("POST", "http://127.0.0.1/v1/rates", None, &[], &[]);
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
