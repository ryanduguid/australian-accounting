// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The shared `py.rs` path and text helpers. The same file runs in every crate
//! that holds that copy, so each crate's run measures its own copy. Expected
//! values are what CPython 3.15's `pathlib` and text mode give.
use ato_benchmark_compare::py::{self, PyPath};
use std::path::{MAIN_SEPARATOR, PathBuf};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("family-py-{}-{name}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn a_parent_drops_the_last_part_and_keeps_the_anchor() {
    assert_eq!(PyPath::new("a/b").parent().to_string(), "a");
    assert_eq!(PyPath::new("a").parent().to_string(), ".");
    assert_eq!(
        PyPath::new("/").parent().to_string(),
        MAIN_SEPARATOR.to_string()
    );
}

#[test]
fn resolve_follows_the_existing_prefix_and_normalises_the_rest() {
    let dir = scratch("resolve");
    let real = py::resolve_text(&dir);
    let got = py::resolve_text(&dir.join("missing").join("..").join("x"));
    assert_eq!(got, format!("{real}{MAIN_SEPARATOR}x"));
    // Python shows a Windows path without its verbatim prefix.
    assert!(!real.starts_with("\\\\?\\"));
    std::fs::remove_dir_all(dir).unwrap();
}

// CPython 3.15 on a drive letter that is not there: Path(r"Y:\no\such.csv").resolve()
// is the same text, r"Y:\no\..\x.csv" is r"Y:\x.csv" and r"Y:\no\" is r"Y:\no".
#[cfg(windows)]
#[test]
fn a_path_on_a_missing_drive_resolves_lexically_on_windows() {
    let drive = ('D'..='Z')
        .rev()
        .find(|d| !std::path::Path::new(&format!(r"{d}:\")).exists())
        .expect("a drive letter with no drive");
    let resolve = |p: &str| py::resolve_text(std::path::Path::new(&format!("{drive}:{p}")));
    assert_eq!(resolve(r"\no\such.csv"), format!(r"{drive}:\no\such.csv"));
    assert_eq!(resolve(r"\no\..\x.csv"), format!(r"{drive}:\x.csv"));
    assert_eq!(resolve(r"\no\"), format!(r"{drive}:\no"));
}

#[cfg(not(windows))]
#[test]
fn resolved_paths_compare_exactly_on_posix() {
    assert!(py::same_resolved("/a/b", "/a/b"));
    assert!(!py::same_resolved("/a/b", "/A/b"));
}

#[cfg(windows)]
#[test]
fn resolved_paths_compare_without_case_on_windows() {
    assert!(py::same_resolved("C:\\a\\b", "c:\\A\\B"));
    assert!(!py::same_resolved("C:\\a\\b", "C:\\a\\c"));
}

#[test]
fn text_mode_reads_every_line_end_as_a_newline() {
    assert_eq!(py::universal_newlines("a\r\nb\rc\n"), "a\nb\nc\n");
    assert_eq!(py::universal_newlines("a\nb"), "a\nb");
}
