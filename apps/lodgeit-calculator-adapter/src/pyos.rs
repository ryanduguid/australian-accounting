//! The `os` calls the adapter makes, failing with the text of CPython's `OSError`.
//!
//! On Windows, `open` goes through the C runtime and reports an errno; the
//! directory and stat calls report the Win32 error. A message names the path
//! as Python passed it: the `str` of a `Path`.
use std::io;

use crate::py;

/// Which layer reported the error, which decides how Python renders it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Style {
    /// The C runtime's errno (`open`, file reads and writes).
    Errno,
    /// The platform call itself (Win32 on Windows).
    Native,
}

/// An `OSError`: its `str()` and the condition callers branch on.
#[derive(Debug)]
pub struct OsError {
    pub kind: io::ErrorKind,
    pub text: String,
}

impl OsError {
    pub fn new(error: &io::Error, filename: Option<&str>, style: Style) -> Self {
        let text = match filename {
            Some(name) => py::oserror_str(error, name, style.into()),
            None => without_filename(error, style),
        };
        OsError {
            kind: error.kind(),
            text,
        }
    }

    /// The `OSError` subclass Python raises for this error.
    pub fn class(&self) -> &'static str {
        let number = |prefix: &str| {
            self.text
                .strip_prefix(prefix)
                .and_then(|rest| rest.split(']').next())
                .and_then(|n| n.parse::<i32>().ok())
        };
        // CPython's winerror_to_errno for the Win32 errors these calls report.
        let errno = number("[Errno ").or_else(|| {
            number("[WinError ").map(|winerror| match winerror {
                2 | 3 | 15 | 53 | 67 | 161 | 206 => 2,
                5 | 32 | 33 => 13,
                80 | 183 => 17,
                267 => 20,
                _ => 0,
            })
        });
        match errno {
            Some(2) => "FileNotFoundError",
            Some(1 | 13) => "PermissionError",
            Some(17) => "FileExistsError",
            Some(20) => "NotADirectoryError",
            Some(21) => "IsADirectoryError",
            Some(32) => "BrokenPipeError",
            _ => "OSError",
        }
    }
}

impl From<Style> for py::OsStyle {
    fn from(style: Style) -> Self {
        match style {
            Style::Errno => py::OsStyle::Errno,
            Style::Native => py::OsStyle::Native,
        }
    }
}

impl std::fmt::Display for OsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text)
    }
}

/// `str(OSError)` with no filename, as a failed `CreateProcess` reports it.
fn without_filename(error: &io::Error, style: Style) -> String {
    let with = py::oserror_str(error, "", style.into());
    with.strip_suffix(": ''").unwrap_or(&with).to_string()
}

/// `os.path.isdir(path)`.
fn isdir(path: &str) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_dir())
}

/// `Path(path).mkdir(parents=True, exist_ok=True)`.
pub fn makedirs(path: &crate::pypath::PyPath) -> Result<(), OsError> {
    let text = path.display();
    match std::fs::create_dir(&text) {
        Ok(()) => Ok(()),
        // A path that is its own parent is not retried; its error is raised
        // below, as `isdir` is false for a path that is not there.
        Err(error)
            if error.kind() == io::ErrorKind::NotFound && path.parent().display() != text =>
        {
            makedirs(&path.parent())?;
            match std::fs::create_dir(&text) {
                Ok(()) => Ok(()),
                Err(_) if isdir(&text) => Ok(()),
                Err(error) => Err(OsError::new(&error, Some(&text), Style::Native)),
            }
        }
        Err(_) if isdir(&text) => Ok(()),
        Err(error) => Err(OsError::new(&error, Some(&text), Style::Native)),
    }
}
