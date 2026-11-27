// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `repr()` and `len()` against the answers CPython 3.15 gave.

use super::*;

#[test]
fn repr_escapes_backslash_tab_newline_and_return() {
    assert_eq!(repr("a\\b\tc\nd\re"), r"'a\\b\tc\nd\re'");
}

#[test]
fn len_counts_code_points() {
    assert_eq!(py_len("\u{10F7FF}\u{10F800}"), 2);
    assert_eq!(py_len("caf\u{e9}"), 4);
}
