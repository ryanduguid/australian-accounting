// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `json.loads` on text and on bytes in each encoding `json.detect_encoding` picks,
//! against the answers CPython 3.15.0 gave (written by a script that
//! ran each input through it). Values compare by their Python `repr`.

use super::*;
use crate::pyvalue::repr;

const TEXT_CASES: &[(&str, Result<&str, &str>)] = &[
    ("[Infinity, -Infinity, NaN]", Ok("[inf, -inf, nan]")),
    (
        "{\u{22}a\u{22} 1}",
        Err("Expecting ':' delimiter: line 1 column 6 (char 5)"),
    ),
    (
        "[1 2]",
        Err("Expecting ',' delimiter: line 1 column 4 (char 3)"),
    ),
    (
        "[1,]",
        Err("Illegal trailing comma before end of array: line 1 column 3 (char 2)"),
    ),
    (
        "{\u{22}a\u{22}: 1 \u{22}b\u{22}: 2}",
        Err("Expecting ',' delimiter: line 1 column 9 (char 8)"),
    ),
    (
        "\u{22}a\u{1}b\u{22}",
        Err("Invalid control character at: line 1 column 3 (char 2)"),
    ),
    (
        "\u{22}\u{5c}/\u{5c}b\u{5c}f\u{5c}n\u{5c}r\u{5c}t\u{22}",
        Ok("'/\u{5c}x08\u{5c}x0c\u{5c}n\u{5c}r\u{5c}t'"),
    ),
    ("1e", Err("Extra data: line 1 column 2 (char 1)")),
    ("1e+", Err("Extra data: line 1 column 2 (char 1)")),
    ("1e5", Ok("100000.0")),
    ("-1.5E-3", Ok("-0.0015")),
    (
        "[\u{a}1,\u{a}2,\u{a} x]",
        Err("Expecting value: line 4 column 2 (char 9)"),
    ),
    (
        "{\u{22}a\u{22}: 1, \u{22}a\u{22}: 2, \u{22}b\u{22}: [true, false, null]}",
        Ok("{'a': 2, 'b': [True, False, None]}"),
    ),
    ("\u{22}\u{5c}ud800\u{22}", Ok("'\u{5c}ud800'")),
    ("\u{22}x\u{5c}u00e9\u{22}", Ok("'x\u{e9}'")),
];

#[test]
fn text_decodes_or_fails_as_json_loads_does() {
    for (text, want) in TEXT_CASES {
        let got = loads_standard(text).map(|v| repr(&v));
        let want = want.map_err(|message| DecodeError::Invalid(message.into()));
        assert_eq!(got.as_deref().map_err(Clone::clone), want, "{text:?}");
    }
}

