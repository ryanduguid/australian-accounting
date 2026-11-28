// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `py::read_text` and `py::resolve_text` on real files, as `Path.read_text`
//! and `Path.resolve` behave.

use std::path::Path;

use payday_super_checker::py::{ReadError, read_text, resolve_text};

#[test]
fn read_text_reads_utf8_with_universal_newlines() {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("py-read-text");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("in.txt");
    std::fs::write(&path, b"a\r\nb\rc").unwrap();
    assert_eq!(read_text(&path).unwrap(), "a\nb\nc");
    std::fs::write(&path, b"\xff").unwrap();
    assert!(matches!(read_text(&path), Err(ReadError::Decode(_))));
    assert!(matches!(
        read_text(&dir.join("missing")),
        Err(ReadError::Os(_))
    ));
    let resolved = resolve_text(&dir.join("gone").join("..").join("in.txt"));
    assert_eq!(resolved, resolve_text(&path));
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A verbatim path keeps `.` as a component; resolving it must still drop it.
#[cfg(windows)]
#[test]
fn a_verbatim_path_resolves_without_its_dot() {
    let dotted = resolve_text(Path::new("\\\\?\\C:\\payday-no-such-dir\\.\\missing"));
    assert_eq!(
        dotted,
        resolve_text(Path::new("\\\\?\\C:\\payday-no-such-dir\\missing"))
    );
    assert!(!dotted.contains("\\.\\"), "{dotted}");
}
