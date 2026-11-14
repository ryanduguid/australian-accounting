# Aus Accounting MCP core

The tool call of the native Aus Accounting MCP server without a transport, for
the 6 tools a hosted connector can serve: `calculate_tax_worksheet`,
`calc_payday_super_deadline`, `get_div7a_benchmark_rate`, `review_div7a_loan`,
`refuse_div7a` and `get_data_freshness`. `call(name, arguments)` checks the
arguments, applies the effective input models, runs the linked engine and
validates the result against the tool's declared output schema, returning the
complete `tools/call` result. It reads no clock, file or network and has no MCP
SDK, so it builds for `wasm32-unknown-unknown` as well as natively.

The code and data come from the draft native server (pull request 599, branch
`ryan/b1-rust-mcp-2026-11-08`, head `caff9b7`): `AccountingServer::call`, the
adapters for these tools, `contracts.rs`, `validation.rs`, and those tools'
entries in `catalogue.v1.json` and `effective-inputs.v1.json`. The draft's other
tools, its retrieval modules and its stdio server are not here. Its
`get_data_freshness` also reports engines this core does not link (benchmarks,
lodgment program, GST rules and reviewed rates), so here it reports only the
datasets of the tools above; `freshness::report_on` takes the extra rows.

The engines are linked by path, so results follow `packages/` at this commit.

One change from the draft: `review_div7a_loan` refuses a remaining term that
cannot hold on a loan the gate passed, as the Python server on `main` does
(`_remaining_term_refusal`, `_term_left_refusal` and `_refuse` in
`aus_accounting_mcp/adapters/div7a.py`). The engine's public
`minimum_yearly_repayment` takes no loan term and checks no remaining-term
limit, so the draft's adapter worked out a lower repayment for, say, a remaining
term of 8 years on a loan whose maximum term is 7. `adapters/div7a.rs` now adds
`REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM`,
`REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM` and
`REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED` with the same conditions
and wording. [`tests/div7a_python.rs`](tests/div7a_python.rs) compares the
verdicts, codes, reasons and figures with the Python server's results, recorded
in `tests/fixtures/python-div7a.json`. The draft server needs this adapter, or
these refusals, before it replaces the Python server.

## Check

Use Rust 1.99. From this directory:

```sh
rustup target add wasm32-unknown-unknown
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --target wasm32-unknown-unknown --lib -- -D warnings
cargo test --locked
cargo build --locked --release --target wasm32-unknown-unknown
```

`cargo test` replays the 32 cases in [`../conformance/cases.json`](../conformance/cases.json)
through `call`.

Licensed under the [Ryan Duguid Public Attribution Licence 1.0](../../../LICENSE); see [NOTICE](../../../NOTICE).
