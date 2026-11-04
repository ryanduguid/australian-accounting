// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Fuzz the CSV reader with fabricated files: as a loan register read from the
//! bytes, and as a benchmark rate table read from the decoded lines through
//! `Reader::from_lines`. A refusal is a result; a panic is a crash.
#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    div7a_loan_review::fuzzing::read_csv(data);
});
