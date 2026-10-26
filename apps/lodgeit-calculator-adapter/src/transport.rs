//! `transport.py`: one bounded HTTP request, as urllib and CPython 3.15's
//! `http.client` run it with the adapter's handlers and response class.
//!
//! - Redirects are never followed. A 3xx is a refusal, because a redirect is
//!   the provider changing the destination after the allowlist check passed.
//! - The response is read with a hard byte ceiling, at most 64 KiB at a time.
//! - Retries happen only for a transport failure or a 5xx, never on a 4xx,
//!   and only when the caller marks the request `retry_safe`.
//! - `read_timeout` is the only timeout, given to the dialler.
//! - Nothing here logs a request or response body.
//!
//! The request is built as urllib builds it (`Request`, `do_request_`, the
//! adapter's `_open`) and written as `http.client` writes it, so a URL,
//! method or header Python refused before connecting is refused here before
//! dialling. Dialling is the [`Dial`] the caller passes: gate B1 supplies TCP
//! and TLS, the differential a script.

use std::io::{self, BufRead, BufReader, Read, Write};
use std::time::Instant;

use num_bigint::BigInt;
use num_traits::ToPrimitive;

use crate::config::AdapterConfig;
use crate::errors::{CONFIGURATION, DISALLOWED_TARGET, TRANSPORT};
use crate::https::{Failure, header_lines, latin1, parse_header_lines, read_line, status_line};
use crate::pyops::{Exc, R};
use crate::{py, pyurl, sre};

/// `http.client._MAXLINE` and `_MAXHEADERS`.
const MAXLINE: usize = 65536;
const MAXHEADERS: usize = 100;
const MAXINTERIM: usize = 100;
/// urllib's default `User-agent`, added when the request names none.
const URLLIB_AGENT: &str = "Python-urllib/3.15";

/// What a request needs from its configuration: `transport._RequestConfig`.
pub trait RequestConfig {
    fn require_enabled(&self) -> R<()>;
    /// `check_url(url)`, with the route required.
    fn check_url(&self, url: &str) -> R<()>;
    fn read_timeout(&self) -> f64;
    fn max_response_bytes(&self) -> &BigInt;
    fn max_attempts(&self) -> u32;
    fn retry_backoff_seconds(&self) -> f64;
    fn user_agent(&self) -> &str;
    fn extra_headers(&self) -> &[(String, String)];
}

impl RequestConfig for AdapterConfig {
    fn require_enabled(&self) -> R<()> {
        AdapterConfig::require_enabled(self)
    }
    fn check_url(&self, url: &str) -> R<()> {
        AdapterConfig::check_url(self, url, true)
    }
    fn read_timeout(&self) -> f64 {
        self.read_timeout
    }
    fn max_response_bytes(&self) -> &BigInt {
        &self.max_response_bytes
    }
    fn max_attempts(&self) -> u32 {
        self.attempts()
    }
    fn retry_backoff_seconds(&self) -> f64 {
        self.retry_backoff_seconds
    }
    fn user_agent(&self) -> &str {
        &self.user_agent
    }
    fn extra_headers(&self) -> &[(String, String)] {
        &self.extra_headers
    }
}

/// A connected socket, plain or TLS.
pub trait Connection: Read + Write {}
impl<T: Read + Write> Connection for T {}

/// `socket.create_connection((host, port), timeout)`, wrapped in TLS when
/// `tls` is set: an error is the `OSError` Python's connect raised, or,
/// carrying [`Raised`], an exception `_attempt` does not catch.
pub trait Dial {
    fn dial(
        &mut self,
        host: &str,
        port: u16,
        tls: bool,
        timeout: f64,
    ) -> io::Result<Box<dyn Connection>>;
}

/// The payload of a dial error that `request` raises as it is, as Python's
/// `OverflowError` from `settimeout` passes through `_attempt`.
#[derive(Debug)]
pub struct Raised(pub Exc);

impl std::fmt::Display for Raised {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.0.class)
    }
}

impl std::error::Error for Raised {}

/// What came back, before anything interprets it.
#[derive(Clone, Debug, PartialEq)]
pub struct RawResponse {
    pub status: u16,
    /// `{key.lower(): value}` over the response's headers, in dict order.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub url: String,
    pub elapsed_ms: u64,
}

impl RawResponse {
    /// `text()`: the body as strict UTF-8.
    pub fn text(&self) -> R<String> {
        String::from_utf8(self.body.clone())
            .map_err(|_| Exc::msg(TRANSPORT, "response is not UTF-8"))
    }
}

/// One request: `transport.request`'s arguments after `config`.
pub struct Call<'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub body: Option<&'a [u8]>,
    pub content_type: Option<&'a str>,
    pub retry_safe: bool,
    pub bearer_token: Option<&'a str>,
}

/// `_AttemptFailure.kind`. Python also has "close", for a response whose
/// `close()` raised; dropping a Rust connection cannot fail.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Open,
    Read,
    Ceiling,
    Redirect,
}

