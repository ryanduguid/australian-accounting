//! `HTTPSConnection(host, timeout=60)`, `request("GET", path)`,
//! `getresponse()` and `read()`, as CPython 3.15's `http.client` runs them for
//! `rates ato-tables`: the request it writes, and `HTTPResponse`'s status
//! line, header, framing and read rules with their exception messages.
//!
//! TLS is rustls with the operating system's certificate verifier, where
//! Python uses OpenSSL with the system's root certificates, so a TLS failure
//! is worded differently; so are name-resolution failures, which Python
//! reports as `getaddrinfo`'s errors. The connect, each socket read and each
//! write may take 60 seconds, as Python's socket timeout allows.
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(crate) const TIMEOUT: Duration = Duration::from_secs(60);
/// `http.client`'s limits: line length, header lines, interim responses.
const MAXLINE: usize = 65536;
const MAXHEADERS: usize = 100;
const MAXINTERIM: usize = 100;

/// A response: its status and, for a 200, its body.
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: Vec<u8>,
}

/// The `str()` of the `OSError` or `HTTPException` the exchange raised.
pub type Failure = String;

/// GET `path` from `host` over HTTPS. The body is read only for a 200, as
/// the caller raises before reading any other.
pub fn get(host: &str, path: &str) -> Result<Response, Failure> {
    let addresses = (host, 443)
        .to_socket_addrs()
        .map_err(|e| format!("getaddrinfo failed: {e}"))?;
    exchange(connect(host, addresses, tls_config)?, host, path)
}

/// One GET over a connected stream, which is dropped, closing the
/// connection, however the exchange ends.
fn exchange<S: Read + Write>(mut stream: S, host: &str, path: &str) -> Result<Response, Failure> {
    let mut message = format!("GET {path} HTTP/1.1\r\n").into_bytes();
    message.extend_from_slice(format!("Host: {host}\r\n").as_bytes());
    message.extend_from_slice(b"Accept-Encoding: identity\r\n\r\n");
    stream
        .write_all(&message)
        .and_then(|()| stream.flush())
        .map_err(|e| os_error(&e))?;
    let mut wire = BufReader::with_capacity(1 << 16, stream);
    let head = begin(&mut wire)?;
    if head.status != 200 {
        return Ok(Response {
            status: head.status,
            body: Vec::new(),
        });
    }
    let body = if head.chunked {
        read_chunked(&mut wire)?
    } else {
        match head.length {
            None => read_upto(&mut wire, u64::MAX)?,
            Some(length) => safe_read(&mut wire, length)?,
        }
    };
    Ok(Response {
        status: head.status,
        body,
    })
}

/// `socket.create_connection((host, 443), 60)` over the resolved addresses,
/// then `wrap_socket` with the host as the server name.
fn connect(
    host: &str,
    addresses: impl IntoIterator<Item = SocketAddr>,
    config: fn() -> Result<Arc<rustls::ClientConfig>, Failure>,
) -> Result<Stream, Failure> {
    let mut last = None;
    let mut socket = None;
    for address in addresses {
        match TcpStream::connect_timeout(&address, TIMEOUT) {
            Ok(s) => {
                socket = Some(s);
                break;
            }
            Err(e) => last = Some(e),
        }
    }
    let mut socket = match (socket, last) {
        (Some(socket), _) => socket,
        (None, Some(error)) => return Err(os_error(&error)),
        (None, None) => return Err("getaddrinfo returns an empty list".into()),
    };
    let _ = socket.set_nodelay(true);
    set_timeouts(&socket, TIMEOUT).map_err(|e| os_error(&e))?;
    let config = config()?;
    let name = rustls::pki_types::ServerName::try_from(host)
        .map_err(|e| format!("SSL: {e}"))?
        .to_owned();
    let mut session =
        rustls::ClientConnection::new(config, name).map_err(|e| format!("SSL: {e}"))?;
    handshake(&mut session, &mut socket, Instant::now() + TIMEOUT)?;
    set_timeouts(&socket, TIMEOUT).map_err(|e| os_error(&e))?;
    Ok(Stream(Box::new(rustls::StreamOwned::new(session, socket))))
}

