//! `tests/test_core_clients.py`: fabricated Core catalogue and transport
//! cases; no tenant or secret is accessed.
//!
//! Python replaces `transport._opener` with a fake urllib opener. Here the
//! seam is a scripted `transport::Dial` given to `CoreClient::list_clients_via`:
//! each dial records where it went and what it was sent, and replies with
//! fabricated HTTP bytes, so the real transport parses every response.

use std::cell::RefCell;
use std::io::{self, Read, Write};
use std::rc::Rc;

use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::core::{
    CORE_URL, CoreCategory, CoreClient, CoreClientPage, CoreConfig, CoreContinuation, CoreError,
};
use lodgeit_calculator_adapter::dial;
use lodgeit_calculator_adapter::errors::{CONFIGURATION, DISALLOWED_TARGET};
use lodgeit_calculator_adapter::pyjson::{self, Json, Layout};
use lodgeit_calculator_adapter::transport::{self, Call, Connection, Dial, RequestConfig};

const ID: &str = "00000000-0000-0000-0000-000000000001";
const TOKEN: &str = "fabricated-token-for-offline-test";
const MIB: i64 = 1024 * 1024;
const REQUEST_LINE: &str = "POST /core/clients/v1-preview/list HTTP/1.1\r\n";

/// `item(**changes)`: the fabricated client, as `json.dumps` writes it, with
/// each change replacing a key in place or appended after the others. Values
/// are JSON text.
fn item(changes: &[(&str, &str)]) -> String {
    let id = format!("\"{ID}\"");
    let mut pairs: Vec<(&str, String)> = vec![
        ("id", id),
        ("code", "\"DEMO-090\"".into()),
        ("archived", "false".into()),
        ("private", "true".into()),
        ("profile", "null".into()),
    ];
    for (key, value) in changes {
        match pairs.iter_mut().find(|(k, _)| k == key) {
            Some(slot) => slot.1 = (*value).into(),
            None => pairs.push((key, (*value).into())),
        }
    }
    let members: Vec<String> = pairs.iter().map(|(k, v)| format!("\"{k}\": {v}")).collect();
    format!("{{{}}}", members.join(", "))
}

fn items(items: &[String]) -> Vec<u8> {
    format!("{{\"items\": [{}]}}", items.join(", ")).into_bytes()
}

/// An HTTP/1.1 reply with a Content-Length.
fn response(status: &str, headers: &str, body: &[u8]) -> Vec<u8> {
    let mut out = format!(
        "HTTP/1.1 {status}\r\n{headers}Content-Length: {}\r\n\r\n",
        body.len()
    )
    .into_bytes();
    out.extend_from_slice(body);
    out
}

/// The fake opener's 200, with `Content-Type: application/json`.
fn ok(body: &[u8]) -> Reply {
    Reply::Bytes(response(
        "200 OK",
        "Content-Type: application/json\r\n",
        body,
    ))
}

/// What every dial of a [`Script`] does.
#[derive(Clone)]
enum Reply {
    Bytes(Vec<u8>),
    /// These bytes, then an I/O error on the next read.
    BytesThenError(Vec<u8>),
    /// The connection is refused.
    Refused,
}

/// A connection that replies with fixed bytes and records what it is sent.
struct Scripted {
    input: io::Cursor<Vec<u8>>,
    fail_at_end: bool,
    sent: Rc<RefCell<Vec<u8>>>,
}

impl Read for Scripted {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let n = self.input.read(buf)?;
        if n == 0 && self.fail_at_end && !buf.is_empty() {
            return Err(io::Error::other("private-close-canary"));
        }
        Ok(n)
    }
}

impl Write for Scripted {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sent.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// The fake opener: every dial gets the same reply, and is recorded.
struct Script {
    reply: Reply,
    dials: Vec<(String, u16, bool)>,
    sent: Vec<Rc<RefCell<Vec<u8>>>>,
}

impl Script {
    fn new(reply: Reply) -> Script {
        Script {
            reply,
            dials: Vec::new(),
            sent: Vec::new(),
        }
    }

