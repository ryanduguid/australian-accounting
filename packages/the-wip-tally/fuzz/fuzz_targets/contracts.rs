// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Fuzz the contracts CSV reader with fabricated schedules under the default
//! mapping, then measure each contract read. A refusal is a result; a panic
//! is a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    the_wip_tally::fuzzing::read_contracts(data);
});
