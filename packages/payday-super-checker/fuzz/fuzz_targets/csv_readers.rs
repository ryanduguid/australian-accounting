// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Fuzz the CSV readers with fabricated exports: the payroll and super-export
//! importers with vendor detection, and the canonical row parser under the
//! default column mapping. A refusal is a result; a panic is a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    payday_super_checker::fuzzing::read_export(data);
});
