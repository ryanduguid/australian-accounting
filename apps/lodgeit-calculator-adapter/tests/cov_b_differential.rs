// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.
//! The `differential` build's fixed clock, which the harness compares with
//! Python's fixed `_now()`.
#![cfg(feature = "differential")]

use lodgeit_calculator_adapter::client;

#[test]
fn the_differential_clock_reads_the_fixed_instant() {
    assert_eq!(client::now(), "2026-01-01T00:00:00+00:00");
}
