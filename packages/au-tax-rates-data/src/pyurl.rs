//! `urllib.parse.urlsplit` of CPython 3.15 for `str` URLs.
//!
//! The checks use the scheme, netloc and path; the `ValueError`s
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

    #[test]
    fn splits_like_urllib() {
        let s = urlsplit(" \tMAILTO:ryan@example.test?subject=x").unwrap();
        assert_eq!(
            (s.scheme.as_str(), s.path.as_str(), s.query.as_str()),
            ("mailto", "ryan@example.test", "subject=x")
        );
        let s = urlsplit("https://User@WWW.LinkedIn.com:443/in/x/#f").unwrap();
        assert_eq!(s.netloc, "User@WWW.LinkedIn.com:443");
        assert_eq!(s.path, "/in/x/");
        assert!(urlsplit("https://[::1/x").is_err());
        assert!(urlsplit("https://[1.2.3.4]/").is_err());
        assert!(urlsplit("https://[fe80::1%25eth0]/").is_ok());
        assert_eq!(urlsplit("1http:x").unwrap().scheme, "");
        assert!(urlsplit("https://ex\u{2100}ample/").is_err());
    }
}
