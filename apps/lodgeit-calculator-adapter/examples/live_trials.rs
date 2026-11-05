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
use std::path::Path;
use std::process::ExitCode;

use lodgeit_calculator_adapter::trials::live;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let value = |flag: &str| {
        args.iter()
            .position(|arg| arg == flag)
            .and_then(|at| args.get(at + 1))
    };
    let (Some(base_url), Some(out), true) = (
        value("--base-url"),
        value("--out"),
        args.iter().any(|arg| arg == "--enable-network"),
    ) else {
        eprintln!("usage: live_trials --enable-network --base-url URL --out FILE");
        return ExitCode::from(2);
    };
    match live::run_to_file(base_url, Path::new(out)) {
        Ok(None) => ExitCode::SUCCESS,
        Ok(Some(case)) => {
            eprintln!("stopped at {case}; read {out}");
            ExitCode::from(1)
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}
