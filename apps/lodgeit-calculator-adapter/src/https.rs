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
use std::net::{TcpStream, ToSocketAddrs};
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
    let mut stream = connect(host)?;
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

/// `socket.create_connection((host, 443), 60)`, then `wrap_socket` with the
/// host as the server name.
fn connect(host: &str) -> Result<Stream, Failure> {
    let addresses = (host, 443)
        .to_socket_addrs()
        .map_err(|e| format!("getaddrinfo failed: {e}"))?;
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
    let config = tls_config()?;
    let name = rustls::pki_types::ServerName::try_from(host)
        .map_err(|e| format!("SSL: {e}"))?
        .to_owned();
    let mut session =
        rustls::ClientConnection::new(config, name).map_err(|e| format!("SSL: {e}"))?;
    let deadline = Instant::now() + TIMEOUT;
    while session.is_handshaking() || session.wants_write() {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err("_ssl.c: The handshake operation timed out".into());
        }
        set_timeouts(&socket, left).map_err(|e| os_error(&e))?;
        if session.wants_write() {
            session.write_tls(&mut socket).map_err(|e| os_error(&e))?;
        } else if !session.wants_read() {
            return Err("SSL: the handshake stalled".into());
        } else if session.read_tls(&mut socket).map_err(|e| os_error(&e))? == 0 {
            return Err(
                "[SSL: UNEXPECTED_EOF_WHILE_READING] EOF occurred in violation of protocol".into(),
            );
        } else if let Err(e) = session.process_new_packets() {
            let _ = session.write_tls(&mut socket);
            return Err(format!("SSL: {e}"));
        }
    }
    set_timeouts(&socket, TIMEOUT).map_err(|e| os_error(&e))?;
    Ok(Stream(Box::new(rustls::StreamOwned::new(session, socket))))
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

/// `int(text)` of a str of Latin-1 text: surrounding whitespace allowed.
fn py_int(text: &str) -> Option<num_bigint::BigInt> {
    crate::py::parse_int(text.trim_matches(py_space), 10).ok()
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
    let bad = || crate::py::repr_str(&line);
    if !version.starts_with("HTTP/") {
        return Err(bad());
    }
    let status = py_int(&status)
        .and_then(|n| u16::try_from(&n).ok())
        .filter(|n| (100..=999).contains(n))
        .ok_or_else(bad)?;
    Ok((version, status))
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
        return Err(crate::py::repr_str(&version));
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
            return Err("chunk size".into());
        }
        let Some(size) = chunk_size(&line) else {
            return Err(incomplete(&value));
        };
        if size == 0 {
            let mut count = 0;
            loop {
                let line = read_line(reader)?;
                if line.len() > MAXLINE {
                    return Err("trailer line".into());
                }
                if matches!(line.as_slice(), b"" | b"\r\n" | b"\n") {
                    return Ok(value);
                }
                count += 1;
                if count > MAXHEADERS {
                    return Err("got more than 100 headers".into());
                }
            }
        }
        let chunk = read_upto(reader, size)?;
        value.extend_from_slice(&chunk);
        if (chunk.len() as u64) < size {
            return Err(incomplete(&value));
        }
        // The CRLF that ends a chunk.
        if read_upto(reader, 2)?.len() < 2 {
            return Err(incomplete(&value));
        }
    }
}

/// `int(line, 16)` of a chunk-size line without its extensions; a negative
/// size is refused.
fn chunk_size(line: &[u8]) -> Option<u64> {
    let digits = line.split(|&b| b == b';').next().unwrap_or_default();
    let text = std::str::from_utf8(digits).ok()?;
    if !text.is_ascii() || text.contains(['\x1c', '\x1d', '\x1e', '\x1f']) {
        return None;
    }
    let n = crate::py::parse_int(text, 16).ok()?;
    if n.sign() == num_bigint::Sign::Minus {
        return None;
    }
    Some(u64::try_from(&n).unwrap_or(u64::MAX))
}
