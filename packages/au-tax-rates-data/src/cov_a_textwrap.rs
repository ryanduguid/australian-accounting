// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `textwrap.wrap` on words longer than the width, against CPython 3.15's answers.

use super::*;

#[test]
fn a_long_word_breaks_after_its_last_hyphen_that_fits() {
    assert_eq!(wrap("a-bcdefgh", 4), ["a-", "bcde", "fgh"]);
    assert_eq!(wrap("xy-abcdefgh", 5), ["xy-ab", "cdefg", "h"]);
}

#[test]
fn a_hyphen_with_only_hyphens_before_it_is_not_a_break() {
    assert_eq!(wrap("--abcdefgh", 4), ["--ab", "cdef", "gh"]);
    assert_eq!(wrap("abcdefgh", 3), ["abc", "def", "gh"]);
}
