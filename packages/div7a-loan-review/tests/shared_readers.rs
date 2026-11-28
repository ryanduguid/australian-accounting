// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The Python-compatibility code this crate shares with its siblings (`py`,
//! `pyjson`, `csvread`), tested from outside so the shared files stay the same
//! bytes. Each expected value and message is what CPython 3.15 gives.

use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use div7a_loan_review::Failure;
use div7a_loan_review::py::{self, OsStyle, PyPath};
use div7a_loan_review::{rates, register};
use num_bigint::BigInt;

const SEP: char = std::path::MAIN_SEPARATOR;

/// A fresh file holding `data`, named for this process and call.
fn file(data: &[u8]) -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("div7a-shared-{}-{n}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("scratch directory");
    let path = dir.join("input");
    std::fs::write(&path, data).expect("scratch file");
    path
}

fn py_path(path: &std::path::Path) -> PyPath {
    PyPath::new(&path.to_string_lossy())
}

#[test]
fn int_reads_signs_prefixes_and_unicode_digits() {
    for (text, base, want) in [
        ("-12", 10, -12),
        ("+7", 10, 7),
        ("0xff", 16, 255),
        ("0o17", 8, 15),
        ("0X_1f", 16, 31),
        ("ff", 16, 255),
        ("1_000", 10, 1000),
        ("\u{661}\u{662}", 10, 12),
    ] {
        assert_eq!(py::parse_int(text, base), Ok(BigInt::from(want)), "{text}");
    }
}

#[test]
fn int_refusals_read_as_python_words_them() {
    for (text, shown) in [
        ("1__0", "'1__0'"),
        ("1_", "'1_'"),
        ("_1", "'_1'"),
        ("12a", "'12a'"),
        ("", "''"),
        ("1\u{20ac}", "'1\u{20ac}'"),
    ] {
        assert_eq!(
            py::parse_int(text, 10).unwrap_err(),
            format!("invalid literal for int() with base 10: {shown}"),
            "{text}"
        );
    }
    let quoted = format!("{}'", "\u{e9}".repeat(150));
    assert_eq!(
        py::parse_int(&quoted, 10).unwrap_err(),
        format!("invalid literal for int() with base 10: \"{quoted}\"")
    );
    assert_eq!(
        py::parse_int(&"1".repeat(4301), 10).unwrap_err(),
        "Exceeds the limit (4300 digits) for integer string conversion: value has 4301 \
         digits; use sys.set_int_max_str_digits() to increase the limit"
    );
}

#[test]
fn repr_escapes_newlines_and_unprintable_astral_characters() {
    assert_eq!(py::repr_str("a\nb\rc"), "'a\\nb\\rc'");
    assert_eq!(py::repr_str("\u{e0001}"), "'\\U000e0001'");
}

#[test]
fn an_error_without_an_os_code_prints_its_own_message() {
    let error = std::io::Error::other("no code here");
    assert_eq!(
        py::oserror_str(&error, "f.csv", OsStyle::Errno),
        "no code here"
    );
}

#[test]
fn joining_an_anchored_path_replaces_the_base() {
    assert_eq!(
        PyPath::new("a").join("/etc/x").to_string(),
        format!("{SEP}etc{SEP}x")
    );
}

#[cfg(unix)]
#[test]
fn two_leading_slashes_stay_an_anchor_on_posix() {
    assert_eq!(PyPath::new("//srv/x").display(), "//srv/x");
    assert_eq!(PyPath::new("///srv/x").display(), "/srv/x");
}

/// What `load_override` says of a file holding `json`.
fn override_error(json: &str) -> String {
    let path = file(json.as_bytes());
    match rates::load_override(&py_path(&path)) {
        Err(Failure::Value(message)) => message.replace(&py_path(&path).display(), "<f>"),
        other => panic!("{json}: not a refusal: {other:?}"),
    }
}

fn json_error(json: &str) -> String {
    let message = override_error(json);
    match message.strip_prefix("<f> is not valid JSON: ") {
        Some(rest) => rest.to_string(),
        None => panic!("{json}: not a JSON error: {message}"),
    }
}

#[test]
fn json_containers_and_literals_parse_before_the_override_checks() {
    let holds = |kind: &str| {
        format!(
            "<f> must be a JSON object with 'verified_until', 'citation' and 'rates'; it \
             holds a {kind}"
        )
    };
    assert_eq!(
        override_error("[true, false, null, {}, -12, -0, 1.5e3, 2E-2, 7e+1]"),
        holds("list")
    );
    assert_eq!(override_error("[Infinity, -Infinity]"), holds("list"));
    assert_eq!(override_error("\"\\u00e9\""), holds("str"));
    assert!(override_error("{\"a\": {}, \"b\": [true]}").starts_with("<f> has no 'citation'"));
}

