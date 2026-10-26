//! Port of `tests/test_transport_failures.py`: fabricated regressions for
//! transport failures and status-driven retries.
//!
//! Python patches urllib's opener, `http.client`'s sockets and TLS objects.
//! Here every connection comes from a scripted [`Dial`]: each dial yields a
//! [`Reply`] whose bytes arrive one segment per read, then a clean end of
//! stream or an I/O error. A connection is "released" once the transport has
//! dropped it, which the backoff callback checks before every pause.
//!
//! Python-only failure kinds have no Rust counterpart and are left out of the
//! loops, as the mapping table records: a `close()` that raises (dropping a
//! Rust connection cannot fail) and a non-I/O exception (`KeyboardInterrupt`,
//! `SystemExit`, `TypeError`, `RuntimeError`) raised by a read or a close (a
//! Rust read can only return `io::Error`).

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::rc::Rc;

use lodgeit_calculator_adapter::client::{Client, Outcome, Status};
use lodgeit_calculator_adapter::config::AdapterConfig;
use lodgeit_calculator_adapter::contract;
use lodgeit_calculator_adapter::errors::{DISALLOWED_TARGET, TRANSPORT};
use lodgeit_calculator_adapter::pyjson::Json;
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::transport::{Call, Connection, Dial, Raised, RawResponse, request};

const URL: &str = "http://127.0.0.1:9/v1/calculators";
const CANARY: &str = "fabricated-private-canary";

// -- the scripted dialler ----------------------------------------------------

/// What one connection replies: segments, one per read, then the end.
#[derive(Clone)]
struct Reply {
    segments: Vec<Vec<u8>>,
    /// `None` is a clean end of stream; otherwise the read error after it.
    end: Option<io::ErrorKind>,
}

/// The whole reply in one segment, then a clean end of stream.
fn wire(bytes: &[u8]) -> Reply {
    Reply {
        segments: vec![bytes.to_vec()],
        end: None,
    }
}

/// The head and the body in separate segments, so a read of the body shows.
fn split(head: &[u8], body: &[u8]) -> Reply {
    Reply {
        segments: vec![head.to_vec(), body.to_vec()],
        end: None,
    }
}

/// What the dialler saw, shared with the connections it made.
#[derive(Default)]
struct Log {
    dials: RefCell<Vec<(String, u16, bool)>>,
    open: Cell<usize>,
    /// Reads attempted once the first segment was spent.
    later_reads: Cell<usize>,
    sent: RefCell<Vec<u8>>,
}

impl Log {
    fn dials(&self) -> usize {
        self.dials.borrow().len()
    }

    fn released(&self) -> bool {
        self.open.get() == 0
    }
}

struct Conn {
    segments: VecDeque<Vec<u8>>,
    end: Option<io::ErrorKind>,
    spent: usize,
    log: Rc<Log>,
}

impl Read for Conn {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.spent > 0 {
                self.log.later_reads.set(self.log.later_reads.get() + 1);
            }
            let Some(front) = self.segments.front_mut() else {
                return match self.end {
                    Some(kind) => Err(io::Error::new(kind, CANARY)),
                    None => Ok(0),
                };
            };
            if front.is_empty() {
                self.segments.pop_front();
                self.spent += 1;
                continue;
            }
            let n = buf.len().min(front.len());
            buf[..n].copy_from_slice(&front[..n]);
            front.drain(..n);
            return Ok(n);
        }
    }
}

impl Write for Conn {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.log.sent.borrow_mut().extend_from_slice(buf);
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Drop for Conn {
    fn drop(&mut self) {
        self.log.open.set(self.log.open.get() - 1);
    }
}

/// Each dial asks `next` with the dial's index for its reply or its error.
struct Scripted<F> {
    next: F,
    log: Rc<Log>,
}

impl<F: FnMut(usize) -> io::Result<Reply>> Dial for Scripted<F> {
    fn dial(
        &mut self,
        host: &str,
        port: u16,
        tls: bool,
        _timeout: f64,
    ) -> io::Result<Box<dyn Connection>> {
        let index = self.log.dials();
        self.log
            .dials
            .borrow_mut()
            .push((host.to_owned(), port, tls));
        let reply = (self.next)(index)?;
        self.log.open.set(self.log.open.get() + 1);
        Ok(Box::new(Conn {
            segments: reply.segments.into(),
            end: reply.end,
            spent: 0,
            log: Rc::clone(&self.log),
        }))
    }
}

/// A dial error `request` raises as it is, as Python's opener raising a
/// non-I/O exception.
fn raised(class: &'static str) -> io::Error {
    io::Error::other(Raised(exc(class)))
}

fn exc(class: &'static str) -> Exc {
    Exc {
        class,
        text: CANARY.chars().map(u32::from).collect(),
        notes: Vec::new(),
    }
}

// -- configuration and calls -------------------------------------------------

fn config() -> AdapterConfig {
    config_limit(2 * 1024 * 1024)
}

fn config_limit(max_response_bytes: usize) -> AdapterConfig {
    AdapterConfig {
        enabled: true,
        base_url: Some("http://127.0.0.1:9".into()),
        allow_loopback: true,
        max_attempts: 3.into(),
        retry_backoff_seconds: 0.01,
        max_response_bytes: max_response_bytes.into(),
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a valid configuration")
}

fn get(retry_safe: bool) -> Call<'static> {
    call("GET", URL, retry_safe)
}

fn call<'a>(method: &'a str, url: &'a str, retry_safe: bool) -> Call<'a> {
    Call {
        method,
        url,
        body: None,
        content_type: None,
        retry_safe,
        bearer_token: None,
    }
}

/// POST with a canary body and a canary bearer token.
fn post_canary(retry_safe: bool) -> Call<'static> {
    Call {
        body: Some(CANARY.as_bytes()),
        bearer_token: Some(CANARY),
        ..call("POST", URL, retry_safe)
    }
}

