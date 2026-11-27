//! The parts of CPython 3.15's `http.client` that `transport` reuses: the
//! status line, header and line reading rules with their exception messages,
//! and the TLS client configuration `dial` connects with.
//!
//! TLS is rustls with the operating system's certificate verifier, where
//! Python uses OpenSSL with the system's root certificates, so a TLS failure
//! is worded differently.
use std::io::{self, BufRead, Read};
use std::sync::Arc;

/// The `str()` of the `OSError` or `HTTPException` the exchange raised.
pub type Failure = String;

/// `http.client`'s limits: line length and header lines.
const MAXLINE: usize = 65536;
const MAXHEADERS: usize = 100;

pub(crate) fn tls_config() -> Result<Arc<rustls::ClientConfig>, Failure> {
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
pub(crate) fn read_line<R: BufRead>(reader: &mut R) -> Result<Vec<u8>, Failure> {
    let mut line = Vec::new();
    reader
        .by_ref()
        .take(MAXLINE as u64 + 1)
        .read_until(b'\n', &mut line)
        .map_err(|e| os_error(&e))?;
    Ok(line)
}

pub(crate) fn latin1(bytes: &[u8]) -> String {
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

/// The version and status code from a status line already read.
pub(crate) fn status_line(raw: &[u8]) -> Result<(String, u16), Failure> {
    if raw.len() > MAXLINE {
        return Err("got more than 65536 bytes when reading status line".into());
    }
    let line = latin1(raw);
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

/// `_read_headers` over any line source.
pub(crate) fn header_lines(
    mut next: impl FnMut() -> Result<Vec<u8>, Failure>,
) -> Result<Vec<Vec<u8>>, Failure> {
    let mut lines = Vec::new();
    loop {
        let line = next()?;
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

/// `http.client.parse_headers`: the email package's compat32 parse of the
/// header lines, reduced to the name and value of each field.
pub(crate) fn parse_header_lines(raw: &[Vec<u8>]) -> Vec<(String, String)> {
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

#[cfg(test)]
mod tests {
    //! Expected values are CPython 3.15's: `HTTPResponse._read_status()` on
    //! the same bytes (a `BadStatusLine` shown as the `repr` of its `line`),
    //! and `email.parser.Parser(_class=HTTPMessage).parsestr(...).items()`.
    use super::*;

    fn lines(raw: &[&[u8]]) -> Vec<(String, String)> {
        parse_header_lines(&raw.iter().map(|line| line.to_vec()).collect::<Vec<_>>())
    }

    fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
        expected
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect()
    }

    #[test]
    fn a_status_line_is_read_as_python_reads_it() {
        assert_eq!(
            status_line(b"HTTP/1.1 200\r\n"),
            Ok(("HTTP/1.1".into(), 200)),
            "a status without a reason"
        );
        assert_eq!(
            status_line(b""),
            Err("Remote end closed connection without response".into())
        );
        assert_eq!(status_line(b"HTTP/1.1\r\n"), Err("'HTTP/1.1\\r\\n'".into()));
        assert_eq!(
            status_line(&vec![b'x'; 65537]),
            Err("got more than 65536 bytes when reading status line".into())
        );
    }

    #[test]
    fn a_header_line_past_the_limit_is_refused() {
        let mut given = vec![b"A: 1\r\n".to_vec(), vec![b'a'; 65537]].into_iter();
        assert_eq!(
            header_lines(|| Ok(given.next().unwrap_or_default())),
            Err("got more than 65536 bytes when reading header line".into())
        );
    }

    #[test]
    fn header_lines_may_end_in_a_lone_cr_or_lf_or_nothing() {
        assert_eq!(
            lines(&[b"A: 1\n", b"B: 2"]),
            pairs(&[("A", "1"), ("B", "2")])
        );
        assert_eq!(
            lines(&[b"A: 1\r", b"B: 2\r\n"]),
            pairs(&[("A", "1"), ("B", "2")])
        );
    }

    #[test]
    fn continuation_lines_join_the_field_before_them() {
        assert_eq!(
            lines(&[b"A: 1\r\n", b" more\r\n", b"\tand\r\n", b"B: 2\r\n"]),
            pairs(&[("A", "1\r\n more\r\n\tand"), ("B", "2")])
        );
        // One before any field has nothing to join and is dropped.
        assert_eq!(lines(&[b" lead\r\n", b"A: 1\r\n"]), pairs(&[("A", "1")]));
    }

    #[test]
    fn a_from_line_or_a_nameless_field_is_skipped_and_a_non_field_ends_them() {
        assert_eq!(
            lines(&[b"From someone\r\n", b"A: 1\r\n"]),
            pairs(&[("A", "1")])
        );
        assert_eq!(lines(&[b":empty\r\n", b"A: 1\r\n"]), pairs(&[("A", "1")]));
        assert_eq!(
            lines(&[b"A: 1\r\n", b"not a header\r\n", b"B: 2\r\n"]),
            pairs(&[("A", "1")])
        );
    }

    struct Failing(io::ErrorKind);

    impl Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(self.0, "bad record mac"))
        }
    }

    #[test]
    fn a_read_failure_is_worded_as_the_socket_error() {
        let read = |kind| read_line(&mut io::BufReader::new(Failing(kind)));
        assert_eq!(read(io::ErrorKind::TimedOut), Err("timed out".into()));
        assert_eq!(
            read(io::ErrorKind::UnexpectedEof),
            Err(
                "[SSL: UNEXPECTED_EOF_WHILE_READING] EOF occurred in violation of protocol (_ssl.c:1144)"
                    .into()
            )
        );
        // rustls's alert after the handshake, which Python words as OpenSSL's.
        assert_eq!(
            read(io::ErrorKind::InvalidData),
            Err("SSL: bad record mac".into())
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn another_socket_error_reads_as_pythons_oserror() {
        struct Reset;
        impl Read for Reset {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::from_raw_os_error(104))
            }
        }
        // str(ConnectionResetError(104, os.strerror(104))) on Linux.
        assert_eq!(
            read_line(&mut io::BufReader::new(Reset)),
            Err("[Errno 104] Connection reset by peer".into())
        );
    }
}
