// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.
//! `evidence.read`, `write`, `_canonical` and `verify` on inputs Python
//! refuses. Each expected value is what the retired Python
//! `lodgeitadapter.evidence` raised under CPython 3.15; a stack figure in a
//! `RecursionError` is the platform's, so only its wording is checked.
use std::path::PathBuf;

use lodgeit_calculator_adapter::evidence::{self, ReadError};
use lodgeit_calculator_adapter::pyjson::{self, Json};
use lodgeit_calculator_adapter::pyops::Exc;
use lodgeit_calculator_adapter::pypath::PyPath;

fn text(points: &[u32]) -> String {
    points.iter().filter_map(|&cp| char::from_u32(cp)).collect()
}

fn refusal<T>(result: Result<T, Exc>) -> (&'static str, String) {
    match result {
        Ok(_) => panic!("expected a refusal"),
        Err(error) => (error.class, text(&error.text)),
    }
}

fn uncaught(path: &PyPath) -> (&'static str, String) {
    match evidence::read(path) {
        Err(ReadError::Uncaught(error)) => (error.class, text(&error.text)),
        Err(ReadError::Reported(m)) => panic!("reported, not raised: {m}"),
        Ok(_) => panic!("expected a refusal"),
    }
}

fn scratch(name: &str) -> (PathBuf, PyPath) {
    let dir = std::env::temp_dir().join(format!(
        "lodgeit-cov-b-evidence-{}-{name}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    let file = dir.join("record.json");
    let path = PyPath::new(file.to_str().expect("a UTF-8 temp path"));
    (dir, path)
}

fn on_a_big_stack(test: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(test)
        .expect("spawn the test thread")
        .join()
        .expect("the test thread ran");
}

fn nested(levels: usize) -> Json {
    let mut node = Json::Array(Vec::new());
    for _ in 0..levels {
        node = Json::Array(vec![node]);
    }
    node
}

fn surrogate() -> Json {
    pyjson::loads_standard(r#"{"a": "\ud800"}"#).expect("valid JSON")
}

#[test]
fn a_record_that_is_not_utf8_escapes_as_a_unicode_decode_error() {
    let (dir, path) = scratch("utf8");
    std::fs::write(path.fs(), b"{\"a\": \"\xff\"}").expect("write the record");
    assert_eq!(
        uncaught(&path),
        (
            "UnicodeDecodeError",
            "'utf-8' codec can't decode byte 0xff in position 7: invalid start byte".to_owned()
        )
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn an_integer_past_the_digit_limit_escapes_as_a_value_error() {
    let (dir, path) = scratch("digits");
    std::fs::write(path.fs(), format!(r#"{{"x": {}}}"#, "1".repeat(4301))).expect("write");
    assert_eq!(
        uncaught(&path),
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
    on_a_big_stack(|| {
        let (dir, path) = scratch("deep");
        std::fs::write(path.fs(), "[".repeat(200_000)).expect("write the record");
        let (class, message) = uncaught(&path);
        assert_eq!(class, "RecursionError");
        assert_eq!(message, pyjson::recursion_message("array"));
        assert!(message.ends_with(" while decoding a JSON array from a unicode string"));
        let _ = std::fs::remove_dir_all(&dir);
    });
}

#[test]
fn a_lone_surrogate_cannot_be_written_or_canonicalised() {
    let (dir, path) = scratch("surrogate");
    // Path.write_text opens the file before it encodes, so an empty file is
    // left behind. CPython 3.15 on Windows reports position 11, as the
    // newline it counts past is written "\r\n"; on Linux it is 10.
    let position = if cfg!(windows) { 11 } else { 10 };
    assert_eq!(
        refusal(evidence::write(&surrogate(), &path)),
        (
            "UnicodeEncodeError",
            format!(
                "'utf-8' codec can't encode character '\\ud800' in position {position}: \
                 surrogates not allowed"
            )
        )
    );
    assert_eq!(std::fs::read(path.fs()).expect("the file exists"), b"");
    assert_eq!(
        refusal(evidence::canonical(&surrogate())),
        (
            "UnicodeEncodeError",
            "'utf-8' codec can't encode character '\\ud800' in position 6: surrogates not allowed"
                .to_owned()
        )
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_lets_a_recursion_error_escape() {
    on_a_big_stack(|| {
        let deep = nested(100_000);
        let calculation = Json::obj([
            ("schema", Json::str(evidence::SCHEMA)),
            ("calculation", Json::obj([("x", deep.clone())])),
        ]);
        let (class, message) = refusal(evidence::verify(&calculation));
        assert_eq!(class, "RecursionError");
        assert!(message.starts_with("Stack overflow (used "));
        assert!(message.ends_with(" kB) while encoding a JSON object"));
        let schema = Json::obj([("schema", deep)]);
        let (class, message) = refusal(evidence::verify(&schema));
        assert_eq!(class, "RecursionError");
        assert!(message.starts_with("Stack overflow (used "));
        assert!(message.ends_with(" kB) while getting the repr of an object"));
    });
}

#[test]
fn a_record_that_is_not_json_is_reported() {
    let (dir, path) = scratch("notjson");
    std::fs::write(path.fs(), "{not json").expect("write the record");
    match evidence::read(&path) {
        Err(ReadError::Reported(m)) => assert_eq!(
            m,
            "Expecting property name enclosed in double quotes: line 1 column 2 (char 1)"
        ),
        _ => panic!("a JSONDecodeError is reported"),
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn writing_over_a_folder_raises_the_open_error() {
    // CPython 3.15 on Windows: PermissionError [Errno 13]; on Linux
    // IsADirectoryError [Errno 21].
    let (dir, path) = scratch("folder");
    std::fs::create_dir(path.fs()).expect("make the folder");
    let shown = path.display().replace('\\', "\\\\");
    let expected = if cfg!(windows) {
        (
            "PermissionError",
            format!("[Errno 13] Permission denied: '{shown}'"),
        )
    } else {
        (
            "IsADirectoryError",
            format!("[Errno 21] Is a directory: '{shown}'"),
        )
    };
    assert_eq!(
        refusal(evidence::write(&Json::Object(Vec::new()), &path)),
        expected
    );
    let _ = std::fs::remove_dir_all(&dir);
}