struct Run {
    result: Result<RawResponse, Exc>,
    log: Rc<Log>,
    pauses: Vec<f64>,
}

/// `transport.request` through the scripted dialler, with a backoff that
/// requires every connection released before it pauses.
fn run(config: &AdapterConfig, call: &Call, next: impl FnMut(usize) -> io::Result<Reply>) -> Run {
    let log = Rc::new(Log::default());
    let mut dial = Scripted {
        next,
        log: Rc::clone(&log),
    };
    let mut pauses = Vec::new();
    let watch = Rc::clone(&log);
    let result = request(config, &mut dial, call, &mut |seconds| {
        assert!(watch.released(), "release the connection before backoff");
        pauses.push(seconds);
        Ok(())
    });
    Run {
        result,
        log,
        pauses,
    }
}

/// The same reply for every dial.
fn run_wire(config: &AdapterConfig, call: &Call, reply: &Reply) -> Run {
    run(config, call, |_| Ok(reply.clone()))
}

/// `LodgeitClient(config, contract).discover()` through the scripted dialler.
fn discover(
    config: &AdapterConfig,
    next: impl FnMut(usize) -> io::Result<Reply>,
) -> (Outcome, Rc<Log>) {
    let contract = contract::load("lodgeit-calculators", None).expect("the snapshot loads");
    let log = Rc::new(Log::default());
    let mut dial = Scripted {
        next,
        log: Rc::clone(&log),
    };
    let watch = Rc::clone(&log);
    let mut sleep = |_: f64| {
        assert!(watch.released());
        Ok(())
    };
    let outcome = Client {
        config,
        contract: &contract,
        dial: &mut dial,
        sleep: &mut sleep,
    }
    .discover()
    .expect("discover returns an outcome");
    (outcome, log)
}

fn assert_unavailable(outcome: &Outcome) {
    assert_eq!(outcome.status, Status::UpstreamUnavailable);
    assert_eq!(outcome.result, Json::Null);
    assert!(outcome.values.is_empty());
}

fn pauses(attempts: usize) -> Vec<f64> {
    (1..attempts).map(|attempt| 0.01 * attempt as f64).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn error_of(run: Run) -> (Exc, Rc<Log>, Vec<f64>) {
    match run.result {
        Ok(raw) => panic!("expected an error, got HTTP {}", raw.status),
        Err(error) => (error, run.log, run.pauses),
    }
}

fn success(run: Run) -> (RawResponse, Rc<Log>, Vec<f64>) {
    match run.result {
        Ok(raw) => (raw, run.log, run.pauses),
        Err(error) => panic!("expected a response, got {}", error.traceback()),
    }
}

/// `assert_detached`: the error says nothing it was not built with. Python
/// also walks `__cause__`, `__context__`, `vars()` and the transport's frame
/// locals; a Rust `Exc` has only its class, text and notes.
fn assert_detached(error: &Exc) {
    assert!(!error.traceback().contains(CANARY), "{}", error.traceback());
    assert!(!format!("{error:?}").contains(CANARY));
    assert!(error.notes.iter().all(|note| !text(note).contains(CANARY)));
}

fn assert_class(error: &Exc, class: &str) {
    assert_eq!(error.class, class, "{}", error.traceback());
}

fn assert_message(error: &Exc, fragment: &str) {
    let message = text(&error.text);
    assert!(message.contains(fragment), "{message:?} lacks {fragment:?}");
}

/// The value of a response header, as `raw.headers[name]`.
fn header<'a>(raw: &'a RawResponse, name: &str) -> Option<&'a str> {
    raw.headers
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

// -- ErrorOpener: a status whose body fails ----------------------------------

/// `ErrorOpener(status, failure)`: the status with a canary reason, then a
/// body that fails as `ErrorBody(failure)` does. Python raises the HTTPError
/// itself, so a 3xx never meets the redirect handler; here a 3xx carries a
/// Location urllib would not follow, the wire path to the same HTTPError.
fn error_reply(status: u16, failure: &str) -> Reply {
    let canary = CANARY.as_bytes();
    let (length, body, end) = match failure {
        // Python's "close" body reads cleanly; only its close() fails.
        "none" | "close" => (canary.len(), canary.to_vec(), None),
        // IncompleteRead: fewer bytes than promised, then the end.
        "parser" => (canary.len() + 99, canary.to_vec(), None),
        "value" => (canary.len(), Vec::new(), Some(io::ErrorKind::InvalidData)),
        "timeout" => (canary.len(), Vec::new(), Some(io::ErrorKind::TimedOut)),
        "oversize" => (canary.len() * 2, canary.repeat(2), None),
        other => panic!("unknown failure {other}"),
    };
    let location = if (300..400).contains(&status) {
        format!("Location: mailto:{CANARY}\r\n")
    } else {
        String::new()
    };
    let head = format!("HTTP/1.1 {status} {CANARY}\r\n{location}Content-Length: {length}\r\n\r\n");
    Reply {
        segments: vec![head.into_bytes(), body],
        end,
    }
}

fn limit_for(failure: &str) -> usize {
    if failure == "oversize" { 4 } else { 1024 }
}

#[test]
fn failed_http_body_respects_status_and_retry_budget() {
    // "close" is left out: Python fails on the body's close(), which a Rust
    // connection cannot do, so its body would read cleanly.
    for status in [400, 422, 429, 503] {
        for failure in ["parser", "value", "timeout", "oversize"] {
            for retry_safe in [false, true] {
                let mut config = config_limit(limit_for(failure));
                config.extra_headers = vec![("X-Fabricated".into(), CANARY.into())];
                let (error, log, slept) = error_of(run_wire(
                    &config,
                    &post_canary(retry_safe),
                    &error_reply(status, failure),
                ));
                let attempts = if status == 503 && retry_safe { 3 } else { 1 };
                assert_eq!(log.dials(), attempts, "{status} {failure} {retry_safe}");
                assert_eq!(slept, pauses(attempts));
                assert!(log.released());
                assert_class(&error, TRANSPORT);
                assert_detached(&error);
            }
        }
    }
}

