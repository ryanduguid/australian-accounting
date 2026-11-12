// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Offline supplementary evaluation; recordings are never executed.
#[path = "../evaluation/mod.rs"]
mod evaluation;

#[tokio::main]
async fn main() {
    std::process::exit(evaluation::main(std::env::args_os().skip(1).collect()).await);
}
