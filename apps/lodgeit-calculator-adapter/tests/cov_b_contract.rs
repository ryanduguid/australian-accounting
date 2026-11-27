// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.
//! `contract.check_period` and `contract.load` on unusual snapshots. Each
//! expected value is what the retired Python `lodgeitadapter.contract`
//! returned or raised under CPython 3.15.
use std::path::PathBuf;

use lodgeit_calculator_adapter::contract::{self, Contract};
use lodgeit_calculator_adapter::errors::CONTRACT;
use lodgeit_calculator_adapter::pyjson::{self, Json};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::pypath::PyPath;

fn pts(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn refusal<T>(result: Result<T, Exc>) -> (&'static str, String) {
    match result {
        Ok(_) => panic!("expected a refusal"),
        Err(error) => (error.class, text(&error.text)),
    }
}

fn with_periods(supported: &str) -> Contract {
    let calculators = format!(r#"{{"c": {{"supported_periods": {supported}}}}}"#);
    Contract {
        provider: Json::str("p"),
        snapshot_id: Json::str("s1"),
        read_at: Json::str("t"),
        read_by: Json::str("r"),
        calculators: pyjson::loads_standard(&calculators).expect("valid JSON"),
        response_contract: Json::Object(Vec::new()),
        openapi_input_schemas: None,
        notes: Vec::new(),
        sha256: String::new(),
        path: PyPath::new("x.json"),
    }
}

fn check(supported: &str) -> Result<(), Exc> {
    with_periods(supported).check_period(&pts("c"), &pts("p1"))
}

#[test]
fn supported_periods_of_any_container_answer_python_in() {
    // `"p1" in "p1p2"` is a substring test; `in` a dict tests its keys.
    assert!(check(r#""p1p2""#).is_ok());
    assert!(check(r#"{"p1": 1}"#).is_ok());
    assert_eq!(
        refusal(check("5")),
        (
            "TypeError",
            "argument of type 'int' is not a container or iterable".to_owned()
        )
    );
}

#[test]
fn a_refused_period_lists_what_the_snapshot_accepts() {
    assert_eq!(
        refusal(check(r#"["a", "b"]"#)),
        (
            CONTRACT,
            "c does not accept p1 in snapshot s1. It accepts: a, b.".to_owned()
        )
    );
    assert_eq!(
        refusal(check("[]")),
        (
            CONTRACT,
            "c does not accept p1 in snapshot s1. It accepts: nothing recorded.".to_owned()
        )
    );
    assert_eq!(
        refusal(check("[5]")),
        (
            "TypeError",
            "sequence item 0: expected str instance, int found".to_owned()
        )
    );
}

fn snapshot_dir(name: &str, payload: &str) -> (PathBuf, PyPath) {
    let dir = std::env::temp_dir().join(format!(
        "lodgeit-cov-b-contract-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    std::fs::write(dir.join(format!("{name}.json")), payload).expect("write the snapshot");
    let shown = PyPath::new(dir.to_str().expect("a UTF-8 temp path"));
    (dir, shown)
}

#[test]
fn a_snapshot_without_notes_loads_with_none() {
    let payload = r#"{"provider": "p", "snapshot_id": "s1", "read_at": "t", "read_by": "r",
        "source_urls": [], "calculators": {}, "response_contract": {}}"#;
    let (dir, path) = snapshot_dir("nonotes", payload);
    let loaded = contract::load("nonotes", Some(&path)).expect("the snapshot loads");
    assert!(loaded.notes.is_empty());
    assert!(loaded.openapi_input_schemas.is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_integer_past_the_digit_limit_escapes_as_a_value_error() {
    // json.loads raises a plain ValueError here, which `except
    // json.JSONDecodeError` does not catch.
    let payload = format!(r#"{{"x": {}}}"#, "1".repeat(4301));
    let (dir, path) = snapshot_dir("digits", &payload);
    assert_eq!(
        refusal(contract::load("digits", Some(&path))),
        (
            "ValueError",
            "Exceeds the limit (4300 digits) for integer string conversion: value has 4301 \
             digits; use sys.set_int_max_str_digits() to increase the limit"
                .to_owned()
        )
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn nesting_past_the_stack_escapes_as_a_recursion_error() {
    // CPython 3.15 on Windows: 'Stack overflow (used 1952 kB) while decoding a
    // JSON array from a unicode string'; the figure is the platform's.
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            for (name, open, kind) in [("deep", "[", "array"), ("deepobj", r#"{"a":"#, "object")] {
                let (dir, path) = snapshot_dir(name, &open.repeat(200_000));
                assert_eq!(
                    refusal(contract::load(name, Some(&path))),
                    ("RecursionError", pyjson::recursion_message(kind))
                );
                assert!(pyjson::recursion_message(kind).starts_with("Stack overflow (used "));
                let _ = std::fs::remove_dir_all(&dir);
            }
        })
        .expect("spawn the test thread")
        .join()
        .expect("the test thread ran");
}
