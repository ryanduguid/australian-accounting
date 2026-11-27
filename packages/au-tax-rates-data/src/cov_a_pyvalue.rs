// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `json.dumps`, `type()`, `repr()`, `int()` and `==` against the answers CPython
//! 3.15.0 gave (written by a script that ran each value through it).

use super::*;
use crate::pyjson::loads_standard;

fn text(points: Vec<u32>) -> String {
    points
        .into_iter()
        .map(|cp| char::from_u32(cp).unwrap_or('?'))
        .collect()
}

#[test]
fn dumps_with_writes_what_json_dumps_writes() {
    let cases: &[(&str, Option<usize>, bool, bool, &str)] = &[
        (
            "{\u{22}b\u{22}: [1, 2.5, -0.0, 1e300, null, true, false], \u{22}a\u{22}: {\u{22}z\u{22}: [], \u{22}y\u{22}: {}}}",
            None,
            false,
            false,
            "{\u{22}b\u{22}: [1, 2.5, -0.0, 1e+300, null, true, false], \u{22}a\u{22}: {\u{22}z\u{22}: [], \u{22}y\u{22}: {}}}",
        ),
        (
            "{\u{22}b\u{22}: [1, 2.5, -0.0, 1e300, null, true, false], \u{22}a\u{22}: {\u{22}z\u{22}: [], \u{22}y\u{22}: {}}}",
            Some(2),
            true,
            true,
            "{\u{a}  \u{22}a\u{22}: {\u{a}    \u{22}y\u{22}: {},\u{a}    \u{22}z\u{22}: []\u{a}  },\u{a}  \u{22}b\u{22}: [\u{a}    1,\u{a}    2.5,\u{a}    -0.0,\u{a}    1e+300,\u{a}    null,\u{a}    true,\u{a}    false\u{a}  ]\u{a}}",
        ),
        (
            "{\u{22}b\u{22}: [1, 2.5, -0.0, 1e300, null, true, false], \u{22}a\u{22}: {\u{22}z\u{22}: [], \u{22}y\u{22}: {}}}",
            Some(1),
            false,
            true,
            "{\u{a} \u{22}a\u{22}: {\u{a}  \u{22}y\u{22}: {},\u{a}  \u{22}z\u{22}: []\u{a} },\u{a} \u{22}b\u{22}: [\u{a}  1,\u{a}  2.5,\u{a}  -0.0,\u{a}  1e+300,\u{a}  null,\u{a}  true,\u{a}  false\u{a} ]\u{a}}",
        ),
        (
            "{\u{22}b\u{22}: [1, 2.5, -0.0, 1e300, null, true, false], \u{22}a\u{22}: {\u{22}z\u{22}: [], \u{22}y\u{22}: {}}}",
            None,
            true,
            false,
            "{\u{22}b\u{22}: [1, 2.5, -0.0, 1e+300, null, true, false], \u{22}a\u{22}: {\u{22}z\u{22}: [], \u{22}y\u{22}: {}}}",
        ),
        (
            "[\u{22}q\u{5c}\u{22}b\u{5c}\u{5c}n\u{5c}nr\u{5c}rb\u{5c}bf\u{5c}ft\u{5c}tc\u{5c}u0001\u{22}, \u{22}\u{5c}u00e9\u{5c}u4e2d\u{22}, \u{22}\u{5c}ud83d\u{5c}ude00\u{22}, \u{22}\u{5c}ud800\u{22}]",
            None,
            false,
            false,
            "[\u{22}q\u{5c}\u{22}b\u{5c}\u{5c}n\u{5c}nr\u{5c}rb\u{5c}bf\u{5c}ft\u{5c}tc\u{5c}u0001\u{22}, \u{22}\u{e9}\u{4e2d}\u{22}, \u{22}\u{1f600}\u{22}, \u{22}?\u{22}]",
        ),
        (
            "[\u{22}q\u{5c}\u{22}b\u{5c}\u{5c}n\u{5c}nr\u{5c}rb\u{5c}bf\u{5c}ft\u{5c}tc\u{5c}u0001\u{22}, \u{22}\u{5c}u00e9\u{5c}u4e2d\u{22}, \u{22}\u{5c}ud83d\u{5c}ude00\u{22}, \u{22}\u{5c}ud800\u{22}]",
            Some(2),
            true,
            true,
            "[\u{a}  \u{22}q\u{5c}\u{22}b\u{5c}\u{5c}n\u{5c}nr\u{5c}rb\u{5c}bf\u{5c}ft\u{5c}tc\u{5c}u0001\u{22},\u{a}  \u{22}\u{5c}u00e9\u{5c}u4e2d\u{22},\u{a}  \u{22}\u{5c}ud83d\u{5c}ude00\u{22},\u{a}  \u{22}\u{5c}ud800\u{22}\u{a}]",
        ),
        (
            "[\u{22}q\u{5c}\u{22}b\u{5c}\u{5c}n\u{5c}nr\u{5c}rb\u{5c}bf\u{5c}ft\u{5c}tc\u{5c}u0001\u{22}, \u{22}\u{5c}u00e9\u{5c}u4e2d\u{22}, \u{22}\u{5c}ud83d\u{5c}ude00\u{22}, \u{22}\u{5c}ud800\u{22}]",
            Some(1),
            false,
            true,
            "[\u{a} \u{22}q\u{5c}\u{22}b\u{5c}\u{5c}n\u{5c}nr\u{5c}rb\u{5c}bf\u{5c}ft\u{5c}tc\u{5c}u0001\u{22},\u{a} \u{22}\u{e9}\u{4e2d}\u{22},\u{a} \u{22}\u{1f600}\u{22},\u{a} \u{22}?\u{22}\u{a}]",
        ),
        (
            "[\u{22}q\u{5c}\u{22}b\u{5c}\u{5c}n\u{5c}nr\u{5c}rb\u{5c}bf\u{5c}ft\u{5c}tc\u{5c}u0001\u{22}, \u{22}\u{5c}u00e9\u{5c}u4e2d\u{22}, \u{22}\u{5c}ud83d\u{5c}ude00\u{22}, \u{22}\u{5c}ud800\u{22}]",
            None,
            true,
            false,
            "[\u{22}q\u{5c}\u{22}b\u{5c}\u{5c}n\u{5c}nr\u{5c}rb\u{5c}bf\u{5c}ft\u{5c}tc\u{5c}u0001\u{22}, \u{22}\u{5c}u00e9\u{5c}u4e2d\u{22}, \u{22}\u{5c}ud83d\u{5c}ude00\u{22}, \u{22}\u{5c}ud800\u{22}]",
        ),
        (
            "[NaN, Infinity, -Infinity, 5e-324]",
            None,
            false,
            false,
            "[NaN, Infinity, -Infinity, 5e-324]",
        ),
        (
            "[NaN, Infinity, -Infinity, 5e-324]",
            Some(2),
            true,
            true,
            "[\u{a}  NaN,\u{a}  Infinity,\u{a}  -Infinity,\u{a}  5e-324\u{a}]",
        ),
        (
            "[NaN, Infinity, -Infinity, 5e-324]",
            Some(1),
            false,
            true,
            "[\u{a} NaN,\u{a} Infinity,\u{a} -Infinity,\u{a} 5e-324\u{a}]",
        ),
        (
            "[NaN, Infinity, -Infinity, 5e-324]",
            None,
            true,
            false,
            "[NaN, Infinity, -Infinity, 5e-324]",
        ),
        ("[]", None, false, false, "[]"),
        ("[]", Some(2), true, true, "[]"),
        ("[]", Some(1), false, true, "[]"),
        ("[]", None, true, false, "[]"),
        ("{}", None, false, false, "{}"),
        ("{}", Some(2), true, true, "{}"),
        ("{}", Some(1), false, true, "{}"),
        ("{}", None, true, false, "{}"),
        ("\u{22}x\u{22}", None, false, false, "\u{22}x\u{22}"),
        ("\u{22}x\u{22}", Some(2), true, true, "\u{22}x\u{22}"),
        ("\u{22}x\u{22}", Some(1), false, true, "\u{22}x\u{22}"),
        ("\u{22}x\u{22}", None, true, false, "\u{22}x\u{22}"),
        ("7", None, false, false, "7"),
        ("7", Some(2), true, true, "7"),
        ("7", Some(1), false, true, "7"),
        ("7", None, true, false, "7"),
    ];
    for (doc, indent, ascii, sort, want) in cases {
        let value = loads_standard(doc).unwrap();
        assert_eq!(
            text(dumps_with(&value, *indent, *ascii, *sort)),
            *want,
            "{doc} {indent:?} {ascii} {sort}"
        );
    }
}

