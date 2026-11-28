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
        ("\\to-absent\\child", "\\absent\\child"),
        ("\\d\\to-missing", "\\d\\missing"),
        ("\\loop-a", "\\loop-b"),
    ] {
        assert_eq!(
            realpath(&format!("{tmp}{input}")).unwrap(),
            format!("{top}{want}"),
            "{input}"
        );
    }
    assert_eq!(
        final_path_name_nonstrict(&tmp).unwrap(),
        final_path_name(&tmp).unwrap()
    );
    assert_eq!(
        final_path_name_nonstrict("no_such_relative_entry").unwrap(),
        "no_such_relative_entry"
    );
    let cwd = std::env::current_dir().unwrap();
    assert_eq!(
        realpath("nope-rel").unwrap(),
        format!("{}\\nope-rel", realpath(&cwd.to_string_lossy()).unwrap())
    );
    std::fs::remove_dir_all(&base).unwrap();
}

fn unmapped_drive_in_mask(mask: u32) -> Option<char> {
    if mask == 0 {
        return None;
    }
    (3u8..26)
        .find(|&bit| (mask & (1u32 << bit)) == 0)
        .map(|bit| char::from(b'A' + bit))
}

fn unmapped_drive() -> Option<char> {
    unsafe extern "system" {
        fn GetLogicalDrives() -> u32;
    }
    // SAFETY: GetLogicalDrives takes no arguments and returns the assigned-drive bitmask.
    // nosemgrep: rust.lang.security.unsafe-usage.unsafe-usage
    unmapped_drive_in_mask(unsafe { GetLogicalDrives() })
}

fn check_unmapped_drive(drive: Option<char>) {
    let Some(drive) = drive else {
        return;
    };
    let missing_root = PyPath::new(&format!("{drive}:\\"));
    let err = crate::pyos::makedirs(&missing_root).unwrap_err();
    assert_eq!(err.class(), "FileNotFoundError");
    let missing_child = format!("{drive}:\\missing");
    assert_eq!(realpath(&missing_child).unwrap(), missing_child);
}

fn not_symlink(_: &str) -> bool {
    false
}

/// Short drive-rooted and UNC symlink targets regain CPython's verbatim prefix.
#[test]
fn restore_verbatim_prefix_covers_drive_and_unc_targets() {
    use std::os::windows::fs::symlink_file;
    for (input, want) in [
        ("C:\\a\\b", "\\\\?\\C:\\a\\b"),
        ("\\\\server\\share\\a", "\\\\?\\UNC\\server\\share\\a"),
        ("\\\\?\\C:\\a", "\\\\?\\C:\\a"),
        (
            "\\\\?\\UNC\\server\\share\\a",
            "\\\\?\\UNC\\server\\share\\a",
        ),
        ("\\\\.\\NUL", "\\\\.\\NUL"),
        ("C:relative", "C:relative"),
        ("missing", "missing"),
    ] {
        assert_eq!(restore_verbatim_prefix(input.into()), want, "{input}");
    }
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join(format!("cov-a7-unc-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).unwrap();
    let link = base.join("to-unc");
    symlink_file("\\\\server\\share\\target", &link).unwrap();
    let raw = std::fs::read_link(&link)
        .unwrap()
        .to_string_lossy()
        .into_owned();
    assert_eq!(
        restore_verbatim_prefix(raw),
        "\\\\?\\UNC\\server\\share\\target"
    );
    std::fs::remove_dir_all(&base).unwrap();
}

/// Two-link UNC loops stop on CPython 3.15's terminal link for both ordinary and
/// verbatim inputs; non-symlink relative reparse points stop on the link itself.
#[test]
fn unc_readlink_loop_and_non_symlink_reparse_match_cpython() {
    let unc_loop = |p: &str| match p {
        "\\\\server\\share\\uloop-a" | "\\\\?\\UNC\\server\\share\\uloop-a" => {
            Some(PathBuf::from("\\\\server\\share\\uloop-b"))
        }
        "\\\\server\\share\\uloop-b" | "\\\\?\\UNC\\server\\share\\uloop-b" => {
            Some(PathBuf::from("\\\\server\\share\\uloop-a"))
        }
        _ => None,
    };
    assert_eq!(
        readlink_deep_with("\\\\server\\share\\uloop-a", unc_loop, not_symlink),
        "\\\\?\\UNC\\server\\share\\uloop-b"
    );
    assert_eq!(
        readlink_deep_with("\\\\?\\UNC\\server\\share\\uloop-a", unc_loop, not_symlink),
        "\\\\?\\UNC\\server\\share\\uloop-a"
    );
    assert_eq!(
        readlink_deep_with("\\\\server\\share\\other", unc_loop, not_symlink),
        "\\\\server\\share\\other"
    );
    assert_eq!(
        readlink_deep_with("C:\\a\\j", |_| Some(PathBuf::from("rel")), not_symlink),
        "C:\\a\\j"
    );
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

    check_unmapped_drive(unmapped_drive());
    check_unmapped_drive(unmapped_drive_in_mask(0));
    check_unmapped_drive(unmapped_drive_in_mask(u32::MAX));

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

#[test]
fn nonstrict_error_and_verbatim_prefix_stripping_match_ntpath() {
    let disallowed = |_: &str| Err(std::io::Error::from_raw_os_error(1117));
    let err = final_path_name_nonstrict_with("C:\\io", disallowed, str::to_string).unwrap_err();
    assert!(err.text.starts_with("[WinError 1117] "), "{}", err.text);
    assert!(err.text.ends_with(": 'C:\\\\io'"), "{}", err.text);

    // Ordinary UNC input loses \\?\UNC\ when re-resolving the short path agrees.
    let unc_same = |p: &str| {
        assert_eq!(p, "\\\\server\\share\\x");
        Ok("\\\\?\\UNC\\server\\share\\x".to_string())
    };
    assert_eq!(
        realpath_with("\\\\server\\share\\x", unc_same, str::to_string).unwrap(),
        "\\\\server\\share\\x"
    );

    // If re-resolving the short path disagrees or fails with a new error, keep the verbatim prefix.
    let ok_mismatch = |p: &str| {
        Ok(if p == "C:\\a\\b" {
            "\\\\?\\C:\\other\\b"
        } else {
            assert_eq!(p, "C:\\other\\b");
            "\\\\?\\C:\\third\\b"
        }
        .to_string())
    };
    assert_eq!(
        realpath_with("C:\\a\\b", ok_mismatch, str::to_string).unwrap(),
        "\\\\?\\C:\\other\\b"
    );

    let err_mismatch = |p: &str| match p {
        "C:\\a\\link" => Err(std::io::Error::from_raw_os_error(2)),
        _ => Err(std::io::Error::from_raw_os_error(5)),
    };
    let to_verbatim = |p: &str| {
        assert_eq!(p, "C:\\a\\link");
        "\\\\?\\C:\\a\\target".to_string()
    };
    assert_eq!(
        realpath_with("C:\\a\\link", err_mismatch, to_verbatim).unwrap(),
        "\\\\?\\C:\\a\\target"
    );
}
