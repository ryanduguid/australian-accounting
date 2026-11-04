// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: FSL-1.1-ALv2

//! Fuzz the contracts CSV reader with fabricated schedules under the default
//! mapping, then measure each contract read. A refusal is a result; a panic
//! is a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    the_wip_tally::fuzzing::read_contracts(data);
});
