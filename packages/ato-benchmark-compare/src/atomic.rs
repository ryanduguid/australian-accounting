// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! `atomic_write_text`: stage beside the destination, flush, then replace it
//! in one filesystem operation so a failed rewrite cannot truncate it.

use std::io::Write;

use crate::Failure;
use crate::py::{self, OsStyle, PyPath};

pub fn atomic_write(path: &PyPath, bytes: &[u8]) -> Result<(), Failure> {
    let parent = path.parent();
    let os_error =
        |e: std::io::Error, name: &str| Failure::Value(py::oserror_str(&e, name, OsStyle::Native));
    std::fs::create_dir_all(parent.fs()).map_err(|e| os_error(e, &parent.display()))?;
    let (temporary, mut handle) = create_temporary(&parent, path.name())?;
    let written = handle.write_all(bytes).and_then(|()| handle.sync_all());
    drop(handle);
    let result = written
        .map_err(|e| os_error(e, &temporary.display().to_string()))
        .and_then(|()| {
            std::fs::rename(&temporary, path.fs())
                .map_err(|e| os_error(e, &temporary.display().to_string()))
        });
    if result.is_err() {
        let _ = std::fs::remove_file(&temporary);
    }
    result
}

/// `tempfile.mkstemp(dir=parent, prefix=f".{name}.", suffix=".tmp")`.
fn create_temporary(
    parent: &PyPath,
    name: &str,
) -> Result<(std::path::PathBuf, std::fs::File), Failure> {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_nanos() as u64)
        ^ u64::from(std::process::id()).rotate_left(32);
    create_temporary_from(parent, name, seed)
}

/// The 8-letter suffixes `mkstemp` tries, from one seed.
fn suffixes(mut seed: u64) -> impl Iterator<Item = String> {
    const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789_";
    std::iter::repeat_with(move || {
        (0..8)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                LETTERS[(seed % LETTERS.len() as u64) as usize] as char
            })
            .collect()
    })
}

fn create_temporary_from(
    parent: &PyPath,
    name: &str,
    seed: u64,
) -> Result<(std::path::PathBuf, std::fs::File), Failure> {
    for suffix in suffixes(seed).take(100) {
        let candidate = parent.fs().join(format!(".{name}.{suffix}.tmp"));
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        // `mkstemp` creates its file readable by the owner only.
        #[cfg(unix)]
        std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
        match options.open(&candidate) {
            Ok(file) => return Ok((candidate, file)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => {
                return Err(Failure::Value(py::oserror_str(
                    &e,
                    &candidate.display().to_string(),
                    OsStyle::Errno,
                )));
            }
        }
    }
    Err(Failure::Value(
        "[Errno 17] No usable temporary file name found".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn staged(dir: &std::path::Path) -> Vec<String> {
        std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .filter(|n| n.ends_with(".tmp"))
            .collect()
    }

    #[test]
    fn a_failed_replacement_keeps_the_destination_and_no_staged_file() {
        let dir = std::env::temp_dir().join(format!("ato-atomic-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        // A non-empty directory cannot be replaced by a file on any platform.
        let target = dir.join("comparison.json");
        std::fs::create_dir_all(&target).unwrap();
        std::fs::write(target.join("kept"), "old reviewed result\n").unwrap();
        let path = PyPath::new(&target.to_string_lossy());
        assert!(atomic_write(&path, b"new result\n").is_err());
        assert_eq!(
            std::fs::read_to_string(target.join("kept")).unwrap(),
            "old reviewed result\n"
        );
        assert!(staged(&dir).is_empty());
        let fresh = PyPath::new(&dir.join("written.json").to_string_lossy());
        atomic_write(&fresh, b"new result\n").unwrap();
        assert_eq!(
            std::fs::read(dir.join("written.json")).unwrap(),
            b"new result\n"
        );
        assert!(staged(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A fresh folder under the crate's ignored `target/`, the working
    /// directory cargo runs unit tests in.
    fn scratch(tag: &str) -> std::path::PathBuf {
        let dir =
            std::path::Path::new("target").join(format!("atomic-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_taken_name_moves_on_to_the_next_suffix() {
        let dir = scratch("taken");
        let parent = PyPath::new(&dir.to_string_lossy());
        let names: Vec<String> = suffixes(7).take(2).collect();
        assert_ne!(names[0], names[1]);
        std::fs::write(dir.join(format!(".out.{}.tmp", names[0])), "taken").unwrap();
        let (path, _) = create_temporary_from(&parent, "out", 7).unwrap();
        assert_eq!(path, dir.join(format!(".out.{}.tmp", names[1])));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_hundred_taken_names_is_errno_17() {
        let dir = scratch("full");
        let parent = PyPath::new(&dir.to_string_lossy());
        for suffix in suffixes(7).take(100) {
            std::fs::write(dir.join(format!(".out.{suffix}.tmp")), "taken").unwrap();
        }
        assert_eq!(
            create_temporary_from(&parent, "out", 7).unwrap_err(),
            Failure::Value("[Errno 17] No usable temporary file name found".into())
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_missing_folder_is_reported_with_its_errno() {
        let dir = scratch("missing").join("absent");
        let parent = PyPath::new(&dir.to_string_lossy());
        let failure = create_temporary_from(&parent, "out", 7)
            .map(|_| ())
            .unwrap_err();
        assert!(matches!(failure, Failure::Value(_)));
        let message = failure.to_string();
        assert!(
            message.starts_with("[Errno 2] No such file or directory: "),
            "{message}"
        );
        let _ = std::fs::remove_dir_all(dir.parent().unwrap());
    }
}
