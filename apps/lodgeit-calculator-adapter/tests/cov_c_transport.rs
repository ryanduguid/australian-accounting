//! `transport::request`: the headers urllib and `http.client` put on the
//! wire, the requests they refuse before connecting, and a chunked body cut
//! short. Expected bytes and refusals are CPython 3.15.0's
//! (`urllib.request.urlopen` against a loopback listener, and
//! `http.client.HTTPConnection.putheader`).
use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::rc::Rc;

use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::errors::TRANSPORT;
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::transport::{Call, Connection, Dial, RawResponse, request};

const OK: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";

/// Replies with fixed bytes and records what it is sent.
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

/// Every dial gets the same reply; the count shows whether one happened.
struct Same {
    reply: &'static [u8],
    dials: usize,
    sent: Rc<RefCell<Vec<u8>>>,
}

impl Dial for Same {
    fn dial(&mut self, _: &str, _: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
        self.dials += 1;
        Ok(Box::new(Canned {
            reply: io::Cursor::new(self.reply.to_vec()),
            sent: Rc::clone(&self.sent),
        }))
    }
}

struct Sent {
    result: Result<RawResponse, Exc>,
    dials: usize,
    wire: String,
}

fn send(url: &str, extra_headers: &[(&str, &str)], reply: &'static [u8]) -> Sent {
    let config = AdapterConfig {
        enabled: true,
        base_url: Some("http://127.0.0.1".into()),
        allow_loopback: true,
        user_agent: "fixture-ua/1.0".into(),
        extra_headers: extra_headers
            .iter()
            .map(|&(name, value)| (name.to_owned(), value.to_owned()))
            .collect(),
        ..AdapterConfig::default()
    };
    let sent = Rc::default();
    let mut dial = Same {
        reply,
        dials: 0,
        sent: Rc::clone(&sent),
    };
    let call = Call {
        method: "GET",
        url,
        body: None,
        content_type: None,
        retry_safe: true,
        bearer_token: None,
    };
    let result = request(&config, &mut dial, &call, &mut |_| Ok(()));
    Sent {
        result,
        dials: dial.dials,
        wire: String::from_utf8(sent.take()).expect("an ASCII request"),
    }
}

fn wire_with(headers: &str) -> String {
    format!(
        "GET /v1/rates HTTP/1.1\r\nAccept-Encoding: identity\r\nHost: 127.0.0.1\r\n\
         Accept: application/json\r\n{headers}Connection: close\r\n\r\n"
    )
}

fn refusal(error: &Exc) -> (&'static str, String) {
    let text = error
        .text
        .iter()
        .filter_map(|&c| char::from_u32(c))
        .collect();
    (error.class, text)
}

const URL: &str = "http://127.0.0.1/v1/rates";

#[test]
fn the_configured_user_agent_is_sent() {
    let sent = send(URL, &[], OK);
    assert_eq!(sent.result.expect("a 200").body, b"ok");
    assert_eq!(sent.wire, wire_with("User-Agent: fixture-ua/1.0\r\n"));
}

#[test]
fn an_extra_user_agent_header_in_any_case_replaces_the_configured_one() {
    // urllib capitalizes both keys to `User-agent`, so the later one wins.
    let sent = send(URL, &[("user-agent", "fixture-override/2")], OK);
    assert_eq!(sent.result.expect("a 200").status, 200);
    assert_eq!(sent.wire, wire_with("User-Agent: fixture-override/2\r\n"));
}

#[test]
fn a_header_name_is_titled_after_an_uncased_underscore() {
    let sent = send(URL, &[("x_fixture", "1")], OK);
    assert_eq!(sent.result.expect("a 200").status, 200);
    assert_eq!(
        sent.wire,
        wire_with("User-Agent: fixture-ua/1.0\r\nX_Fixture: 1\r\n")
    );
}

#[test]
fn a_header_http_client_refuses_fails_every_attempt_before_connecting() {
    // ValueError for the name or the line break, UnicodeEncodeError for the
    // non-ASCII name or the value past Latin-1; each raised before connecting.
    for header in [
        ("Bad:Name", "1"),
        ("N\u{e4}me", "1"),
        ("X-Fixture", "a\nb"),
        ("X-Fixture", "\u{20ac}"),
    ] {
        let sent = send(URL, &[header], OK);
        let error = sent.result.expect_err("refused");
        assert_eq!(
            refusal(&error),
            (TRANSPORT, "no response after 2 attempt(s)".into()),
            "{header:?}"
        );
        assert_eq!(sent.dials, 0, "{header:?}");
    }
}

#[test]
fn a_tab_inside_the_scheme_passes_urlsplit_but_urllib_has_no_handler() {
    // urlsplit drops the tab (scheme 'http'); urllib's Request keeps it and
    // raises "unknown url type: ht\ttp" before connecting.
    let sent = send("ht\ttp://127.0.0.1/v1/rates", &[], OK);
    let error = sent.result.expect_err("refused");
    assert_eq!(
        refusal(&error),
        (TRANSPORT, "no response after 2 attempt(s)".into())
    );
    assert_eq!(sent.dials, 0);
}

#[test]
fn a_chunk_shorter_than_its_size_is_an_unreadable_body() {
    // http.client raises IncompleteRead; a 200 whose body failed is retried.
    let short: &[u8] = b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nab";
    let sent = send(URL, &[], short);
    let error = sent.result.expect_err("unreadable");
    assert_eq!(
        refusal(&error),
        (
            TRANSPORT,
            "HTTP 200 body could not be read after 2 attempt(s)".into()
        )
    );
    assert_eq!(sent.dials, 2);
}
