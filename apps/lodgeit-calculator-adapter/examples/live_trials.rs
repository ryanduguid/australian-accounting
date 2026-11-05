// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! One manual pass over the trial cases against the live service. Read
//! `docs/live-probe.md` first:
//!
//!   cargo run --locked --example live_trials -- --enable-network \
//!     --base-url https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app \
//!     --out out/live-trials.json
//!
//! Without `--enable-network` it refuses and sends nothing.
use std::process::ExitCode;

use lodgeit_calculator_adapter::trials::live;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    ExitCode::from(live::cli(&args))
}
