# Flat Glama badges

A separate HTTP adapter for public Glama score badges, using the Mermaid badge
palette. It does not link to or run the accounting engines or MCP application.
The MCP application keeps its local stdio transport.
The [Cloudflare Worker candidate](../glama-badge-worker/) reuses this library with
its native HTTP dependencies disabled.

This component is a local deployment candidate. No public host is configured and
the repository's badge references still use Glama directly.

## Run and check

Use the repository's Rust 1.99 toolchain. From this directory:

```sh
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
cargo run --locked -- --listen 127.0.0.1:8080
```

For example, request
`http://127.0.0.1:8080/glama/servers/ryanduguid/australian-accounting.svg`.
The click-through link remains the original Glama listing.

The default listener is `127.0.0.1:8080`. A public deployment needs an explicitly
selected HTTPS ingress with connection, header-read and request-rate limits.
Do not expose the native listener directly to the internet.

## Source and appearance

`allowed.txt` contains 2,947 distinct public Glama identities observed in the
badge migration inventory. Only those server and connector score routes are
accepted. New identities require a reviewed allowlist change. Legacy Glama
routes and queries are not included.

The transformer retains Glama's glyph paths, order, positions, title and
description. Black, ivory, platinum and gold define the flat frame and wordmark.
Positive provider states use Rhodesian green, intermediate states gold, failed
states red and missing states platinum. It never infers a passing grade from a
description or replaces Glama's scoring logic. Provider format drift fails.

Only public Glama HTTPS requests leave the adapter. It has no credential or
client-data input. Redirects and environment proxies are disabled. Requests
have a 3-second connection deadline and an 8-second total deadline, and the
response body is limited to 32 KiB. Four upstream requests may run at once;
extra cache misses receive HTTP 503 without a waiting queue.

Successful transformed badges are cached for at most 5 minutes in a cache of
at most 256 entries. Downstream cache time is capped at the remaining freshness
window. Expired badges are removed; upstream errors return HTTP 502 with
`no-store`. Unsupported SVG, oversized responses and redirects are errors.
No last-known positive result is served after expiry or a failed refresh.

## Verification limits

The tests cover captured SVG preservation and rejection, route restrictions,
upstream status and media-type failures, redirects, response size, cache reuse,
expiry and concurrent-fetch capacity. They use loopback fixtures. Deployment,
public HTTPS ingress, provider quotas and live end-to-end availability require
separate validation before any README is migrated.
