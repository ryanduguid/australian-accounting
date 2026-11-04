// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: FSL-1.1-ALv2

//! The fuzz target's body, kept in the library so `cargo test` replays the
//! seed corpus through exactly what the fuzzer runs.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::py::PyPath;
use crate::{rates, register};

/// A directory this process created under an unpredictable name. `create_dir`
/// fails on any existing path, link or not, so no one else can redirect the
/// input file.
fn scratch() -> Option<&'static Path> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        (0..16).find_map(|_| {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()?
                .subsec_nanos();
            let dir =
                std::env::temp_dir().join(format!("div7a-fuzz-{}-{nanos}", std::process::id()));
            std::fs::create_dir(&dir).ok().map(|()| dir)
        })
    })
    .as_deref()
}

/// Read untrusted bytes as a loan register, whose CSV reader decodes the bytes
/// itself, and as a benchmark rate table, whose reader takes the decoded text's
/// lines (`Reader::from_lines`). Refusals are results; only a panic is a defect.
pub fn read_csv(data: &[u8]) {
    let Some(dir) = scratch() else { return };
    let path = dir.join("input.csv");
    if std::fs::write(&path, data).is_err() {
        return;
    }
    let path = PyPath::new(&path.to_string_lossy());
    // The `load-rows` command takes any column list; requiring none sends every
    // row past the header check into the reader.
    let _ = register::load_rows(&path, &[]);
    let _ = rates::load_table(&path);
}

#[cfg(all(test, unix))]
mod tests {
    use super::read_csv;

    /// A link planted where the scratch directory used to be, a predictable
    /// path, must not receive the input.
    #[test]
    fn a_planted_link_does_not_redirect_the_input() {
        let base = std::env::temp_dir();
        let victim = base.join(format!("div7a-victim-{}", std::process::id()));
        let planted = base.join(format!("div7a-fuzz-{}", std::process::id()));
        std::fs::create_dir_all(&victim).unwrap();
        let _ = std::fs::remove_file(&planted);
        std::os::unix::fs::symlink(&victim, &planted).unwrap();
        read_csv(b"a,b\n1,2\n");
        let leaked = victim.join("input.csv").exists();
        let _ = std::fs::remove_file(&planted);
        let _ = std::fs::remove_dir_all(&victim);
        assert!(!leaked, "the input followed a planted link");
    }
}
