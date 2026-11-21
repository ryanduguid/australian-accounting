// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Fuzz the CSV reader with fabricated files: as a loan register read from the
//! bytes, and as a benchmark rate table read from the decoded lines through
//! `Reader::from_lines`. A refusal is a result; a panic is a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    div7a_loan_review::fuzzing::read_csv(data);
});
