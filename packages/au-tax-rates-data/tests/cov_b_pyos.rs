//! `OSError` subclasses, `mkdir`, `makedirs` and OS name decoding as `rates` reports them.
use au_tax_rates_data::pyos::{OsError, makedirs, mkdir, os_name_points};
use au_tax_rates_data::pypath::PyPath;

fn class(text: &str) -> &'static str {
    OsError { text: text.into() }.class()
}

#[test]
fn win32_errors_raise_the_subclass_cpython_maps_them_to() {
    // winerror_to_errno: not found, access denied, exists, not a directory, other.
    for (winerror, expected) in [
        (2, "FileNotFoundError"),
        (3, "FileNotFoundError"),
        (206, "FileNotFoundError"),
        (5, "PermissionError"),
        (32, "PermissionError"),
        (80, "FileExistsError"),
        (183, "FileExistsError"),
        (267, "NotADirectoryError"),
        (87, "OSError"),
    ] {
        let text = format!("[WinError {winerror}] message: 'x'");
        assert_eq!(class(&text), expected, "{winerror}");
    }
}

#[test]
fn errno_values_raise_their_subclass() {
    for (errno, expected) in [
        (1, "PermissionError"),
        (13, "PermissionError"),
        (17, "FileExistsError"),
        (20, "NotADirectoryError"),
        (21, "IsADirectoryError"),
        (32, "BrokenPipeError"),
        (5, "OSError"),
    ] {
        assert_eq!(class(&format!("[Errno {errno}] message")), expected);
    }
    assert_eq!(class("no number"), "OSError");
}

#[test]
fn an_os_error_prints_as_its_text() {
    let error = OsError {
        text: "[Errno 2] No such file or directory: 'x'".into(),
    };
    assert_eq!(error.to_string(), error.text);
}

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("cov_b_pyos-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("make the scratch folder");
    dir
}

fn py(path: &std::path::Path) -> PyPath {
    PyPath::new(&path.to_string_lossy())
}

#[test]
fn mkdir_accepts_an_existing_folder_and_refuses_a_missing_parent() {
    let dir = scratch("mkdir");
    let made = dir.join("reports");
    assert!(mkdir(&py(&made)).is_ok());
    assert!(made.is_dir());
    assert!(mkdir(&py(&made)).is_ok(), "exist_ok=True");
    let error = mkdir(&py(&dir.join("no").join("reports"))).unwrap_err();
    assert_eq!(error.class(), "FileNotFoundError", "{}", error.text);
    let file = dir.join("file");
    std::fs::write(&file, b"").unwrap();
    let error = mkdir(&py(&file)).unwrap_err();
    assert_eq!(error.class(), "FileExistsError", "{}", error.text);
}

#[test]
fn makedirs_makes_every_missing_parent_and_refuses_a_file_in_the_way() {
    let dir = scratch("makedirs");
    let deep = dir.join("a").join("b").join("c");
    assert!(makedirs(&py(&deep)).is_ok());
    assert!(deep.is_dir());
    assert!(makedirs(&py(&deep)).is_ok(), "exist_ok=True");
    let file = dir.join("file");
    std::fs::write(&file, b"").unwrap();
    let error = makedirs(&py(&file)).unwrap_err();
    assert_eq!(error.class(), "FileExistsError", "{}", error.text);
    let error = makedirs(&py(&file.join("below"))).unwrap_err();
    assert_ne!(error.class(), "FileNotFoundError", "{}", error.text);
}

#[test]
fn makedirs_tries_again_once_the_parent_of_a_dotdot_path_exists() {
    // CPython 3.15: `Path(dir, "missing", "..").mkdir(parents=True, exist_ok=True)` succeeds;
    // POSIX finds `missing` absent, makes it and retries, Windows resolves `..` lexically first.
    let dir = scratch("dotdot");
    assert!(makedirs(&py(&dir.join("missing").join(".."))).is_ok());
    assert_eq!(dir.join("missing").is_dir(), cfg!(unix));
    // The retry for `gone/../file` meets a file: FileExistsError naming the path as given.
    std::fs::write(dir.join("file"), b"").unwrap();
    let path = dir.join("gone").join("..").join("file");
    let error = makedirs(&py(&path)).unwrap_err();
    assert_eq!(error.class(), "FileExistsError", "{}", error.text);
    let sep = if cfg!(windows) { "\\\\" } else { "/" };
    assert!(
        error.text.ends_with(&format!("..{sep}file'")),
        "{}",
        error.text
    );
}

#[cfg(windows)]
#[test]
fn makedirs_on_a_missing_drive_raises_the_roots_file_not_found() {
    // CPython 3.15 on Windows: FileNotFoundError [WinError 3] ... 'Y:\\', the drive root itself.
    let letter = ('D'..='Z')
        .rev()
        .find(|c| !std::path::Path::new(&format!("{c}:\\")).exists())
        .expect("a free drive letter");
    let error = makedirs(&PyPath::new(&format!("{letter}:\\x"))).unwrap_err();
    assert_eq!(error.class(), "FileNotFoundError", "{}", error.text);
    assert!(error.text.starts_with("[WinError 3] "), "{}", error.text);
    assert!(
        error.text.ends_with(&format!("'{letter}:\\\\'")),
        "{}",
        error.text
    );
}

#[cfg(unix)]
#[test]
fn undecodable_name_bytes_become_surrogate_escapes() {
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::OsStr::from_bytes(b"a\xff\xe2\x82b");
    assert_eq!(
        os_name_points(name),
        vec![0x61, 0xDCFF, 0xDCE2, 0xDC82, 0x62]
    );
}

#[cfg(windows)]
#[test]
fn unpaired_surrogates_in_a_name_are_kept() {
    use std::os::windows::ffi::OsStringExt;
    let name = std::ffi::OsString::from_wide(&[0x61, 0xD800, 0x62]);
    assert_eq!(os_name_points(&name), vec![0x61, 0xD800, 0x62]);
}
