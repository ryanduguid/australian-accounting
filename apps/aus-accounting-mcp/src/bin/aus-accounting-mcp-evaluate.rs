// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

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
