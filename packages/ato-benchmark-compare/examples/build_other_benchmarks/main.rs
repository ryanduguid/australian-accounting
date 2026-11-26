// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Build the other benchmark ranges from the ATO's industry pages.
//!
//! Development-time script. The engine only reads its output. Save each
//! industry's "in detail" page as text into a directory holding `index.json`
//! and one text file per page, then run:
//!
//!     cargo run --example build_other_benchmarks -- \
//!         --pages-dir <saved pages> \
//!         --dataset data/benchmarks-2023-24.json \
//!         --retrieved 2026-09-29 \
//!         --out data/other-benchmarks-2023-24.json

#[path = "../shared/mod.rs"]
mod shared;

mod build;
mod page;
#[cfg(test)]
mod tests;
mod text;

use std::collections::BTreeSet;
use std::fs;
use std::process::ExitCode;

use ato_benchmark_compare::py::PyPath;

use shared::{Outcome, Spec, last, to_json_indented};

const SPEC: Spec = Spec {
    prog: "build_other_benchmarks",
    usage: "usage: build_other_benchmarks [-h] --pages-dir PAGES_DIR --dataset DATASET \
            --retrieved RETRIEVED --out OUT [--expect-excluded BUSINESS_TYPE]",
    flags: &[
        ("--pages-dir", true),
        ("--dataset", true),
        ("--retrieved", true),
        ("--out", true),
        ("--expect-excluded", false),
    ],
};

const HELP: &str = "\n\nBuild the other benchmark ranges from the ATO's industry pages.\n\noptions:\n  \
     -h, --help            show this help message and exit\n  \
     --pages-dir PAGES_DIR\n  \
     --dataset DATASET\n  \
     --retrieved RETRIEVED\n                        ISO date the pages were saved\n  \
     --out OUT\n  \
     --expect-excluded BUSINESS_TYPE\n                        \
     a business type whose page is known to disagree; repeat for each one\n";

fn run(args: &[String]) -> Outcome {
    let values = match SPEC.parse(args) {
        Ok(Some(values)) => values,
        Ok(None) => {
            return Outcome {
                code: 0,
                stdout: format!("{}{HELP}", SPEC.usage),
                stderr: String::new(),
            };
        }
        Err(text) => return Outcome::fail(2, text),
    };
    let expect: BTreeSet<String> = values
        .get("--expect-excluded")
        .into_iter()
        .flatten()
        .cloned()
        .collect();
    let out = PyPath::new(&last(&values, "--out"));
    let built = match build::build(
        &PyPath::new(&last(&values, "--pages-dir")).fs(),
        &PyPath::new(&last(&values, "--dataset")).fs(),
        &last(&values, "--retrieved"),
        &expect,
    ) {
        Ok(built) => built,
        Err(err) => return Outcome::fail(1, format!("error: {}\n", err.message())),
    };
    if let Err(e) = fs::write(out.fs(), to_json_indented(&built).as_bytes()) {
        return Outcome::fail(1, format!("error: {}: {e}\n", out.display()));
    }
    let count = built["business_type_count"].as_u64().unwrap_or(0);
    let stderr = built["excluded_pages"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|item| {
            format!(
                "excluded: {}\n",
                item["reason"].as_str().unwrap_or_default()
            )
        })
        .collect();
    Outcome {
        code: 0,
        stdout: format!(
            "wrote {}: {count} business types, {} ranges\n",
            out.display(),
            build::count_filled_ranges(&built)
        ),
        stderr,
    }
}

fn main() -> ExitCode {
    run(&shared::os_args()).emit()
}