#[test]
fn bytes_decode_in_the_encoding_json_detects() {
    type Case<'a> = (&'a [u8], Result<&'a str, Option<DecodeError>>);
    let cases: &[Case] = &[
        (b"\xef\xbb\xbf{\x22k\x22: [\x22\xc3\xa9\x22, 1]}", Ok("{'k': ['\u{e9}', 1]}")),
        (b"\xff\xfe{\x00\x22\x00k\x00\x22\x00:\x00 \x00[\x00\x22\x00\xe9\x00\x22\x00,\x00 \x001\x00]\x00}\x00", Ok("{'k': ['\u{e9}', 1]}")),
        (b"\x00{\x00\x22\x00k\x00\x22\x00:\x00 \x00[\x00\x22\x00\xe9\x00\x22\x00,\x00 \x001\x00]\x00}", Ok("{'k': ['\u{e9}', 1]}")),
        (b"{\x00\x22\x00k\x00\x22\x00:\x00 \x00[\x00\x22\x00\xe9\x00\x22\x00,\x00 \x001\x00]\x00}\x00", Ok("{'k': ['\u{e9}', 1]}")),
        (b"\xff\xfe\x00\x00{\x00\x00\x00\x22\x00\x00\x00k\x00\x00\x00\x22\x00\x00\x00:\x00\x00\x00 \x00\x00\x00[\x00\x00\x00\x22\x00\x00\x00\xe9\x00\x00\x00\x22\x00\x00\x00,\x00\x00\x00 \x00\x00\x001\x00\x00\x00]\x00\x00\x00}\x00\x00\x00", Ok("{'k': ['\u{e9}', 1]}")),
        (b"\x00\x00\x00{\x00\x00\x00\x22\x00\x00\x00k\x00\x00\x00\x22\x00\x00\x00:\x00\x00\x00 \x00\x00\x00[\x00\x00\x00\x22\x00\x00\x00\xe9\x00\x00\x00\x22\x00\x00\x00,\x00\x00\x00 \x00\x00\x001\x00\x00\x00]\x00\x00\x00}", Ok("{'k': ['\u{e9}', 1]}")),
        (b"{\x00\x00\x00\x22\x00\x00\x00k\x00\x00\x00\x22\x00\x00\x00:\x00\x00\x00 \x00\x00\x00[\x00\x00\x00\x22\x00\x00\x00\xe9\x00\x00\x00\x22\x00\x00\x00,\x00\x00\x00 \x00\x00\x001\x00\x00\x00]\x00\x00\x00}\x00\x00\x00", Ok("{'k': ['\u{e9}', 1]}")),
        (b"\xfe\xff\x00", Err(None)),
        (b"\xff\xfe\x00\x00\x00", Err(None)),
        (b"\x00\x001", Err(Some(DecodeError::Invalid("Expecting value: line 1 column 1 (char 0)".into())))),
        (b"1\x00", Ok("1")),
        (b"\x001", Ok("1")),
        (b"\x22\x00=\xd8\x00\xde\x22\x00", Ok("'\u{1f600}'")),
        (b"\x22\x00\x00\xd8\x22\x00", Ok("'\u{5c}ud800'")),
        (b"\x00\x00\x00[\x00\x00\x00]", Ok("[]")),
        (b"[\x00\x00\x00]\x00\x00\x00", Ok("[]")),
        (b"\x00\x00\x00\x22\x00\x11\x00\x00\x00\x00\x00\x22", Err(None)),
        (b"\x22\xff\x22", Err(None)),
    ];
    for (data, want) in cases {
        let got = loads_bytes(data).map(|v| repr(&v));
        assert_eq!(
            got.as_deref().map_err(Clone::clone),
            want.clone(),
            "{data:x?}"
        );
    }
}

#[test]
fn truthiness_is_python_s() {
    assert!(!loads_standard("null").unwrap().truthy(), "null");
    assert!(loads_standard("true").unwrap().truthy(), "true");
    assert!(!loads_standard("false").unwrap().truthy(), "false");
    assert!(!loads_standard("0").unwrap().truthy(), "0");
    assert!(loads_standard("7").unwrap().truthy(), "7");
    assert!(!loads_standard("0.0").unwrap().truthy(), "0.0");
    assert!(loads_standard("0.5").unwrap().truthy(), "0.5");
    assert!(
        !loads_standard("\u{22}\u{22}").unwrap().truthy(),
        "\u{22}\u{22}"
    );
    assert!(
        loads_standard("\u{22}a\u{22}").unwrap().truthy(),
        "\u{22}a\u{22}"
    );
    assert!(
        loads_standard("\u{22}\u{5c}ud800\u{22}").unwrap().truthy(),
        "\u{22}\u{5c}ud800\u{22}"
    );
    assert!(!loads_standard("[]").unwrap().truthy(), "[]");
    assert!(loads_standard("[0]").unwrap().truthy(), "[0]");
    assert!(!loads_standard("{}").unwrap().truthy(), "{{}}");
    assert!(
        loads_standard("{\u{22}a\u{22}: 1}").unwrap().truthy(),
        "{{\u{22}a\u{22}: 1}}"
    );
}

#[test]
fn accessors_refuse_the_wrong_kind() {
    let lone = loads_standard("\"\\ud800\"").unwrap();
    let Json::Str(s) = &lone else {
        panic!("a string")
    };
    assert_eq!(s.as_str(), None);
    assert_eq!(s.code_points(), [0xd800]);
    assert_eq!(loads_standard("1").unwrap().as_str(), None);
    assert!(!loads_standard("[1]").unwrap().has_exactly(&[]));
}
