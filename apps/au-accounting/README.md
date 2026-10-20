# au-accounting

One command for the 7 Rust engines in this repository. `au-accounting ENGINE ...`
passes the remaining arguments to that engine's library, so each subcommand takes
the same arguments, reads the same JSON on stdin and prints the same output as the
engine's own command.

| Subcommand | Engine | Engine's own command |
| --- | --- | --- |
| `benchmarks` | [ATO Benchmark Compare](../../packages/ato-benchmark-compare/README.md) | `ato-benchmark-compare` |
| `div7a` | [Division 7A Loan Review](../../packages/div7a-loan-review/README.md) | `div7a-loan-review` |
| `exchequer` | [The Exchequer Tally](../../packages/the-exchequer-tally/README.md) | `the-exchequer-tally` |
| `payday-super` | [Payday Super Checker](../../packages/payday-super-checker/README.md) | `payday-super-check` |
| `solomons` | [Solomon's Sword](../../packages/solomons-sword/README.md) | `solomons-sword` |
| `tax` | [Australian Tax Calculators](../../packages/australian-tax-calculators/README.md) | `australian-tax-calculators` |
| `wip` | [The WIP Tally](../../packages/the-wip-tally/README.md) | `wip-tally` |

The engine's own command name also works as the subcommand, for example
`au-accounting div7a-loan-review`. GST Review is still a Python package, so it has
no subcommand; install it from [PyPI](https://pypi.org/project/gst-review/).

## Install

With [Rust](https://rustup.rs/) 1.99 or later:

```bash
cargo install --locked --git https://github.com/ryanduguid/australian-accounting au-accounting
```

Prebuilt binaries are not published yet. Once a release publishes them,
`cargo binstall au-accounting` will download one for Windows, macOS or Linux; see
the [root README](../../README.md#install-the-rust-engines).

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
Solomon's Sword and The WIP Tally do not enable both on their own, so for them
the JSON commands sort object keys and read numbers as the default parser
does, which gives the same output. `tests/parity.py` holds every engine to its
own command, byte for byte, over each engine's frozen corpora and edge-case
JSON; CI runs it on Windows, macOS and Linux. What remains different:

- For an integer beyond the 64-bit range sent to those 2 engines, the nearest
  f64 can differ in its last digit, because their own parser rounds such
  integers differently. Both engines take amounts as decimal strings.
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
