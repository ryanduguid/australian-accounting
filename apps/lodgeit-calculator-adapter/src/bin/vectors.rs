// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: AGPL-3.0-or-later
// Additional terms under AGPL-3.0 section 7(b): see NOTICE.

//! The differential vector runner: JSON cases on stdin, results on stdout.
//! It runs on the CLI's 512 MB stack, which deep cases need.
fn main() -> std::io::Result<()> {
    std::thread::Builder::new()
        .stack_size(512 << 20)
        .spawn(|| {
            lodgeit_calculator_adapter::vectors::run(
                std::io::stdin().lock(),
                std::io::BufWriter::new(std::io::stdout().lock()),
            )
        })?
        .join()
        .expect("the runner thread")
}
