// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `charity-check` against the committed copy of the ACNC register. A new copy
//! fails these until its date, counts and digest are rechecked and updated.
use australian_tax_calculators::api;
use serde_json::{Value, json};

const DATA: &str = include_str!("../data/acnc-register.jsonl");

fn header() -> Value {
    serde_json::from_str(DATA.lines().next().unwrap()).unwrap()
}

#[test]
fn the_committed_copy_is_the_one_read_and_attributed() {
    let first = header();
    assert_eq!(first["snapshot_date"], "2026-11-22");
    assert_eq!(first["last_modified"], "2026-11-22T17:39:55");
    assert_eq!(
        first["csv_sha256"],
        "559c29245442d98abc473ffa60a2832b4d25ead3ebbf6734dd2857237fef328d"
    );
    assert_eq!(first["rows_read"], 66_645);
    let kept = first["charities"].as_u64().unwrap();
    let dropped = first["abn_withheld_dropped"].as_u64().unwrap()
        + first["abn_invalid_dropped"].as_u64().unwrap();
    assert_eq!(first["abn_invalid_dropped"], 2);
    assert_eq!(kept + dropped, 66_645);
    assert_eq!(DATA.lines().count() as u64, kept + 1);
    assert_eq!(
        first["licence_url"],
        "https://creativecommons.org/licenses/by/3.0/au/"
    );
    for line in DATA.lines().skip(1) {
        let row: Value = serde_json::from_str(line).unwrap();
        assert_eq!(row.as_array().unwrap().len(), 4, "{line}");
    }
}

#[test]
fn a_registered_charity_reads_as_on_the_copy() {
    // Australian Red Cross Society, registered from 3 December 2012 on the ACNC
    // register as ABN Lookup showed it on 24 November 2026.
    let out = api::evaluate(
        "charity-check",
        &json!({"abns": ["50 169 561 394"], "as_at": "2026-11-24"}),
    )
    .unwrap();
    let result = &out["charities"][0];
    assert_eq!(result["on_register_snapshot"], true);
    assert_eq!(result["legal_name"], "Australian Red Cross Society");
    assert_eq!(result["registration_date"], "2012-12-03");
    assert_eq!(
        result["subtypes"][0]["subtype"],
        "public benevolent institution"
    );
    assert_eq!(out["data"]["snapshot_date"], "2026-11-22");
    assert_eq!(out["data"]["attribution"], header()["attribution"]);
}

#[test]
fn the_committed_copy_is_refused_once_stale() {
    let refused = api::evaluate_coded(
        "charity-check",
        &json!({"abns": ["50169561394"], "as_at": "2026-12-24"}),
    )
    .unwrap_err();
    assert_eq!(refused.code, "charity_check.snapshot_stale");
}
