// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `validate_record` against what `rates.py` (24fcf24) returned or raised
//! under CPython 3.15 for malformed variants of the shipped records: wrong
//! types and missing keys, values for each unit, tax scales, periods, source
//! addresses, timestamps, quotes and value patterns (which also run the
//! regular expression port). `tests/data/cov_a2_validate_record.json` holds
//! the cases; a script that ran each through `rates.py` wrote it.

use super::*;
use crate::pyjson::loads_standard;

const CASES: &str = include_str!("../tests/data/cov_a2_validate_record.json");

/// `dict(record, **set)` without the `drop` keys, keeping Python's key order.
fn variant(base: &Json, case: &Json) -> Json {
    let mut record = base.clone();
    if let Json::Object(pairs) = &mut record {
        if let Some(Json::Object(set)) = case.get("set") {
            for (key, value) in set {
                match pairs.iter_mut().find(|(k, _)| k == key) {
                    Some(pair) => pair.1 = value.clone(),
                    None => pairs.push((key.clone(), value.clone())),
                }
            }
        }
        if let Some(Json::Array(drop)) = case.get("drop") {
            pairs.retain(|(k, _)| !drop.contains(&Json::Str(k.clone())));
        }
    }
    record
}

/// The problems, or `!Class: message` for the exception Python raised.
fn answer(got: R<Vec<S>>) -> Vec<S> {
    match got {
        Ok(problems) => problems,
        Err(e) => vec![[pts("!"), pts(e.name()), pts(": "), e.text].concat()],
    }
}

#[test]
fn validate_record_answers_as_rates_py_does() {
    let ctx = Ctx::new();
    let records = load_records(&ctx).expect("the committed records load");
    let doc = loads_standard(CASES).expect("case file");
    let cases = iterate(doc.at("cases")).expect("a list of cases");
    assert!(cases.len() > 2000, "{} cases", cases.len());
    let wrong: Vec<String> = cases
        .iter()
        .filter_map(|case| check(&ctx, &records, case))
        .collect();
    let shown = wrong.join("\n");
    assert!(
        wrong.is_empty(),
        "{} of {}:\n{shown}",
        wrong.len(),
        cases.len()
    );
}

/// `None` when `validate_record` answers the case as `rates.py` did.
fn check(ctx: &Ctx, records: &[Json], case: &Json) -> Option<String> {
    let id = case.at("base");
    let base = records.iter().find(|r| r.get("id") == Some(id));
    let base = base.expect("base record");
    let page = if *case.at("page") == Json::Null {
        None
    } else if case.at("page").as_str() == Some("snapshot") {
        let url = pyvalue::str_points(base.at("source_url"));
        let snap = snapshot_path(ctx, &url).expect("snapshot path");
        Some(pts(&snap.read().expect("snapshot text")))
    } else {
        Some(pyvalue::str_points(case.at("page")))
    };
    let want = iterate(case.at("want")).expect("a list of lines");
    let want: Vec<S> = want.iter().map(pyvalue::str_points).collect();
    let record = variant(base, case);
    let got = answer(validate_record(&record, page.as_deref()));
    let shown = repr(&Json::Array(got.iter().cloned().map(json_str).collect()));
    (got != want).then(|| format!("{} got {shown}", repr(case.at("why"))))
}
