//! `pyos`: the `OSError` subclass a message maps to, and
//! `Path.mkdir(parents=True, exist_ok=True)`. Expected classes and messages
//! are CPython 3.15's (`OSError(errno, ...)`, `OSError(None, ..., None,
//! winerror)` and `pathlib.Path.mkdir` on the same paths).
use std::io;
use std::path::PathBuf;

use lodgeit_calculator_adapter::pyos::{OsError, makedirs};
use lodgeit_calculator_adapter::pypath::PyPath;

fn class_of(text: &str) -> &'static str {
    OsError {
        kind: io::ErrorKind::Other,
        text: text.into(),
    }
    .class()
}

fn tmp_dir(name: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("lodgeit-cov-c-pyos-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test folder");
    dir
}

#[test]
fn an_errno_names_the_subclass_python_raises() {
    let cases = [
        (1, "PermissionError"),
        (2, "FileNotFoundError"),
        (5, "OSError"),
        (13, "PermissionError"),
        (17, "FileExistsError"),
        (20, "NotADirectoryError"),
        (21, "IsADirectoryError"),
        (32, "BrokenPipeError"),
    ];
    for (errno, class) in cases {
        assert_eq!(
            class_of(&format!("[Errno {errno}] x")),
            class,
            "errno {errno}"
        );
    }
}

#[test]
fn a_win32_error_maps_through_winerror_to_errno() {
    let cases = [
        (2, "FileNotFoundError"),
        (3, "FileNotFoundError"),
        (5, "PermissionError"),
        (15, "FileNotFoundError"),
        (32, "PermissionError"),
        (33, "PermissionError"),
        (53, "FileNotFoundError"),
        (67, "FileNotFoundError"),
        (80, "FileExistsError"),
        (161, "FileNotFoundError"),
        (183, "FileExistsError"),
        (206, "FileNotFoundError"),
        (267, "NotADirectoryError"),
        (87, "OSError"),
    ];
    for (winerror, class) in cases {
        assert_eq!(
            class_of(&format!("[WinError {winerror}] x")),
            class,
            "winerror {winerror}"
        );
    }
}

#[test]
fn a_message_without_a_number_is_a_plain_oserror() {
    assert_eq!(class_of("timed out"), "OSError");
    assert_eq!(class_of("[Errno x] y"), "OSError");
}

#[test]
fn makedirs_makes_every_missing_parent_and_accepts_an_existing_folder() {
    let root = tmp_dir("nested");
    let leaf = root.join("a").join("b").join("c");
    let path = PyPath::new(&leaf.display().to_string());
    makedirs(&path).expect("every parent is made");
    assert!(leaf.is_dir());
    makedirs(&path).expect("exist_ok accepts the folder");
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn makedirs_refuses_a_path_that_is_a_file() {
    let root = tmp_dir("file");
    let file = root.join("taken");
    std::fs::write(&file, b"x").unwrap();
    let name = file.display().to_string();
    let error = makedirs(&PyPath::new(&name)).unwrap_err();
    assert_eq!(error.class(), "FileExistsError");
    if cfg!(unix) {
        assert_eq!(error.text, format!("[Errno 17] File exists: '{name}'"));
    }
    std::fs::remove_dir_all(&root).unwrap();
}

#[cfg(unix)]
#[test]
fn makedirs_raises_the_error_of_a_parent_it_cannot_make() {
    // mkdir reports the missing folder first, then the long name once the
    // folder exists; Python raises that from the parent's own mkdir.
    let root = tmp_dir("long");
    let long = root.join("missing").join("n".repeat(300));
    let name = long.display().to_string();
    let error = makedirs(&PyPath::new(&format!("{name}/leaf"))).unwrap_err();
    assert_eq!(
        error.text,
        format!("[Errno 36] File name too long: '{name}'")
    );
    assert_eq!(error.class(), "OSError");
    assert!(
        root.join("missing").is_dir(),
        "the folders before it stay made"
    );
    std::fs::remove_dir_all(&root).unwrap();
}