/// `do_handshake()` under one deadline for the whole handshake, as CPython's
/// `_ssl` runs it with a socket timeout.
///
/// Whilst rustls is handshaking it can receive no application data and
/// ignores no close_notify, so with nothing to write it always wants to read.
fn handshake(
    session: &mut rustls::ClientConnection,
    socket: &mut TcpStream,
    deadline: Instant,
) -> Result<(), Failure> {
    while session.is_handshaking() || session.wants_write() {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err("_ssl.c: The handshake operation timed out".into());
        }
        set_timeouts(socket, left).map_err(|e| os_error(&e))?;
        if session.wants_write() {
            session.write_tls(socket).map_err(|e| os_error(&e))?;
        } else if session.read_tls(socket).map_err(|e| os_error(&e))? == 0 {
            return Err(
                "[SSL: UNEXPECTED_EOF_WHILE_READING] EOF occurred in violation of protocol".into(),
            );
        } else if let Err(e) = session.process_new_packets() {
            let _ = session.write_tls(socket);
            return Err(format!("SSL: {e}"));
        }
    }
    Ok(())
}

fn tls_config() -> Result<Arc<rustls::ClientConfig>, Failure> {
    use rustls_platform_verifier::BuilderVerifierExt;
    let provider = Arc::new(rustls::crypto::ring::default_provider());
    let mut config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .and_then(BuilderVerifierExt::with_platform_verifier)
        .map_err(|e| format!("SSL: {e}"))?
        .with_no_client_auth();
    // _create_https_context offers HTTP/1.1 by ALPN.
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(Arc::new(config))
}

fn set_timeouts(socket: &TcpStream, timeout: Duration) -> io::Result<()> {
    socket.set_read_timeout(Some(timeout))?;
    socket.set_write_timeout(Some(timeout))
}

/// TLS over TCP. A TCP close without TLS's close_notify is an error, as
/// OpenSSL 3 reports it to Python's `ssl` (`suppress_ragged_eofs` does not
/// cover it).
struct Stream(Box<rustls::StreamOwned<rustls::ClientConnection, TcpStream>>);

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

/// `str()` of the `OSError` a socket call raised.
fn os_error(error: &io::Error) -> Failure {
    match error.kind() {
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock => "timed out".into(),
        io::ErrorKind::UnexpectedEof => {
            "[SSL: UNEXPECTED_EOF_WHILE_READING] EOF occurred in violation of protocol (_ssl.c:1144)"
                .into()
        }
        // rustls reports a TLS failure after the handshake this way.
        io::ErrorKind::InvalidData => format!("SSL: {error}"),
        _ => crate::pyos::OsError::new(error, None, crate::pyos::Style::Native).text,
    }
}

/// `fp.readline(_MAXLINE + 1)`.
fn read_line<R: BufRead>(reader: &mut R) -> Result<Vec<u8>, Failure> {
    let mut line = Vec::new();
    reader
        .by_ref()
        .take(MAXLINE as u64 + 1)
        .read_until(b'\n', &mut line)
        .map_err(|e| os_error(&e))?;
    Ok(line)
}

/// `fp.read(n)`: `n` bytes or fewer at the end of the stream.
fn read_upto<R: BufRead>(reader: &mut R, n: u64) -> Result<Vec<u8>, Failure> {
    let mut data = Vec::new();
    reader
        .by_ref()
        .take(n)
        .read_to_end(&mut data)
        .map_err(|e| os_error(&e))?;
    Ok(data)
}

/// `_safe_read(amt)`: exactly `amt` bytes or `IncompleteRead`.
fn safe_read<R: BufRead>(reader: &mut R, amt: u64) -> Result<Vec<u8>, Failure> {
    let data = read_upto(reader, amt)?;
    if (data.len() as u64) < amt {
        return Err(format!(
            "IncompleteRead({} bytes read, {} more expected)",
            data.len(),
            amt - data.len() as u64
        ));
    }
    Ok(data)
}

fn latin1(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| char::from(b)).collect()
}

/// `str.isspace()` for Latin-1 text.
fn py_space(c: char) -> bool {
    matches!(c, '\t'..='\r' | '\x1c'..='\x1f' | ' ' | '\u{85}' | '\u{a0}')
}

