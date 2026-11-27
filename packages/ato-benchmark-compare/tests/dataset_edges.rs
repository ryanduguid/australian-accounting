// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Dataset loading edges: the canonical digest's escapes, schema versions of
//! other JSON types and data files that cannot be read.
use ato_benchmark_compare::Failure;
use ato_benchmark_compare::dataset::{self, DataDir};
use ato_benchmark_compare::py::PyPath;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "ato-benchmark-compare-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn the_digest_matches_python_canonical_json() {
    // hashlib.sha256(json.dumps(json.loads(text), sort_keys=True,
    // separators=(",", ":"), ensure_ascii=True).encode()).hexdigest()
    let text = r#"{"b":"q\"\\\n\r\t\b\f\u0001é😀~","a":[1.5,NaN,-Infinity,Infinity,null,true,false,10,1e22]}"#;
    assert_eq!(
        dataset::canonical_digest(text).unwrap(),
        "4077f7fd846fe658ff5114bc9e8f3fcc30151e31303f03bac08f840b17d1d42f"
    );
}

/// The JSON reader's refusals, as CPython's json.loads words them.
#[test]
fn the_digest_refuses_json_as_cpython_does() {
    let refusal = |text: &str| match dataset::canonical_digest(text) {
        Err(Failure::Value(message)) => message,
        other => panic!("{text:?} gave {other:?}"),
    };
    assert_eq!(
        refusal("[\n x]"),
        "not valid JSON: Expecting value: line 2 column 2 (char 3)"
    );
    assert_eq!(
        refusal("-x"),
        "not valid JSON: Expecting value: line 1 column 1 (char 0)"
    );
    // An exponent marker without digits ends the number before it.
    assert_eq!(
        refusal("1ex"),
        "not valid JSON: Extra data: line 1 column 2 (char 1)"
    );
    assert_eq!(
        refusal("1e+"),
        "not valid JSON: Extra data: line 1 column 2 (char 1)"
    );
    // CPython keeps '\ud83dA'; a Rust string cannot hold the lone surrogate.
    assert_eq!(
        refusal(r#""\ud83dA""#),
        "not valid JSON: Unpaired surrogate in \\uXXXX escape: line 1 column 2 (char 1)"
    );
    // json.dumps({}) and json.dumps([1.25, 100000.0, 0.002]) hashed.
    assert_eq!(
        dataset::canonical_digest("{}").unwrap(),
        "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
    );
    assert_eq!(
        dataset::canonical_digest("[1.25,1e+5,2E-3]").unwrap(),
        dataset::canonical_digest("[1.25, 100000.0, 0.002]").unwrap()
    );
}

#[test]
fn an_unknown_ratio_key_is_its_own_label() {
    assert_eq!(dataset::ratio_label("rent_to_turnover"), "Rent to turnover");
    assert_eq!(
        dataset::ratio_label("wages_to_turnover"),
        "wages_to_turnover"
    );
}

#[test]
fn a_schema_version_compares_as_python_numbers_do() {
    let refused = |version: &str, shown: &str| {
        let text = format!(r#"{{"schema_version": {version}}}"#);
        let Err(Failure::Value(message)) = dataset::loads(&text, "x.json") else {
            panic!("schema version {version} was accepted");
        };
        assert_eq!(
            message,
            format!("x.json: schema version {shown} is not supported by this release (expected 1)")
        );
    };
    refused("2.5", "2.5");
    refused("false", "False");
    refused(r#""1""#, "'1'");
    // 1.0 == 1 and True == 1 in Python, so both pass the version check.
    for version in ["1.0", "true"] {
        let text = format!(r#"{{"schema_version": {version}}}"#);
        let Err(Failure::Value(message)) = dataset::loads(&text, "x.json") else {
            panic!("an empty dataset loaded");
        };
        assert_eq!(message, "x.json: benchmark_year is missing");
    }
}

#[test]
fn a_data_file_that_is_not_utf8_is_a_decode_error() {
    let dir = scratch("decode");
    std::fs::write(dir.join("benchmarks-2023-24.json"), b"\xff{}").unwrap();
    let path = PyPath::new(&dir.to_string_lossy());
    let Err(Failure::Uncaught(message)) = dataset::load(None, &DataDir::Path(&path)) else {
        panic!("undecodable data loaded");
    };
    assert!(
        message
            .starts_with("UnicodeDecodeError: 'utf-8' codec can't decode byte 0xff in position 0"),
        "{message}"
    );
}

#[cfg(unix)]
#[test]
fn a_data_file_that_cannot_be_opened_is_an_oserror() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("denied");
    let file = dir.join("benchmarks-2023-24.json");
    std::fs::write(&file, b"{}").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000)).unwrap();
    if std::fs::read(&file).is_ok() {
        // Root reads any file; the refusal cannot be shown here.
        return;
    }
    let path = PyPath::new(&dir.to_string_lossy());
    let Err(Failure::Value(message)) = dataset::load(None, &DataDir::Path(&path)) else {
        panic!("an unreadable data file loaded");
    };
    assert!(
        message.starts_with("[Errno 13] Permission denied: "),
        "{message}"
    );
}

/// One shipped business type, with its turnover bands replaced or removed.
fn one_type(bands: Option<&str>) -> String {
    let bands = bands.map_or(String::new(), |b| format!(r#", "turnover_bands": {b}"#));
    format!(
        r#"{{"schema_version": 1, "benchmark_year": "2023-24", "business_types": [{{"name": "Cafes", "key_ratio": "total_expenses_to_turnover"{bands}}}]}}"#
    )
}

fn refusal(result: Result<dataset::Dataset, Failure>) -> Failure {
    match result {
        Err(failure) => failure,
        Ok(_) => panic!("the dataset loaded"),
    }
}

#[test]
fn turnover_bands_are_iterated_as_python_iterates_them() {
    let value = |m: &str| Failure::Value(m.into());
    // No key: `entry.get("turnover_bands", [])` gives no bands.
    assert_eq!(
        refusal(dataset::loads(&one_type(None), "x.json")),
        value("x.json: Cafes: no turnover bands")
    );
    // A dict iterates its keys and a str its characters: neither is a band object.
    for bands in [r#"{"low": {}}"#, r#""low""#] {
        assert_eq!(
            refusal(dataset::loads(&one_type(Some(bands)), "x.json")),
            value("x.json: Cafes: a turnover band is not an object")
        );
    }
    assert_eq!(
        refusal(dataset::loads(&one_type(Some("5")), "x.json")),
        Failure::Uncaught("TypeError: 'int' object is not iterable".into())
    );
}

/// The shipped 2023-24 dataset before its page ranges, and the page file edited by `edit`.
fn merged_with(edit: impl FnOnce(&mut serde_json::Value)) -> Result<dataset::Dataset, Failure> {
    let data = dataset::loads(include_str!("../data/benchmarks-2023-24.json"), "b.json").unwrap();
    let mut pages: serde_json::Value =
        serde_json::from_str(include_str!("../data/other-benchmarks-2023-24.json")).unwrap();
    edit(&mut pages);
    dataset::merge_page_ranges(&data, &pages.to_string(), "p.json", None)
}

#[test]
fn the_shipped_page_file_merges_unedited() {
    let merged = merged_with(|_| {}).unwrap();
    assert!(merged.page_source.is_some());
}

#[test]
fn a_malformed_page_file_is_refused() {
    let value = |m: &str| Failure::Value(m.into());
    assert_eq!(
        refusal(merged_with(
            |p| p["excluded_pages"][0] = serde_json::json!({"reason": "no name"})
        )),
        value("p.json: excluded_pages is missing or malformed")
    );
    assert_eq!(
        refusal(merged_with(|p| p["excluded_pages"] = serde_json::json!({}))),
        value("p.json: excluded_pages is missing or malformed")
    );
    assert_eq!(
        refusal(merged_with(
            |p| p["business_types"][0]["name"] = serde_json::json!(5)
        )),
        value("p.json: 5 is not a business type in 2023-24")
    );
    let shown = "p.json: Air conditioning, refrigeration and heating services: turnover bands do not match the dataset";
    assert_eq!(
        refusal(merged_with(
            |p| p["business_types"][0]["turnover_bands"] = serde_json::json!({})
        )),
        value(shown)
    );
    assert_eq!(
        refusal(merged_with(
            |p| p["business_types"][0]["turnover_bands"][1]["band"] = serde_json::json!("low")
        )),
        value(shown)
    );
}
