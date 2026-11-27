//! `urlsplit`'s netloc checks against CPython 3.15: NFKC normalisation, bracketed hosts, IPvFuture
//! and IPv6 literals. Expected values are what `tests/data/cov_b_pyurl.py` prints.
use au_tax_rates_data::pyurl::urlsplit;

fn netloc(url: &str) -> Result<String, String> {
    urlsplit(url).map(|s| s.netloc)
}

fn not_an_address(host: &str) -> Result<String, String> {
    Err(format!(
        "'{host}' does not appear to be an IPv4 or IPv6 address"
    ))
}

#[test]
fn non_ascii_netlocs_pass_unless_nfkc_adds_a_delimiter() {
    // Already NFKC: kept as written.
    assert_eq!(
        netloc("https://b\u{fc}cher.de/x"),
        Ok("b\u{fc}cher.de".into())
    );
    // Fullwidth 'a' normalises to 'a', which is harmless, so the netloc is kept unnormalised.
    assert_eq!(netloc("https://\u{ff41}.com/x"), Ok("\u{ff41}.com".into()));
    // U+2100 normalises to 'a/c', bringing in a slash.
    assert_eq!(
        netloc("https://evil\u{2100}.com/x"),
        Err(
            "netloc 'evil\u{2100}.com' contains invalid characters under NFKC normalization".into()
        )
    );
}

#[test]
fn bracketed_hosts_are_checked_as_python_checks_them() {
    assert_eq!(netloc("http://user@[::1]:80/x"), Ok("user@[::1]:80".into()));
    // Text before the bracket, or after it but before the port, is refused.
    assert_eq!(netloc("http://x[::1]/x"), Err("Invalid IPv6 URL".into()));
    assert_eq!(netloc("http://[::1]x/x"), Err("Invalid IPv6 URL".into()));
    // The last '@' leaves no bracket in the host, so the host is the text before any ':'.
    assert_eq!(netloc("http://[v1.x]@host]/x"), not_an_address("host]"));
    assert_eq!(
        netloc("http://[1.2.3.4]/x"),
        Err("An IPv4 address cannot be in brackets".into())
    );
    assert_eq!(netloc("http://[zz]/x"), not_an_address("zz"));
}

#[test]
fn ipvfuture_needs_hex_a_dot_and_more() {
    assert_eq!(netloc("http://[v1.fe]/x"), Ok("[v1.fe]".into()));
    assert_eq!(netloc("http://[V1.fe]/x"), Ok("[V1.fe]".into()));
    let invalid = Err("IPvFuture address is invalid".to_string());
    assert_eq!(netloc("http://[vz.x]/x"), invalid);
    assert_eq!(netloc("http://[v1.]/x"), invalid);
}

#[test]
fn ipv6_literals_follow_ipaddress() {
    let long = "1".repeat(46);
    assert_eq!(netloc(&format!("http://[{long}]/x")), not_an_address(&long));
    // A trailing ':' after '::' leaves a hextet missing.
    assert_eq!(netloc("http://[1::2:]/x"), not_an_address("1::2:"));
    // '::' must stand for at least one hextet.
    assert_eq!(
        netloc("http://[1:2:3:4::5:6:7:8]/x"),
        not_an_address("1:2:3:4::5:6:7:8")
    );
    assert_eq!(
        netloc("http://[1:2:3:4:5:6:1.2.3.4]/x"),
        Ok("[1:2:3:4:5:6:1.2.3.4]".into())
    );
}
