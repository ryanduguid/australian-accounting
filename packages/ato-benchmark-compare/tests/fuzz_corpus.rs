// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Replay the fuzz seed corpus so a broken harness fails ordinary tests.

use std::path::Path;

#[test]
fn every_seed_replays_without_a_panic() {
    let corpus = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz/corpus");
    let mut seeds: Vec<_> = std::fs::read_dir(corpus)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    seeds.sort();
    assert!(!seeds.is_empty());
    for seed in seeds {
        ato_benchmark_compare::fuzzing::read_csv(&std::fs::read(&seed).unwrap());
    }
}
