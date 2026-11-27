// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! How a failure reads: `str(exc)` and the last traceback line Python printed,
//! an OS error with no errno, and command-line bytes that are not UTF-8.

use payday_super_checker::py::OsStyle;
use payday_super_checker::{Failure, OsError};

#[test]
fn failures_read_as_python_printed_them() {
    // CPython: date.max + timedelta(1) raises "OverflowError: date value out of range".
    assert_eq!(Failure::Overflow.message(), "date value out of range");
    assert_eq!(
        Failure::Overflow.traceback_line(),
        "OverflowError: date value out of range"
    );
    let value = Failure::Value("bad row".into());
    assert_eq!(value.traceback_line(), "ValueError: bad row");
    let os = Failure::Os(OsError {
        errno: 13,
        strerror: "Permission denied".into(),
        filename: Some("r.csv".into()),
        text: "[Errno 13] Permission denied: 'r.csv'".into(),
    });
    assert_eq!(
        os.traceback_line(),
        "OSError: [Errno 13] Permission denied: 'r.csv'"
    );
}

#[test]
fn an_error_with_no_errno_keeps_its_text_when_renamed() {
    let e = OsError::from_io(&std::io::Error::other("boom"), "in.csv", OsStyle::Errno);
    assert_eq!((e.errno, e.strerror.as_str()), (0, ""));
    assert_eq!(e.filename.as_deref(), Some("in.csv"));
    assert!(!e.not_found());
    // OSError(exc.errno, exc.strerror or str(exc), filename): no strerror, so str(exc).
    let renamed = e.renamed("out.csv");
    assert_eq!(renamed.strerror, e.text);
    assert_eq!(renamed.filename.as_deref(), Some("out.csv"));
}

#[cfg(unix)]
#[test]
fn a_non_utf8_argument_prints_as_its_surrogate_escape() {
    use payday_super_checker::cli::{decode_os, output_bytes};
    use std::os::unix::ffi::OsStrExt;
    // CPython: b"a\xffb".decode(errors="surrogateescape").encode(errors="backslashreplace")
    // == b"a\\udcffb".
    let text = decode_os(std::ffi::OsStr::from_bytes(b"a\xffb"));
    assert_eq!(output_bytes(&text), b"a\\udcffb");
}