    /// What request `index` put on the wire: its head and its body.
    fn request(&self, index: usize) -> (String, String) {
        let sent = String::from_utf8(self.sent[index].borrow().clone()).unwrap();
        let (head, body) = sent.split_once("\r\n\r\n").unwrap();
        (head.to_owned(), body.to_owned())
    }

    /// `request.full_url == CORE_URL and request.get_method() == "POST"`.
    fn assert_post_to_core(&self, index: usize) {
        assert_eq!(self.dials[index], ("api.lodgeit.com".to_owned(), 443, true));
        assert!(self.request(index).0.starts_with(REQUEST_LINE));
    }
}

impl Dial for Script {
    fn dial(
        &mut self,
        host: &str,
        port: u16,
        tls: bool,
        _: f64,
    ) -> io::Result<Box<dyn Connection>> {
        self.dials.push((host.into(), port, tls));
        let sent = Rc::default();
        self.sent.push(Rc::clone(&sent));
        let (bytes, fail_at_end) = match &self.reply {
            Reply::Bytes(bytes) => (bytes.clone(), false),
            Reply::BytesThenError(bytes) => (bytes.clone(), true),
            Reply::Refused => return Err(io::ErrorKind::ConnectionRefused.into()),
        };
        Ok(Box::new(Scripted {
            input: io::Cursor::new(bytes),
            fail_at_end,
            sent,
        }))
    }
}

fn enabled() -> CoreClient {
    CoreClient {
        config: CoreConfig::new(true, 20.0, MIB).unwrap(),
    }
}

fn list(
    script: &mut Script,
    bearer_token: &str,
    page_size: i64,
    continuation: Option<&CoreContinuation>,
) -> Result<CoreClientPage, CoreError> {
    enabled().list_clients_via(script, bearer_token, page_size, continuation)
}

/// Everything a caller can see of an error: `str()` and `repr()`.
fn shown(error: &CoreError) -> String {
    format!("{error} {error:?}")
}

/// `error.__cause__ is error.__context__ is None`: a `CoreError` has no source.
fn detached(error: &CoreError) -> bool {
    std::error::Error::source(error).is_none()
}

fn continuation(value: &str) -> CoreContinuation {
    CoreContinuation {
        value: value.into(),
    }
}

#[test]
fn default_and_environment_cannot_enable_core() {
    // Rust's Core client reads no environment at all; the default config is
    // disabled whatever `LODGEIT_ADAPTER_*` holds.
    let mut script = Script::new(ok(b"{\"items\": []}"));
    let client = CoreClient {
        config: CoreConfig::default(),
    };
    let error = client
        .list_clients_via(&mut script, TOKEN, 100, None)
        .unwrap_err();
    assert_eq!(error.category, CoreCategory::Disabled);
    assert!(script.dials.is_empty());
}

#[test]
fn exact_request_projection_and_sensitive_representations() {
    let token = "https://evil.example/../../admin";
    let body = format!(
        "{{\"items\": [{}], \"nextPageToken\": \"{token}\"}}",
        item(&[])
    );
    let mut script = Script::new(ok(body.as_bytes()));
    let client = CoreClient {
        config: CoreConfig::new(true, 20.0, MIB).unwrap(),
    };
    let page = client
        .list_clients_via(&mut script, TOKEN, 5, None)
        .unwrap();
    assert!(page.items[0].id == ID && page.items[0].code.as_deref() == Some("DEMO-090"));
    assert!(page.items[0].private);
    assert_eq!(page.continuation.as_ref().unwrap().value, token);
    assert_eq!(script.dials.len(), 1);
    script.assert_post_to_core(0);
    let (head, payload) = script.request(0);
    assert!(head.contains(&format!("\r\nAuthorization: Bearer {TOKEN}\r\n")));
    assert_eq!(
        payload,
        r#"{"archived": false, "expansion": null, "paging": {"size": 5, "token": null}}"#
    );
    // `CoreClient` has no `Debug`; its config and the page are what can show.
    let representations = format!("{:?}{page:?}", client.config);
    assert!(!representations.contains(TOKEN));
    let next = page.continuation.as_ref().unwrap();
    assert!(!format!("{:?}", page.items[0]).contains(ID) && !format!("{next:?}").contains(token));
    client
        .list_clients_via(&mut script, "another-fabricated-token", 100, Some(next))
        .unwrap();
    script.assert_post_to_core(1);
    let (head1, payload1) = script.request(1);
    let sent = pyjson::loads(&payload1).unwrap();
    assert_eq!(
        sent.get("paging").and_then(|p| p.get("token")),
        Some(&Json::str(token))
    );
    assert!(
        script
            .request(0)
            .0
            .contains(&format!("\r\nAuthorization: Bearer {TOKEN}\r\n"))
    );
    assert!(head1.contains("\r\nAuthorization: Bearer another-fabricated-token\r\n"));
}

#[test]
fn only_the_exact_read_route_is_admitted() {
    let config = CoreConfig::new(true, 20.0, MIB).unwrap();
    let local = CORE_URL.replace("https", "http");
    let port = CORE_URL.replace("api.lodgeit.com", "api.lodgeit.com:443");
    let route = CORE_URL.replace("list", "modify-profile");
    for url in [
        &format!("{CORE_URL}/"),
        &format!("{CORE_URL}?x=1"),
        &format!("{CORE_URL}#x"),
        &local,
        &port,
        &route,
        "http://127.0.0.1/core/clients/v1-preview/list",
    ] {
        let error = RequestConfig::check_url(&config, url).unwrap_err();
        assert_eq!(error.class, DISALLOWED_TARGET, "{url}");
        let error = AdapterConfig::default()
            .check_url(CORE_URL, true)
            .unwrap_err();
        assert_eq!(error.class, DISALLOWED_TARGET);
    }
}

#[test]
fn invalid_tokens_never_open_transport() {
    // `None` and `1` are not strings; Rust's `&str` parameter refuses them
    // at compile time.
    let long = "a".repeat(8193);
    for token in [
        "",
        "Bearer token",
        "a\nb",
        "a\rb",
        " a",
        "a b",
        "é",
        long.as_str(),
    ] {
        let mut script = Script::new(ok(b"{\"items\": []}"));
        let error = list(&mut script, token, 100, None).unwrap_err();
        assert_eq!(error.category, CoreCategory::InvalidInput, "{token:?}");
        assert!(detached(&error));
        assert!(script.dials.is_empty());
    }
}

#[test]
fn invalid_page_size_never_sends() {
    // `True`, `"100"` and `1.5` are not integers; Rust's `i64` parameter
    // refuses them at compile time.
    for size in [0, 501] {
        let mut script = Script::new(ok(b"{\"items\": []}"));
        assert!(list(&mut script, TOKEN, size, None).is_err(), "{size}");
        assert!(script.dials.is_empty());
    }
}

#[test]
fn invalid_continuation_never_sends() {
    // A raw string is not a `CoreContinuation`, and a Rust `String` cannot
    // hold the lone surrogate `"\ud800"`; both are refused by the types.
    for value in [String::new(), "a".repeat(8193)] {
        let mut script = Script::new(ok(b"{\"items\": []}"));
        let error = list(&mut script, TOKEN, 100, Some(&continuation(&value))).unwrap_err();
        assert_eq!(
            error.category,
            CoreCategory::InvalidInput,
            "{}",
            value.len()
        );
        assert!(script.dials.is_empty());
    }
}

#[test]
fn captured_contract_pins_the_read_request_and_required_projection() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("contracts")
        .join("lodgeit-core-clients.json");
    let snapshot = pyjson::loads(&std::fs::read_to_string(path).unwrap()).unwrap();
    let route = snapshot.at("paths").at("/core/clients/v1-preview/list");
    assert!(route.has_exactly(&["post"]));
    let security = pyjson::dumps(route.at("post").at("security"), Layout::Canonical);
    assert!(security.contains("read:core.clients"));
    assert_eq!(
        snapshot.at("schemas").at("ClientV1").at("required"),
        &Json::Array(
            ["id", "archived", "private", "profile"]
                .map(Json::str)
                .to_vec()
        )
    );
    assert_eq!(
        snapshot
            .at("schemas")
            .at("ClientPagingArgsV1")
            .at("properties")
            .at("size")
            .at("maximum"),
        &Json::Int(500.into())
    );
    assert_eq!(
        snapshot.at("source_sha256").as_str().map(str::len),
        Some(64)
    );
}

