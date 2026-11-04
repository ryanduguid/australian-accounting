// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! The fuzz target's body, kept in the library so `cargo test` replays the
//! seed corpus through exactly what the fuzzer runs.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::py::PyPath;
use crate::{mapping, pnl};

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
            let dir = std::env::temp_dir().join(format!("ato-fuzz-{}-{nanos}", std::process::id()));
            std::fs::create_dir(&dir).ok().map(|()| dir)
        })
    })
    .as_deref()
}

/// Read untrusted bytes as a profit and loss export, in both of its forms, and
/// as an account mapping. Refusals are results; only a panic is a defect.
pub fn read_csv(data: &[u8]) {
    read_csv_in(
        scratch().expect("a fuzz run needs a scratch directory"),
        data,
    );
}

/// A run that cannot place its input has read nothing, so it must not pass.
fn read_csv_in(dir: &Path, data: &[u8]) {
    let path = dir.join("input.csv");
    std::fs::write(&path, data).expect("the input must reach the readers");
    let path = PyPath::new(&path.to_string_lossy());
    let _ = pnl::read(&path, None);
    // The `--amount-column` route picks the column by name instead of by shape.
    let _ = pnl::read(&path, Some("amount"));
    let _ = mapping::read_mapping(&path);
}

#[cfg(test)]
mod unwritable {
    use super::read_csv_in;

    /// A file stands in for the directory, which refuses the write on every
    /// platform and for every user, root included.
    #[test]
    #[should_panic(expected = "the input must reach the readers")]
    fn an_input_that_cannot_be_written_fails_the_run() {
        read_csv_in(&std::env::current_exe().unwrap(), b"a,b\n1,2\n");
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::read_csv;

    /// A link planted where the scratch directory used to be, a predictable
    /// path, must not receive the input.
    #[test]
    fn a_planted_link_does_not_redirect_the_input() {
        let base = std::env::temp_dir();
        let victim = base.join(format!("ato-victim-{}", std::process::id()));
        let planted = base.join(format!("ato-fuzz-{}", std::process::id()));
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
