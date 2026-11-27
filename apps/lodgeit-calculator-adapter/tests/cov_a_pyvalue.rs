// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Python's `repr`, `==`, `hash`, `json.dumps` and `json.dump` on decoded
//! values, against CPython 3.15: each expected text is what Python writes.
use num_bigint::BigInt;

use lodgeit_calculator_adapter::decimal::Decimal;
use lodgeit_calculator_adapter::pyjson::{Json, PyStr};
use lodgeit_calculator_adapter::pyvalue::{
    self, Key, Stop, dump_until_stop, dumps_line, dumps_sorted, encode_print, float_to_int, key,
    py_eq, repr, repr_points, str_for_stderr, type_name, utf8_strict_error,
};

fn points(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    pyvalue::points_lossy(points)
}

const LONE: u32 = 0xd800;

#[test]
fn type_names_and_reprs_match_python() {
    assert_eq!(type_name(&Json::Bool(true)), "bool");
    assert_eq!(type_name(&Json::Float(1.0)), "float");
    assert_eq!(type_name(&Json::Dec(Decimal::from_int(1))), "Decimal");
    assert_eq!(repr(&Json::Bool(false)), "False");
    // repr('a\\') and repr('a\'"'): the backslash and the chosen quote are escaped.
    assert_eq!(repr_points(&points("a\\")), r"'a\\'");
    assert_eq!(repr_points(&points("a'\"")), r#"'a\'"'"#);
}

#[test]
fn stderr_text_backslash_replaces_a_lone_surrogate() {
    let value = Json::Str(PyStr::Lone(vec![u32::from('a'), LONE]));
    assert_eq!(str_for_stderr(&value), "a\\ud800");
}

#[test]
fn equality_compares_numbers_by_value() {
    assert!(py_eq(&Json::Float(1.5), &Json::Float(1.5)));
    assert!(!py_eq(&Json::Float(0.5), &Json::Int(BigInt::from(0))));
    assert!(py_eq(&Json::Null, &Json::Null));
    assert!(!py_eq(&Json::str("a"), &Json::Array(vec![])));
    assert_eq!(float_to_int(-3.0), BigInt::from(-3));
    assert_eq!(float_to_int(-0.0), BigInt::from(0));
}

#[test]
fn none_is_hashable_and_a_list_is_not() {
    assert_eq!(key(&Json::Null), Ok(Key::None));
    assert_eq!(key(&Json::Array(vec![])), Err("unhashable type: 'list'".to_string()));
}

#[test]
fn dumps_escapes_controls_and_non_ascii() {
    let value = Json::str("\\\n\r\t\u{8}\u{c}\u{1}\u{1F600}\u{e9}");
    assert_eq!(
        text(&dumps_line(&value, true)),
        r#""\\\n\r\t\b\f\u0001\ud83d\ude00\u00e9""#
    );
}

#[test]
fn dumps_writes_every_scalar_as_python_does() {
    let value = Json::Array(vec![
        Json::Null,
        Json::Bool(true),
        Json::Bool(false),
        Json::Int(BigInt::from(1)),
        Json::Float(f64::NAN),
        Json::Float(f64::INFINITY),
        Json::Float(f64::NEG_INFINITY),
        Json::Float(1.5),
        Json::Dec(Decimal::parse("1.0").expect("valid")),
    ]);
    assert_eq!(
        text(&dumps_line(&value, true)),
        r#"[null, true, false, 1, NaN, Infinity, -Infinity, 1.5, "1.0"]"#
    );
    let pairs = Json::obj([("a", Json::Int(BigInt::from(1))), ("b", Json::Int(BigInt::from(2)))]);
    assert_eq!(text(&dumps_line(&pairs, true)), r#"{"a": 1, "b": 2}"#);
}

#[test]
fn dumps_sorted_indents_and_orders_keys() {
    let value = Json::obj([("b", Json::Int(BigInt::from(1))), ("a", Json::Int(BigInt::from(2)))]);
    assert_eq!(text(&dumps_sorted(&value, Some(2))), "{\n  \"a\": 2,\n  \"b\": 1\n}");
}

#[test]
fn json_dump_to_a_strict_file_stops_at_the_chunk_holding_a_lone_surrogate() {
    let lone = || Json::Str(PyStr::Lone(vec![LONE]));
    let quoted = vec![u32::from('"'), LONE, u32::from('"')];

    let (written, stop) = dump_until_stop(&Json::Array(vec![Json::str("a"), lone()]), None, false, usize::MAX);
    assert_eq!(text(&written), "[\"a\"");
    let mut chunk = points(", ");
    chunk.extend(&quoted);
    assert_eq!(stop, Some(Stop::Unencodable(chunk)));

    let keyed = Json::Object(vec![(PyStr::Lone(vec![LONE]), Json::Int(BigInt::from(1)))]);
    let (written, stop) = dump_until_stop(&keyed, None, true, usize::MAX);
    assert_eq!(text(&written), "{");
    assert_eq!(stop, Some(Stop::Unencodable(quoted.clone())));

    let (written, stop) = dump_until_stop(&lone(), None, false, usize::MAX);
    assert!(written.is_empty());
    assert_eq!(stop, Some(Stop::Unencodable(quoted)));
}

#[test]
fn json_dump_stops_at_a_container_nested_past_the_limit() {
    let nested = Json::Array(vec![Json::Array(vec![Json::Int(BigInt::from(1))])]);
    let (written, stop) = dump_until_stop(&nested, None, false, 1);
    assert_eq!(text(&written), "[");
    assert_eq!(stop, Some(Stop::Depth));
}

#[test]
fn a_run_of_unmappable_surrogates_is_one_encoding_error() {
    assert_eq!(
        encode_print(&[LONE, LONE + 1]),
        Err("'utf-8' codec can't encode characters in position 0-1: surrogates not allowed".into())
    );
    assert_eq!(
        utf8_strict_error(&[u32::from('a'), LONE, LONE + 1]),
        Some("'utf-8' codec can't encode characters in position 1-2: surrogates not allowed".into())
    );
}