#[test]
fn type_names_are_python_s() {
    assert_eq!(type_name(&loads_standard("null").unwrap()), "NoneType");
    assert_eq!(type_name(&loads_standard("true").unwrap()), "bool");
    assert_eq!(type_name(&loads_standard("1").unwrap()), "int");
    assert_eq!(type_name(&loads_standard("1.5").unwrap()), "float");
    assert_eq!(type_name(&loads_standard("\u{22}s\u{22}").unwrap()), "str");
    assert_eq!(type_name(&loads_standard("[]").unwrap()), "list");
    assert_eq!(type_name(&loads_standard("{}").unwrap()), "dict");
}

#[test]
fn repr_points_quotes_as_python_does() {
    assert_eq!(
        repr_points(&[0x61, 0x27, 0x62, 0x22, 0x63, 0x5c, 0x64]),
        "'a\u{5c}'b\u{22}c\u{5c}\u{5c}d'"
    );
    assert_eq!(
        repr_points(&[0x69, 0x74, 0x27, 0x73, 0x20, 0x5c]),
        "\u{22}it's \u{5c}\u{5c}\u{22}"
    );
    assert_eq!(
        repr_points(&[0x73, 0x61, 0x79, 0x20, 0x22, 0x68, 0x69, 0x22]),
        "'say \u{22}hi\u{22}'"
    );
    assert_eq!(repr_points(&[0xd800, 0x78]), "'\u{5c}ud800x'");
}

