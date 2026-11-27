// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `textwrap.wrap` on words longer than the line: each expected value is what
//! CPython 3.15's `textwrap.wrap(text, width)` returned.
use lodgeit_calculator_adapter::textwrap::wrap;

#[test]
fn long_words_break_after_a_hyphen_that_fits_as_in_cpython() {
    let rows: &[(&str, usize, &[&str])] = &[
        ("1-2-3-4-5-6-7-8-9", 5, &["1-2-", "3-4-", "5-6-", "7-8-9"]),
        ("see x-1234567 now", 4, &["see", "x-", "1234", "567", "now"]),
        ("a1-b2-c3-d4", 4, &["a1-", "b2-", "c3-", "d4"]),
        // A leading hyphen is not a break point.
        ("-12345678", 4, &["-123", "4567", "8"]),
        ("ab-cd-1234567", 5, &["ab-cd", "-1234", "567"]),
    ];
    for &(text, width, want) in rows {
        assert_eq!(wrap(text, width), want, "{text:?} at {width}");
    }
}