/// `str.split(None, maxsplit)`.
fn split_ws(s: &str, maxsplit: usize) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut parts = Vec::new();
    let mut i = 0;
    while parts.len() < maxsplit {
        while i < chars.len() && py_space(chars[i]) {
            i += 1;
        }
        if i == chars.len() {
            break;
        }
        let start = i;
        while i < chars.len() && !py_space(chars[i]) {
            i += 1;
        }
        parts.push(chars[start..i].iter().collect());
    }
    while i < chars.len() && py_space(chars[i]) {
        i += 1;
    }
    if i < chars.len() {
        parts.push(chars[i..].iter().collect());
    }
    parts
}

/// `int(text)`, which strips whitespace but, unlike `str.split`, not
/// `\x1c` to `\x1f`.
fn py_int(text: &str) -> Option<num_bigint::BigInt> {
    crate::py::parse_int(text, 10).ok()
}

/// `HTTPResponse._read_status()`: the version and status code.
fn read_status<R: BufRead>(reader: &mut R) -> Result<(String, u16), Failure> {
    let raw = read_line(reader)?;
    if raw.len() > MAXLINE {
        return Err("got more than 65536 bytes when reading status line".into());
    }
    let line = latin1(&raw);
    if line.is_empty() {
        return Err("Remote end closed connection without response".into());
    }
    let (version, status) = match split_ws(&line, 2).as_slice() {
        [version, status, _] => (version.clone(), status.clone()),
        _ => match split_ws(&line, 1).as_slice() {
            [version, status] => (version.clone(), status.clone()),
            _ => (String::new(), String::new()),
        },
    };
    // BadStatusLine(line), whose str() is the line itself.
    if !version.starts_with("HTTP/") {
        return Err(line);
    }
    match py_int(&status)
        .and_then(|n| u16::try_from(&n).ok())
        .filter(|n| (100..=999).contains(n))
    {
        Some(status) => Ok((version, status)),
        None => Err(line),
    }
}

/// `_read_headers(fp)`: the raw lines up to the blank line or the end.
fn read_headers<R: BufRead>(reader: &mut R) -> Result<Vec<Vec<u8>>, Failure> {
    let mut lines = Vec::new();
    loop {
        let line = read_line(reader)?;
        if line.len() > MAXLINE {
            return Err("got more than 65536 bytes when reading header line".into());
        }
        if matches!(line.as_slice(), b"\r\n" | b"\n" | b"") {
            return Ok(lines);
        }
        lines.push(line);
        if lines.len() > MAXHEADERS {
            return Err("got more than 100 headers".into());
        }
    }
}

struct Head {
    status: u16,
    chunked: bool,
    length: Option<u64>,
}

/// `HTTPResponse.begin()`: skip `100 Continue`s, check the version, read the
/// headers and settle the body's framing.
fn begin<R: BufRead>(reader: &mut R) -> Result<Head, Failure> {
    let mut found = None;
    for _ in 0..MAXINTERIM {
        let (version, status) = read_status(reader)?;
        if status != 100 {
            found = Some((version, status));
            break;
        }
        read_headers(reader)?;
    }
    let (version, status) = found.ok_or("got more than 100 interim responses")?;
    if !(version == "HTTP/1.0" || version == "HTTP/0.9" || version.starts_with("HTTP/1.")) {
        // UnknownProtocol(version), whose str() is the version.
        return Err(version);
    }
    let headers = parse_header_lines(&read_headers(reader)?);
    let header = |name: &str| {
        headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    };
    let chunked = header("transfer-encoding").is_some_and(|v| v.eq_ignore_ascii_case("chunked"));
    let mut length = match header("content-length") {
        // int(length); an invalid or negative length is ignored.
        Some(text) if !text.is_empty() && !chunked => py_int(text)
            .filter(|n| n.sign() != num_bigint::Sign::Minus)
            .map(|n| u64::try_from(&n).unwrap_or(u64::MAX)),
        _ => None,
    };
    if matches!(status, 204 | 304) || (100..200).contains(&status) {
        length = Some(0);
    }
    Ok(Head {
        status,
        chunked,
        length,
    })
}

