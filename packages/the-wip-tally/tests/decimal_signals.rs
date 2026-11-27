// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Decimal signals the Python original raised uncaught. Each expectation was
//! read from CPython running the 0.2.0 Python package on the same input.

use serde_json::{Value, json};
use the_wip_tally::csv_io::Failure;
use the_wip_tally::decimal::Signal;
use the_wip_tally::{api, cli, money};

/// A measurable cost-to-cost contract with every amount zero but the price.
fn contract(changes: &[(&str, &str)]) -> Value {
    let mut v = json!({
        "contract_id": "J1", "line_number": 2, "customer": "", "description": "",
        "original_contract_sum": "1000", "constraint_include_ratio": "1",
        "gst_rate": "0.1", "retention_classification": "review",
        "progress_method": "cost_to_cost", "outcome_reasonably_measurable": true,
    });
    for key in [
        "approved_variations",
        "unapproved_variations_estimate",
        "costs_incurred",
        "inefficiency_rework_wastage",
        "uninstalled_materials",
        "estimated_cost_to_complete",
        "certified_billings",
        "uncertified_claims",
        "retention_withheld",
        "committed_outstanding",
    ] {
        v[key] = json!("0");
    }
    for (key, value) in changes {
        v[*key] = json!(value);
    }
    v
}

#[test]
fn variable_consideration_too_large_for_cents_raises_invalid_operation() {
    // quantize to cents of 1E+27 needs 30 digits, past the 28-digit context.
    let v = contract(&[("unapproved_variations_estimate", "1E27")]);
    assert_eq!(
        api::evaluate("measure", &v),
        Err("decimal.InvalidOperation".to_string())
    );
}

#[test]
fn stale_spend_too_large_for_cents_raises_invalid_operation() {
    // costs_incurred - prior_costs_incurred is -1E+27 when the remaining cost is unchanged.
    let v = contract(&[
        ("estimated_cost_to_complete", "10"),
        ("prior_costs_incurred", "1E27"),
        ("prior_estimated_cost_to_complete", "10"),
    ]);
    assert_eq!(
        api::evaluate("measure", &v),
        Err("decimal.InvalidOperation".to_string())
    );
}

#[test]
fn revenue_rounding_past_the_largest_cents_value_raises_invalid_operation() {
    // 99999999999999999999999999.99 less 0.015 rounds to .98 at 28 digits; adding
    // the 0.015 back rounds to 1E+26, which cents cannot hold.
    let v = contract(&[
        ("original_contract_sum", "99999999999999999999999999.99"),
        ("costs_incurred", "1"),
        ("uninstalled_materials", "0.015"),
    ]);
    assert_eq!(
        api::evaluate("measure", &v),
        Err("decimal.InvalidOperation".to_string())
    );
}

#[test]
fn a_million_digit_percentage_overflows_the_ratio_parser() {
    let raw = format!("-{}%", "1".repeat(1_000_001));
    let error = money::parse_ratio(&raw, "ratio").unwrap_err();
    assert_eq!(error, money::AmountError::Decimal(Signal::Overflow));
    assert_eq!(error.to_string(), "decimal.Overflow");
    assert_eq!(Failure::from(error), Failure::Decimal(Signal::Overflow));
    assert_eq!(
        money::AmountError::Message("ratio: no ratio given".into()).to_string(),
        "ratio: no ratio given"
    );
}

#[test]
fn schedule_cli_reports_an_uncaught_signal_and_writes_nothing() {
    let folder = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("wip-signal-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let source = folder.join("stale.csv");
    let output = folder.join("schedule.csv");
    std::fs::write(
        &source,
        "contract_id,original_contract_sum,costs_incurred,estimated_cost_to_complete,\
         certified_billings,outcome_reasonably_measurable,prior_costs_incurred,\
         prior_estimated_cost_to_complete\n\
         STALE-1,1000,90000000000000000000000000.00,0,0,yes,(90000000000000000000000000.00),0\n",
    )
    .unwrap();
    let (code, stdout, stderr) = cli::run(&[
        "schedule".into(),
        source.to_string_lossy().into(),
        "--as-at".into(),
        "2026-10-25".into(),
        "--output".into(),
        output.to_string_lossy().into(),
    ]);
    assert_eq!(
        (code, stdout.as_str(), stderr.as_str()),
        (1, "", "error: decimal.InvalidOperation\n")
    );
    assert!(!output.exists());
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn schedule_as_at_skips_blank_rows_as_dict_reader_does() {
    let folder = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("wip-asat-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    let path = folder.join("schedule.csv");
    std::fs::write(&path, b"as_at,x\r\n\r\n2026-09-30,1\r\n").unwrap();
    let path = the_wip_tally::py::PyPath::new(&path.to_string_lossy());
    assert_eq!(cli::schedule_as_at(&path).as_deref(), Some("2026-09-30"));
    std::fs::remove_dir_all(folder).unwrap();
}

#[cfg(unix)]
#[test]
fn an_undecodable_argument_byte_is_echoed_as_its_surrogate_escape() {
    use std::os::unix::ffi::OsStrExt;
    // CPython: b"a\xffb" decodes to "a\udcffb", which backslashreplace writes back.
    let text = cli::decode_os(std::ffi::OsStr::from_bytes(b"a\xffb"));
    assert_eq!(cli::output_bytes(&text), b"a\\udcffb");
}
