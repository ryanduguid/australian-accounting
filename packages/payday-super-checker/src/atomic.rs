//! Atomic output for generated CSV and Markdown workpapers.
//!
//! The operator chooses the files the command reads and the name of the file
//! it writes. Generated outputs are staged beside the destination with
//! owner-only permissions, then moved over it in one operation, so an
//! existing destination symlink is replaced rather than followed.

use std::io::Write;

use crate::py::{OsStyle, PyPath};
use crate::{Failure, OsError, Result};

/// `Path(path).suffix`, as Python 3.15 takes it.
pub fn suffix(path: &PyPath) -> String {
    let name = path.name().trim_start_matches('.');
    name.rfind('.')
        .map(|i| name[i..].to_string())
        .unwrap_or_default()
}

fn destination(path: &str, wanted: &str, kind: &str) -> Result<PyPath> {
    let destination = PyPath::new(path);
    if !suffix(&destination).eq_ignore_ascii_case(wanted) {
        return Err(Failure::Value(format!(
            "generated output must use a {kind} filename: {destination}"
        )));
    }
    Ok(destination)
}

/// Refuse a generated-output name that does not end in `.csv`.
pub fn csv_destination(path: &str) -> Result<PyPath> {
    destination(path, ".csv", ".csv")
}

/// Refuse a generated-output name that does not end in `.md`.
pub fn markdown_destination(path: &str) -> Result<PyPath> {
    destination(path, ".md", ".md")
}

/// Stage `bytes` beside `path`, then replace it. Any failure names the
/// destination the caller selected, never the staging file.
pub fn write(path: &str, bytes: &[u8], validator: fn(&str) -> Result<PyPath>) -> Result<()> {
    let destination = validator(path)?;
    let shown = destination.display();
    let renamed = |e: &std::io::Error, style: OsStyle| {
        Failure::Os(OsError::from_io(e, &shown, style).renamed(&shown))
    };
    let parent = destination.parent();
    let (temporary, mut file) =
        create_temporary(&parent).map_err(|e| renamed(&e, OsStyle::Errno))?;
    let result = file
        .write_all(bytes)
        .map_err(|e| renamed(&e, OsStyle::Errno))
        .and_then(|()| {
            drop(file);
            std::fs::rename(&temporary, destination.fs()).map_err(|e| renamed(&e, OsStyle::Native))
        });
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

/// `tempfile.mkstemp(prefix=".payday-super-checker-", suffix=".tmp", dir=parent)`.
fn create_temporary(parent: &PyPath) -> std::io::Result<(std::path::PathBuf, std::fs::File)> {
    const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789_";
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_nanos() as u64)
        ^ u64::from(std::process::id()).rotate_left(32);
    let mut last = None;
    for _ in 0..100 {
        let name: String = (0..8)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                LETTERS[(seed % LETTERS.len() as u64) as usize] as char
            })
            .collect();
        let candidate = parent
            .fs()
            .join(format!(".payday-super-checker-{name}.tmp"));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        match options.open(&candidate) {
            Ok(file) => return Ok((candidate, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => last = Some(e),
            Err(e) => return Err(e),
        }
    }
    Err(last.expect("every attempt collided"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suffixes_follow_python_3_15() {
        let s = |n: &str| suffix(&PyPath::new(n));
        assert_eq!(s("a."), ".");
        assert_eq!(s(".csv"), "");
        assert_eq!(s("..csv"), "");
        assert_eq!(s("a..csv"), ".csv");
        assert_eq!(s("a. csv"), ". csv");
        assert!(csv_destination("x/REPORT.CSV").is_ok());
        assert!(csv_destination("report.txt").is_err());
    }

    #[test]
    fn a_failed_replacement_keeps_the_destination_and_no_staged_file() {
        let dir = std::env::temp_dir().join(format!("payday-atomic-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let target = dir.join("report.csv");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("kept"), "old\n").unwrap();
        assert!(write(&target.to_string_lossy(), b"new\n", csv_destination).is_err());
        assert_eq!(
            std::fs::read_to_string(target.join("kept")).unwrap(),
            "old\n"
        );
        let staged = std::fs::read_dir(&dir)
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .ends_with(".tmp")
            })
            .count();
        assert_eq!(staged, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_csv_name_is_checked_before_anything_is_staged() {
        let dir = std::env::temp_dir().join(format!("payday-atomic-name-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("existing-report.txt");
        let err = write(&target.to_string_lossy(), b"new\n", csv_destination).unwrap_err();
        assert!(
            err.message().contains("must use a .csv filename"),
            "{}",
            err.message()
        );
        assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