#[test]
fn calculator_client_returns_unavailable_for_failed_5xx() {
    // Python's "close" 503 reads cleanly and fails on close(); the Rust 503
    // reads cleanly, which the client reports the same way.
    for failure in ["parser", "value", "close", "oversize"] {
        let reply = error_reply(503, failure);
        let (outcome, log) = discover(&config_limit(limit_for(failure)), |_| Ok(reply.clone()));
        assert_unavailable(&outcome);
        assert_eq!(log.dials(), 3, "{failure}");
        assert!(outcome.findings.iter().all(|f| !text(f).contains(CANARY)));
    }
}

#[test]
fn response_failures_are_normalised_without_retaining_diagnostics() {
    // "close" (a 200 whose close() raises) has no Rust counterpart.
    for retry_safe in [false, true] {
        for failure in ["bad-status", "bad-chunk", "socket", "value"] {
            let (error, log, _) = error_of(run(&config(), &post_canary(retry_safe), |_| {
                match failure {
                    "bad-status" => Ok(wire(format!("{CANARY}\r\n").as_bytes())),
                    "bad-chunk" => Ok(wire(
                        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n\
                          5\r\nhello\r\ninvalid-size\r\n",
                    )),
                    // URLError and ValueError from the opener: a failed connect.
                    "socket" => Err(io::Error::other(CANARY)),
                    _ => Err(io::Error::new(io::ErrorKind::InvalidInput, CANARY)),
                }
            }));
            assert_eq!(log.dials(), if retry_safe { 3 } else { 1 }, "{failure}");
            assert_class(&error, TRANSPORT);
            assert_detached(&error);
        }
    }
}

#[test]
fn redirect_refusal_never_retries_or_retains_location() {
    // A close() failure on the refused body has no Rust counterpart.
    for status in [301, 302, 303, 307, 308] {
        for location in [
            "https://[".to_owned(),
            format!("https://outside.invalid/{CANARY}"),
        ] {
            let head = format!(
                "HTTP/1.1 {status} {CANARY}\r\nLocation: {location}\r\nContent-Length: {}\r\n\r\n",
                CANARY.len()
            );
            let reply = split(head.as_bytes(), CANARY.as_bytes());
            let (error, log, slept) =
                error_of(run_wire(&config(), &call("POST", URL, true), &reply));
            assert_eq!(log.dials(), 1);
            assert_eq!(
                log.later_reads.get(),
                0,
                "a refused redirect body must not be drained"
            );
            assert!(log.released() && slept.is_empty());
            assert_class(&error, DISALLOWED_TARGET);
            assert_detached(&error);
        }
    }
}

#[test]
fn successful_oversized_response_is_not_retried() {
    // Python runs this with and without a failing close(); Rust has only the
    // first.
    let reply = wire(format!("HTTP/1.1 200 OK\r\n\r\n{CANARY}{CANARY}").as_bytes());
    let (error, log, _) = error_of(run_wire(&config_limit(CANARY.len()), &get(true), &reply));
    assert_message(&error, "byte ceiling");
    assert_eq!(log.dials(), 1);
    assert_detached(&error);
}

#[test]
fn process_cancellation_is_not_converted_or_retried() {
    // The opener's KeyboardInterrupt or SystemExit is the dialler's `Raised`.
    for class in ["KeyboardInterrupt", "SystemExit"] {
        let (error, log, slept) = error_of(run(&config(), &get(true), |_| Err(raised(class))));
        assert_class(&error, class);
        assert_eq!(log.dials(), 1);
        assert!(slept.is_empty());
    }
}

#[test]
fn readable_error_response_preserves_provider_data() {
    for status in [302, 400, 422, 429, 503] {
        for retry_safe in [false, true] {
            let (raw, log, _) = success(run_wire(
                &config(),
                &get(retry_safe),
                &error_reply(status, "none"),
            ));
            assert_eq!(raw.status, status);
            assert_eq!(raw.body, CANARY.as_bytes());
            assert_eq!(raw.url, URL);
            assert_eq!(log.dials(), if status == 503 && retry_safe { 3 } else { 1 });
            assert!(log.released());
            // The control for the unread-body checks below: a read body shows.
            assert!(log.later_reads.get() > 0);
        }
    }
}

#[test]
fn failed_5xx_recovers_to_success_within_budget() {
    for failure in ["none", "parser", "value", "close", "oversize"] {
        let first = error_reply(503, failure);
        let ok = wire(format!("HTTP/1.1 200 OK\r\nX-Provider: {CANARY}\r\n\r\n{{}}").as_bytes());
        let config = config_limit(limit_for(failure));
        let (raw, log, slept) = success(run(&config, &post_canary(true), |index| {
            Ok(if index == 0 {
                first.clone()
            } else {
                ok.clone()
            })
        }));
        assert_eq!(raw.status, 200);
        assert_eq!(raw.body, b"{}");
        assert_eq!(
            raw.headers,
            vec![("x-provider".to_owned(), CANARY.to_owned())]
        );
        assert_eq!(log.dials(), 2, "{failure}");
        assert!(log.released());
        assert_eq!(slept, vec![0.01]);
    }
}

#[test]
fn known_redirect_with_failed_body_is_never_retried() {
    // "close" is left out: the Rust body reads cleanly and the 302 returns.
    for failure in ["parser", "value", "timeout", "oversize"] {
        let (error, log, slept) = error_of(run_wire(
            &config_limit(limit_for(failure)),
            &get(true),
            &error_reply(302, failure),
        ));
        assert_eq!(log.dials(), 1, "{failure}");
        assert!(log.released());
        assert!(slept.is_empty(), "redirect retry");
        assert_class(&error, TRANSPORT);
        assert_detached(&error);
    }
}