#[test]
fn bad_success_is_transactionally_rejected_without_context() {
    let payloads: Vec<Vec<u8>> = vec![
        items(&[item(&[("profile", r#"{"sensitive": "canary"}"#)])]),
        items(&[item(&[("groups", "[]")])]),
        items(&[item(&[("assignments", "{}")])]),
        items(&[item(&[("relations", "[]")])]),
        items(&[item(&[("unexpected", "\"canary\"")])]),
        items(&[item(&[("archived", "true")])]),
        items(&[item(&[("private", "1")])]),
        items(&[item(&[("id", "\"not-a-uuid\"")])]),
        items(&[item(&[("code", "1.2")])]),
        items(&[item(&[]), item(&[])]),
        b"{\"items\": [], \"nextPageToken\": \"\"}".to_vec(),
        format!(
            "{{\"items\": [], \"nextPageToken\": \"{}\"}}",
            "a".repeat(8193)
        )
        .into_bytes(),
        b"{\"items\":[],\"items\":[]}".to_vec(),
        b"{\"items\":NaN}".to_vec(),
        b"\xff".to_vec(),
        b"{\"items\":canary}".to_vec(),
    ];
    for payload in payloads {
        let mut script = Script::new(ok(&payload));
        let error = list(&mut script, TOKEN, 100, None).unwrap_err();
        let label = String::from_utf8_lossy(&payload);
        assert_eq!(error.category, CoreCategory::Protocol, "{label}");
        assert!(detached(&error));
        assert!(!shown(&error).contains("canary"), "{label}");
    }
}

#[test]
fn http_error_bodies_are_discarded_and_never_retried() {
    for (status, category) in [
        (401, CoreCategory::Authentication),
        (403, CoreCategory::Authorisation),
        (400, CoreCategory::RequestRejected),
        (422, CoreCategory::RequestRejected),
        (429, CoreCategory::Throttled),
        (503, CoreCategory::Upstream),
        (302, CoreCategory::Protocol),
    ] {
        // Python's fake opener raises `HTTPError(302)` past the redirect
        // handler. Through the real transport a 302 reaches that step only
        // when its Location is one urllib would not follow.
        let headers = if status == 302 {
            "Location: file:///private-redirect-canary\r\n"
        } else {
            ""
        };
        let wire = response(&format!("{status} X"), headers, b"private-provider-canary");
        let mut script = Script::new(Reply::Bytes(wire));
        let error = list(&mut script, TOKEN, 100, None).unwrap_err();
        assert_eq!(error.category, category, "{status}");
        assert_eq!(error.http_status, Some(status));
        assert!(detached(&error));
        assert!(!shown(&error).contains("canary"));
        assert_eq!(script.dials.len(), 1);
    }
}

#[test]
fn http_parser_failures_have_no_payload_or_bearer_context() {
    // Python also checks `list_clients` cleared `bearer_token` from its frame
    // locals; a Rust error carries no frames, so the contract is that the
    // error shows neither the payload nor the bearer.
    for wire in [
        &b"private-upstream-canary\r\n"[..],
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\
          Transfer-Encoding: chunked\r\n\r\n5\r\ncanary\r\ninvalid-size\r\n",
    ] {
        let mut script = Script::new(Reply::Bytes(wire.to_vec()));
        let error = list(&mut script, TOKEN, 100, None).unwrap_err();
        assert_eq!(error.category, CoreCategory::Transport);
        assert!(detached(&error));
        assert!(!shown(&error).contains("canary") && !shown(&error).contains(TOKEN));
    }
}

#[test]
fn http_error_close_failure_is_sanitised() {
    // Python's 503 body reads whole and then `close()` raises. Closing a Rust
    // connection cannot fail, so the step below it fails instead: the 503's
    // body read raises part way, which Python's transport reduces to the same
    // TransportError ("HTTP 503 body could not be read").
    let mut wire = b"HTTP/1.1 503 fabricated\r\nContent-Type: application/problem+json\r\n\
X-Private: private-header-canary\r\nContent-Length: 64\r\n\r\n"
        .to_vec();
    wire.extend_from_slice(b"{\"private\":\"response-canary\"}");
    let mut script = Script::new(Reply::BytesThenError(wire));
    let error = list(
        &mut script,
        "private-bearer-canary",
        100,
        Some(&continuation("private-continuation-canary")),
    )
    .unwrap_err();
    assert_eq!(script.dials.len(), 1);
    script.assert_post_to_core(0);
    assert_eq!(error.category, CoreCategory::Transport);
    assert_eq!(error.http_status, None);
    assert!(detached(&error));
    assert!(!shown(&error).contains("canary"));
}

#[test]
fn every_core_failure_clears_sensitive_frame_locals() {
    // Python walks the traceback's frame locals for the secrets. A Rust
    // `CoreError` holds only its category and status, so the contract is
    // that no error at any stage carries a source or shows either secret.
    let (token, secret) = ("fabricated-bearer-canary", "fabricated-continuation-canary");
    for stage in [
        "disabled",
        "size",
        "continuation",
        "bearer",
        "transport",
        "status",
        "malformed",
    ] {
        let mut bearer = token.to_owned();
        let mut page_size = 100;
        // Python passes the raw string, which the type check refuses; the
        // nearest refusal Rust can reach is an over-long continuation.
        let mut next = continuation(secret);
        match stage {
            "size" => page_size = 0,
            "continuation" => next = continuation(&secret.repeat(300)),
            "bearer" => bearer.push('\n'),
            _ => {}
        }
        let reply = match stage {
            "transport" => Reply::Refused,
            "status" => Reply::Bytes(response("503 X", "", b"{\"items\": []}")),
            "malformed" => ok(b"malformed"),
            _ => ok(b"{\"items\": []}"),
        };
        let client = CoreClient {
            config: CoreConfig::new(stage != "disabled", 20.0, MIB).unwrap(),
        };
        let error = client
            .list_clients_via(&mut Script::new(reply), &bearer, page_size, Some(&next))
            .unwrap_err();
        assert!(detached(&error), "{stage}");
        let shown = shown(&error);
        assert!(!shown.contains(token) && !shown.contains(secret), "{stage}");
    }
}

#[test]
fn actual_redirect_handler_refuses_without_following_or_leaking() {
    for status in [301, 302, 303, 307, 308] {
        for (location, category) in [
            (
                "https://outside.invalid/private-redirect-canary",
                CoreCategory::Transport,
            ),
            ("https://[", CoreCategory::Transport),
            ("https://[not-an-ip]/", CoreCategory::Transport),
            ("file:///private-redirect-canary", CoreCategory::Protocol),
        ] {
            let wire = response(
                &format!("{status} redirect"),
                &format!("Location: {location}\r\n"),
                b"private-response-canary",
            );
            let mut script = Script::new(Reply::Bytes(wire));
            let error = list(
                &mut script,
                "private-bearer-canary",
                100,
                Some(&continuation("private-continuation-canary")),
            )
            .unwrap_err();
            assert_eq!(script.dials.len(), 1);
            script.assert_post_to_core(0);
            assert_eq!(error.category, category, "{status} {location}");
            let expected = (category == CoreCategory::Protocol).then_some(status);
            assert_eq!(error.http_status, expected);
            assert!(detached(&error));
            let shown = shown(&error);
            assert!(!shown.contains("canary") && !shown.contains(location));
        }
    }
}

#[test]
fn huge_timeouts_are_configuration_errors() {
    // `10**400` has no `f64`; as a float it is infinite, and the largest
    // finite values are refused as well.
    for timeout in [f64::INFINITY, f64::NEG_INFINITY, f64::MAX, -f64::MAX] {
        let error = CoreConfig::new(false, timeout, MIB).unwrap_err();
        assert_eq!(error.class, CONFIGURATION, "{timeout}");
    }
}

#[test]
fn response_ceiling_and_media_type_are_enforced() {
    let body = b"{\"items\": []}";
    for (reply, config) in [
        (
            Reply::Bytes(response("200 OK", "Content-Type: text/html\r\n", body)),
            CoreConfig::new(true, 20.0, MIB).unwrap(),
        ),
        (ok(body), CoreConfig::new(true, 20.0, 1).unwrap()),
    ] {
        let client = CoreClient { config };
        assert!(
            client
                .list_clients_via(&mut Script::new(reply), TOKEN, 100, None)
                .is_err()
        );
    }
}

#[test]
fn synthetic_maximum_page_fits_and_contract_projection_is_serialisable() {
    let code = format!("\"{}\"", "x".repeat(1024));
    let clients: Vec<String> = (1..=500)
        .map(|i| {
            // `str(UUID(int=i))`.
            let id = format!("\"00000000-0000-0000-0000-{i:012x}\"");
            item(&[("id", &id), ("code", &code)])
        })
        .collect();
    let mut script = Script::new(ok(&items(&clients)));
    let page = list(&mut script, TOKEN, 500, None).unwrap();
    assert_eq!(page.items.len(), 500);
    // `asdict(page.items[0])["code"]`: the projection's fields are public.
    assert_eq!(
        page.items[0].code.as_deref(),
        Some("x".repeat(1024).as_str())
    );
}

#[test]
fn config_safety_limits() {
    // `enabled=1` and `max_response_bytes=True` are refused by Rust's
    // `bool` and `i64` parameters at compile time.
    for (timeout, limit) in [(f64::NAN, MIB), (61.0, MIB), (20.0, 1_048_577)] {
        let error = CoreConfig::new(false, timeout, limit).unwrap_err();
        assert_eq!(error.class, CONFIGURATION, "{timeout} {limit}");
    }
}

#[test]
fn conflicting_authorisation_is_refused_by_shared_transport() {
    let call = Call {
        method: "POST",
        url: "http://127.0.0.1/v1/calculators",
        body: None,
        content_type: None,
        retry_safe: false,
        bearer_token: Some(TOKEN),
    };
    // Python's config names no base URL, so `require_enabled` raises the
    // ConfigurationError before the headers are compared; the second config
    // has one, so the conflicting header is what refuses it.
    for base_url in [None, Some("http://127.0.0.1".to_owned())] {
        let config = AdapterConfig {
            enabled: true,
            base_url: base_url.clone(),
            allow_loopback: true,
            extra_headers: vec![("authorization".into(), "fabricated".into())],
            ..AdapterConfig::default()
        }
        .checked()
        .unwrap();
        let mut script = Script::new(ok(b"{\"items\": []}"));
        let error = transport::request(&config, &mut script, &call, &mut dial::sleep).unwrap_err();
        assert_eq!(error.class, CONFIGURATION, "{base_url:?}");
        if base_url.is_some() {
            let text: String = error
                .text
                .iter()
                .filter_map(|&c| char::from_u32(c))
                .collect();
            assert_eq!(text, "Invalid per-call bearer authentication.");
        }
        assert!(script.dials.is_empty());
    }
}
