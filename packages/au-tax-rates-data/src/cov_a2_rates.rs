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
    let text = |key: &Json| key.as_str().expect("text keys").to_string();
    let keys = iterate(base).expect("a record is a dict");
    let mut pairs: Vec<(PyStr, Json)> = Vec::new();
    for key in keys.iter().map(text) {
        let value = base.at(&key).clone();
        pairs.push((PyStr::Text(key), value));
    }
    let set = case.at("set");
    for key in iterate(set).expect("set is a dict").iter().map(text) {
        let value = set.at(&key).clone();
        match pairs
            .iter_mut()
            .find(|(k, _)| k.as_str() == Some(key.as_str()))
        {
            Some(pair) => pair.1 = value,
            None => pairs.push((PyStr::Text(key), value)),
        }
    }
    let drop = iterate(case.at("drop")).expect("drop is a list");
    pairs.retain(|(k, _)| !drop.contains(&Json::Str(k.clone())));
    Json::Object(pairs)
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
    let shown = format!("{} of {}:\n{}", wrong.len(), cases.len(), wrong.join("\n"));
    assert!(wrong.is_empty(), "{shown}");
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

const ATO_CASES: &str = include_str!("../tests/data/cov_a2_ato_table_report.json");

/// The base tables with each `[table, row, column, value]` cell replaced.
fn edited(tables: &Json, cells: &Json) -> Json {
    let index = |n: &Json| {
        as_int(n)
            .and_then(|n| usize::try_from(n).ok())
            .expect("an index")
    };
    let mut out = tables.clone();
    for cell in iterate(cells).expect("cells are a list") {
        let cell = iterate(&cell).expect("a cell");
        let name = cell[0].as_str().expect("a table name");
        let table = out.at(name);
        let mut rows = iterate(table.at("rows")).expect("rows");
        let mut values = iterate(&rows[index(&cell[1])]).expect("a row");
        values[index(&cell[2])] = cell[3].clone();
        rows[index(&cell[1])] = Json::Array(values);
        let columns = table.at("columns").clone();
        let table = Json::obj([("columns", columns), ("rows", Json::Array(rows))]);
        let keys = iterate(&out).expect("tables are a dict");
        let pairs = keys.iter().map(|key| {
            let key = key.as_str().expect("a table name");
            let value = if key == name {
                table.clone()
            } else {
                out.at(key).clone()
            };
            (PyStr::Text(key.to_string()), value)
        });
        out = Json::Object(pairs.collect());
    }
    out
}

/// `(name, table)` pairs, in the dict's order.
fn table_pairs(tables: &Json) -> Vec<(String, Json)> {
    let keys = iterate(tables).expect("tables are a dict");
    let name = |key: &Json| key.as_str().expect("a table name").to_string();
    keys.iter()
        .map(|k| (name(k), tables.at(&name(k)).clone()))
        .collect()
}

#[test]
fn ato_table_report_answers_as_rates_py_does() {
    let doc = loads_standard(ATO_CASES).expect("case file");
    let base = doc.at("base");
    let cases = iterate(doc.at("cases")).expect("a list of cases");
    assert!(cases.len() > 400, "{} cases", cases.len());
    let mut wrong = Vec::new();
    for case in &cases {
        let records = iterate(case.get("records").unwrap_or(base.at("records")));
        let records = records.expect("records are a list");
        let tables = match case.get("tables") {
            Some(tables) => tables.clone(),
            None => edited(base.at("tables"), case.at("cells")),
        };
        let pairs = table_pairs(&tables);
        let pairs: Vec<(&str, Json)> = pairs.iter().map(|(k, v)| (k.as_str(), v.clone())).collect();
        let got = match ato_table_report(&records, &pairs) {
            Ok((lines, findings)) => [lines, vec![pts(&format!("findings: {findings}"))]].concat(),
            Err(e) => answer(Err(e)),
        };
        let want = iterate(case.at("want")).expect("a list of lines");
        let want: Vec<S> = want.iter().map(pyvalue::str_points).collect();
        let shown = repr(&Json::Array(got.iter().cloned().map(json_str).collect()));
        wrong.extend((got != want).then(|| format!("{} got {shown}", repr(case.at("why")))));
    }
    let shown = format!("{} of {}:\n{}", wrong.len(), cases.len(), wrong.join("\n"));
    assert!(wrong.is_empty(), "{shown}");
}