#[test]
fn json_number_and_position_errors_match_the_c_scanner() {
    assert_eq!(
        json_error("[\n1 2]"),
        "Expecting ',' delimiter: line 2 column 3 (char 4)"
    );
    // An exponent marker with no digits is not part of the number.
    assert_eq!(
        json_error("[1e]"),
        "Expecting ',' delimiter: line 1 column 3 (char 2)"
    );
    assert_eq!(
        json_error("[x]"),
        "Expecting value: line 1 column 2 (char 1)"
    );
    assert_eq!(
        json_error("[-x]"),
        "Expecting value: line 1 column 2 (char 1)"
    );
}

#[test]
fn json_escape_errors_match_the_c_scanner() {
    // `~` stands for the backslash of a JSON \u escape.
    let escaped = |json: &str| json.replace('~', "\\");
    assert_eq!(
        json_error(&escaped(r#""~ud800~u0041""#)),
        "Unpaired surrogate in \\uXXXX escape: line 1 column 2 (char 1)"
    );
    assert_eq!(
        json_error(&escaped(r#""~u12g4""#)),
        "Invalid \\uXXXX escape: line 1 column 3 (char 2)"
    );
    assert_eq!(
        json_error(&escaped(r#""~ud800~u12g4""#)),
        "Invalid \\uXXXX escape: line 1 column 9 (char 8)"
    );
    // Python keeps a lone surrogate; a Rust string cannot hold one.
    for json in [r#""~ud800""#, r#""~udc00""#] {
        assert_eq!(
            json_error(&escaped(json)),
            "Unpaired surrogate in \\uXXXX escape: line 1 column 2 (char 1)",
            "{json}"
        );
    }
}

/// The `name` cell of each loan row `load_rows` reads from `data`.
fn names(data: &[u8]) -> Vec<String> {
    let rows = register::load_rows(&py_path(&file(data)), &["name"]).expect("rows");
    rows.iter()
        .map(|row| row.get("name").expect("a name cell").py_str())
        .collect()
}

#[test]
fn a_carriage_return_at_a_chunk_boundary_joins_its_newline() {
    // The reader decodes 8,192 bytes at a time; the \r is the last of the first chunk.
    let long = "x".repeat(8192 - 7);
    let mut data = format!("name\r\n{long}").into_bytes();
    assert_eq!(data.len(), 8191);
    data.extend(b"\r\ny\n");
    assert_eq!(names(&data), vec![long.clone(), "y".to_string()]);
    // A \r that ends the data still ends its row.
    data.truncate(8192);
    assert_eq!(names(&data), vec![long.clone()]);
    data.extend(b"z");
    assert_eq!(names(&data), vec![long, "z".to_string()]);
    assert_eq!(names(b"name\ra\r"), vec!["a".to_string()]);
}

#[test]
fn a_register_that_is_not_utf8_names_its_bad_byte() {
    let path = file(b"name\n\xff\n");
    let shown = py_path(&path).display();
    assert_eq!(
        register::load_rows(&py_path(&path), &["name"]).unwrap_err(),
        Failure::Value(format!(
            "{shown} is not valid UTF-8 text (invalid byte 0xff). Re-save it as \
             'CSV UTF-8 (Comma delimited)' and run again."
        ))
    );
}

#[test]
fn json_string_escapes_decode_and_a_bad_one_is_placed() {
    let holds_str = "<f> must be a JSON object with 'verified_until', 'citation' and 'rates'; it \
                     holds a str";
    // `~` stands for a backslash.
    let escaped = |json: &str| json.replace('~', "\\");
    assert_eq!(override_error(&escaped(r#""~"~~~/~b~f~n~r~t""#)), holds_str);
    // A surrogate pair joins into one code point.
    assert_eq!(override_error(&escaped(r#""~ud83d~ude00""#)), holds_str);
    assert_eq!(
        json_error(&escaped(r#""~x""#)),
        r"Invalid \escape: line 1 column 2 (char 1)"
    );
}

#[test]
fn a_rate_table_field_over_the_csv_limit_is_refused_as_python_raises_it() {
    let mut text =
        "# reviewed_until: 2026-27\n# reviewed_on: 2026-07-01\nyear_of_income,rate\n".to_string();
    text.push_str(&"x".repeat(131_073));
    text.push('\n');
    let path = file(text.as_bytes());
    assert_eq!(
        rates::load_table(&py_path(&path)).unwrap_err(),
        Failure::Uncaught("_csv.Error: field larger than field limit (131072)".into())
    );
}
