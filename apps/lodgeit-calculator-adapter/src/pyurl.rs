//! `urllib.parse.urlsplit` and `urljoin` of CPython 3.15 for `str` URLs, with `.hostname`.
//!
//! The checks use the scheme, netloc, path and hostname; the `ValueError`s
//! urlsplit raises for malformed brackets or NFKC tricks are returned as
//! errors, which the callers report as Python reported the traceback.
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, PartialEq)]
pub struct Split {
    pub scheme: String,
    pub netloc: String,
    pub path: String,
    pub query: String,
    pub fragment: String,
}

fn c0_or_space(c: char) -> bool {
    (c as u32) <= 0x20
}

/// `_urlsplit`: components absent from the URL are `None`, as CPython 3.15 keeps them.
struct Parts {
    scheme: Option<String>,
    netloc: Option<String>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

fn split_parts(url: &str) -> Result<Parts, String> {
    let mut url: String = url
        .trim_start_matches(c0_or_space)
        .chars()
        .filter(|c| !matches!(c, '\t' | '\r' | '\n'))
        .collect();
    let mut scheme = None;
    if let Some(i) = url.find(':')
        && i > 0
        && url.as_bytes()[0].is_ascii_alphabetic()
        && url[..i]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
    {
        scheme = Some(url[..i].to_ascii_lowercase());
        url = url[i + 1..].to_owned();
    }
    let mut netloc = None;
    if url.starts_with("//") {
        let rest = &url[2..];
        let end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
        let found = rest[..end].to_owned();
        url = rest[end..].to_owned();
        if found.contains('[') != found.contains(']') {
            return Err("Invalid IPv6 URL".into());
        }
        if found.contains('[') && found.contains(']') {
            check_bracketed_netloc(&found)?;
        }
        netloc = Some(found);
    }
    let mut fragment = None;
    if let Some((before, after)) = url.split_once('#') {
        fragment = Some(after.to_owned());
        url = before.to_owned();
    }
    let mut query = None;
    if let Some((before, after)) = url.split_once('?') {
        query = Some(after.to_owned());
        url = before.to_owned();
    }
    check_netloc(netloc.as_deref().unwrap_or(""))?;
    Ok(Parts {
        scheme,
        netloc,
        path: url,
        query,
        fragment,
    })
}

pub fn urlsplit(url: &str) -> Result<Split, String> {
    let parts = split_parts(url)?;
    Ok(Split {
        scheme: parts.scheme.unwrap_or_default(),
        netloc: parts.netloc.unwrap_or_default(),
        path: parts.path,
        query: parts.query.unwrap_or_default(),
        fragment: parts.fragment.unwrap_or_default(),
    })
}

const USES_RELATIVE: [&str; 20] = [
    "", "ftp", "http", "gopher", "nntp", "imap", "wais", "file", "https", "shttp", "mms",
    "prospero", "rtsp", "rtsps", "rtspu", "sftp", "svn", "svn+ssh", "ws", "wss",
];

const USES_NETLOC: [&str; 27] = [
    "",
    "ftp",
    "http",
    "gopher",
    "nntp",
    "telnet",
    "imap",
    "wais",
    "file",
    "mms",
    "https",
    "shttp",
    "snews",
    "prospero",
    "rtsp",
    "rtsps",
    "rtspu",
    "rsync",
    "svn",
    "svn+ssh",
    "sftp",
    "nfs",
    "git",
    "git+ssh",
    "ws",
    "wss",
    "itms-services",
];

fn unsplit(
    scheme: Option<&str>,
    netloc: Option<&str>,
    path: &str,
    query: Option<&str>,
    fragment: Option<&str>,
) -> String {
    let mut url = path.to_owned();
    if let Some(netloc) = netloc {
        if !url.is_empty() && !url.starts_with('/') {
            url.insert(0, '/');
        }
        url = format!("//{netloc}{url}");
    } else if url.starts_with("//") {
        url = format!("//{url}");
    }
    if let Some(scheme) = scheme.filter(|s| !s.is_empty()) {
        url = format!("{scheme}:{url}");
    }
    if let Some(query) = query {
        url = format!("{url}?{query}");
    }
    if let Some(fragment) = fragment {
        url = format!("{url}#{fragment}");
    }
    url
}

/// `urllib.parse.urlunsplit(parts)` for a `urlsplit` result: empty components are dropped,
/// except the empty netloc a netloc scheme keeps before an absolute path.
pub fn urlunsplit(parts: &Split) -> String {
    let some = |s: &str| (!s.is_empty()).then_some(s.to_owned());
    let netloc = if parts.netloc.is_empty() {
        (USES_NETLOC.contains(&parts.scheme.as_str())
            && !parts.scheme.is_empty()
            && (parts.path.is_empty() || parts.path.starts_with('/')))
        .then(String::new)
    } else {
        Some(parts.netloc.clone())
    };
    unsplit(
        some(&parts.scheme).as_deref(),
        netloc.as_deref(),
        &parts.path,
        some(&parts.query).as_deref(),
        some(&parts.fragment).as_deref(),
    )
}

/// `urllib.parse.urljoin(base, url)` of CPython 3.15.
pub fn urljoin(base: &str, url: &str) -> Result<String, String> {
    if base.is_empty() {
        return Ok(url.to_owned());
    }
    if url.is_empty() {
        return Ok(base.to_owned());
    }
    let b = split_parts(base)?;
    let Parts {
        scheme,
        mut netloc,
        mut path,
        mut query,
        mut fragment,
    } = split_parts(url)?;
    let scheme = scheme.or(b.scheme.clone());
    let named = scheme.as_deref().filter(|s| !s.is_empty());
    if scheme != b.scheme || named.is_some_and(|s| !USES_RELATIVE.contains(&s)) {
        return Ok(url.to_owned());
    }
    if named.is_none_or(|s| USES_NETLOC.contains(&s)) {
        if netloc.as_deref().is_some_and(|n| !n.is_empty()) {
            return Ok(unsplit(
                scheme.as_deref(),
                netloc.as_deref(),
                &path,
                query.as_deref(),
                fragment.as_deref(),
            ));
        }
        netloc = b.netloc.clone();
    }
    if path.is_empty() {
        path = b.path.clone();
        if query.is_none() {
            query = b.query.clone();
            if fragment.is_none() {
                fragment = b.fragment.clone();
            }
        }
        return Ok(unsplit(
            scheme.as_deref(),
            netloc.as_deref(),
            &path,
            query.as_deref(),
            fragment.as_deref(),
        ));
    }
    let mut base_parts: Vec<&str> = b.path.split('/').collect();
    if base_parts.last() != Some(&"") {
        base_parts.pop();
    }
    let segments: Vec<&str> = if path.starts_with('/') {
        path.split('/').collect()
    } else {
        let mut all = base_parts;
        all.extend(path.split('/'));
        // segments[1:-1] = filter(None, segments[1:-1])
        if all.len() > 2 {
            let last = all.len() - 1;
            let mut kept = vec![all[0]];
            kept.extend(all[1..last].iter().filter(|s| !s.is_empty()));
            kept.push(all[last]);
            kept
        } else {
            all
        }
    };
    let mut resolved: Vec<&str> = Vec::new();
    for seg in &segments {
        match *seg {
            ".." => {
                resolved.pop();
            }
            "." => {}
            _ => resolved.push(seg),
        }
    }
    if matches!(segments.last(), Some(&".") | Some(&"..")) {
        resolved.push("");
    }
    let joined = resolved.join("/");
    Ok(unsplit(
        scheme.as_deref(),
        netloc.as_deref(),
        if joined.is_empty() { "/" } else { &joined },
        query.as_deref(),
        fragment.as_deref(),
    ))
}

fn check_netloc(netloc: &str) -> Result<(), String> {
    if netloc.is_empty() || netloc.is_ascii() {
        return Ok(());
    }
    let n: String = netloc
        .chars()
        .filter(|c| !matches!(c, '@' | ':' | '#' | '?'))
        .collect();
    let normalised: String = n.nfkc().collect();
    if n == normalised {
        return Ok(());
    }
    if normalised.contains(['/', '?', '#', '@', ':']) {
        return Err(format!(
            "netloc '{netloc}' contains invalid characters under NFKC normalization"
        ));
    }
    Ok(())
}

fn check_bracketed_netloc(netloc: &str) -> Result<(), String> {
    let host_and_port = netloc.rsplit_once('@').map_or(netloc, |(_, h)| h);
    let hostname = match host_and_port.split_once('[') {
        Some((before, bracketed)) => {
            if !before.is_empty() {
                return Err("Invalid IPv6 URL".into());
            }
            let (hostname, port) = bracketed.split_once(']').unwrap_or((bracketed, ""));
            if !port.is_empty() && !port.starts_with(':') {
                return Err("Invalid IPv6 URL".into());
            }
            hostname
        }
        None => host_and_port
            .split_once(':')
            .map_or(host_and_port, |(h, _)| h),
    };
    if hostname.starts_with(['v', 'V']) {
        let valid = hostname[1..].split_once('.').is_some_and(|(hex, rest)| {
            !hex.is_empty()
                && hex.chars().all(|c| c.is_ascii_hexdigit())
                && !rest.is_empty()
                && !rest.contains('\n')
        });
        return if valid {
            Ok(())
        } else {
            Err("IPvFuture address is invalid".into())
        };
    }
    // ipaddress.ip_address tries IPv4 first, which brackets refuse.
    if is_ipv4(hostname) {
        return Err("An IPv4 address cannot be in brackets".into());
    }
    if !is_ipv6(hostname) {
        return Err(format!(
            "{} does not appear to be an IPv4 or IPv6 address",
            crate::py::repr_str(hostname)
        ));
    }
    Ok(())
}

/// `ipaddress.IPv4Address(text)` accepts it: 4 dotted ASCII decimal octets
/// of at most 3 digits, without leading zeros, each at most 255.
fn is_ipv4(text: &str) -> bool {
    let octets: Vec<&str> = text.split('.').collect();
    octets.len() == 4
        && octets.iter().all(|o| {
            !o.is_empty()
                && o.len() <= 3
                && o.bytes().all(|b| b.is_ascii_digit())
                && (*o == "0" || !o.starts_with('0'))
                && o.parse::<u16>().is_ok_and(|n| n <= 255)
        })
}

/// `ipaddress.IPv6Address(text)` accepts it, following `_split_scope_id` and
/// `_ip_int_from_string`: a `%scope` must be non-empty and hold no `%`.
fn is_ipv6(text: &str) -> bool {
    if text.contains('/') {
        return false;
    }
    let address = match text.split_once('%') {
        Some((_, scope)) if scope.is_empty() || scope.contains('%') => return false,
        Some((address, _)) => address,
        None => text,
    };
    if address.is_empty() || address.chars().count() > 45 {
        return false;
    }
    let mut parts: Vec<&str> = address.splitn(10, ':').collect();
    if parts.len() < 3 {
        return false;
    }
    if parts[parts.len() - 1].contains('.') {
        // An IPv4 suffix stands for 2 hextets.
        if !is_ipv4(parts.pop().expect("3 parts")) {
            return false;
        }
        parts.extend(["0", "0"]);
    }
    let n = parts.len();
    if n > 9 {
        return false;
    }
    let mut skip = None;
    for (i, part) in parts.iter().enumerate().take(n - 1).skip(1) {
        if part.is_empty() {
            if skip.is_some() {
                return false;
            }
            skip = Some(i);
        }
    }
    let (hi, lo) = match skip {
        Some(at) => {
            let (mut hi, mut lo) = (at, n - at - 1);
            if parts[0].is_empty() {
                hi -= 1;
                if hi != 0 {
                    return false;
                }
            }
            if parts[n - 1].is_empty() {
                lo -= 1;
                if lo != 0 {
                    return false;
                }
            }
            if hi + lo >= 8 {
                return false;
            }
            (hi, lo)
        }
        None if n != 8 || parts[0].is_empty() || parts[n - 1].is_empty() => return false,
        None => (n, 0),
    };
    let hextet =
        |h: &&str| !h.is_empty() && h.len() <= 4 && h.bytes().all(|b| b.is_ascii_hexdigit());
    parts[..hi].iter().all(hextet) && parts[n - lo..].iter().all(hextet)
}

/// `.hostname` of a split URL: lower-cased, without user information, port or brackets.
pub fn hostname(netloc: &str) -> Option<String> {
    if netloc.is_empty() {
        return None;
    }
    let hostinfo = netloc.rsplit_once('@').map_or(netloc, |(_, h)| h);
    let host = match hostinfo.split_once('[') {
        Some((_, bracketed)) => bracketed.split_once(']').map_or(bracketed, |(h, _)| h),
        None => hostinfo.split_once(':').map_or(hostinfo, |(h, _)| h),
    };
    if host.is_empty() {
        return None;
    }
    let (address, zone) = host
        .split_once('%')
        .map_or((host, None), |(a, z)| (a, Some(z)));
    Some(match zone {
        Some(zone) => format!("{}%{zone}", address.to_lowercase()),
        None => address.to_lowercase(),
    })
}

/// `.port` of a split URL, or the ValueError's message: the text after the
/// host's `:`, which must be ASCII digits within 0-65535.
pub fn port(netloc: &str) -> Result<Option<u32>, String> {
    let hostinfo = netloc.rsplit_once('@').map_or(netloc, |(_, h)| h);
    let port = match hostinfo.split_once('[') {
        Some((_, bracketed)) => bracketed
            .split_once(']')
            .and_then(|(_, rest)| rest.split_once(':'))
            .map_or("", |(_, p)| p),
        None => hostinfo.split_once(':').map_or("", |(_, p)| p),
    };
    if port.is_empty() {
        return Ok(None);
    }
    if !port.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!(
            "Port could not be cast to integer value as {}",
            crate::py::repr_str(port)
        ));
    }
    match port.trim_start_matches('0') {
        "" => Ok(Some(0)),
        digits => match digits.parse::<u32>() {
            Ok(value) if value <= 65535 => Ok(Some(value)),
            _ => Err("Port out of range 0-65535".into()),
        },
    }
}