#[test]
fn invalid_utf8_error_does_not_retain_response() {
    let raw = RawResponse {
        status: 200,
        headers: vec![("x-provider".into(), CANARY.into())],
        body: [CANARY.as_bytes(), b"\xff"].concat(),
        url: URL.into(),
        elapsed_ms: 0,
    };
    let error = raw.text().unwrap_err();
    assert_class(&error, TRANSPORT);
    assert_message(&error, "not UTF-8");
    assert_detached(&error);
}

#[test]
fn normally_returned_server_error_uses_status_for_retry() {
    for status in [503, 600] {
        for retry_safe in [false, true] {
            let reply = wire(
                format!("HTTP/1.1 {status} X\r\nX-Provider: {CANARY}\r\n\r\n{CANARY}").as_bytes(),
            );
            let (raw, log, _) = success(run_wire(&config(), &get(retry_safe), &reply));
            assert_eq!(log.dials(), if retry_safe { 3 } else { 1 });
            assert!(log.released());
            assert_eq!(raw.status, status);
            assert_eq!(raw.body, CANARY.as_bytes());
            assert_eq!(
                raw.headers,
                vec![("x-provider".to_owned(), CANARY.to_owned())]
            );
        }
    }
}

#[test]
fn readable_5xx_then_open_failure_never_returns_stale_response() {
    // Python's HTTPError and returned-response variants are one wire reply
    // here. Its frame-locals check maps to the release check in the backoff.
    for _as_http_error in [false, true] {
        let first = error_reply(503, "none");
        let (error, log, _) = error_of(run(&config(), &get(true), |index| {
            if index == 0 {
                Ok(first.clone())
            } else {
                Err(io::Error::other(CANARY))
            }
        }));
        assert_message(&error, "no response after 3");
        assert_eq!(log.dials(), 3);
        assert!(log.released());
        assert_detached(&error);
    }
}

#[test]
fn final_error_detaches_ambient_context_and_opaque_callback() {
    // Python's ambient exception context has no Rust counterpart; the
    // nearest contract is that a canary held by the backoff callback, a
    // closure or an opaque object, never reaches the error.
    struct Callback {
        secret: String,
    }
    for target in ["request", "text"] {
        for closure in [false, true] {
            let hidden = CANARY.to_owned();
            let opaque = Callback {
                secret: CANARY.to_owned(),
            };
            let mut callback = |_: f64| -> Result<(), Exc> {
                let held = if closure { &hidden } else { &opaque.secret };
                assert_eq!(held, CANARY);
                Ok(())
            };
            let error = if target == "request" {
                let mut dial = Scripted {
                    next: |_| Err(io::Error::other(CANARY)),
                    log: Rc::default(),
                };
                request(&config(), &mut dial, &get(false), &mut callback).unwrap_err()
            } else {
                RawResponse {
                    status: 200,
                    headers: Vec::new(),
                    body: b"\xff".to_vec(),
                    url: URL.into(),
                    elapsed_ms: 0,
                }
                .text()
                .unwrap_err()
            };
            assert_class(&error, TRANSPORT);
            assert_detached(&error);
        }
    }
}

#[test]
fn backoff_exceptions_propagate_after_closure() {
    for class in [
        "KeyboardInterrupt",
        "SystemExit",
        "TypeError",
        "RuntimeError",
    ] {
        let log = Rc::new(Log::default());
        let reply = error_reply(503, "none");
        let mut dial = Scripted {
            next: |_| Ok(reply.clone()),
            log: Rc::clone(&log),
        };
        let watch = Rc::clone(&log);
        let error = request(&config(), &mut dial, &get(true), &mut |_| {
            assert!(watch.released());
            Err(exc(class))
        })
        .unwrap_err();
        assert_class(&error, class);
        assert_eq!(log.dials(), 1);
    }
}

#[test]
fn unrelated_open_errors_are_not_converted() {
    for class in ["TypeError", "RuntimeError"] {
        let (error, log, _) = error_of(run(&config(), &get(true), |_| Err(raised(class))));
        assert_class(&error, class);
        assert_eq!(log.dials(), 1);
    }
}

#[test]
fn close_failure_preserves_primary_body_failure() {
    // The failing close() itself has no Rust counterpart; the primary body
    // failure it must not replace is what remains.
    for (status, failure) in [(400, "parser"), (503, "parser"), (503, "oversize")] {
        let message = if failure == "oversize" {
            "byte ceiling".to_owned()
        } else {
            format!("HTTP {status} body could not be read")
        };
        let (error, log, _) = error_of(run_wire(
            &config_limit(limit_for(failure)),
            &get(true),
            &error_reply(status, failure),
        ));
        assert_message(&error, &message);
        assert_eq!(log.dials(), if status == 503 { 3 } else { 1 });
        assert!(log.released());
        assert_detached(&error);
    }
}

// -- wire_sockets: the real opener over scripted bytes -----------------------

fn canary_bytes() -> Vec<u8> {
    CANARY.as_bytes().to_vec()
}

fn many_headers(value: &str) -> Vec<u8> {
    format!("X-Fabricated: {value}\r\n")
        .repeat(101)
        .into_bytes()
}

