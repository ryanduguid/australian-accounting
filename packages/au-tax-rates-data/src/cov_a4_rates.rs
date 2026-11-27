// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The Python-semantics helpers against CPython 3.15.0: `a < b` and `a <= b`,
//! `sorted(set(v))`, hashing a pair, `Decimal(v)` (the tuple form included)
//! and the `UnicodeDecodeError` text of `json.loads(bytes)`.
//! `tests/data/cov_a4_helpers.json` holds each input with what CPython
//! returned or raised; `tests/data/cov_a4_helpers.py` wrote it.

use super::*;
use crate::pyjson::loads_standard;

const CASES: &str = include_str!("../tests/data/cov_a4_helpers.json");

/// `(input, expected)` pairs of one kind of case.
fn cases(kind: &str) -> Vec<(String, String)> {
    let doc = loads_standard(CASES).expect("case file");
    assert_eq!(doc.at("python").as_str(), Some("3.15.0"));
    let list = iterate(doc.at("cases").at(kind)).expect("a list of cases");
    let text = |value: &Json| value.as_str().expect("text").to_string();
    let pair = |case: &Json| {
        let pair = iterate(case).expect("an input and an answer");
        (text(&pair[0]), text(&pair[1]))
    };
    list.iter().map(pair).collect()
}

fn value(text: &str) -> Json {
    loads_standard(text).expect("a JSON value")
}

/// The result as the oracle printed it: `repr(result)` or `!Class: message`.
fn shown(got: R<String>) -> String {
    match got {
        Ok(text) => text,
        Err(e) => {
            let text: String = e
                .text
                .iter()
                .map(|&c| char::from_u32(c).unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect();
            format!("!{}: {text}", e.name())
        }
    }
}

/// Every case of `kind` answers as CPython did.
fn check(kind: &str, run: impl Fn(&str) -> String) {
    let (got, want): (Vec<String>, Vec<String>) = cases(kind)
        .into_iter()
        .map(|(input, want)| {
            (
                format!("{input} {}", run(&input)),
                format!("{input} {want}"),
            )
        })
        .unzip();
    assert_eq!(got, want, "{kind}");
}

fn bool_repr(b: bool) -> String {
    String::from(if b { "True" } else { "False" })
}

fn compare(op: &str) -> impl Fn(&str) -> String {
    move |input| {
        let pair = iterate(&value(input)).expect("a pair");
        shown(py_compare(&pair[0], &pair[1], op).map(bool_repr))
    }
}

#[test]
fn comparisons_answer_as_cpython_does() {
    check("lt", compare("<"));
    check("le", compare("<="));
}

#[test]
fn sorted_set_answers_as_cpython_does() {
    let sorted = |input: &str| {
        let values = iterate(&value(input)).expect("a list");
        shown(sorted_set(values).map(|v| repr(&Json::Array(v))))
    };
    check("sorted_set", sorted);
    // CPython's set order picks which pair the TypeError names, so only the
    // class is compared.
    let class = |input: &str| {
        sorted(input)
            .split(':')
            .next()
            .unwrap_or_default()
            .to_string()
    };
    check("sorted_set_class", class);
}

#[test]
fn a_pair_hashes_as_cpython_hashes_a_tuple() {
    check("tuple_key", |input| {
        let pair = iterate(&value(input)).expect("a pair");
        shown(tuple_key(&[&pair[0], &pair[1]]).map(|_| String::from("1")))
    });
}

#[test]
fn decimal_of_a_json_value_answers_as_cpython_does() {
    check("decimal", |input| {
        shown(to_decimal(&value(input)).map(|d| d.to_string()))
    });
}

fn hex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
        .collect()
}

#[test]
fn utf8_decode_errors_are_worded_as_cpython_words_them() {
    check("bytes", |input| {
        let data = hex(input);
        assert!(matches!(pyjson::loads_bytes(&data), Err(None)), "{input}");
        bytes_decode_error(&data)
    });
}

#[test]
fn a_utf8_encoded_surrogate_decodes_as_surrogatepass_allows() {
    let doc = loads_standard(CASES).expect("case file");
    for input in iterate(doc.at("cases").at("decodes")).expect("a list") {
        let data = hex(input.as_str().expect("hex text"));
        assert!(pyjson::loads_bytes(&data).is_ok(), "{input:?}");
    }
}

const WIDE: &str = "'utf-16' or 'utf-32' codec can't decode bytes";

/// For UTF-16 and UTF-32 only the codec family is named: CPython's text
/// (`'utf-16-le' codec can't decode ...`) names the exact codec and position.
#[test]
fn wide_decode_errors_name_the_codec_family() {
    for (input, python) in cases("wide") {
        let data = hex(&input);
        assert!(matches!(pyjson::loads_bytes(&data), Err(None)), "{input}");
        let codec = python.starts_with("'utf-16-") || python.starts_with("'utf-32-");
        assert!(codec, "{python}");
        assert_eq!(bytes_decode_error(&data), WIDE, "{input}");
    }
}