/// `http.client.parse_headers`: the email package's compat32 parse of the
/// header lines, reduced to the name and value of each field.
fn parse_header_lines(raw: &[Vec<u8>]) -> Vec<(String, String)> {
    let text = latin1(&raw.concat());
    let mut lines = Vec::new();
    let mut rest = text.as_str();
    while !rest.is_empty() {
        let end = match rest.find(['\r', '\n']) {
            Some(i) if rest[i..].starts_with("\r\n") => i + 2,
            Some(i) => i + 1,
            None => rest.len(),
        };
        lines.push(&rest[..end]);
        rest = &rest[end..];
    }
    let is_field = |line: &str| {
        if line.starts_with("From ") || line.starts_with([' ', '\t']) {
            return true;
        }
        let end = line
            .find(|c: char| !matches!(c, '\x21'..='\x39' | '\x3b'..='\x7e'))
            .unwrap_or(line.len());
        line[end..].starts_with(':')
    };
    let mut headers = Vec::new();
    let mut pending: Vec<&str> = Vec::new();
    let finish = |pending: &[&str]| {
        let (name, first) = pending[0].split_once(':').unwrap_or((pending[0], ""));
        let value: String = [first].iter().chain(&pending[1..]).copied().collect();
        let value = value
            .trim_start_matches([' ', '\t', '\r', '\n'])
            .trim_end_matches(['\r', '\n']);
        (name.to_owned(), value.to_owned())
    };
    for line in lines.into_iter().take_while(|line| is_field(line)) {
        if line.starts_with([' ', '\t']) {
            if !pending.is_empty() {
                pending.push(line);
            }
            continue;
        }
        if !pending.is_empty() {
            headers.push(finish(&pending));
            pending.clear();
        }
        if line.starts_with("From ") || line.starts_with(':') {
            continue;
        }
        pending.push(line);
    }
    if !pending.is_empty() {
        headers.push(finish(&pending));
    }
    headers
}

/// `_read_chunked(None)`.
fn read_chunked<R: BufRead>(reader: &mut R) -> Result<Vec<u8>, Failure> {
    let mut value = Vec::new();
    let incomplete = |value: &[u8]| format!("IncompleteRead({} bytes read)", value.len());
    loop {
        let line = read_line(reader)?;
        if line.len() > MAXLINE {
            return Err("got more than 65536 bytes when reading chunk size".into());
        }
        let Some(size) = chunk_size(&line) else {
            return Err(incomplete(&value));
        };
        if size.sign() == num_bigint::Sign::Minus {
            // _safe_read(negative) reads to the end, so the chunk's CRLF is missing.
            value.extend(read_upto(reader, u64::MAX)?);
            return Err(incomplete(&value));
        }
        let size = u64::try_from(&size).unwrap_or(u64::MAX);
        if size == 0 {
            let mut count = 0;
            loop {
                let line = read_line(reader)?;
                if line.len() > MAXLINE {
                    return Err("got more than 65536 bytes when reading trailer line".into());
                }
                if matches!(line.as_slice(), b"" | b"\r\n" | b"\n") {
                    return Ok(value);
                }
                count += 1;
                if count > MAXHEADERS {
                    return Err("got more than 100 trailers".into());
                }
            }
        }
        let chunk = read_upto(reader, size)?;
        // A short chunk is not kept: _safe_read raises before it is appended.
        if (chunk.len() as u64) < size {
            return Err(incomplete(&value));
        }
        value.extend_from_slice(&chunk);
        // The CRLF that ends a chunk.
        if read_upto(reader, 2)?.len() < 2 {
            return Err(incomplete(&value));
        }
    }
}

