// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `super-projection`, `objection-deadline` and `lessee-lease` against the
//! site's own engines (ryanduguid.github.io `main` at d876df72, checked
//! 26 November 2026): `data/site_mirror_cases.json` holds every stored
//! case of the site's super projection, objection deadline and AASB 16 lease
//! pages, and 220 seeded cases, each run through the site's
//! `superProjection`, `objectionDeadline` and `lease` and written with the
//! site's figures as JSON pointers into this crate's output. A case the site
//! refuses must be refused here too, with the code the fixture gives.
use australian_tax_calculators::api;
use serde_json::Value;

#[test]
fn every_site_case_agrees() {
    let cases: Vec<Value> =
        serde_json::from_str(include_str!("data/site_mirror_cases.json")).expect("cases parse");
    assert_eq!(cases.len(), 249);
    for case in &cases {
        let (command, origin) = (case["command"].as_str().unwrap(), &case["origin"]);
        if let Some(code) = case.get("code") {
            let err = api::evaluate_coded(command, &case["input"]).unwrap_err();
            assert_eq!(err.code, code, "{command} {origin}: the site refuses it");
            continue;
        }
        let out = api::evaluate(command, &case["input"]);
        let out = out.unwrap_or_else(|e| panic!("{command} {origin}: {e}"));
        for (pointer, want) in case["expected"].as_object().unwrap() {
            let got = match pointer.as_str() {
                "/income_years_count" => Value::from(out["income_years"].as_array().unwrap().len()),
                "/schedule_count" => Value::from(out["schedule"].as_array().unwrap().len()),
                p => out.pointer(p).cloned().unwrap_or(Value::Null),
            };
            assert_eq!(&got, want, "{command} {origin} {pointer}");
        }
    }
}
