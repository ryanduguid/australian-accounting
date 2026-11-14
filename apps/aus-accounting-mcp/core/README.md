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

`review_div7a_loan` keeps the draft's adapter, which calls the engine's public
`minimum_yearly_repayment` and so lacks the remaining-term refusals the Python
server on `main` adds (`REFUSED_REMAINING_TERM_ABOVE_MAXIMUM_TERM`,
`REFUSED_REMAINING_TERM_ABOVE_LOAN_TERM` and
`REFUSED_REMAINING_TERM_ABOVE_TERM_LESS_YEARS_ELAPSED`). With a remaining term
of 8 years on a loan whose maximum term is 7, this core reports the minimum
yearly repayment as met where the Python server refuses. Close that gap before
anything serves this tool.

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
