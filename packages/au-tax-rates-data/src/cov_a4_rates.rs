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
    // CPython 3.15.0: `json.loads(bytes.fromhex("22eda08022"))` is '\ud800'.
    let lone = pyjson::loads_bytes(&hex("22eda08022")).ok();
    assert_eq!(lone, loads_standard(r#""\ud800""#).ok());
    assert!(lone.is_some());
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

/// What `subprocess.run(command, capture_output=True)` raised under CPython
/// 3.15.0 on Linux before the program ran.
#[cfg(not(windows))]
#[test]
fn a_command_that_cannot_start_raises_as_subprocess_does() {
    let missing = "/nonexistent-rates-browser";
    let not_str = "TypeError: expected str, bytes or os.PathLike object, not int";
    let cases = [
        (
            vec![Json::str(missing)],
            format!("FileNotFoundError: [Errno 2] No such file or directory: '{missing}'"),
        ),
        (
            vec![Json::str("a\0b")],
            String::from("ValueError: embedded null byte"),
        ),
        (vec![Json::Int(BigInt::from(1))], String::from(not_str)),
        (
            vec![Json::str(missing), Json::Int(BigInt::from(2))],
            String::from(not_str),
        ),
    ];
    for (command, want) in cases {
        let got = run_with_limit(&command, Duration::from_secs(5)).map(|_| String::new());
        let got = shown(got).replacen('!', "", 1);
        assert_eq!(got, want);
    }
}

/// The first and last lines CPython 3.15.0 wrote to stderr for an uncaught
/// `ValueError('\ud800x')`, `MemoryError()` and `json.loads('x')`'s error;
/// the exit status is 1.
#[test]
fn an_uncaught_exception_reports_as_cpython_does() {
    let lone = Exc::new("ValueError", vec![0xD800, u32::from('x')]);
    let json = Exc::msg(
        "json.decoder.JSONDecodeError",
        "Expecting value: line 1 column 1 (char 0)",
    );
    let first = "Traceback (most recent call last):";
    let cases = [
        (lone, "ValueError: \\ud800x"),
        (Exc::msg("MemoryError", ""), "MemoryError"),
        (
            json,
            "json.decoder.JSONDecodeError: Expecting value: line 1 column 1 (char 0)",
        ),
    ];
    for (e, last) in cases {
        assert_eq!(traceback(&e), format!("{first}\n{last}\n"));
        assert_eq!(report(Err(e)), 1);
    }
    assert_eq!(report(Ok(3)), 3);
}

/// pathlib joins a snapshot name holding a lone surrogate as it joins any
/// other name (CPython 3.15.0: `str(Path('/r/snapshots') / 'a\ud800.txt')`,
/// its `.name` and `.relative_to('/r')`); no OS path can hold it.
#[test]
fn a_snapshot_name_with_a_lone_surrogate_joins_like_any_name() {
    let ctx = Ctx::new();
    let url = |last: u32| p!("https://example.gov.au/rates/a", vec![last], "/");
    let plain = snapshot_path(&ctx, &url(0x2603)).expect("plain name");
    let lone = snapshot_path(&ctx, &url(0xD800)).expect("lone surrogate");
    let swap = |text: &S| -> S {
        let swap = |&c: &u32| if c == 0x2603 { 0xD800 } else { c };
        text.iter().map(swap).collect()
    };
    assert_eq!(lone.display, swap(&plain.display));
    assert_eq!(lone.relative, plain.relative.clone().map(|t| swap(&t)));
    assert_eq!(lone.name, p!("a", vec![0xD800], ".txt"));
    assert!(plain.fs.is_some());
    assert_eq!(lone.fs, None);
}

/// A URL already using every stand-in code point leaves none for its lone
/// surrogate: a refusal, not a wrong path.
#[test]
fn a_url_with_no_free_stand_in_is_refused() {
    let mut url = pts("https://example.gov.au/");
    url.extend((0x4_0000..0xE_0000).chain(0xF_0000..=0x10_FFFF));
    url.push(0xD800);
    let e = url_path(&url).expect_err("refused");
    let text: String = e.text.iter().filter_map(|&c| char::from_u32(c)).collect();
    assert_eq!(
        (e.name(), text.as_str()),
        (
            "ValueError",
            "no code point is free to stand in for a lone surrogate"
        )
    );
}

/// `show` of a bracket from 10**5000 fails as CPython's `str(from - 1)` does.
#[test]
fn a_bracket_too_long_to_print_raises_the_digit_limit() {
    let from = calc(Decimal::parse("1E+5000")).map_err(|e| e.name());
    let from = from.expect("a decimal");
    let zero = decimal(0);
    let figure = Figure::Scale(vec![(from, None, zero.clone(), decimal(1))]);
    let e = show(&figure).expect_err("refused");
    let text: String = e.text.iter().filter_map(|&c| char::from_u32(c)).collect();
    let limit = "Exceeds the limit (4300 digits) for integer string conversion; \
                 use sys.set_int_max_str_digits() to increase the limit";
    assert_eq!((e.name(), text.as_str()), ("ValueError", limit));
}
