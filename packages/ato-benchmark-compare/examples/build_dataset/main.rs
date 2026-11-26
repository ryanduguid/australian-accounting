// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! Convert an ATO small business benchmarks workbook into the shipped JSON dataset.
//!
//! Development-time script using `[dev-dependencies]` (`zip`, `roxmltree`), so
//! the release binary links none of it. Run when the ATO publishes a new year:
//!
//!     cargo run --example build_dataset -- \
//!         --xlsx small-business-benchmarks-2023-24-data.xlsx \
//!         --year 2023-24 \
//!         --resource-url https://data.gov.au/... \
//!         --resource-name "2023-24 Benchmarks" \
//!         --resource-last-modified 2026-03-15T22:07:39.563377 \
//!         --retrieved 2026-08-13 \
//!         --out data/benchmarks-2023-24.json

#[path = "../shared/mod.rs"]
mod shared;

mod bands;
#[cfg(test)]
mod tests;
mod xlsx;

use std::fs;
use std::process::ExitCode;

use ato_benchmark_compare::py::PyPath;

use bands::Resource;
use shared::{Outcome, Spec, last, to_json_indented};

const SPEC: Spec = Spec {
    prog: "build_dataset",
    usage: "usage: build_dataset [-h] --xlsx XLSX --year YEAR --resource-name RESOURCE_NAME \
            --resource-url RESOURCE_URL --resource-last-modified RESOURCE_LAST_MODIFIED \
            --retrieved RETRIEVED --out OUT",
    flags: &[
        ("--xlsx", true),
        ("--year", true),
        ("--resource-name", true),
        ("--resource-url", true),
        ("--resource-last-modified", true),
        ("--retrieved", true),
        ("--out", true),
    ],
};

const HELP: &str = "\n\nConvert an ATO small business benchmarks workbook into the shipped JSON dataset.\n\noptions:\n  \
     -h, --help            show this help message and exit\n  \
     --xlsx XLSX\n  \
     --year YEAR           benchmark year, for example \"2023-24\"\n  \
     --resource-name RESOURCE_NAME\n  \
     --resource-url RESOURCE_URL\n  \
     --resource-last-modified RESOURCE_LAST_MODIFIED\n  \
     --retrieved RETRIEVED\n                        ISO date the file was downloaded\n  \
     --out OUT\n";

/// Writes the dataset to `out`, creating its folder as `build_dataset.py` did.
fn write_dataset(out: &PyPath, text: &str) -> Result<(), String> {
    let out_fs = out.fs();
    if let Some(parent) = out_fs.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
    }
    fs::write(&out_fs, text.as_bytes()).map_err(|e| format!("{}: {e}", out.display()))
}

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
    let resource = Resource {
        year: last(&values, "--year"),
        name: last(&values, "--resource-name"),
        url: last(&values, "--resource-url"),
        last_modified: last(&values, "--resource-last-modified"),
        retrieved: last(&values, "--retrieved"),
    };
    let (xlsx, out) = (
        PyPath::new(&last(&values, "--xlsx")),
        PyPath::new(&last(&values, "--out")),
    );
    let (dataset, anomalies) = match bands::build(&xlsx.fs(), &resource) {
        Ok(built) => built,
        Err(msg) => return Outcome::fail(1, format!("error: {msg}\n")),
    };
    let stderr: String = anomalies
        .iter()
        .map(|l| format!("anomaly: {l}\n"))
        .collect();
    if let Err(msg) = write_dataset(&out, &to_json_indented(&dataset)) {
        return Outcome::fail(1, format!("{stderr}error: {msg}\n"));
    }
    let count = dataset["business_type_count"].as_u64().unwrap_or(0);
    let sha = dataset["source"]["sha256"].as_str().unwrap_or_default();
    Outcome {
        code: 0,
        stdout: format!(
            "wrote {}: {count} business types, sha256 {}\n",
            out.display(),
            &sha[..16]
        ),
        stderr,
    }
}

fn main() -> ExitCode {
    run(&shared::os_args()).emit()
}
