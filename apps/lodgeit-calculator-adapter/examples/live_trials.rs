// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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
    // Discard the executable name; `live::cli` sends nothing without `--enable-network`.
    // nosemgrep: rust.lang.security.args.args
    let args: Vec<String> = std::env::args().skip(1).collect();
    ExitCode::from(live::cli(&args))
}
