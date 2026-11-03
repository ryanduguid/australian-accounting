//! The fuzz target's body, kept in the library so `cargo test` replays the
//! seed corpus through exactly what the fuzzer runs.

use std::path::PathBuf;

use crate::py::PyPath;
use crate::{rates, register};

fn scratch() -> PathBuf {
    std::env::temp_dir().join(format!("div7a-fuzz-{}", std::process::id()))
}

/// Read untrusted bytes as a loan register, whose CSV reader decodes the bytes
/// itself, and as a benchmark rate table, whose reader takes the decoded text's
/// lines (`Reader::from_lines`). Refusals are results; only a panic is a defect.
pub fn read_csv(data: &[u8]) {
    let dir = scratch();
    let _ = std::fs::create_dir_all(&dir);
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
