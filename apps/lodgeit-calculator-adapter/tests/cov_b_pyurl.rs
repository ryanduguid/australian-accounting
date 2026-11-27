// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `urllib.parse` against CPython 3.15: each row's expected value is what
//! CPython 3.15 gave for the same input (`urlsplit`'s 5 parts, `urlunsplit`
//! of them, `.hostname` and `.port`, `urljoin`, or the `ValueError` text).
use lodgeit_calculator_adapter::pyurl::{hostname, port, urljoin, urlsplit, urlunsplit};

type Split = (
    [&'static str; 5],
    &'static str,
    Option<&'static str>,
    Result<Option<u32>, &'static str>,
);

#[test]
fn urls_split_and_rejoin_as_cpython_does() {
    let rows: &[(&str, Result<Split, &str>)] = &[
        // BEGIN CPYTHON ROWS
        (
            "https://user:@Host.Test:8443/a/b?q=1#f",
            Ok((
                ["https", "user:@Host.Test:8443", "/a/b", "q=1", "f"],
                "https://user:@Host.Test:8443/a/b?q=1#f",
                Some("host.test"),
                Ok(Some(8443)),
            )),
        ),
        (
            "http://[::1]:80/x",
            Ok((
                ["http", "[::1]:80", "/x", "", ""],
                "http://[::1]:80/x",
                Some("::1"),
                Ok(Some(80)),
            )),
        ),
        (
            "http://[::1%25eth0]/",
            Ok((
                ["http", "[::1%25eth0]", "/", "", ""],
                "http://[::1%25eth0]/",
                Some("::1%25eth0"),
                Ok(None),
            )),
        ),
        (
            "http://[v1.fe]/",
            Ok((
                ["http", "[v1.fe]", "/", "", ""],
                "http://[v1.fe]/",
                Some("v1.fe"),
                Ok(None),
            )),
        ),
        ("http://[v1.]/", Err("IPvFuture address is invalid")),
        ("http://[vx.a]/", Err("IPvFuture address is invalid")),
        ("http://[v.a]/", Err("IPvFuture address is invalid")),
        ("http://a[::1]/", Err("Invalid IPv6 URL")),
        ("http://[::1]x/", Err("Invalid IPv6 URL")),
        ("http://[::1", Err("Invalid IPv6 URL")),
        ("http://::1]/", Err("Invalid IPv6 URL")),
        (
            "http://[1:2:3:4:5:6:7:8:9]/",
            Err("'1:2:3:4:5:6:7:8:9' does not appear to be an IPv4 or IPv6 address"),
        ),
        (
            "http://[1::2::3]/",
            Err("'1::2::3' does not appear to be an IPv4 or IPv6 address"),
        ),
        (
            "http://[::ffff:1.2.3.4]/",
            Ok((
                ["http", "[::ffff:1.2.3.4]", "/", "", ""],
                "http://[::ffff:1.2.3.4]/",
                Some("::ffff:1.2.3.4"),
                Ok(None),
            )),
        ),
        (
            "http://[1:2:3:4:5:6:1.2.3.4]/",
            Ok((
                ["http", "[1:2:3:4:5:6:1.2.3.4]", "/", "", ""],
                "http://[1:2:3:4:5:6:1.2.3.4]/",
                Some("1:2:3:4:5:6:1.2.3.4"),
                Ok(None),
            )),
        ),
        (
            "http://[1:2:3:4:5:6:7:1.2.3.4]/",
            Err("'1:2:3:4:5:6:7:1.2.3.4' does not appear to be an IPv4 or IPv6 address"),
        ),
        (
            "http://[::g]/",
            Err("'::g' does not appear to be an IPv4 or IPv6 address"),
        ),
        (
            "http://[]/",
            Err("'' does not appear to be an IPv4 or IPv6 address"),
        ),
        (
            "http://[1111111111111111111111111111111111111111111111]/",
            Err(
                "'1111111111111111111111111111111111111111111111' does not appear to be an IPv4 or IPv6 address",
            ),
        ),
        (
            "http://[1:2:3:4:5:6:7::]/",
            Ok((
                ["http", "[1:2:3:4:5:6:7::]", "/", "", ""],
                "http://[1:2:3:4:5:6:7::]/",
                Some("1:2:3:4:5:6:7::"),
                Ok(None),
            )),
        ),
        (
            "http://[::1:2:3:4:5:6:7]/",
            Ok((
                ["http", "[::1:2:3:4:5:6:7]", "/", "", ""],
                "http://[::1:2:3:4:5:6:7]/",
                Some("::1:2:3:4:5:6:7"),
                Ok(None),
            )),
        ),
        (
            "http://[1:2:3:4:5:6:7:8]/",
            Ok((
                ["http", "[1:2:3:4:5:6:7:8]", "/", "", ""],
                "http://[1:2:3:4:5:6:7:8]/",
                Some("1:2:3:4:5:6:7:8"),
                Ok(None),
            )),
        ),
        (
            "http://h:99999/",
            Ok((
                ["http", "h:99999", "/", "", ""],
                "http://h:99999/",
                Some("h"),
                Err("Port out of range 0-65535"),
            )),
        ),
        (
            "http://h:-1/",
            Ok((
                ["http", "h:-1", "/", "", ""],
                "http://h:-1/",
                Some("h"),
                Err("Port could not be cast to integer value as '-1'"),
            )),
        ),
        (
            "http://h:+1/",
            Ok((
                ["http", "h:+1", "/", "", ""],
                "http://h:+1/",
                Some("h"),
                Err("Port could not be cast to integer value as '+1'"),
            )),
        ),
        (
            "http://h:1_0/",
            Ok((
                ["http", "h:1_0", "/", "", ""],
                "http://h:1_0/",
                Some("h"),
                Err("Port could not be cast to integer value as '1_0'"),
            )),
        ),
        (
            "http://h: 1/",
            Ok((
                ["http", "h: 1", "/", "", ""],
                "http://h: 1/",
                Some("h"),
                Err("Port could not be cast to integer value as ' 1'"),
            )),
        ),
        (
            "http://h:/",
            Ok((
                ["http", "h:", "/", "", ""],
                "http://h:/",
                Some("h"),
                Ok(None),
            )),
        ),
        (
            "http://h:\u{661}/",
            Ok((
                ["http", "h:\u{661}", "/", "", ""],
                "http://h:\u{661}/",
                Some("h"),
                Err("Port could not be cast to integer value as '\u{661}'"),
            )),
        ),
        (
            "http://@/",
            Ok((["http", "@", "/", "", ""], "http://@/", None, Ok(None))),
        ),
        (
            "http://u@/",
            Ok((["http", "u@", "/", "", ""], "http://u@/", None, Ok(None))),
        ),
        (
            "http://%41/",
            Ok((
                ["http", "%41", "/", "", ""],
                "http://%41/",
                Some("%41"),
                Ok(None),
            )),
        ),
        (
            "http://[::1%eth0]/",
            Ok((
                ["http", "[::1%eth0]", "/", "", ""],
                "http://[::1%eth0]/",
                Some("::1%eth0"),
                Ok(None),
            )),
        ),
        (
            "//netloc//p",
            Ok((
                ["", "netloc", "//p", "", ""],
                "//netloc//p",
                Some("netloc"),
                Ok(None),
            )),
        ),
        (
            "////p",
            Ok((["", "", "//p", "", ""], "////p", None, Ok(None))),
        ),
        (
            "/p?q#f",
            Ok((["", "", "/p", "q", "f"], "/p?q#f", None, Ok(None))),
        ),
        (
            "mailto:x@y",
            Ok((["mailto", "", "x@y", "", ""], "mailto:x@y", None, Ok(None))),
        ),
        (
            "file:///etc",
            Ok((["file", "", "/etc", "", ""], "file:///etc", None, Ok(None))),
        ),
        (
            "a:b:c",
            Ok((["a", "", "b:c", "", ""], "a:b:c", None, Ok(None))),
        ),
        (
            "1a:b",
            Ok((["", "", "1a:b", "", ""], "1a:b", None, Ok(None))),
        ),
        ("?q", Ok((["", "", "", "q", ""], "?q", None, Ok(None)))),
        ("#f", Ok((["", "", "", "", "f"], "#f", None, Ok(None)))),
        (
            "http://h\u{ff03}/",
            Err("netloc 'h\u{ff03}' contains invalid characters under NFKC normalization"),
        ),
        (
            "http://h/\u{a}",
            Ok((["http", "h", "/", "", ""], "http://h/", Some("h"), Ok(None))),
        ),
        (
            "HTTP://H/",
            Ok((["http", "H", "/", "", ""], "http://H/", Some("h"), Ok(None))),
        ),
        (
            "http://h?x",
            Ok((
                ["http", "h", "", "x", ""],
                "http://h?x",
                Some("h"),
                Ok(None),
            )),
        ),
        (
            "http://h#x",
            Ok((
                ["http", "h", "", "", "x"],
                "http://h#x",
                Some("h"),
                Ok(None),
            )),
        ),
        // END CPYTHON ROWS
    ];
    let mut wrong = Vec::new();
    for (url, want) in rows {
        let got = urlsplit(url).map(|p| {
            let parts =
                [&p.scheme, &p.netloc, &p.path, &p.query, &p.fragment].map(|s| s.to_string());
            (parts, urlunsplit(&p), hostname(&p.netloc), port(&p.netloc))
        });
        let want = want
            .map(|(parts, rejoined, host, port)| {
                (
                    parts.map(str::to_string),
                    rejoined.to_string(),
                    host.map(str::to_string),
                    port.map_err(str::to_string),
                )
            })
            .map_err(str::to_string);
        if got != want {
            wrong.push(format!("{url:?}: got {got:?}, CPython {want:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn urls_join_as_cpython_does() {
    let rows: &[(&str, &str, Result<&str, &str>)] = &[
        // BEGIN CPYTHON JOINS
        ("http://a/b/c/d;p?q", "g", Ok("http://a/b/c/g")),
        ("http://a/b/c/d;p?q", "../../../g", Ok("http://a/g")),
        ("http://a/b/c/d;p?q", "//g", Ok("http://g")),
        ("http://a/b/c/d;p?q", "", Ok("http://a/b/c/d;p?q")),
        ("http://a/b/c/d;p?q", "#s", Ok("http://a/b/c/d;p?q#s")),
        ("http://a/b/c/d;p?q", "?y", Ok("http://a/b/c/d;p?y")),
        ("http://a/b/c/d;p?q#f", "", Ok("http://a/b/c/d;p?q#f")),
        ("http://a/b", "#", Ok("http://a/b#")),
        ("", "x", Ok("x")),
        ("x", "", Ok("x")),
        ("mailto:a", "b", Ok("b")),
        ("http://a/b", "https://c/d", Ok("https://c/d")),
        ("http://a/b", "http:g", Ok("http://a/g")),
        ("http://a", "g", Ok("http://a/g")),
        ("http://a/b/", ".", Ok("http://a/b/")),
        ("http://a/b/c", "./", Ok("http://a/b/")),
        ("http://a/b/c", "g/..", Ok("http://a/b/")),
        ("http://a/b?q", "#f", Ok("http://a/b?q#f")),
        ("http://a/b#f", "?q", Ok("http://a/b?q")),
        ("//a/b", "c", Ok("//a/c")),
        ("http://a/b", "//[::1", Err("Invalid IPv6 URL")),
        ("http://[::1", "x", Err("Invalid IPv6 URL")),
        ("svn+ssh://a/b", "c", Ok("svn+ssh://a/c")),
        ("http:///b", "c", Ok("http:///c")),
        ("http://a/b", "http:", Ok("http://a/b")),
        ("http://a/b/c", "../../..", Ok("http://a/")),
        ("foo:/b", "c", Ok("c")),
        // END CPYTHON JOINS
    ];
    let mut wrong = Vec::new();
    for (base, url, want) in rows {
        let got = urljoin(base, url);
        if got.as_deref().map_err(String::as_str) != *want {
            wrong.push(format!("{base:?} + {url:?}: got {got:?}, CPython {want:?}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn an_empty_netloc_has_no_hostname() {
    // CPython 3.15: urlsplit('file:///x').hostname is None.
    assert_eq!(hostname(""), None);
}
