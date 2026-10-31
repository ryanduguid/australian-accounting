# au-accounting

One command for the 8 Rust engines in this repository. `au-accounting ENGINE ...`
passes the remaining arguments to that engine's library, so each subcommand takes
the same arguments, reads the same JSON on stdin and prints the same output as the
engine's own command.

| Subcommand | Engine | Engine's own command |
| --- | --- | --- |
| `benchmarks` | [ATO Benchmark Compare](../../packages/ato-benchmark-compare/README.md) | `ato-benchmark-compare` |
| `div7a` | [Division 7A Loan Review](../../packages/div7a-loan-review/README.md) | `div7a-loan-review` |
| `exchequer` | [The Exchequer Tally](../../packages/the-exchequer-tally/README.md) | `the-exchequer-tally` |
| `gst-review` | [GST Review](../../packages/gst-review/README.md) | `gst-review` |
| `payday-super` | [Payday Super Checker](../../packages/payday-super-checker/README.md) | `payday-super-check` |
| `solomons` | [Solomon's Sword](../../packages/solomons-sword/README.md) | `solomons-sword` |
| `tax` | [Australian Tax Calculators](../../packages/australian-tax-calculators/README.md) | `australian-tax-calculators` |
| `wip` | [The WIP Tally](../../packages/the-wip-tally/README.md) | `wip-tally` |

The engine's own command name also works as the subcommand, for example
`au-accounting div7a-loan-review`. GST Review takes `review-coding` or
`reconcile-bas` with one JSON object on stdin.

## Install

With [Rust](https://rustup.rs/) 1.99 or later:

```bash
cargo install --locked --git https://github.com/ryanduguid/australian-accounting au-accounting
```

Prebuilt binaries are not published yet; see the
[root README](../../README.md#install-the-rust-engines).

## Use

```bash
au-accounting --help
au-accounting div7a --help
au-accounting div7a rate --year 2026-27
au-accounting payday-super --api-help
```

Each engine's README documents its commands, JSON schemas, scope and limits. Help
and error text name the engine's own command, as that engine prints them.

## Differences from the engines' own commands

One binary builds every engine against one `serde_json` with the
`preserve_order` and `arbitrary_precision` features. The Exchequer Tally,
Solomon's Sword and The WIP Tally do not enable both on their own. The JSON
commands sort keys where required and adapt numbers to the engine's own
features. Default-number engines use serde_json's decimal conversion rather
than Rust's string parser, which can round differently. Numbers are converted
once, before evaluation. `tests/parity.py` holds every engine to its
own command, byte for byte, over each engine's frozen corpora and edge-case
JSON; CI runs it on Windows, macOS and Linux. What remains different:

- Australian Tax Calculators decodes non-UTF-8 arguments lossily instead of
  stopping.

## Checks

From this directory:

```bash
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

Outputs are review aids, not advice or lodgements. Each engine's `DISCLAIMER.md`
applies to its subcommand.