#[test]
fn invalid_framing_uses_real_opener_and_status_budget() {
    for status in [200, 400, 503, 600] {
        for retry_safe in [false, true] {
            for framing in ["negative-chunk", "huge-negative-chunk", "truncated"] {
                let mut body = canary_bytes();
                let headers: Vec<u8> = if framing == "truncated" {
                    format!("Content-Length: {}\r\n", body.len() + 1).into_bytes()
                } else {
                    let size: &[u8] = if framing == "negative-chunk" {
                        b"-1"
                    } else {
                        b"-1000000000000000000000000000000"
                    };
                    body = [
                        format!("{:x}\r\n", body.len()).as_bytes(),
                        &body,
                        b"\r\n",
                        size,
                        b"\r\n",
                    ]
                    .concat();
                    b"Transfer-Encoding: chunked\r\n".to_vec()
                };
                let bytes = [
                    format!("HTTP/1.1 {status} Test\r\n").as_bytes(),
                    &headers,
                    b"\r\n",
                    &body,
                ]
                .concat();
                let (error, log, slept) =
                    error_of(run_wire(&config(), &post_canary(retry_safe), &wire(&bytes)));
                let attempts = if retry_safe && status != 400 { 3 } else { 1 };
                assert_eq!(log.dials(), attempts, "{status} {retry_safe} {framing}");
                assert_eq!(slept, pauses(attempts));
                assert!(log.released());
                assert_detached(&error);
            }
        }
    }
}

#[test]
fn status_parsed_before_bad_headers_controls_retry() {
    for status in [302, 400, 422, 429, 503, 600] {
        for retry_safe in [false, true] {
            let bytes = [
                format!("HTTP/1.1 {status} Test\r\n").into_bytes(),
                many_headers(CANARY),
                b"\r\n".to_vec(),
            ]
            .concat();
            let (error, log, slept) =
                error_of(run_wire(&config(), &get(retry_safe), &wire(&bytes)));
            let attempts = if retry_safe && status >= 500 { 3 } else { 1 };
            assert_eq!(log.dials(), attempts, "{status} {retry_safe}");
            assert_eq!(slept, pauses(attempts));
            assert!(log.released());
            assert_class(&error, TRANSPORT);
            assert_detached(&error);
        }
    }
}

type Framed = (
    u16,
    &'static str,
    &'static [u8],
    &'static [u8],
    &'static [u8],
);

fn assert_framed(cases: &[Framed], check: impl Fn(&[u8], &RawResponse)) {
    for &(status, method, headers, body, expected) in cases {
        let bytes = [
            format!("HTTP/1.1 {status} Test\r\n").as_bytes(),
            headers,
            b"\r\n",
            body,
        ]
        .concat();
        let (raw, log, _) = success(run_wire(
            &config_limit(2),
            &call(method, URL, false),
            &wire(&bytes),
        ));
        assert_eq!(raw.status, status);
        assert_eq!(raw.body, expected, "{status} {method} {headers:?}");
        assert_eq!(log.dials(), 1);
        assert!(log.released());
        check(headers, &raw);
    }
}

#[test]
fn effective_framing_preserves_valid_and_bodyless_responses() {
    assert_framed(
        &[
            (200, "GET", b"Content-Length: 2\r\n", b"[]", b"[]"),
            (200, "GET", b"", b"[]", b"[]"),
            (
                200,
                "GET",
                b"Transfer-Encoding: chunked\r\n",
                b"2\r\n[]\r\n0\r\n\r\n",
                b"[]",
            ),
            (200, "HEAD", b"Content-Length: 20\r\n", b"", b""),
            (204, "GET", b"Content-Length: 20\r\n", b"", b""),
            (304, "GET", b"Content-Length: 20\r\n", b"", b""),
        ],
        |_, _| {},
    );
}

