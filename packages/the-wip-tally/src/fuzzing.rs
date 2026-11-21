// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The fuzz target's body, kept in the library so `cargo test` replays the
//! seed corpus through exactly what the fuzzer runs.

use crate::{csv_io, engine};

/// Read untrusted bytes as a contracts CSV under the default mapping, then
/// measure each contract read. Refusals are results; only a panic is a defect.
pub fn read_contracts(data: &[u8]) {
    let mapping = csv_io::load_mapping(None).expect("the default mapping must load");
    if let Ok(contracts) = csv_io::read_contracts("contracts.csv", data, &mapping) {
        for contract in &contracts {
            let _ = engine::measure(contract);
        }
    }
}
