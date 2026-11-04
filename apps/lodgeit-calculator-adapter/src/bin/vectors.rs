// Copyright 2026 Ryan Duguid
// SPDX-License-Identifier: FSL-1.1-ALv2

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