enum Attempt {
    Raw(RawResponse),
    Failed(Kind, Option<u16>),
    Raised(Exc),
}

/// `request(config, method, url, ...)`: one HTTP request with every bound
/// the config sets, retried as `retry_safe` allows. `sleep` is
/// `time.sleep`, given each backoff in seconds.
pub fn request(
    config: &dyn RequestConfig,
    dial: &mut dyn Dial,
    call: &Call,
    sleep: &mut dyn FnMut(f64) -> R<()>,
) -> R<RawResponse> {
    config.require_enabled()?;
    config.check_url(call.url)?;
    let mut headers = Vec::new();
    put(&mut headers, "Accept", "application/json");
    put(&mut headers, "User-Agent", config.user_agent());
    for (name, value) in config.extra_headers() {
        put(&mut headers, name, value);
    }
    if let Some(token) = call.bearer_token {
        if py::char_len(token) > 8192
            || !bearer_shape(token)
            || config
                .extra_headers()
                .iter()
                .any(|(name, _)| lower(name) == "authorization")
        {
            return Err(Exc::msg(
                CONFIGURATION,
                "Invalid per-call bearer authentication.",
            ));
        }
        put(&mut headers, "Authorization", &format!("Bearer {token}"));
    }
    if let Some(content_type) = call.content_type.filter(|c| !c.is_empty()) {
        put(&mut headers, "Content-Type", content_type);
    }
    let attempts = if call.retry_safe {
        config.max_attempts()
    } else {
        1
    };
    let limit = config.max_response_bytes();
    let mut last = None;
    for attempt in 1..=attempts {
        let result = one_attempt(dial, call, &headers, config.read_timeout(), limit);
        match result {
            Attempt::Raw(raw) => {
                if raw.status < 500 || attempt == attempts {
                    return Ok(raw);
                }
            }
            Attempt::Raised(exc) => return Err(exc),
            Attempt::Failed(kind, status) => {
                let server_error = status.is_some_and(|s| s >= 500);
                let eligible = kind != Kind::Redirect
                    && (server_error
                        || status.is_none_or(|s| (200..300).contains(&s) && kind != Kind::Ceiling));
                if !eligible || attempt == attempts {
                    last = Some((kind, status, attempt));
                    break;
                }
            }
        }
        sleep(config.retry_backoff_seconds() * f64::from(attempt))?;
    }
    let Some((kind, status, attempt)) = last else {
        return Err(Exc::msg(
            TRANSPORT,
            "request attempt limit must be positive",
        ));
    };
    let message = match (kind, status) {
        (Kind::Redirect, _) => {
            return Err(Exc::msg(
                DISALLOWED_TARGET,
                "provider redirected; redirects are refused",
            ));
        }
        (Kind::Ceiling, _) => {
            format!("response exceeded the {limit} byte ceiling and was abandoned")
        }
        (_, None) => format!("no response after {attempt} attempt(s)"),
        (_, Some(status)) => {
            format!("HTTP {status} body could not be read after {attempt} attempt(s)")
        }
    };
    Err(Exc::msg(TRANSPORT, &message))
}

/// `re.fullmatch(r"[A-Za-z0-9._~+/-]+=*", token)`.
fn bearer_shape(token: &str) -> bool {
    let body = token.trim_end_matches('=');
    !body.is_empty()
        && body
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "._~+/-".contains(c))
}

/// `dict[key] = value`: a new key goes last, a known one keeps its place.
fn put(map: &mut Vec<(String, String)>, key: &str, value: &str) {
    match map.iter_mut().find(|(k, _)| k == key) {
        Some(entry) => entry.1 = value.to_owned(),
        None => map.push((key.to_owned(), value.to_owned())),
    }
}

fn has(map: &[(String, String)], key: &str) -> bool {
    map.iter().any(|(k, _)| k == key)
}

fn lower(text: &str) -> String {
    sre::str_lower(&crate::pyops::pts(text))
        .into_iter()
        .filter_map(char::from_u32)
        .collect()
}

fn upper_char(c: char) -> char {
    char::from_u32(sre::upper(u32::from(c))).unwrap_or(c)
}

fn lower_char(c: char) -> char {
    char::from_u32(sre::lower(u32::from(c))).unwrap_or(c)
}