/// `.username or .password` of a split URL: user information before the
/// last `@` that is not empty, either side of its first `:`.
pub fn has_credentials(netloc: &str) -> bool {
    netloc.rsplit_once('@').is_some_and(|(info, _)| {
        let (user, password) = info.split_once(':').unwrap_or((info, ""));
        !user.is_empty() || !password.is_empty()
    })
}

/// `urllib.parse.unquote(text)`: each maximal ASCII run is percent-decoded and
/// read as UTF-8 with U+FFFD for invalid bytes; other text passes through.
pub fn unquote(text: &str) -> String {
    if !text.contains('%') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while !rest.is_empty() {
        let (ascii, tail) = rest.split_at(rest.find(|c: char| !c.is_ascii()).unwrap_or(rest.len()));
        out.push_str(&String::from_utf8_lossy(&percent_decode(ascii.as_bytes())));
        let (other, tail) = tail.split_at(tail.find(|c: char| c.is_ascii()).unwrap_or(tail.len()));
        out.push_str(other);
        rest = tail;
    }
    out
}

/// `%XX` to its byte where both are hex digits; any other `%` stays.
fn percent_decode(bytes: &[u8]) -> Vec<u8> {
    let hex = |b: u8| (b as char).to_digit(16).map(|d| d as u8);
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%'
            && let (Some(&a), Some(&b)) = (bytes.get(i + 1), bytes.get(i + 2))
            && let (Some(a), Some(b)) = (hex(a), hex(b))
        {
            out.push(a * 16 + b);
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Expected values from CPython 3.15's `ipaddress.IPv6Address`.
    #[test]
    fn ipv6_addresses_are_read_as_ipaddress_reads_them() {
        let long = format!("{}:", "0:".repeat(20));
        for (text, valid) in [
            ("fe80::%1%25ETH0", false),
            ("1.2.3.4", false),
            ("::ffff:1.2.3.4", true),
            ("::ffff:1.2.3.04", false),
            ("1:2:3:4:5:6:7::", true),
            ("::1:2:3:4:5:6:7", true),
            ("1::2::3", false),
            (":1::", false),
            ("1:2:3:4:5:6:7:8:9", false),
            ("12345::", false),
            ("::%", false),
            ("::1/64", false),
            ("0:0:0:0:0:0:0:0", true),
            ("::ffff:1.2.3", false),
            ("1:2:3:4:5:6:1.2.3.4", true),
            ("1:2:3:4:5:6:7:1.2.3.4", false),
            (long.as_str(), false),
            ("\u{ff11}::", false),
            ("::1%\u{e9}", true),
            ("1:2:3:4:5:6:7:8::", false),
            ("::", true),
            (":::", false),
            ("::1.2.3.4", true),
            ("a:b:c:d:e:f:1:2:3:4", false),
        ] {
            assert_eq!(is_ipv6(text), valid, "{text}");
        }
        assert!(is_ipv4("1.2.3.4") && !is_ipv4("1.2.3.4%x") && !is_ipv4("01.2.3.4"));
    }

    /// Expected values from CPython 3.15's urljoin.
    #[test]
    fn joins_like_urllib() {
        for (base, url, expected) in [
            (
                "https://duguid.com.au/rates/example/",
                "rates.csv",
                "https://duguid.com.au/rates/example/rates.csv",
            ),
            (
                "https://duguid.com.au/rates/example/",
                "../",
                "https://duguid.com.au/rates/",
            ),
            (
                "https://duguid.com.au/rates/example/",
                "../../../../x",
                "https://duguid.com.au/x",
            ),
            (
                "https://duguid.com.au/rates/example/",
                "//cdn.example.test/x",
                "https://cdn.example.test/x",
            ),
            (
                "https://duguid.com.au/rates/example/",
                "?q=1",
                "https://duguid.com.au/rates/example/?q=1",
            ),
            (
                "https://duguid.com.au/rates/example/",
                "#f",
                "https://duguid.com.au/rates/example/#f",
            ),
            (
                "https://duguid.com.au/rates/example/",
                "mailto:x@y",
                "mailto:x@y",
            ),
            (
                "https://duguid.com.au/rates/example/",
                "HTTP://Example.TEST/A",
                "HTTP://Example.TEST/A",
            ),
            ("https://a.test/b/c", "./", "https://a.test/b/"),
            ("https://a.test/b/c", "..", "https://a.test/"),
            ("https://a.test/b/c?x#y", "", "https://a.test/b/c?x#y"),
            ("https://a.test/b/c?x#y", "#z", "https://a.test/b/c?x#z"),
            ("https://a.test/b/c?x#y", "?", "https://a.test/b/c?"),
            ("https://a.test", "x", "https://a.test/x"),
            ("https://a.test/b/", "a//b/./c/..", "https://a.test/b/a/b/"),
            ("http://a/b/c/d;p?q", "g;x?y#s", "http://a/b/c/g;x?y#s"),
            ("http://a/b/c/d;p?q", "../../../g", "http://a/g"),
            ("http://a/b/c/d;p?q", "/./g", "http://a/g"),
            ("", "x", "x"),
            ("foo:bar", "x", "x"),
            ("https://a.test/b", "ftp://h/x", "ftp://h/x"),
            ("https://a.test/b", "https:x", "https://a.test/x"),
            ("https://a.test/b", "tel:1", "tel:1"),
        ] {
            assert_eq!(urljoin(base, url).unwrap(), expected, "{base} + {url}");
        }
        assert!(urljoin("https://a.test/", "http://[::1").is_err());
    }

    #[test]
    fn splits_like_urllib() {
        let s = urlsplit(" \tMAILTO:ryan@example.test?subject=x").unwrap();
        assert_eq!(
            (s.scheme.as_str(), s.path.as_str(), s.query.as_str()),
            ("mailto", "ryan@example.test", "subject=x")
        );
        let s = urlsplit("https://User@WWW.LinkedIn.com:443/in/x/#f").unwrap();
        assert_eq!(hostname(&s.netloc).as_deref(), Some("www.linkedin.com"));
        assert_eq!(s.path, "/in/x/");
        assert!(urlsplit("https://[::1/x").is_err());
        assert!(urlsplit("https://[1.2.3.4]/").is_err());
        assert!(urlsplit("https://[fe80::1%25eth0]/").is_ok());
        assert_eq!(urlsplit("1http:x").unwrap().scheme, "");
        assert!(urlsplit("https://ex\u{2100}ample/").is_err());
    }
}
