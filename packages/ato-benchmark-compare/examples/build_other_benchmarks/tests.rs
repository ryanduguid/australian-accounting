// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

use ato_benchmark_compare::dataset::{canonical_digest, normalise};
use serde_json::Value;

use crate::shared::{parity, to_json_indented};

const RECORDED: &str = include_str!("../../tests/data/builders/build_other_benchmarks-python.json");

/// Every recorded answer of the Python `build_other_benchmarks.py`, replayed.
#[test]
fn port_answers_as_the_python_script_did() {
    let cases = parity::cases(RECORDED);
    assert!(cases.len() >= 30, "recorded cases");
    for case in &cases {
        let dir = parity::scratch("build-other", case["name"].as_str().unwrap());
        parity::write_files(&dir, case);
        let got = crate::run(&parity::args(&dir, case));
        parity::check(&dir, case, &got);
    }
}

#[test]
fn help_and_repeated_flags_follow_argparse() {
    let help = crate::run(&["--he".to_string()]);
    assert_eq!(help.code, 0);
    assert!(
        help.stdout
            .starts_with("usage: build_other_benchmarks [-h]")
    );
    let args: Vec<String> = ["--out", "a", "--out=b", "--", "-x"]
        .map(String::from)
        .to_vec();
    let err = crate::SPEC.parse(&args).unwrap_err();
    assert!(err.ends_with(
        "error: the following arguments are required: --pages-dir, --dataset, --retrieved\n"
    ));
    let negative = [
        "--pages-dir",
        "-1",
        "--dataset",
        "d",
        "--retrieved",
        "r",
        "--out",
        "o",
    ]
    .map(String::from)
    .to_vec();
    let values = crate::SPEC.parse(&negative).unwrap().unwrap();
    assert_eq!(values["--pages-dir"], ["-1"]);
}

#[test]
fn shipped_page_file_digest_and_layout_match_canonical_and_indent_one() {
    let dataset = include_str!("../../data/benchmarks-2023-24.json");
    let pages_raw = include_str!("../../data/other-benchmarks-2023-24.json");
    let pages: Value = serde_json::from_str(pages_raw).unwrap();
    assert_eq!(
        canonical_digest(dataset).unwrap(),
        pages["source"]["dataset_json_sha256"].as_str().unwrap()
    );
    assert_eq!(to_json_indented(&pages), pages_raw);
}

#[test]
fn builder_folds_names_as_engine_does() {
    let cases = [
        ("Fish and chips shops", "fish and chips shops"),
        (
            "Bakeries \u{2013} and \u{2018}hot\u{2019} bread",
            "bakeries and hot bread",
        ),
        ("Caf\u{00e9}s", "cafe\u{0301}s"),
        ("STRASSE \u{00df}", "strasse ss"),
        ("  spaced\tout  ", "spaced out"),
    ];
    for (text, folded) in cases {
        assert_eq!(normalise(text), folded);
    }
}
