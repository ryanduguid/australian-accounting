//! The `os` calls the rates tool makes, failing with the text of CPython's `OSError`.
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

/// `open(path, "rb").read()`.
pub fn read(path: &std::path::Path, name: &str) -> Result<Vec<u8>, OsError> {
    std::fs::read(path).map_err(|e| OsError::new(&e, Some(name), Style::Errno))
}

/// `os.path.isdir(path)`.
pub fn isdir(path: &str) -> bool {
    std::fs::metadata(path).is_ok_and(|m| m.is_dir())
}

/// `os.path.exists(path)`: any failure, a NUL in the name included, is False.
pub fn exists(path: &std::path::Path) -> bool {
    std::fs::metadata(path).is_ok()
}

/// `Path(path).mkdir(exist_ok=True)`.
pub fn mkdir(path: &crate::pypath::PyPath) -> Result<(), OsError> {
    let text = path.display();
    match std::fs::create_dir(&text) {
        Ok(()) => Ok(()),
        Err(_) if isdir(&text) => Ok(()),
        Err(error) => Err(OsError::new(&error, Some(&text), Style::Native)),
    }
}

/// `Path(path).mkdir(parents=True, exist_ok=True)`.
pub fn makedirs(path: &crate::pypath::PyPath) -> Result<(), OsError> {
    let text = path.display();
    match std::fs::create_dir(&text) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            let parent = path.parent();
            if parent.display() == text {
                return Err(OsError::new(&error, Some(&text), Style::Native));
            }
            makedirs(&parent)?;
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

/// The code points of the str Python gives an OS name: on POSIX, UTF-8 with
/// `surrogateescape` (each byte outside a valid sequence becomes U+DC80..U+DCFF);
/// on Windows, UTF-16 with an unpaired surrogate kept as itself.
pub fn os_name_points(name: &std::ffi::OsStr) -> Vec<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        let mut bytes = name.as_bytes();
        let mut points = Vec::with_capacity(bytes.len());
        loop {
            match std::str::from_utf8(bytes) {
                Ok(text) => {
                    points.extend(text.chars().map(u32::from));
                    return points;
                }
                Err(error) => {
                    let (valid, rest) = bytes.split_at(error.valid_up_to());
                    let valid = std::str::from_utf8(valid).expect("checked prefix");
                    points.extend(valid.chars().map(u32::from));
                    let bad = error.error_len().unwrap_or(rest.len());
                    points.extend(rest[..bad].iter().map(|&b| 0xDC00 + u32::from(b)));
                    bytes = &rest[bad..];
                }
            }
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStrExt;
        char::decode_utf16(name.encode_wide())
            .map(|unit| unit.map_or_else(|e| u32::from(e.unpaired_surrogate()), u32::from))
            .collect()
    }
}

/// The OS string for a Python str: on Windows each code point as UTF-16 (a
/// lone surrogate as itself); on POSIX UTF-8 with `surrogateescape`, where
/// any other surrogate is `os.fsencode`'s `UnicodeEncodeError`.
pub fn os_string(points: &[u32]) -> Result<std::ffi::OsString, String> {
    #[cfg(windows)]
    {
        use std::os::windows::ffi::OsStringExt;
        let mut wide = Vec::with_capacity(points.len());
        for &cp in points {
            match char::from_u32(cp) {
                Some(c) => {
                    let mut buf = [0u16; 2];
                    wide.extend_from_slice(c.encode_utf16(&mut buf));
                }
                None => wide.push(cp as u16),
            }
        }
        Ok(std::ffi::OsString::from_wide(&wide))
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        crate::pyvalue::encode_print(points).map(std::ffi::OsString::from_vec)
    }
}
