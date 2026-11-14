# Aus Accounting MCP calculators on Cloudflare Workers

An undeployed candidate: a stateless Streamable HTTP MCP endpoint for 6 tools of
the native Aus Accounting MCP server, `calculate_tax_worksheet`,
`calc_payday_super_deadline`, `get_div7a_benchmark_rate`, `review_div7a_loan`,
`refuse_div7a` and `get_data_freshness`. Each tool call runs
[the core](../../apps/aus-accounting-mcp/core/) compiled to WebAssembly, so the
engines' results pass through unchanged. Nothing in this repository deploys it,
and no hosted address serves it.

## Build and check

Use Rust 1.99 and Node.js 26.11.1. From this directory:

```sh
rustup target add wasm32-unknown-unknown
cargo install worker-build --version 0.8.7 --locked
npm ci --ignore-scripts
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo clippy --locked --target wasm32-unknown-unknown --lib -- -D warnings
cargo test --locked
AUS_ACCOUNTING_CONNECTOR_SERVE_UNTIL=2000-01-01 worker-build --release --out-dir build-expired
worker-build --release
npm test
npm run dry-run
```

`npm test` runs both bundles in local workerd through Miniflare, with every
outbound request recorded and refused. It posts the 32 conformance cases, and
the tools without required arguments, to `/mcp` in both protocol eras and
compares each complete `result` byte for byte with
[`tests/fixtures/reference.json`](tests/fixtures/reference.json), the native
stdio server's output (see its `source`). The one difference allowed is
`_meta["io.modelcontextprotocol/serverInfo"]` on 2026-07-28 results, which
names the server producing them. `get_data_freshness` without a tool is
compared after dropping the datasets of engines this Worker does not link.
`npm run measure` prints local wall times for each case; they are not
Cloudflare's CPU accounting.

To record the fixture again, build the native server's `aus-accounting-mcp`
binary and run `node tests/record-reference.mjs <binary> <its commit>`. Draft
PR 599 needs the core's PAYG withholding entry in `NARROWED_SCOPE` first (see
[the core README](../../apps/aus-accounting-mcp/core/README.md)); the fixture
here was recorded with it.

`worker` and `worker-build` stay at 0.8.7, as in the badge Worker: the 0.9.0
build imports its WebAssembly in a form the stable Miniflare 4.20260730.0
runtime does not run, so that Worker hung on every request in the harness.

Do not enable Cargo release stripping: the wasm-bindgen packaging step needs
the target feature metadata.

## HTTP contract

Only `POST /mcp` reaches the handler; a query string is ignored and any other
path gets 404. The handler speaks MCP 2026-07-28 (`server/discover`,
`tools/list`, `tools/call`) and, without sessions, the `initialize` handshake of
2025-11-25 and 2025-06-18. It never sends `Mcp-Session-Id` and ignores one it
receives.

| Request | Reply |
| --- | --- |
| GET, DELETE or another method | 405, `Allow: POST` |
| `Origin` other than claude.ai, claude.com, chatgpt.com or duguid.com.au | 403 |
| Body over 16 KiB, by `Content-Length` or as read | 413, before parsing |
| Body not `application/json` | 415 |
| JSON-RPC response, batch or malformed JSON | 400 |
| Notification | 202, no body |
| Protocol header and body version differ, or a missing `Mcp-Method` or `Mcp-Name` | 400, `-32020` |
| Unsupported protocol version | 400, `-32022` with the supported versions |
| Unknown method | 404, `-32601` (200 before 2026-07-28, where 404 ends a session) |
| Over 60 requests a minute from one address in one location | 429, `Retry-After: 60` |
| Rate limiter unavailable | 503 |

Each release refuses every tool call from its `serve_until` date, fixed by
[`build.rs`](build.rs) as the earliest of the day before the next 1 July, the end
of the GIC table's last quarter and the business-day calendar's
`verified_until`. The release, its commit and that date are in the
`x-connector-release` header of every reply and in `serverInfo`.

## Deployment status

Undeployed and unreleased: no workflow tags, attests, publishes or deploys it.
Deploying needs a separate approval and Ryan's own Cloudflare sign-in;
[SECURITY.md](SECURITY.md) lists what must hold first.
[`live-check.mjs`](live-check.mjs) checks a running endpoint with the
conformance cases; its tests run it only against the local Worker.

Licensed under the [Ryan Duguid Public Attribution Licence 1.0](../../LICENSE); see [NOTICE](../../NOTICE).