#[test]
fn float_to_int_truncates_as_int_does() {
    assert_eq!(float_to_int(f64::from_bits(0x1)).to_string(), "0");
    assert_eq!(
        float_to_int(f64::from_bits(0xc000000000000000)).to_string(),
        "-2"
    );
    assert_eq!(
        float_to_int(f64::from_bits(0x8000000000000000)).to_string(),
        "0"
    );
    assert_eq!(
        float_to_int(f64::from_bits(0x43b0000000000000)).to_string(),
        "1152921504606846976"
    );
    assert_eq!(
        float_to_int(f64::from_bits(0xc00e000000000000)).to_string(),
        "-3"
    );
}

#[test]
fn equality_follows_python() {
    let v = |doc: &str| loads_standard(doc).unwrap();
    assert!(py_eq(&v("null"), &v("null")), "null == null");
    assert!(
        !py_eq(&v("null"), &v("\u{22}a\u{22}")),
        "null == \u{22}a\u{22}"
    );
    assert!(py_eq(&v("1.5"), &v("1.5")), "1.5 == 1.5");
    assert!(!py_eq(&v("1.5"), &v("2.5")), "1.5 == 2.5");
    assert!(py_eq(&v("1"), &v("1.0")), "1 == 1.0");
    assert!(py_eq(&v("[1.5]"), &v("[1.5]")), "[1.5] == [1.5]");
    assert!(py_eq(&v("true"), &v("1")), "true == 1");
    assert!(!py_eq(&v("[null]"), &v("[0]")), "[null] == [0]");
    // A NaN is unequal to itself, but a list holding it compares the item by identity.
    let nan = v("NaN");
    assert!(!py_eq(&nan, &nan));
    assert!(item_eq(&nan, &nan));
}

#[test]
fn a_run_of_lone_surrogates_is_one_encode_error() {
    assert_eq!(
        utf8_strict_error(&[0x61, 0xd800, 0xd801, 0x62]).as_deref(),
        Some("'utf-8' codec can't encode characters in position 1-2: surrogates not allowed")
    );
    assert_eq!(
        encode_print(&[0x61, 0xd800, 0xd801, 0x62]).unwrap_err(),
        "'utf-8' codec can't encode characters in position 1-2: surrogates not allowed"
    );
    assert_eq!(
        utf8_strict_error(&[0xd800, 0xdc80]).as_deref(),
        Some("'utf-8' codec can't encode characters in position 0-1: surrogates not allowed")
    );
    assert_eq!(
        encode_print(&[0xd800, 0xdc80]).unwrap_err(),
        "'utf-8' codec can't encode characters in position 0-1: surrogates not allowed"
    );
    assert_eq!(
        utf8_strict_error(&[0x61, 0xd800]).as_deref(),
        Some(
            "'utf-8' codec can't encode character '\u{5c}ud800' in position 1: surrogates not allowed"
        )
    );
    assert_eq!(
        encode_print(&[0x61, 0xd800]).unwrap_err(),
        "'utf-8' codec can't encode character '\u{5c}ud800' in position 1: surrogates not allowed"
    );
}