#[test]
fn truncated_valid_json_discovery_cannot_be_computed() {
    let reply = wire(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\n[]");
    let (outcome, log) = discover(&config(), |_| Ok(reply.clone()));
    assert_unavailable(&outcome);
    assert_eq!(log.dials(), 3);
    assert!(log.released());
}

#[test]
fn real_opener_ceiling_stops_without_retry() {
    let cases: [(&[u8], &[u8]); 3] = [
        (b"Content-Length: 3\r\n", b"abc"),
        (b"", b"abc"),
        (b"Transfer-Encoding: chunked\r\n", b"3\r\nabc\r\n0\r\n\r\n"),
    ];
    for (headers, body) in cases {
        let bytes = [b"HTTP/1.1 200 OK\r\n", headers, b"\r\n", body].concat();
        let (error, log, _) = error_of(run_wire(&config_limit(2), &get(true), &wire(&bytes)));
        assert_message(&error, "byte ceiling");
        assert_eq!(log.dials(), 1);
        assert!(log.released());
    }
}

#[test]
fn real_opener_pre_status_failure_still_retries() {
    let reply = wire(format!("{CANARY}\r\n").as_bytes());
    let (error, log, _) = error_of(run_wire(&config(), &get(true), &reply));
    assert_message(&error, "no response after 3");
    assert_eq!(log.dials(), 3);
    assert!(log.released());
    assert_detached(&error);
}

#[test]
fn https_installs_framing_guards_with_verified_tls_defaults() {
    // Python checks the TLS context (CERT_REQUIRED, check_hostname) and that
    // the stdlib response classes stay untouched. Here the dialler is asked
    // for TLS, which `dial::Net` verifies with the platform verifier, and the
    // same framing guards apply to what the TLS connection returns.
    let mut repeated = b"HTTP/1.1 429 Throttled\r\n".to_vec();
    repeated.extend(b"X-Fabricated: ignored\r\n".repeat(101));
    repeated.extend(b"\r\n");
    let wires: [Vec<u8>; 3] = [
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n-1\r\n".to_vec(),
        b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\n[]".to_vec(),
        repeated,
    ];
    let target = "https://127.0.0.1:9";
    let config = AdapterConfig {
        enabled: true,
        base_url: Some(target.into()),
        allow_loopback: true,
        max_attempts: 1.into(),
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a valid configuration");
    let url = format!("{target}/v1/calculators");
    for bytes in wires {
        let call = Call {
            bearer_token: Some(CANARY),
            ..call("GET", &url, false)
        };
        let (error, log, _) = error_of(run_wire(&config, &call, &wire(&bytes)));
        assert_eq!(*log.dials.borrow(), vec![("127.0.0.1".to_owned(), 9, true)]);
        assert!(log.released());
        assert_class(&error, TRANSPORT);
        assert_detached(&error);
    }
}

#[test]
fn premature_chunk_trailer_eof_is_not_completion() {
    let tails: [&[u8]; 3] = [b"0\r\n", b"0", b"0\r\nX-Trailer: unfinished"];
    for status in [200, 400, 503, 600] {
        for retry_safe in [false, true] {
            for tail in tails {
                let bytes = [
                    format!("HTTP/1.1 {status} Test\r\nTransfer-Encoding: chunked\r\n\r\n")
                        .as_bytes(),
                    b"2\r\n[]\r\n",
                    tail,
                ]
                .concat();
                let (error, log, slept) =
                    error_of(run_wire(&config(), &get(retry_safe), &wire(&bytes)));
                let attempts = if retry_safe && status != 400 { 3 } else { 1 };
                assert_eq!(log.dials(), attempts, "{status} {retry_safe} {tail:?}");
                assert_eq!(slept, pauses(attempts));
                assert!(log.released());
                assert_detached(&error);
            }
        }
    }
}

#[test]
fn invalid_length_is_not_close_delimited_success() {
    let lengths: [&[u8]; 7] = [
        b"2, 3",
        b"2\r\nContent-Length: 3",
        b"-2",
        b"two",
        b"",
        b"3, 3",
        b"3\r\nContent-Length: 3",
    ];
    for status in [200, 400, 503] {
        for length in lengths {
            let bytes = [
                format!("HTTP/1.1 {status} Test\r\nContent-Length: ").as_bytes(),
                length,
                b"\r\n\r\n[]",
            ]
            .concat();
            let (error, log, _) = error_of(run_wire(&config(), &get(true), &wire(&bytes)));
            assert_eq!(
                log.dials(),
                if status == 400 { 1 } else { 3 },
                "{status} {length:?}"
            );
            assert!(log.released());
            assert_detached(&error);
        }
    }
}

#[test]
fn initial_header_eof_is_not_completion() {
    for status in [200, 429, 503] {
        let reply = wire(format!("HTTP/1.1 {status} Test\r\nContent-Length: 0\r\n").as_bytes());
        let (error, log, _) = error_of(run_wire(&config(), &get(true), &reply));
        assert_eq!(log.dials(), if status == 429 { 1 } else { 3 }, "{status}");
        assert!(log.released());
        assert_detached(&error);
    }
}

#[test]
fn known_status_survives_secondary_opener_cleanup() {
    // The failing stream and socket closes have no Rust counterpart; the
    // status parsed before the header failure still sets the budget.
    for status in [302, 400, 422, 429, 503, 600] {
        let bytes = [
            format!("HTTP/1.1 {status} Test\r\n").into_bytes(),
            many_headers("ignored"),
        ]
        .concat();
        let (error, log, slept) = error_of(run_wire(&config(), &get(true), &wire(&bytes)));
        let attempts = if status >= 500 { 3 } else { 1 };
        assert_eq!(log.dials(), attempts, "{status}");
        assert_eq!(slept, pauses(attempts));
        assert!(log.released());
        assert_class(&error, TRANSPORT);
        assert_detached(&error);
    }
}

#[test]
fn completed_framing_and_bodyless_controls() {
    assert_framed(
        &[
            (200, "GET", b"Content-Length: 0\r\n", b"", b""),
            (200, "GET", b"Content-Length: 2, 2\r\n", b"[]", b"[]"),
            (
                200,
                "GET",
                b"Content-Length: 2\r\nContent-Length: 2\r\n",
                b"[]",
                b"[]",
            ),
            (
                200,
                "GET",
                b"Transfer-Encoding: chunked\r\n",
                b"2\r\n[]\r\n0\r\n\r\n",
                b"[]",
            ),
            (
                200,
                "GET",
                b"Transfer-Encoding: chunked\r\n",
                b"2\r\n[]\r\n0\r\nX-Trailer: ignored\r\n\r\n",
                b"[]",
            ),
            (304, "GET", b"Transfer-Encoding: chunked\r\n", b"", b""),
            (200, "HEAD", b"Transfer-Encoding: chunked\r\n", b"", b""),
            (204, "GET", b"Transfer-Encoding: chunked\r\n", b"", b""),
            (304, "GET", b"Content-Length: invalid\r\n", b"", b""),
            (
                200,
                "GET",
                b"Transfer-Encoding: chunked\r\nContent-Length: invalid\r\n",
                b"2\r\n[]\r\n0\r\n\r\n",
                b"[]",
            ),
        ],
        |headers, raw| {
            let has = |needle: &[u8]| headers.windows(needle.len()).any(|w| w == needle);
            if has(b"Transfer-Encoding") {
                assert_eq!(header(raw, "transfer-encoding"), Some("chunked"));
            }
            if has(b"Content-Length: 2, 2") {
                assert_eq!(header(raw, "content-length"), Some("2, 2"));
            }
        },
    );
}

#[test]
fn invalid_message_discovery_cannot_be_computed() {
    let wires: [&[u8]; 2] = [
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n[]\r\n0\r\n",
        b"HTTP/1.1 200 OK\r\nContent-Length: 2, 3\r\n\r\n[]",
    ];
    for bytes in wires {
        let reply = wire(bytes);
        let (outcome, log) = discover(&config(), |_| Ok(reply.clone()));
        assert_unavailable(&outcome);
        assert_eq!(log.dials(), 3);
        assert!(log.released());
    }
}

#[test]
fn https_eof_suppression_branch() {
    // Python fabricates SSLSocket reads; a TLS close without close_notify is
    // the `UnexpectedEof` rustls returns, which the dialler passes through.
    let target = "https://127.0.0.1:9";
    let url = format!("{target}/v1/calculators");
    let settings = AdapterConfig {
        enabled: true,
        base_url: Some(target.into()),
        allow_loopback: true,
        max_attempts: 3.into(),
        retry_backoff_seconds: 0.01,
        ..AdapterConfig::default()
    }
    .checked()
    .expect("a valid configuration");
    for ragged in [false, true] {
        for framing in ["close", "length", "chunked"] {
            for status in [200, 429, 503] {
                let (headers, body): (&[u8], &[u8]) = match framing {
                    "length" => (b"Content-Length: 2\r\n", b"[]"),
                    "chunked" => (b"Transfer-Encoding: chunked\r\n", b"2\r\n[]\r\n0\r\n\r\n"),
                    _ => (b"", b"[]"),
                };
                let bytes = [
                    format!("HTTP/1.1 {status} Test\r\n").as_bytes(),
                    headers,
                    b"\r\n",
                    body,
                ]
                .concat();
                let reply = Reply {
                    segments: vec![bytes],
                    end: ragged.then_some(io::ErrorKind::UnexpectedEof),
                };
                let outcome = run_wire(&settings, &call("GET", &url, true), &reply);
                let failed_close = ragged && framing == "close";
                let (log, slept) = if failed_close {
                    let (error, log, slept) = error_of(outcome);
                    assert_detached(&error);
                    (log, slept)
                } else {
                    let (raw, log, slept) = success(outcome);
                    assert_eq!(raw.status, status);
                    assert_eq!(raw.body, b"[]");
                    (log, slept)
                };
                let attempts = if status == 429 || (status == 200 && !failed_close) {
                    1
                } else {
                    3
                };
                assert_eq!(log.dials(), attempts, "{ragged} {framing} {status}");
                assert!(log.dials.borrow().iter().all(|(_, _, tls)| *tls));
                assert_eq!(slept, pauses(attempts));
                assert!(log.released());
            }
        }
    }
}

#[test]
fn invalid_chunk_transition_is_not_completion() {
    let bodies: [&[u8]; 5] = [
        b"2\r\n[]XX0\r\n\r\n",
        b"+2\r\n[]\r\n0\r\n\r\n",
        b"0x2\r\n[]\r\n0\r\n\r\n",
        b"-0\r\n\r\n",
        b" 2\r\n[]\r\n0\r\n\r\n",
    ];
    for status in [200, 400, 503] {
        for body in bodies {
            let bytes = [
                format!("HTTP/1.1 {status} Test\r\nTransfer-Encoding: chunked\r\n\r\n").as_bytes(),
                body,
            ]
            .concat();
            let (error, log, _) = error_of(run_wire(&config(), &get(true), &wire(&bytes)));
            assert_eq!(
                log.dials(),
                if status == 400 { 1 } else { 3 },
                "{status} {body:?}"
            );
            assert!(log.released());
            assert_detached(&error);
        }
    }
}

#[test]
fn bad_chunk_delimiter_cannot_be_computed() {
    let reply = wire(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n[]XX0\r\n\r\n");
    let (outcome, log) = discover(&config(), |_| Ok(reply.clone()));
    assert_unavailable(&outcome);
    assert_eq!(log.dials(), 3);
    assert!(log.released());
}

#[test]
fn valid_hex_sizes_and_extensions() {
    let reply = wire(
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n\
          A;fabricated=value\r\n0123456789\r\n0\r\n\r\n",
    );
    let (raw, log, _) = success(run_wire(&config(), &get(false), &reply));
    assert_eq!(raw.status, 200);
    assert_eq!(raw.body, b"0123456789");
    assert_eq!(log.dials(), 1);
    assert!(log.released());
}

#[test]
fn transfer_coding_takes_precedence_over_length() {
    let cases: [(&[u8], &[u8], &[u8]); 2] = [
        (
            b"Transfer-Encoding: gzip\r\nContent-Length: 2\r\n",
            b"[]x",
            b"[]x",
        ),
        (
            b"Transfer-Encoding: gzip, chunked\r\nContent-Length: invalid\r\n",
            b"2\r\n[]\r\n0\r\n\r\n",
            b"[]",
        ),
    ];
    for (headers, body, expected) in cases {
        let bytes = [b"HTTP/1.1 200 OK\r\n", headers, b"\r\n", body].concat();
        let (raw, log, _) = success(run_wire(&config(), &get(false), &wire(&bytes)));
        assert_eq!(raw.body, expected);
        assert_eq!(log.dials(), 1);
        assert!(log.released());
    }
}

#[test]
fn transfer_coding_cannot_hide_bytes_above_ceiling() {
    let reply = wire(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: gzip\r\nContent-Length: 2\r\n\r\n[]x");
    let (error, log, _) = error_of(run_wire(&config_limit(2), &get(true), &reply));
    assert_message(&error, "byte ceiling");
    assert_eq!(log.dials(), 1);
    assert!(log.released());
}

/// A 302 with an optional Location and an unread canary body, split so a
/// read of the body shows.
fn redirect(status: u16, location: Option<&str>) -> Reply {
    let location = location.map_or(String::new(), |l| format!("Location: {l}\r\n"));
    let head = format!(
        "HTTP/1.1 {status} Test\r\n{location}Content-Length: {}\r\n\r\n",
        CANARY.len()
    );
    split(head.as_bytes(), CANARY.as_bytes())
}

fn assert_refused_unread(reply: &Reply) {
    let (error, log, slept) = error_of(run_wire(&config(), &get(true), reply));
    assert_eq!(log.dials(), 1);
    assert!(log.released());
    assert_eq!(log.later_reads.get(), 0, "the refused body is never read");
    assert!(slept.is_empty(), "redirect retry");
    assert_class(&error, DISALLOWED_TARGET);
    assert_detached(&error);
}

#[test]
fn redirect_without_location_releases_unread_body() {
    // A failing close() on the body has no Rust counterpart.
    for status in [301, 302, 303, 307, 308] {
        assert_refused_unread(&redirect(status, None));
    }
}

#[test]
fn non_io_close_failure_preserves_recorded_body_failure() {
    // The non-I/O close() failure has no Rust counterpart; the recorded body
    // failure and its retry budget are what remain.
    for (status, failure, retry_safe, attempts) in [
        (429, "parser", true, 1),
        (503, "parser", false, 1),
        (503, "parser", true, 3),
        (200, "oversize", true, 1),
    ] {
        let message = if failure == "oversize" {
            "byte ceiling".to_owned()
        } else {
            format!("HTTP {status} body could not be read")
        };
        let (error, log, _) = error_of(run_wire(
            &config_limit(CANARY.len()),
            &get(retry_safe),
            &error_reply(status, failure),
        ));
        assert_message(&error, &message);
        assert_eq!(log.dials(), attempts, "{status} {failure} {retry_safe}");
        assert!(log.released());
        assert_detached(&error);
    }
}

#[test]
fn redirect_close_exception_keeps_refusal() {
    // The RuntimeError or TypeError from close() has no Rust counterpart.
    for location in [false, true] {
        let target = location.then_some("http://127.0.0.1:9/elsewhere");
        assert_refused_unread(&redirect(302, target));
    }
}

#[test]
fn non_io_header_cleanup_preserves_known_status() {
    // The RuntimeError from the stream and socket closes has no Rust
    // counterpart; the parsed status still sets the budget.
    for status in [429, 503] {
        let bytes = [
            format!("HTTP/1.1 {status} Test\r\n").into_bytes(),
            many_headers("ignored"),
        ]
        .concat();
        let (error, log, _) = error_of(run_wire(&config(), &get(true), &wire(&bytes)));
        assert_eq!(log.dials(), if status == 503 { 3 } else { 1 });
        assert!(log.released());
        assert_class(&error, TRANSPORT);
        assert_detached(&error);
    }
}

// -- framing paths the cases above do not reach ------------------------------

#[test]
fn an_invalid_transfer_coding_list_is_not_completion() {
    let codings: [&[u8]; 2] = [b"chunked, identity", b"gzip, , chunked"];
    for coding in codings {
        let bytes = [
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: ".as_slice(),
            coding,
            b"\r\n\r\n2\r\n[]\r\n0\r\n\r\n",
        ]
        .concat();
        let (error, log, _) = error_of(run_wire(&config(), &get(false), &wire(&bytes)));
        assert_eq!(log.dials(), 1);
        assert!(log.released());
        assert_detached(&error);
    }
}

/// `http.client._MAXLINE` and `_MAXHEADERS`.
const MAXLINE: usize = 65536;
const MAXHEADERS: usize = 100;

#[test]
fn a_chunk_size_line_past_the_line_limit_is_not_completion() {
    let bytes = [
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec(),
        b"0".repeat(MAXLINE + 1),
        b"2\r\n[]\r\n0\r\n\r\n".to_vec(),
    ]
    .concat();
    let (error, log, _) = error_of(run_wire(&config(), &get(false), &wire(&bytes)));
    assert_class(&error, TRANSPORT);
    assert_eq!(log.dials(), 1);
    assert!(log.released());
}

#[test]
fn a_chunk_larger_than_one_read_is_read_across_reads() {
    let body = b"x".repeat(70000);
    let bytes = [
        b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n".to_vec(),
        format!("{:x}\r\n", body.len()).into_bytes(),
        body.clone(),
        b"\r\n0\r\n\r\n".to_vec(),
    ]
    .concat();
    let (raw, log, _) = success(run_wire(&config(), &get(false), &wire(&bytes)));
    assert_eq!(raw.status, 200);
    assert_eq!(raw.body, body);
    assert_eq!(log.dials(), 1);
    assert!(log.released());
}

#[test]
fn an_oversized_or_endless_trailer_is_not_completion() {
    let trailers = [
        [
            b"X-Trailer: ".to_vec(),
            b"t".repeat(MAXLINE + 1),
            b"\r\n\r\n".to_vec(),
        ]
        .concat(),
        [b"X-Trailer: t\r\n".repeat(MAXHEADERS + 1), b"\r\n".to_vec()].concat(),
    ];
    for trailer in trailers {
        let bytes = [
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n[]\r\n0\r\n".to_vec(),
            trailer,
        ]
        .concat();
        let (error, log, _) = error_of(run_wire(&config(), &get(false), &wire(&bytes)));
        assert_eq!(log.dials(), 1);
        assert!(log.released());
        assert_detached(&error);
    }
}

#[test]
fn a_response_without_a_status_is_treated_as_no_response() {
    // Python's HTTPError with no status; on the wire, a body with no status
    // line before it.
    let (error, log, _) = error_of(run_wire(&config(), &get(true), &wire(b"[]")));
    assert!(text(&error.text).starts_with("no response after 3 attempt"));
    assert_eq!(log.dials(), 3);
    assert!(log.released());
}

#[test]
fn nonpositive_attempt_limit_refuses_before_sending() {
    // `checked()` refuses 0, as AdapterConfig's __post_init__ does; the
    // struct is built directly, as Python's object.__setattr__ bypasses it.
    let malformed = AdapterConfig {
        max_attempts: 0.into(),
        ..config()
    };
    let (error, log, _) = error_of(run(&malformed, &get(true), |_| {
        panic!("no attempt may run with a zero limit")
    }));
    assert_class(&error, TRANSPORT);
    assert_message(&error, "attempt limit must be positive");
    assert_eq!(log.dials(), 0);
}
