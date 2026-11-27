// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Command-line refusals that need no stub: an argument that is not valid
//! Unicode, an argument the parser refuses, and request-number paths that are
//! not strings or that walk through a value that is not an object.
use std::ffi::OsString;

use num_bigint::BigInt;

use lodgeit_calculator_adapter::cli;
use lodgeit_calculator_adapter::pyjson::{self, Json};
use lodgeit_calculator_adapter::pyvalue::points_lossy;

#[cfg(unix)]
fn not_unicode() -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(vec![0xff])
}

#[cfg(windows)]
fn not_unicode() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0xd800])
}

#[test]
fn an_argument_that_is_not_unicode_is_refused_with_status_2() {
    assert_eq!(
        cli::main(vec![OsString::from("contract"), not_unicode()]),
        2
    );
}

#[test]
fn an_unknown_option_through_main_is_a_usage_error() {
    assert_eq!(cli::main(vec![OsString::from("--no-such-option")]), 2);
}

#[test]
fn a_number_field_that_is_not_a_string_raises_attribute_error() {
    let body = pyjson::loads_standard(r#"{"amount": "1.00"}"#).expect("valid JSON");
    let error = cli::decimalise(body, &Json::Array(vec![Json::Int(BigInt::from(1))]))
        .expect_err("int has no split");
    assert_eq!(error.class, "AttributeError");
    assert_eq!(
        points_lossy(&error.text),
        "'int' object has no attribute 'split'"
    );
}

#[test]
fn a_number_path_through_a_non_object_leaves_the_body_as_it_was() {
    let body = pyjson::loads_standard(r#"{"loan": 5}"#).expect("valid JSON");
    let converted = cli::decimalise(body.clone(), &Json::Array(vec![Json::str("loan.amount")]))
        .expect("no exception")
        .expect("not refused");
    assert_eq!(converted, body);
}