/// `int(line, 16)` of a chunk-size line without its extensions.
fn chunk_size(line: &[u8]) -> Option<num_bigint::BigInt> {
    let digits = line.split(|&b| b == b';').next().unwrap_or_default();
    let text = std::str::from_utf8(digits).ok()?;
    if !text.is_ascii() || text.contains(['\x1c', '\x1d', '\x1e', '\x1f']) {
        return None;
    }
    crate::py::parse_int(text, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    /// A connection with a canned reply that records what it is sent and
    /// whether it was closed (dropped).
    struct Fake {
        reply: io::Cursor<Vec<u8>>,
        sent: Rc<RefCell<Vec<u8>>>,
        closed: Rc<Cell<bool>>,
        refuse_writes: bool,
    }

    impl Read for Fake {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.reply.read(buf)
        }
    }

    impl Write for Fake {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            if self.refuse_writes {
                return Err(io::ErrorKind::BrokenPipe.into());
            }
            self.sent.borrow_mut().extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl Drop for Fake {
        fn drop(&mut self) {
            self.closed.set(true);
        }
    }

    /// The exchange's result, the bytes sent and whether the stream was closed.
    fn run(reply: &[u8], refuse_writes: bool) -> (Result<Response, Failure>, Vec<u8>, bool) {
        let (sent, closed) = (Rc::default(), Rc::default());
        let fake = Fake {
            reply: io::Cursor::new(reply.to_vec()),
            sent: Rc::clone(&sent),
            closed: Rc::clone(&closed),
            refuse_writes,
        };
        let result = exchange(fake, "ato.example", "/tables/fbt.json");
        (result, sent.take(), closed.get())
    }

    #[test]
    fn a_table_is_one_get_and_the_connection_closes() {
        let (result, sent, closed) = run(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}", false);
        assert_eq!(result.expect("a 200").body, b"{}");
        assert_eq!(
            sent,
            b"GET /tables/fbt.json HTTP/1.1\r\nHost: ato.example\r\nAccept-Encoding: identity\r\n\r\n"
        );
        assert!(closed);
    }

    #[test]
    fn a_redirect_or_error_status_is_not_read_and_the_connection_closes() {
        // The body is malformed chunking: reading it would fail.
        for status in ["301 Moved", "404 Not Found", "500 Error"] {
            let reply = format!(
                "HTTP/1.1 {status}\r\nTransfer-Encoding: chunked\r\nLocation: /x\r\n\r\nzz\r\n"
            );
            let (result, _, closed) = run(reply.as_bytes(), false);
            let response = result.expect("a status");
            assert!(response.status != 200 && response.body.is_empty());
            assert!(closed);
        }
    }

    #[test]
    fn a_failed_request_closes_the_connection() {
        for (reply, refuse) in [
            (
                &b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n5\r\nab"[..],
                false,
            ),
            (b"", true),
            (b"", false),
        ] {
            let (result, _, closed) = run(reply, refuse);
            assert!(result.is_err());
            assert!(closed);
        }
        // Invalid JSON is the caller's to refuse, after the connection has closed.
        let (result, _, closed) = run(b"HTTP/1.1 200 OK\r\n\r\n{,", false);
        assert_eq!(result.expect("a 200").body, b"{,");
        assert!(closed);
    }

    #[test]
    fn a_content_length_int_refuses_is_ignored() {
        // int("2\x1c") raises, so the body runs to the end of the stream.
        let (result, _, _) = run(
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\x1c\r\n\r\n{}JUNK",
            false,
        );
        assert_eq!(result.expect("a 200").body, b"{}JUNK");
        // Headers are Latin-1, so 0x85 is NEL, which int() strips.
        for &space in b"\x0b\x85 " {
            let reply = [
                &b"HTTP/1.1 200 OK\r\nContent-Length: 2"[..],
                &[space],
                b"\r\n\r\n{}JUNK",
            ]
            .concat();
            let (result, _, _) = run(&reply, false);
            assert_eq!(result.expect("a 200").body, b"{}");
        }
    }

    /// The exchange's answer in the form `tests/data/cov_b_https.py` prints.
    fn answer(reply: &[u8]) -> String {
        match run(reply, false).0 {
            Ok(r) if r.status == 200 => format!("body {}", latin1(&r.body)),
            Ok(r) => format!("status {}", r.status),
            Err(e) => format!("error {e}"),
        }
    }

    #[test]
    fn replies_are_read_or_refused_as_cpython_does() {
        let ok = b"HTTP/1.1 200 OK\r\n".to_vec();
        let chunked = [&ok[..], b"Transfer-Encoding: chunked\r\n\r\n"].concat();
        let long = |head: &[u8]| [head, &[b'y'; 65537][..]].concat();
        let with = |head: &[u8], tail: &[u8]| [head, tail].concat();
        // Each answer is what tests/data/cov_b_https.py printed under CPython 3.15.0.
        let cases: [(&str, Vec<u8>, &str); 29] = [
            (
                "short_body",
                with(&ok, b"Content-Length: 5\r\n\r\nab"),
                "error IncompleteRead(2 bytes read, 3 more expected)",
            ),
            ("blank_status", b"\r\n".to_vec(), "error \r\n"),
            (
                "version_only",
                b"HTTP/1.1\r\n\r\n".to_vec(),
                "error HTTP/1.1\r\n",
            ),
            (
                "no_reason",
                b"HTTP/1.1 200\r\nContent-Length: 2\r\n\r\n{}".to_vec(),
                "body {}",
            ),
            (
                "long_status",
                long(b"HTTP/1.1 200 "),
                "error got more than 65536 bytes when reading status line",
            ),
            (
                "not_http",
                b"ICY 200 OK\r\n\r\n".to_vec(),
                "error ICY 200 OK\r\n",
            ),
            (
                "long_header",
                long(&with(&ok, b"X: ")),
                "error got more than 65536 bytes when reading header line",
            ),
            (
                "many_headers",
                with(&with(&ok, &b"X: y\r\n".repeat(101)), b"\r\n{}"),
                "error got more than 100 headers",
            ),
            (
                "continue",
                with(
                    &with(b"HTTP/1.1 100 Continue\r\nX: y\r\n\r\n", &ok),
                    b"Content-Length: 2\r\n\r\n{}",
                ),
                "body {}",
            ),
            (
                "endless_continue",
                b"HTTP/1.1 100 Continue\r\n\r\n".repeat(101),
                "error got more than 100 interim responses",
            ),
            (
                "http2",
                b"HTTP/2.0 200 OK\r\n\r\n".to_vec(),
                "error HTTP/2.0",
            ),
            (
                "no_content",
                b"HTTP/1.1 204 No Content\r\nContent-Length: 9\r\n\r\n".to_vec(),
                "status 204",
            ),
            (
                "bare_lf",
                b"HTTP/1.1 200 OK\nContent-Length: 2\n\n{}JUNK".to_vec(),
                "body {}",
            ),
            (
                "lone_cr",
                with(&ok, b"Content-Length: 2\rX: y\r\n\r\n{}JUNK"),
                "body {}",
            ),
            (
                "header_at_eof",
                with(&ok, b"Content-Length: 2"),
                "error IncompleteRead(0 bytes read, 2 more expected)",
            ),
            (
                "folded",
                with(&ok, b"Content-Length:\r\n 2\r\n\r\n{}JUNK"),
                "body {}",
            ),
            (
                "leading_fold",
                with(&ok, b" X: y\r\nContent-Length: 2\r\n\r\n{}JUNK"),
                "body {}",
            ),
            (
                "from_line",
                with(&ok, b"From nobody\r\n:x\r\nContent-Length: 2\r\n\r\n{}JUNK"),
                "body {}",
            ),
            (
                "chunks",
                with(&chunked, b"2\r\n{}\r\n1;ext=1\r\n \r\n0\r\nX: y\r\n\r\n"),
                "body {} ",
            ),
            (
                "long_chunk_size",
                [&chunked[..], &[b'1'; 65537][..]].concat(),
                "error got more than 65536 bytes when reading chunk size",
            ),
            (
                "bad_chunk_size",
                with(&chunked, b"2\r\n{}\r\nzz\r\n"),
                "error IncompleteRead(2 bytes read)",
            ),
            (
                "non_ascii_chunk_size",
                with(&chunked, b"\xe9\r\n"),
                "error IncompleteRead(0 bytes read)",
            ),
            (
                "separator_chunk_size",
                with(&chunked, b"2\x1c\r\n{}\r\n0\r\n\r\n"),
                "error IncompleteRead(0 bytes read)",
            ),
            (
                "negative_chunk_size",
                with(&chunked, b"-2\r\n{}\r\n0\r\n\r\n"),
                "error IncompleteRead(9 bytes read)",
            ),
            (
                "short_chunk_end",
                with(&chunked, b"2\r\n{}\r"),
                "error IncompleteRead(2 bytes read)",
            ),
            (
                "short_chunk",
                with(&chunked, b"2\r\n{}\r\n5\r\nab"),
                "error IncompleteRead(2 bytes read)",
            ),
            (
                "huge_chunk",
                with(&chunked, &with(&b"f".repeat(20), b"\r\nab")),
                "error IncompleteRead(0 bytes read)",
            ),
            (
                "long_trailer",
                long(&with(&chunked, b"0\r\nX: ")),
                "error got more than 65536 bytes when reading trailer line",
            ),
            (
                "many_trailers",
                with(&with(&chunked, b"0\r\n"), &b"X: y\r\n".repeat(101)),
                "error got more than 100 trailers",
            ),
        ];
        for (name, reply, expected) in cases {
            assert_eq!(answer(&reply), expected, "{name}");
        }
    }

    #[test]
    fn socket_errors_read_as_python_words_them() {
        let text = |kind: io::ErrorKind| os_error(&io::Error::new(kind, "x"));
        assert_eq!(text(io::ErrorKind::TimedOut), "timed out");
        assert_eq!(text(io::ErrorKind::WouldBlock), "timed out");
        assert_eq!(
            text(io::ErrorKind::UnexpectedEof),
            "[SSL: UNEXPECTED_EOF_WHILE_READING] EOF occurred in violation of protocol (_ssl.c:1144)"
        );
        assert_eq!(text(io::ErrorKind::InvalidData), "SSL: x");
    }

    #[test]
    fn the_tls_config_verifies_with_the_platform_and_offers_http_1_1() {
        let config = tls_config().expect("the platform verifier");
        assert_eq!(config.alpn_protocols, [b"http/1.1".to_vec()]);
    }

    use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
    use rustls::crypto::{CryptoProvider, verify_tls13_signature_with_raw_key};
    use rustls::pki_types::{
        CertificateDer, PrivatePkcs8KeyDer, ServerName, SubjectPublicKeyInfoDer, UnixTime,
    };
    use rustls::{DigitallySignedStruct, SignatureScheme};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;

    fn provider() -> Arc<CryptoProvider> {
        Arc::new(rustls::crypto::ring::default_provider())
    }

    /// The loopback server's Ed25519 key (PKCS#8 v1 around a fixed test seed).
    fn server_key() -> Arc<dyn rustls::sign::SigningKey> {
        let mut der = vec![
            0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22,
            0x04, 0x20,
        ];
        der.extend([7u8; 32]);
        rustls::crypto::ring::sign::any_eddsa_type(&PrivatePkcs8KeyDer::from(der))
            .expect("an Ed25519 key")
    }

    /// Trusts the test server's raw public key (RFC 7250) and checks its
    /// handshake signature, in place of the platform's certificate store.
    #[derive(Debug)]
    struct TestKey(Vec<u8>);

    impl ServerCertVerifier for TestKey {
        fn verify_server_cert(
            &self,
            end_entity: &CertificateDer<'_>,
            _: &[CertificateDer<'_>],
            _: &ServerName<'_>,
            _: &[u8],
            _: UnixTime,
        ) -> Result<ServerCertVerified, rustls::Error> {
            assert_eq!(end_entity.as_ref(), self.0.as_slice());
            Ok(ServerCertVerified::assertion())
        }
        fn verify_tls12_signature(
            &self,
            _: &[u8],
            _: &CertificateDer<'_>,
            _: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            Err(rustls::Error::General("TLS 1.3 only".into()))
        }
        fn verify_tls13_signature(
            &self,
            message: &[u8],
            key: &CertificateDer<'_>,
            dss: &DigitallySignedStruct,
        ) -> Result<HandshakeSignatureValid, rustls::Error> {
            let spki = SubjectPublicKeyInfoDer::from(key.as_ref());
            verify_tls13_signature_with_raw_key(
                message,
                &spki,
                dss,
                &provider().signature_verification_algorithms,
            )
        }
        fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
            vec![SignatureScheme::ED25519]
        }
        fn requires_raw_public_keys(&self) -> bool {
            true
        }
    }

    fn spki() -> Vec<u8> {
        server_key()
            .public_key()
            .expect("a public key")
            .as_ref()
            .to_vec()
    }

    fn client_config() -> Result<Arc<rustls::ClientConfig>, Failure> {
        let config = rustls::ClientConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS13])
            .expect("TLS 1.3")
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(TestKey(spki())))
            .with_no_client_auth();
        Ok(Arc::new(config))
    }

    /// A loopback listener whose one connection `serve` handles on its own thread.
    fn listen(serve: impl FnOnce(TcpStream) + Send + 'static) -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
        let address = listener.local_addr().expect("its address");
        thread::spawn(move || serve(listener.accept().expect("a connection").0));
        address
    }

    /// Read until the peer closes, so closing this end sends no reset.
    fn drain(mut socket: TcpStream) {
        let _ = io::copy(&mut socket, &mut io::sink());
    }

    #[test]
    fn a_table_is_fetched_over_tls_from_a_loopback_server() {
        let (tx, rx) = mpsc::channel();
        let address = listen(move |socket| {
            let key =
                rustls::sign::CertifiedKey::new(vec![CertificateDer::from(spki())], server_key());
            let config = rustls::ServerConfig::builder_with_provider(provider())
                .with_protocol_versions(&[&rustls::version::TLS13])
                .expect("TLS 1.3")
                .with_no_client_auth()
                .with_cert_resolver(Arc::new(
                    rustls::server::AlwaysResolvesServerRawPublicKeys::new(Arc::new(key)),
                ));
            let session = rustls::ServerConnection::new(Arc::new(config)).expect("a session");
            let mut tls = rustls::StreamOwned::new(session, socket);
            let mut request = Vec::new();
            while !request.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                tls.read_exact(&mut byte).expect("the request");
                request.push(byte[0]);
            }
            tls.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}")
                .expect("the reply");
            tls.conn.send_close_notify();
            tls.flush().expect("the close");
            tx.send(request).expect("the request back");
            drain(tls.sock);
        });
        let stream = connect("localhost", [address], client_config).expect("a TLS connection");
        let response = exchange(stream, "localhost", "/t.json").expect("a response");
        assert_eq!((response.status, response.body), (200, b"{}".to_vec()));
        assert_eq!(
            rx.recv().expect("the request"),
            b"GET /t.json HTTP/1.1\r\nHost: localhost\r\nAccept-Encoding: identity\r\n\r\n"
        );
    }

    #[test]
    fn a_refused_connection_reads_as_python_words_it() {
        let address = TcpListener::bind("127.0.0.1:0")
            .and_then(|l| l.local_addr())
            .expect("a free port");
        // What str(ConnectionRefusedError) is under CPython 3.15 on each platform.
        let refused = if cfg!(windows) {
            "[WinError 10061] No connection could be made because the target machine actively refused it"
        } else if cfg!(target_os = "macos") {
            "[Errno 61] Connection refused"
        } else {
            "[Errno 111] Connection refused"
        };
        assert_eq!(
            connect("localhost", [address], client_config)
                .err()
                .as_deref(),
            Some(refused)
        );
        // get() connects to port 443, on which nothing listens on loopback here.
        assert_eq!(get("127.0.0.1", "/").err().as_deref(), Some(refused));
    }

    #[test]
    fn no_address_is_refused_as_create_connection_refuses_it() {
        assert_eq!(
            connect("localhost", [], client_config).err().as_deref(),
            Some("getaddrinfo returns an empty list")
        );
    }

    #[test]
    fn a_server_that_closes_during_the_handshake_is_an_eof_error() {
        let address = listen(|mut socket| {
            let mut hello = [0; 1 << 16];
            let _ = socket.read(&mut hello);
            let _ = socket.shutdown(std::net::Shutdown::Write);
            drain(socket);
        });
        assert_eq!(
            connect("localhost", [address], client_config)
                .err()
                .as_deref(),
            Some("[SSL: UNEXPECTED_EOF_WHILE_READING] EOF occurred in violation of protocol")
        );
    }

    #[test]
    fn a_server_that_answers_in_plain_http_is_a_tls_error() {
        let address = listen(|mut socket| {
            let mut hello = [0; 1 << 16];
            let _ = socket.read(&mut hello);
            let _ = socket.write_all(b"HTTP/1.1 400 Bad Request\r\n\r\n");
            drain(socket);
        });
        assert_eq!(
            connect("localhost", [address], client_config)
                .err()
                .as_deref(),
            Some("SSL: received corrupt message of type InvalidContentType")
        );
    }

    #[test]
    fn a_handshake_past_its_deadline_times_out() {
        let address = listen(drain);
        let mut socket = TcpStream::connect(address).expect("a connection");
        let name = ServerName::try_from("localhost").expect("a name");
        let mut session = rustls::ClientConnection::new(client_config().expect("a config"), name)
            .expect("a session");
        assert_eq!(
            handshake(&mut session, &mut socket, Instant::now()),
            Err("_ssl.c: The handshake operation timed out".to_string())
        );
    }
}
