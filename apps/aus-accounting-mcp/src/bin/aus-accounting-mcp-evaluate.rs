// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

//! Offline supplementary evaluation; recordings are never executed.
#[path = "../evaluation/mod.rs"]
mod evaluation;

#[tokio::main]
async fn main() {
    let args = aus_accounting_mcp::command_line::command_words()
        .into_iter()
        .map(std::ffi::OsString::from)
        .collect();
    std::process::exit(evaluation::main(args).await);
}
