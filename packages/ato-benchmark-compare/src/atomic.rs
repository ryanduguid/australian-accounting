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
    const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789_";
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |t| t.as_nanos() as u64)
        ^ u64::from(std::process::id()).rotate_left(32);
    for _ in 0..100 {
        let suffix: String = (0..8)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                LETTERS[(seed % LETTERS.len() as u64) as usize] as char
            })
            .collect();
        let candidate = parent.fs().join(format!(".{name}.{suffix}.tmp"));
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
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
}
