// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The shared `py.rs` (the copy au-tax-rates-data and the lodgeit adapter
//! hold). The same file runs in both crates, so each crate's run measures its
//! own copy. Every expected value is what CPython 3.15 gives.
use lodgeit_calculator_adapter::py::{self, Date, ReadError};
use num_bigint::BigInt;

#[test]
fn character_classes_match_python() {
    // re.fullmatch(r"\w", c) for each character, and int()'s digit value.
    for (c, word) in [
        ('a', true),
        ('_', true),
        ('é', true),
        ('٣', true),
        ('-', false),
        (' ', false),
    ] {
        assert_eq!(py::is_word(c), word, "{c:?}");
    }
    assert_eq!(py::decimal_value('a'), None);
    assert_eq!(py::decimal_value('7'), Some(7));
}

#[test]
fn slices_and_lengths_count_code_points() {
    assert_eq!(py::prefix_chars("abcdef", 3), "abc");
    assert_eq!(py::prefix_chars("ab", 3), "ab");
    assert_eq!(py::char_len("aé€😀"), 4);
}

#[test]
fn repr_escapes_as_python_does() {
    assert_eq!(py::repr_str("\r"), "'\\r'");
    // U+E0001 LANGUAGE TAG is not printable and lies beyond the BMP.
    assert_eq!(py::repr_str("\u{e0001}"), "'\\U000e0001'");
    assert_eq!(py::repr_str("\u{10ffff}"), "'\\U0010ffff'");
}

#[test]
fn float_repr_matches_python() {
    assert_eq!(py::float_repr(f64::INFINITY), "inf");
    assert_eq!(py::float_repr(f64::NEG_INFINITY), "-inf");
    assert_eq!(py::float_repr(f64::NAN), "nan");
    // 2**-1017: its 16-digit shortest spelling is not the nearest 16-digit
    // decimal, which does not read back as the same double.
    assert_eq!(py::float_repr(2f64.powi(-1017)), "7.120236347223045e-307");
    assert_eq!(py::float_repr(2f64.powi(-1007)), "7.291122019556398e-304");
}

#[test]
fn int_parses_signs_prefixes_and_refusals_as_python_does() {
    let int = |s: &str, base: u32| py::parse_int(s, base);
    assert_eq!(int("-5", 10).unwrap(), BigInt::from(-5));
    assert_eq!(int("+5", 10).unwrap(), BigInt::from(5));
    assert_eq!(int("0x1f", 16).unwrap(), BigInt::from(31));
    assert_eq!(int("0o17", 8).unwrap(), BigInt::from(15));
    assert_eq!(int("1f", 16).unwrap(), BigInt::from(31));
    assert_eq!(int("0x_f", 16).unwrap(), BigInt::from(15));
    for bad in ["-", "1_", ""] {
        assert_eq!(
            int(bad, 10).unwrap_err(),
            format!(
                "invalid literal for int() with base 10: {}",
                py::repr_str(bad)
            )
        );
    }
    assert_eq!(
        int(&"1".repeat(4301), 10).unwrap_err(),
        "Exceeds the limit (4300 digits) for integer string conversion: value has 4301 \
         digits; use sys.set_int_max_str_digits() to increase the limit"
    );
}

#[test]
fn int_repr_refuses_more_than_4300_digits_as_python_does() {
    let limit: BigInt = format!("1{}", "0".repeat(4299)).parse().unwrap();
    assert_eq!(py::int_repr(&limit).unwrap().len(), 4300);
    assert_eq!(
        py::int_repr(&(limit * 10)).unwrap_err(),
        "Exceeds the limit (4300 digits) for integer string conversion; use \
         sys.set_int_max_str_digits() to increase the limit"
    );
}

#[test]
fn float_parses_signs_specials_and_refusals_as_python_does() {
    assert_eq!(py::parse_float("-1.5").unwrap(), -1.5);
    assert_eq!(py::parse_float("+2").unwrap(), 2.0);
    assert_eq!(py::parse_float("inf").unwrap(), f64::INFINITY);
    assert_eq!(py::parse_float("-Infinity").unwrap(), f64::NEG_INFINITY);
    assert!(py::parse_float("+NaN").unwrap().is_nan());
    assert_eq!(py::parse_float(".5").unwrap(), 0.5);
    assert_eq!(py::parse_float("-.5").unwrap(), -0.5);
    for bad in ["1e", "1e+"] {
        assert_eq!(
            py::parse_float(bad).unwrap_err(),
            format!("could not convert string to float: '{bad}'")
        );
    }
}

#[test]
fn text_mode_reads_files_with_universal_newlines() {
    let dir = std::env::temp_dir().join(format!("family-py-rates-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let text = dir.join("text.txt");
    std::fs::write(&text, b"a\r\nb\rc\n").unwrap();
    assert_eq!(py::read_text(&text).unwrap(), "a\nb\nc\n");
    let bad = dir.join("bad.txt");
    std::fs::write(&bad, b"ab\xff").unwrap();
    let Err(ReadError::Decode(e)) = py::read_text(&bad) else {
        panic!("invalid UTF-8 is a decode error");
    };
    assert_eq!(
        e.to_string(),
        "'utf-8' codec can't decode byte 0xff in position 2: invalid start byte"
    );
    let missing = dir.join("missing.txt");
    let Err(ReadError::Os(e)) = py::read_text(&missing) else {
        panic!("a missing file is an OS error");
    };
    assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn dates_refuse_a_bad_month_and_read_ordinals_as_python_does() {
    assert_eq!(
        Date::new(2026, 13, 1).unwrap_err(),
        "month must be in 1..12, not 13"
    );
    assert_eq!(
        Date::new(2026, 0, 1).unwrap_err(),
        "month must be in 1..12, not 0"
    );
    // date.fromordinal(36160) is 0100-01-01, where the year estimate is one low.
    let first_of_100 = Date::from_ordinal(36160).unwrap();
    assert_eq!(first_of_100.isoformat(), "0100-01-01");
    assert_eq!(first_of_100.ordinal(), 36160);
    assert_eq!(Date::from_ordinal(1).unwrap().isoformat(), "0001-01-01");
    assert_eq!(
        Date::from_ordinal(3_652_059).unwrap().isoformat(),
        "9999-12-31"
    );
    assert_eq!(Date::from_ordinal(3_652_060), None);
}
