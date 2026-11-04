// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: FSL-1.1-ALv2

//! The fuzz target's body, kept in the library so `cargo test` replays the
//! seed corpus through exactly what the fuzzer runs.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::csv_io::{self, Mapping};
use crate::importers;

/// A directory this process created under an unpredictable name. `create_dir`
/// fails on any existing path, link or not, so no one else can redirect the
/// export file.
fn scratch() -> Option<&'static Path> {
    static DIR: OnceLock<Option<PathBuf>> = OnceLock::new();
    DIR.get_or_init(|| {
        (0..16).find_map(|_| {
            let nanos = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()?
                .subsec_nanos();
            let dir =
                std::env::temp_dir().join(format!("payday-fuzz-{}-{nanos}", std::process::id()));
            std::fs::create_dir(&dir).ok().map(|()| dir)
        })
    })
    .as_deref()
}

/// Read untrusted bytes as every CSV reader would. Refusals are results;
/// only a panic is a defect.
pub fn read_export(data: &[u8]) {
    let Some(dir) = scratch() else { return };
    let path = dir.join("export.csv");
    if std::fs::write(&path, data).is_err() {
        return;
    }
    let path = path.to_string_lossy().into_owned();
    let _ = importers::read_payroll(&path, None);
    let _ = importers::read_super(&path, None);
    let _ = csv_io::parse_rows(&path, &Mapping::default_mapping());
}
