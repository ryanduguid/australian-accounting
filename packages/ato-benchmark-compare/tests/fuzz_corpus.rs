// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

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
