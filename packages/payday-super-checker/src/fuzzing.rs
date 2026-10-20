//! The fuzz target's body, kept in the library so `cargo test` replays the
//! seed corpus through exactly what the fuzzer runs.

use std::path::PathBuf;

use crate::csv_io::{self, Mapping};
use crate::importers;

fn scratch() -> PathBuf {
    std::env::temp_dir().join(format!("payday-fuzz-{}", std::process::id()))
}

/// Read untrusted bytes as every CSV reader would. Refusals are results;
/// only a panic is a defect.
pub fn read_export(data: &[u8]) {
    let dir = scratch();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("export.csv");
    if std::fs::write(&path, data).is_err() {
        return;
    }
    let path = path.to_string_lossy().into_owned();
    let _ = importers::read_payroll(&path, None);
    let _ = importers::read_super(&path, None);
    let _ = csv_io::parse_rows(&path, &Mapping::default_mapping());
}
