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
