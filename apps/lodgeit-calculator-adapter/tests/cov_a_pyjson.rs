// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The JSON port's refusals, byte encodings and writer against CPython 3.15's
//! `json`: each expected message or value is what Python gives for the same
//! input.
use num_bigint::BigInt;

use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::pyjson::{
    self, DecodeError, Json, Layout, PyStr, check_round_trip, loads, loads_bytes, loads_standard,
    recursion_message,
};

fn int(n: i64) -> Json {
    Json::Int(BigInt::from(n))
}

fn refusal(text: &str) -> String {
    match loads(text) {
        Err(DecodeError::Invalid(message)) => message,
        other => panic!("{text:?} gave {other:?}"),
    }
}

#[test]
fn syntax_errors_give_pythons_message_and_position() {
    for (text, expected) in [
        (
            "[\n1 2]",
            "Expecting ',' delimiter: line 2 column 3 (char 4)",
        ),
        (
            "\u{feff}1",
            "Unexpected UTF-8 BOM (decode using utf-8-sig): line 1 column 1 (char 0)",
        ),
        (
            "{\"a\" 1}",
            "Expecting ':' delimiter: line 1 column 6 (char 5)",
        ),
        ("[1 2]", "Expecting ',' delimiter: line 1 column 4 (char 3)"),
        (
            "[1,]",
            "Illegal trailing comma before end of array: line 1 column 3 (char 2)",
        ),
        (
            "[1,\n  2,]",
            "Illegal trailing comma before end of array: line 2 column 4 (char 7)",
        ),
        (
            "{\"a\":1 \"b\":2}",
            "Expecting ',' delimiter: line 1 column 8 (char 7)",
        ),
        ("-", "Expecting value: line 1 column 1 (char 0)"),
        ("[1e]", "Expecting ',' delimiter: line 1 column 3 (char 2)"),
        (
            "\"a\u{1}\"",
            "Invalid control character at: line 1 column 3 (char 2)",
        ),
    ] {
        assert_eq!(refusal(text), expected, "{text:?}");
    }
}

#[test]
fn every_short_escape_decodes() {
    assert_eq!(
        loads(r#""\/\b\f\n\r\t""#),
        Ok(Json::str("/\u{8}\u{c}\n\r\t"))
    );
}

#[test]
fn infinities_decode_as_floats_by_default() {
    assert_eq!(
        loads("[Infinity, -Infinity]"),
        Ok(Json::Array(vec![
            Json::Float(f64::INFINITY),
            Json::Float(f64::NEG_INFINITY)
        ]))
    );
}

#[test]
fn a_plain_dict_keeps_a_repeated_members_first_place_and_last_value() {
    assert_eq!(
        loads_standard(r#"{"a":1,"b":2,"a":3}"#),
        Ok(Json::obj([("a", int(3)), ("b", int(2))]))
    );
    assert_eq!(loads(r#"{"a":1,"a":3}"#), Err(DecodeError::Duplicate));
}

#[test]
fn nesting_past_the_stack_limit_is_a_recursion_error() {
    let arrays = "[".repeat(70_000);
    assert_eq!(loads(&arrays), Err(DecodeError::Recursion("array")));
    let objects = "{\"a\":".repeat(70_000);
    assert_eq!(loads(&objects), Err(DecodeError::Recursion("object")));
}

#[test]
fn recursion_messages_name_what_was_being_decoded() {
    let used = if cfg!(windows) { 1952 } else { 8156 };
    assert_eq!(
        recursion_message("array"),
        format!(
            "Stack overflow (used {used} kB) while decoding a JSON array from a unicode string"
        )
    );
    assert_eq!(
        recursion_message("call"),
        format!("Stack overflow (used {used} kB) while calling a Python object")
    );
    assert_eq!(
        recursion_message("hook"),
        format!("Stack overflow (used {used} kB)")
    );
}

/// On Windows the default `parse_constant` call checks the stack 544 bytes
/// deeper than the arrays around it, so a constant 13,860 arrays down fails
/// where an array there would still open.
#[cfg(windows)]
#[test]
fn a_constant_deep_in_arrays_fails_its_call_on_windows() {
    let depth = 13_860;
    let text = format!("{}NaN{}", "[".repeat(depth), "]".repeat(depth));
    assert_eq!(loads(&text), Err(DecodeError::Recursion("call")));
}

#[test]
fn bytes_are_read_in_the_encoding_json_detects() {
    let utf16 = |text: &str, big: bool| -> Vec<u8> {
        text.encode_utf16()
            .flat_map(|u| {
                if big {
                    u.to_be_bytes()
                } else {
                    u.to_le_bytes()
                }
            })
            .collect()
    };
    let cases: Vec<(Vec<u8>, Json)> = vec![
        (b"\xff\xfe\x00\x001\x00\x00\x00".to_vec(), int(1)),
        (b"\x00\x00\xfe\xff\x00\x00\x001".to_vec(), int(1)),
        (b"\xfe\xff\x001".to_vec(), int(1)),
        (b"\xff\xfe1\x00".to_vec(), int(1)),
        (b"\xef\xbb\xbf1".to_vec(), int(1)),
        (utf16("[1]", false), Json::Array(vec![int(1)])),
        (utf16("[1]", true), Json::Array(vec![int(1)])),
        (b"1\x00\x00\x00".to_vec(), int(1)),
        (b"\x00\x00\x001".to_vec(), int(1)),
        (b"\x001".to_vec(), int(1)),
        (b"1\x00".to_vec(), int(1)),
        (utf16("\"\u{1F600}\"", false), Json::str("\u{1F600}")),
    ];
    for (bytes, expected) in cases {
        assert_eq!(loads_bytes(&bytes), Ok(expected), "{bytes:?}");
    }
}

#[test]
fn bytes_that_do_not_decode_are_a_unicode_error() {
    for bytes in [
        &b"\xfe\xff\x00"[..],
        b"\x00\x00\xfe\xff\x00\x11\x00\x00",
        b"\xff\xfe\x00\x001",
        b"\xff1",
    ] {
        assert_eq!(loads_bytes(bytes), Err(None), "{bytes:?}");
    }
}

#[test]
fn the_fuzz_invariant_holds_for_valid_invalid_and_undecodable_input() {
    for data in [
        &br#"{"a":[1,2.5,"x\u00e9",null,true],"b":{"c":-0.0}}"#[..],
        b"[1,",
        b"\xff\xfe",
    ] {
        assert_eq!(check_round_trip(data), Ok(()), "{data:?}");
    }
}

#[test]
fn control_characters_and_infinities_are_written_as_python_does() {
    assert_eq!(
        pyjson::dumps(&Json::str("\r\t\u{8}\u{c}"), Layout::Canonical),
        r#""\r\t\b\f""#
    );
    assert_eq!(
        pyjson::dumps(
            &Json::Array(vec![
                Json::Float(f64::INFINITY),
                Json::Float(f64::NEG_INFINITY)
            ]),
            Layout::Canonical
        ),
        "[Infinity,-Infinity]"
    );
}

#[test]
fn truthiness_and_key_checks_follow_python() {
    assert!(!Json::Float(0.0).truthy());
    assert!(Json::Float(1.5).truthy());
    assert!(!Json::Dec(Decimal::parse("0.00").expect("valid")).truthy());
    assert!(!Json::str("").truthy());
    assert!(Json::Str(PyStr::Lone(vec![0xd800])).truthy());
    assert!(!Json::Array(vec![]).has_exactly(&[]));
    assert!(Json::obj([("a", Json::Null)]).has_exactly(&["a"]));
}
