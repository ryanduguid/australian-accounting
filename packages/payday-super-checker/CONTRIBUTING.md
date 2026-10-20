# Contributing

This tool checks contributions against the payday-super deadline and estimates SG-charge exposure. A human reads the output and decides what to do with it. Keep that boundary: no contribution should give the tool authority to submit an SGC statement, move money, or present an estimate as an assessment.

## Data boundary

- Use fabricated data. The `.gitignore` blocks `.csv`, `.xls`, `.xlsx`, `.pdf`, `.ofx`, `.qif` and `.aba` files, with exceptions for `examples/*.csv` and the evaluation fixtures named one by one under `evaluation/`. Put new general fixtures in the examples directory.
- Never commit an `.aba` file, redacted or not. It carries the account numbers a payroll run pays into.
- Keep employee names, membership numbers, TFNs, ABNs tied to a real employer, and screenshots of a live payroll system out of the repository.

## Legislation and calendar changes

- Trace every rate, threshold and deadline to a primary source, and cite the provision or instrument in your pull request.
- Business days follow one national calendar. A day is not a business day if it falls on a Saturday, a Sunday, or a public holiday declared for the whole of any State, the ACT or the NT. Regional holidays do not stop the clock.
- Vendor column profiles ship `"verified": false` because no Australian payroll vendor publishes its export header list. To flip a profile to `"verified": true`, quote a real header row from that vendor's export in the pull request. Headers only, no data.

## Local verification

The engine is a Rust crate with the toolchain pinned in `rust-toolchain.toml`.
From this directory:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

`tests/parity.rs` replays the frozen record of the Python 0.2.2 engine's answers;
a change that alters any of them fails it. The calendar generator and the
workbook builder in `tools/` are Python and are tested from the repository root.
CI runs the crate on Ubuntu and Windows, and `fuzz/` holds the cargo-fuzz target
for the CSV readers.

## Pull requests

Name the rule you changed and the test that pins it. A behaviour change needs a focused test under `tests/` or in the module it changes. Run that test against the old code first. If it passes there too, it is not testing your change.

When you change a rule, search for everything else that states or enforces it. An enumeration in the report, a README table and a docstring can all keep asserting the old rule long after you have changed the code.

For a potential security vulnerability, follow [SECURITY.md](SECURITY.md) rather than opening an issue.

## Releases

The Rust 1.0.0 binary is unreleased and has no publisher yet. [RELEASING.md](RELEASING.md)
records how the Python releases through v0.2.2 were published and how to verify
them. The product remains an experimental review aid, not a compliance
determination.
