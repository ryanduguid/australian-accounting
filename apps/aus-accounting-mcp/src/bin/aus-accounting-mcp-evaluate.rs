// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Offline supplementary evaluation; recordings are never executed.
#[path = "../evaluation/mod.rs"]
mod evaluation;

#[tokio::main]
async fn main() {
    let args = std::env::args()
        .skip(1)
        .map(std::ffi::OsString::from)
        .collect();
    std::process::exit(evaluation::main(args).await);
}
