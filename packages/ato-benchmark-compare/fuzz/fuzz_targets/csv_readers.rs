// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Fuzz the CSV readers with fabricated files: as a profit and loss export and
//! as an account mapping. A refusal is a result; a panic is a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    ato_benchmark_compare::fuzzing::read_csv(data);
});
