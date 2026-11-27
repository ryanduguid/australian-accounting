// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Windows `ntpath` cases: each answer is what CPython 3.15.0 on Windows
//! printed for the same input (`tests/data/cov_a7_pypath.py`).

use super::*;

#[test]
fn path_parts_follow_ntpath() {
    assert_eq!(splitroot("\\\\server"), ("\\\\server", "", ""));
    assert_eq!(join("C:\\x", "D:\\y"), "D:\\y");
    assert_eq!(join("\\\\server\\share", "x"), "\\\\server\\share\\x");
    for (path, head, tail) in [
        ("C:\\a\\b", "C:\\a", "b"),
        ("C:\\a\\", "C:\\a", ""),
        ("C:\\", "C:\\", ""),
        ("a/b\\\\c", "a/b", "c"),
        ("b", "", "b"),
        ("\\\\server\\share\\x", "\\\\server\\share\\", "x"),
    ] {
        assert_eq!(split(path), (head.to_string(), tail.to_string()), "{path}");
    }
    for (path, want) in [
        ("C:\\..\\a", "C:\\a"),
        ("\\..\\a", "\\a"),
        ("", "."),
        ("a\\..", "."),
    ] {
        assert_eq!(normpath(path), want, "{path:?}");
    }
}

/// Missing names, broken links and a link loop, in a new folder `T`.
#[test]
fn realpath_keeps_what_it_cannot_resolve_as_ntpath_does() {
    use std::os::windows::fs::symlink_file;
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("cov-a7-pypath-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(base.join("d")).unwrap();
    let tmp = base.to_string_lossy().into_owned();
    let top = realpath(&tmp).unwrap();
    for (target, link) in [
        (format!("{tmp}\\absent"), "to-absent"),
        ("missing".to_string(), "d\\to-missing"),
        (format!("{tmp}\\loop-b"), "loop-a"),
        (format!("{tmp}\\loop-a"), "loop-b"),
    ] {
        symlink_file(target, base.join(link)).unwrap();
    }
    for (input, want) in [
        ("\\nope\\deeper", "\\nope\\deeper"),
        ("\\to-absent", "\\absent"),
        ("\\d\\to-missing", "\\d\\missing"),
        ("\\loop-a", "\\loop-b"),
    ] {
        assert_eq!(
            realpath(&format!("{tmp}{input}")).unwrap(),
            format!("{top}{want}"),
            "{input}"
        );
    }
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(
        realpath("nope-rel").unwrap(),
        format!("{}\\nope-rel", realpath(&cwd.to_string_lossy()).unwrap())
    );
    std::fs::remove_dir_all(&base).unwrap();
}

/// Relative symlink loops and verbatim inputs match Windows CPython 3.15;
/// unmapped drive roots fail `makedirs`, and Windows `pyjson` stops at `CALL_DEPTH`.
#[test]
fn windows_symlink_loops_makedirs_and_json_call_depth_match_cpython() {
    use std::os::windows::fs::symlink_file;
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("cov-a7-extra-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let tmp = base.to_string_lossy().into_owned();
    let top = realpath(&tmp).unwrap();
    for (target, link) in [
        ("rloop-b".to_string(), "rloop-a"),
        ("rloop-a".to_string(), "rloop-b"),
        (format!("{tmp}\\aloop-b"), "aloop-a"),
        (format!("{tmp}\\aloop-a"), "aloop-b"),
    ] {
        symlink_file(target, base.join(link)).unwrap();
    }
    assert_eq!(
        realpath(&format!("{tmp}\\rloop-a")).unwrap(),
        format!("{top}\\rloop-a")
    );
    assert_eq!(
        realpath(&format!("\\\\?\\{tmp}\\aloop-a")).unwrap(),
        format!("\\\\?\\{top}\\aloop-a")
    );
    std::fs::remove_dir_all(&base).unwrap();

    let drive = (b'D'..=b'Z')
        .map(char::from)
        .find(|d| !std::path::Path::new(&format!("{d}:\\")).exists())
        .expect("an unmapped drive letter");
    let missing_root = PyPath::new(&format!("{drive}:\\"));
    let err = crate::pyos::makedirs(&missing_root).unwrap_err();
    assert_eq!(err.class(), "FileNotFoundError");

    let deep = format!("{}NaN{}", "[".repeat(13_861), "]".repeat(13_861));
    assert_eq!(
        crate::pyjson::loads_standard(&deep),
        Err(crate::pyjson::DecodeError::Recursion("call"))
    );
    assert_eq!(
        crate::pyjson::recursion_message("call"),
        "Stack overflow (used 1952 kB) while calling a Python object"
    );
}
