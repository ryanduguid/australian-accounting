// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Names the catalogue does not have: `periods` quotes an unknown worksheet
//! as Python's `repr()` would, escapes included, and the metadata lookups give
//! nothing for it.
use australian_tax_calculators::{api, metadata};
use serde_json::json;

#[test]
fn an_unknown_worksheet_is_quoted_with_its_escapes() {
    let error = api::evaluate_coded("periods", &json!({"kind": "a\b\tc\nd\re"})).unwrap_err();
    assert_eq!(error.code, "worksheet.unknown");
    assert_eq!(error.message, r"unknown worksheet 'a\b\tc\nd\re'");
}

#[test]
fn metadata_has_nothing_for_an_unknown_worksheet() {
    assert_eq!(metadata::source_review("nope"), ("", ""));
    assert!(metadata::input_units("nope").is_empty());
    assert!(metadata::examples("nope").is_empty());
}