/// `_PyUnicode_IsCased`.
fn cased(c: char) -> bool {
    let cp = u32::from(c);
    crate::pycase::CASED
        .binary_search_by(|&(lo, hi)| {
            if hi < cp {
                std::cmp::Ordering::Less
            } else if lo > cp {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

// shortcut: `str.capitalize()` and `str.title()` with the simple case
// mappings; Python's full title mappings differ for ligatures and digraphs
// (`'ﬀ'.title()` is 'Ff'), which only a non-ASCII header name meets.
fn capitalize(text: &str) -> String {
    let mut chars = text.chars();
    chars
        .next()
        .map(upper_char)
        .into_iter()
        .chain(chars.map(lower_char))
        .collect()
}

fn title(text: &str) -> String {
    let mut previous_cased = false;
    text.chars()
        .map(|c| {
            let out = if previous_cased {
                lower_char(c)
            } else {
                upper_char(c)
            };
            previous_cased = cased(c);
            out
        })
        .collect()
}

/// The request as urllib and `http.client` put it on the wire.
struct Prepared {
    host: String,
    port: u16,
    tls: bool,
    head: bool,
    sends: Vec<Vec<u8>>,
}

/// `urllib.request.unwrap(url)`.
fn unwrap(url: &str) -> &str {
    let mut url = py::strip(url);
    if url.starts_with('<') && url.ends_with('>') && url.len() >= 2 {
        url = py::strip(&url[1..url.len() - 1]);
    }
    if let Some(rest) = url.strip_prefix("URL:") {
        url = py::strip(rest);
    }
    url
}

/// `_splittype`: `([^/:]+):(.*)`.
fn split_type(url: &str) -> Option<(String, &str)> {
    let colon = url.find([':', '/'])?;
    if colon == 0 || !url[colon..].starts_with(':') {
        return None;
    }
    Some((lower(&url[..colon]), &url[colon + 1..]))
}

/// `_splithost`: `//([^/#?]*)(.*)`, the path given a leading `/`.
fn split_host(rest: &str) -> (Option<&str>, String) {
    let Some(after) = rest.strip_prefix("//") else {
        return (None, rest.to_owned());
    };
    let end = after.find(['/', '#', '?']).unwrap_or(after.len());
    let path = &after[end..];
    let path = if !path.is_empty() && !path.starts_with('/') {
        format!("/{path}")
    } else {
        path.to_owned()
    };
    (Some(&after[..end]), path)
}

/// `HTTPConnection._get_hostport(host, None)`.
fn host_port(host: &str, default: u16) -> Option<(String, u16)> {
    let mut host = host;
    let mut port = default;
    let colon = host.rfind(':');
    let bracket = host.rfind(']');
    if let Some(i) = colon
        && bracket.is_none_or(|j| i > j)
    {
        let text = &host[i + 1..];
        match py::parse_int(text, 10) {
            // shortcut: a port outside 0-65535 fails in the socket layer,
            // which Python reports as OverflowError; check_url's own port
            // rule keeps such URLs out unless the host is percent-encoded.
            Ok(n) => port = n.to_u16()?,
            Err(_) if text.is_empty() => {}
            Err(_) => return None,
        }
        host = &host[..i];
    }
    if host.len() >= 2 && host.starts_with('[') && host.ends_with(']') {
        host = &host[1..host.len() - 1];
    }
    Some((host.to_owned(), port))
}

/// `_contains_disallowed_url_pchar_re`: `[\x00-\x20\x7f]`.
fn url_char_refused(text: &str) -> bool {
    text.chars().any(|c| c <= ' ' || c == '\x7f')
}

/// `_is_legal_header_name`: `[^:\s][^:\r\n]*` over the name's bytes.
fn legal_name(name: &[u8]) -> bool {
    match name.split_first() {
        Some((first, rest)) => {
            *first != b':'
                && !matches!(first, b' ' | b'\t' | b'\n' | b'\r' | b'\x0b' | b'\x0c')
                && !rest.iter().any(|b| matches!(b, b':' | b'\r' | b'\n'))
        }
        None => false,
    }
}

/// `_is_illegal_header_value`: `\n(?![ \t])|\r(?![ \t\n])`.
fn illegal_value(value: &[u8]) -> bool {
    value.iter().enumerate().any(|(i, &b)| {
        let next = value.get(i + 1).copied();
        (b == b'\n' && !matches!(next, Some(b' ' | b'\t')))
            || (b == b'\r' && !matches!(next, Some(b' ' | b'\t' | b'\n')))
    })
}

/// `putheader(name, value)`: the line, or None for the ValueError or
/// UnicodeEncodeError Python raised before connecting.
fn header_line(name: &str, value: &str) -> Option<Vec<u8>> {
    if !name.is_ascii() || !legal_name(name.as_bytes()) {
        return None;
    }
    let value: Vec<u8> = value
        .chars()
        .map(|c| u8::try_from(u32::from(c)).ok())
        .collect::<Option<_>>()?;
    if illegal_value(&value) {
        return None;
    }
    Some([name.as_bytes(), b": ", &value].concat())
}

/// urllib's `Request`, `do_request_` and the adapter's `_open`, through
/// `http.client`'s `putrequest`, `putheader` and `endheaders`: where to
/// connect and what to send, or None where Python raised before connecting.
fn prepare(call: &Call, headers: &[(String, String)]) -> Option<Prepared> {
    let full = unwrap(call.url);
    let full = full.rsplit_once('#').map_or(full, |(url, _)| url);
    let (scheme, rest) = split_type(full)?;
    let (host, selector) = split_host(rest);
    let host = pyurl::unquote(host.unwrap_or_default());
    // `Request` keeps only a non-empty method; `get_method` supplies the default.
    let method = match call.method {
        "" if call.body.is_some() => "POST",
        "" => "GET",
        method => method,
    };
    let mut asked = Vec::new();
    for (name, value) in headers {
        put(&mut asked, &capitalize(name), value);
    }
    let mut unredirected = Vec::new();
    let known = |asked: &[(String, String)], unredirected: &[(String, String)], key: &str| {
        has(asked, key) || has(unredirected, key)
    };
    if let Some(body) = call.body {
        if !known(&asked, &unredirected, "Content-type") {
            put(
                &mut unredirected,
                "Content-type",
                "application/x-www-form-urlencoded",
            );
        }
        if !known(&asked, &unredirected, "Content-length")
            && !known(&asked, &unredirected, "Transfer-encoding")
        {
            put(&mut unredirected, "Content-length", &body.len().to_string());
        }
    }
    if host.is_empty() {
        return None;
    }
    if !known(&asked, &unredirected, "Host") {
        put(&mut unredirected, "Host", &host);
    }
    if !known(&asked, &unredirected, "User-agent") {
        put(&mut unredirected, "User-agent", URLLIB_AGENT);
    }
    let tls = match scheme.as_str() {
        "https" => true,
        "http" => false,
        _ => return None,
    };
    let (connect_host, port) = host_port(&host, if tls { 443 } else { 80 })?;
    if url_char_refused(&connect_host) {
        return None;
    }
    let mut encode_chunked = known(&asked, &unredirected, "Transfer-encoding");
    let mut merged = unredirected;
    for (name, value) in &asked {
        put(&mut merged, name, value);
    }
    put(&mut merged, "Connection", "close");
    let mut titled = Vec::new();
    for (name, value) in &merged {
        put(&mut titled, &title(name), value);
    }
    let names: Vec<String> = titled.iter().map(|(name, _)| lower(name)).collect();
    let named = |key: &str| names.iter().any(|n| n == key);
    // putrequest
    if method.chars().any(|c| c < ' ') {
        return None;
    }
    let path = if selector.is_empty() { "/" } else { &selector };
    if url_char_refused(path) {
        return None;
    }
    let request_line = format!("{method} {path} HTTP/1.1");
    if !request_line.is_ascii() {
        return None;
    }
    let mut lines = vec![request_line.into_bytes()];
    if !named("accept-encoding") {
        lines.push(b"Accept-Encoding: identity".to_vec());
    }
    if !named("content-length") {
        if !named("transfer-encoding") {
            encode_chunked = false;
            let length = match call.body {
                Some(body) => Some(body.len()),
                None => matches!(
                    method.to_ascii_uppercase().as_str(),
                    "PATCH" | "POST" | "PUT"
                )
                .then_some(0),
            };
            if let Some(length) = length {
                lines.push(format!("Content-Length: {length}").into_bytes());
            }
        }
    } else {
        encode_chunked = false;
    }
    for (name, value) in &titled {
        lines.push(header_line(name, value)?);
    }
    lines.extend([Vec::new(), Vec::new()]);
    let mut sends = vec![lines.join(&b"\r\n"[..])];
    if let Some(body) = call.body {
        if !body.is_empty() {
            sends.push(if encode_chunked {
                [format!("{:X}\r\n", body.len()).as_bytes(), body, b"\r\n"].concat()
            } else {
                body.to_vec()
            });
        }
        if encode_chunked {
            sends.push(b"0\r\n\r\n".to_vec());
        }
    }
    Some(Prepared {
        host: connect_host,
        port,
        tls,
        head: method == "HEAD",
        sends,
    })
}

/// `_attempt`: one connection, with any failure reduced to its kind and
/// the status, if one arrived.
fn one_attempt(
    dial: &mut dyn Dial,
    call: &Call,
    headers: &[(String, String)],
    timeout: f64,
    limit: &BigInt,
) -> Attempt {
    let started = Instant::now();
    let Some(prepared) = prepare(call, headers) else {
        return Attempt::Failed(Kind::Open, None);
    };
    let mut connection = match dial.dial(&prepared.host, prepared.port, prepared.tls, timeout) {
        Ok(connection) => connection,
        Err(error) => {
            return match error.into_inner().map(|e| e.downcast::<Raised>()) {
                Some(Ok(raised)) => Attempt::Raised(raised.0),
                _ => Attempt::Failed(Kind::Open, None),
            };
        }
    };
    // `sendall` per piece; a TLS stream holds records until flushed.
    for data in &prepared.sends {
        if connection.write_all(data).is_err() {
            return Attempt::Failed(Kind::Open, None);
        }
    }
    if connection.flush().is_err() {
        return Attempt::Failed(Kind::Open, None);
    }
    let response = match Response::begin(&mut *connection, prepared.head) {
        Ok(response) => response,
        Err(status) => return Attempt::Failed(Kind::Open, status),
    };
    if matches!(response.status, 301 | 302 | 303 | 307 | 308) {
        // `_NoRedirects.http_error_302`: a Location (or URI) whose scheme
        // urllib would follow is refused unread; one it would not follow
        // is urllib's HTTPError, whose body is read as any other status.
        let target = ["location", "uri"]
            .iter()
            .find_map(|name| response.header(name));
        let followed = |url: &str| match pyurl::urlsplit(url) {
            Ok(parts) => matches!(parts.scheme.as_str(), "http" | "https" | "ftp" | ""),
            Err(_) => true, // urlparse's ValueError, which the handler refuses too
        };
        if target.is_none_or(followed) {
            return Attempt::Failed(Kind::Redirect, Some(response.status));
        }
    }
    consume(response, limit, call.url, started)
}

fn elapsed_ms(started: Instant) -> u64 {
    if cfg!(feature = "differential") {
        return 0;
    }
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

/// `_consume`: the bounded body, or the failure reading it.
fn consume(mut response: Response, limit: &BigInt, url: &str, started: Instant) -> Attempt {
    let status = response.status;
    match response.read_bounded(limit) {
        Err(()) => Attempt::Failed(Kind::Read, Some(status)),
        Ok(None) => Attempt::Failed(Kind::Ceiling, Some(status)),
        Ok(Some(body)) => {
            let mut headers = Vec::new();
            for (name, value) in &response.headers {
                put(&mut headers, &name.to_ascii_lowercase(), value);
            }
            Attempt::Raw(RawResponse {
                status,
                headers,
                body,
                url: url.to_owned(),
                elapsed_ms: elapsed_ms(started),
            })
        }
    }
}

/// Any of `transport._IO_ERRORS` (OSError, HTTPException, ValueError).
type Io<T> = Result<T, ()>;

/// `transport._HTTPResponse` over its connection: `http.client`'s reads with
/// the adapter's stricter framing.
struct Response<'a> {
    /// `fp`, None once `_close_conn` ran.
    fp: Option<BufReader<&'a mut dyn Connection>>,
    head: bool,
    status: u16,
    headers: Vec<(String, String)>,
    chunked: bool,
    /// `chunk_left`: None before a chunk-size line, then the bytes left.
    chunk_left: Option<u64>,
    /// `length`: the bytes the framing still owes, if it says.
    length: Option<u64>,
}

/// `fp.read(n)`: `n` bytes, or fewer at the end of the stream.
fn read_upto(fp: &mut impl BufRead, n: u64) -> Io<Vec<u8>> {
    let mut data = Vec::new();
    fp.by_ref().take(n).read_to_end(&mut data).map_err(drop)?;
    Ok(data)
}

/// `_safe_read(amt)`: exactly `amt` bytes, or IncompleteRead.
fn safe_read(fp: &mut impl BufRead, amt: u64) -> Io<Vec<u8>> {
    let data = read_upto(fp, amt)?;
    if (data.len() as u64) < amt {
        return Err(());
    }
    Ok(data)
}

/// Python's `str.strip()` over Latin-1 text.
fn strip(text: &str) -> &str {
    text.trim_matches(py::is_space)
}

/// `_HeaderReader.readline`: a header-section line must end in a newline,
/// so the end of the stream, or a line cut at the length limit, fails.
fn header_line_of(fp: &mut impl BufRead) -> Result<Vec<u8>, Failure> {
    let line = read_line(fp)?;
    if line.ends_with(b"\n") {
        Ok(line)
    } else {
        Err("incomplete headers".into())
    }
}

impl<'a> Response<'a> {
    /// `getresponse()` and `begin()`: the status and headers, or the status
    /// if one was read before the failure.
    fn begin(connection: &'a mut dyn Connection, head: bool) -> Result<Response<'a>, Option<u16>> {
        let mut fp = BufReader::new(connection);
        let mut found = None;
        for _ in 0..MAXINTERIM {
            let (version, status) =
                status_line(&header_line_of(&mut fp).map_err(|_| None)?).map_err(|_| None)?;
            if status != 100 {
                found = Some((version, status));
                break;
            }
            header_lines(|| header_line_of(&mut fp)).map_err(|_| None)?;
        }
        let (version, status) = found.ok_or(None)?;
        let failed = Some(status);
        if !(version == "HTTP/1.0" || version == "HTTP/0.9" || version.starts_with("HTTP/1.")) {
            return Err(failed);
        }
        let headers =
            parse_header_lines(&header_lines(|| header_line_of(&mut fp)).map_err(|_| failed)?);
        let mut response = Response {
            fp: Some(fp),
            head,
            status,
            headers,
            chunked: false,
            chunk_left: None,
            length: None,
        };
        // Method and status framing take precedence over representation headers.
        if head || matches!(status, 204 | 304) || (100..200).contains(&status) {
            response.length = Some(0);
            return Ok(response);
        }
        let all = |name: &str| -> Vec<String> {
            response
                .headers
                .iter()
                .filter(|(key, _)| key.eq_ignore_ascii_case(name))
                .map(|(_, value)| value.clone())
                .collect()
        };
        let codings = all("transfer-encoding");
        if !codings.is_empty() {
            let codings: Vec<String> = codings
                .iter()
                .flat_map(|value| value.split(','))
                .map(|part| strip(part).to_ascii_lowercase())
                .collect();
            let (last, before) = codings.split_last().expect("one value at least");
            if codings.iter().any(String::is_empty) || before.iter().any(|c| c == "chunked") {
                return Err(failed);
            }
            response.chunked = last == "chunked";
            return Ok(response);
        }
        let lengths = all("content-length");
        if !lengths.is_empty() {
            let values: Vec<&str> = lengths
                .iter()
                .flat_map(|value| value.split(','))
                .map(strip)
                .collect();
            if values
                .iter()
                .any(|v| v.is_empty() || !v.bytes().all(|b| b.is_ascii_digit()))
            {
                return Err(failed);
            }
            // `int(value)` refuses more than 4,300 digits, leading zeros
            // included (`sys.int_info.default_max_str_digits`).
            if values.iter().any(|v| v.len() > 4300) {
                return Err(failed);
            }
            if values
                .iter()
                .any(|v| v.trim_start_matches('0') != values[0].trim_start_matches('0'))
            {
                return Err(failed);
            }
            // A length past u64 is never satisfied, as Python's int is not.
            response.length = Some(values[0].parse().unwrap_or(u64::MAX));
        }
        Ok(response)
    }

    /// The first value of a header, as `headers[name]`.
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }

    /// `read(amt)`, with the adapter's IncompleteRead for a short
    /// Content-Length body.
    fn read(&mut self, amt: usize) -> Io<Vec<u8>> {
        let data = self.read_inner(amt)?;
        if data.is_empty() && amt != 0 && self.length.is_some_and(|l| l > 0) {
            return Err(());
        }
        Ok(data)
    }

    fn read_inner(&mut self, amt: usize) -> Io<Vec<u8>> {
        if self.fp.is_none() {
            return Ok(Vec::new());
        }
        if self.head {
            self.fp = None;
            return Ok(Vec::new());
        }
        if self.chunked {
            return self.read_chunked(amt as u64);
        }
        let mut amt = amt as u64;
        if let Some(length) = self.length
            && amt > length
        {
            amt = length;
        }
        let fp = self.fp.as_mut().expect("open");
        let data = read_upto(fp, amt)?;
        if data.is_empty() && amt != 0 {
            self.fp = None;
        } else if let Some(length) = &mut self.length {
            *length -= data.len() as u64;
            if *length == 0 {
                self.fp = None;
            }
        }
        Ok(data)
    }

    /// `_read_chunked(amt)`.
    fn read_chunked(&mut self, mut amt: u64) -> Io<Vec<u8>> {
        let mut value = Vec::new();
        while let Some(chunk_left) = self.chunk_left()? {
            let fp = self.fp.as_mut().expect("a chunk is open");
            if amt <= chunk_left {
                value.extend(safe_read(fp, amt)?);
                self.chunk_left = Some(chunk_left - amt);
                break;
            }
            value.extend(safe_read(fp, chunk_left)?);
            amt -= chunk_left;
            self.chunk_left = Some(0);
        }
        Ok(value)
    }

    /// `_get_chunk_left()`: the bytes left in the current chunk, reading the
    /// next chunk-size line when it is spent; None after the last chunk.
    fn chunk_left(&mut self) -> Io<Option<u64>> {
        let Some(fp) = self.fp.as_mut() else {
            return Ok(None);
        };
        if let Some(left) = self.chunk_left
            && left > 0
        {
            return Ok(Some(left));
        }
        if self.chunk_left == Some(0) && safe_read(fp, 2)? != b"\r\n" {
            return Err(());
        }
        let size = next_chunk_size(fp)?;
        if size == 0 {
            discard_trailer(fp)?;
            self.fp = None;
            self.chunk_left = None;
            return Ok(None);
        }
        self.chunk_left = Some(size);
        Ok(Some(size))
    }

    /// `_read_bounded(response, limit)`: the body, or None past the ceiling.
    fn read_bounded(&mut self, limit: &BigInt) -> Io<Option<Vec<u8>>> {
        let mut body: Vec<u8> = Vec::new();
        loop {
            let room = limit + BigInt::from(1) - BigInt::from(body.len());
            let amt = room.to_usize().map_or(65536, |room| room.min(65536));
            let chunk = self.read(amt)?;
            if chunk.is_empty() {
                return Ok(Some(body));
            }
            body.extend(chunk);
            if BigInt::from(body.len()) > *limit {
                return Ok(None);
            }
        }
    }
}

/// The adapter's `_read_next_chunk_size`: a complete line whose size, before
/// any extension, is hex digits alone.
fn next_chunk_size(fp: &mut impl BufRead) -> Io<u64> {
    let line = read_line(fp).map_err(drop)?;
    if line.len() > MAXLINE || !line.ends_with(b"\n") {
        return Err(());
    }
    let end = line.len()
        - line
            .iter()
            .rev()
            .take_while(|b| matches!(b, b'\r' | b'\n'))
            .count();
    let token = line[..end].split(|&b| b == b';').next().unwrap_or_default();
    if token.is_empty() || !token.iter().all(u8::is_ascii_hexdigit) {
        return Err(());
    }
    let digits = latin1(token);
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        return Ok(0);
    }
    // A size past u64 is never satisfied.
    Ok(u64::from_str_radix(digits, 16).unwrap_or(u64::MAX))
}

/// The adapter's `_read_and_discard_trailer`: up to 100 complete trailer
/// lines before the blank one.
fn discard_trailer(fp: &mut impl BufRead) -> Io<()> {
    for _ in 0..=MAXHEADERS {
        let line = read_line(fp).map_err(drop)?;
        if line.len() > MAXLINE || !line.ends_with(b"\n") {
            return Err(());
        }
        if matches!(line.as_slice(), b"\r\n" | b"\n") {
            return Ok(());
        }
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    /// A connection that replies with fixed bytes and records what it is sent.
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

    /// Each dial takes the next reply; one past them is refused.
    struct Replies {
        replies: Vec<&'static [u8]>,
        sent: Rc<RefCell<Vec<u8>>>,
    }

    impl Dial for Replies {
        fn dial(&mut self, _: &str, _: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
            if self.replies.is_empty() {
                return Err(io::ErrorKind::ConnectionRefused.into());
            }
            Ok(Box::new(Canned {
                reply: io::Cursor::new(self.replies.remove(0).to_vec()),
                sent: Rc::clone(&self.sent),
            }))
        }
    }

    fn run(
        method: &str,
        body: Option<&[u8]>,
        replies: Vec<&'static [u8]>,
    ) -> (R<RawResponse>, Vec<u8>) {
        let config = AdapterConfig {
            enabled: true,
            base_url: Some("http://127.0.0.1".into()),
            allow_loopback: true,
            ..AdapterConfig::default()
        };
        let sent = Rc::default();
        let mut dial = Replies {
            replies,
            sent: Rc::clone(&sent),
        };
        let call = Call {
            method,
            url: "http://127.0.0.1/v1/rates",
            body,
            content_type: None,
            retry_safe: true,
            bearer_token: None,
        };
        let result = request(&config, &mut dial, &call, &mut |_| Ok(()));
        (result, sent.take())
    }

    #[test]
    fn an_empty_method_is_urllibs_default() {
        let ok: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n";
        let (_, sent) = run("", None, vec![ok]);
        assert!(sent.starts_with(b"GET /v1/rates HTTP/1.1\r\n"));
        let (_, sent) = run("", Some(b"x"), vec![ok]);
        assert!(sent.starts_with(b"POST /v1/rates HTTP/1.1\r\n"));
    }

    #[test]
    fn a_header_section_cut_short_by_the_end_of_the_stream_fails() {
        // _HeaderReader refuses it; a 200 that failed to open is retried.
        let cut: &[u8] = b"HTTP/1.1 200 OK\r\nServer: stub\r\n";
        let (result, _) = run("GET", None, vec![cut, b"HTTP/1.1 200 OK\r\n\r\nok"]);
        assert_eq!(result.expect("the retry succeeds").body, b"ok");
        let (result, _) = run("GET", None, vec![cut, b"HTTP/1.1 503 X\r\nServer: stub"]);
        let text: String = result
            .unwrap_err()
            .text
            .into_iter()
            .filter_map(char::from_u32)
            .collect();
        assert_eq!(text, "HTTP 503 body could not be read after 2 attempt(s)");
    }

    #[test]
    fn a_length_past_ints_digit_limit_fails_to_open() {
        // int() takes 4,300 digits, leading zeros counted, and refuses 4,301;
        // the refusal is a 200 that failed to open, so it is retried.
        let reply = |digits: String| -> &'static [u8] {
            let text = format!("HTTP/1.1 200 OK\r\nContent-Length: {digits}\r\n\r\nab");
            Box::leak(text.into_bytes().into_boxed_slice())
        };
        let ok: &[u8] = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok";
        let at_limit = reply(format!("{}2", "0".repeat(4299)));
        assert_eq!(run("GET", None, vec![at_limit]).0.unwrap().body, b"ab");
        for digits in [format!("{}2", "0".repeat(4300)), "9".repeat(5000)] {
            let (result, _) = run("GET", None, vec![reply(digits), ok]);
            assert_eq!(result.expect("the retry succeeds").body, b"ok");
        }
    }

    /// Bytes, then the error rustls's reader returns for a TCP close
    /// without `close_notify`.
    struct Truncated(io::Cursor<Vec<u8>>);

    impl Read for Truncated {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            match self.0.read(buf)? {
                0 => Err(io::ErrorKind::UnexpectedEof.into()),
                n => Ok(n),
            }
        }
    }

    impl Write for Truncated {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    struct Truncating(&'static [u8]);

    impl Dial for Truncating {
        fn dial(&mut self, _: &str, _: u16, _: bool, _: f64) -> io::Result<Box<dyn Connection>> {
            Ok(Box::new(Truncated(io::Cursor::new(self.0.to_vec()))))
        }
    }

    #[test]
    fn a_tls_truncation_is_not_the_end_of_a_close_delimited_body() {
        // The adapter's suppress_ragged_eofs=False makes it a read failure;
        // `rustls::StreamOwned` reads `UnexpectedEof` for a close without
        // `close_notify`, in the status line, the headers or the body.
        let config = AdapterConfig {
            enabled: true,
            base_url: Some("http://127.0.0.1".into()),
            allow_loopback: true,
            max_attempts: 1.into(),
            ..AdapterConfig::default()
        };
        let call = Call {
            method: "GET",
            url: "http://127.0.0.1/v1/rates",
            body: None,
            content_type: None,
            retry_safe: true,
            bearer_token: None,
        };
        let cases: [(&'static [u8], &str); 3] = [
            (b"HTTP/1.1 2", "no response after 1 attempt(s)"),
            // The status read before the failure stays, as in Python.
            (
                b"HTTP/1.1 200 OK\r\nConn",
                "HTTP 200 body could not be read after 1 attempt(s)",
            ),
            (
                b"HTTP/1.1 200 OK\r\nConnection: close\r\n\r\npartial",
                "HTTP 200 body could not be read after 1 attempt(s)",
            ),
        ];
        for (reply, expected) in cases {
            let error =
                request(&config, &mut Truncating(reply), &call, &mut |_| Ok(())).unwrap_err();
            let text: String = error.text.into_iter().filter_map(char::from_u32).collect();
            assert_eq!(text, expected);
        }
    }

    #[test]
    fn a_backoff_time_sleep_refuses_stops_the_retries() {
        // time.sleep(nan) raises ValueError before the second attempt.
        let config = AdapterConfig {
            enabled: true,
            base_url: Some("http://127.0.0.1".into()),
            allow_loopback: true,
            retry_backoff_seconds: f64::NAN,
            ..AdapterConfig::default()
        };
        let sent = Rc::default();
        let unavailable: &[u8] = b"HTTP/1.1 503 X\r\nContent-Length: 0\r\n\r\n";
        let mut dial = Replies {
            replies: vec![unavailable, unavailable],
            sent: Rc::clone(&sent),
        };
        let call = Call {
            method: "GET",
            url: "http://127.0.0.1/v1/rates",
            body: None,
            content_type: None,
            retry_safe: true,
            bearer_token: None,
        };
        let error = request(&config, &mut dial, &call, &mut crate::dial::sleep).unwrap_err();
        assert_eq!(error.class, "ValueError");
        assert_eq!(dial.replies.len(), 1, "no second dial");
    }

    #[test]
    fn a_real_dial_reaches_loopback_and_an_overflowing_timeout_is_raised() {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let server = std::thread::spawn(move || {
            let (mut peer, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0u8; 1];
                peer.read_exact(&mut byte).unwrap();
                request.push(byte[0]);
            }
            peer.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nok")
                .unwrap();
            request
        });
        let url = format!("http://127.0.0.1:{port}/v1/rates");
        let call = |timeout: f64| {
            let config = AdapterConfig {
                enabled: true,
                base_url: Some(format!("http://127.0.0.1:{port}")),
                allow_loopback: true,
                read_timeout: timeout,
                ..AdapterConfig::default()
            };
            let call = Call {
                method: "GET",
                url: &url,
                body: None,
                content_type: None,
                retry_safe: true,
                bearer_token: None,
            };
            let mut sleeps = 0;
            let result = request(&config, &mut crate::dial::Net, &call, &mut |_| {
                sleeps += 1;
                Ok(())
            });
            (result, sleeps)
        };
        let (result, _) = call(5.0);
        assert_eq!(result.expect("the server answers").body, b"ok");
        assert!(
            server
                .join()
                .unwrap()
                .starts_with(b"GET /v1/rates HTTP/1.1\r\n")
        );
        // Nothing listens now, but the timeout is refused before the connect.
        let (result, sleeps) = call(f64::INFINITY);
        assert_eq!(result.unwrap_err().class, "OverflowError");
        assert_eq!(sleeps, 0, "an uncaught exception is not retried");
    }
}
