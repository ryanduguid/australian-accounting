# Flat Glama badges on Cloudflare Workers

This Worker renders public Glama score badges in the flat Mermaid palette for
repository READMEs. It uses the renderer and public allowlist in
[glama-badge-server](../glama-badge-server/), without the native HTTP runtime.

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
worker-build --release
npm test
npm run dry-run
```

The Node.js harness uses Cloudflare's Miniflare runtime to exercise the compiled
Worker with captured public SVGs and intercepted upstream requests. It makes no
live provider requests. The production component is Rust. Miniflare is pinned
to stable 4.20260730.0; Wrangler 4.151.3 also bundles a newer beta runtime for its
own commands. Both local integration tests and Wrangler packaging must pass.
The stable Miniflare test dependency overrides Sharp to 0.35.5 and Undici to
7.29.1 for their published security fixes, without changing the direct pins.

Do not enable Cargo release stripping: it removes target feature metadata
required by the current wasm-bindgen packaging step. worker-build runs wasm-opt.

## HTTP contract

GET and HEAD accept only `/glama/<allowed-identity>.svg`, without queries.
The 2,947 identities are compiled from the native adapter's public allowlist.
Click-through links remain the original Glama listings.

Every miss fetches the fixed Glama HTTPS score route with fresh headers, no
caller credentials and no redirects. Fetching and reading share an 8-second
deadline. Source bodies are streamed with a 32 KiB bound; transformed and cached
bodies have a 64 KiB bound. Unsupported SVG returns 502 with `no-store`.

Successful SVGs use the Worker Cache API for at most 5 minutes. The original
fetch timestamp bounds every cache hit; client freshness is capped at 60 seconds
and decreases near expiry. Expired positives cannot mask a failed refresh.
There are at most 32 simultaneous source requests per isolate. Extra misses
return 503 without joining a queue. Separate isolates have separate limits.

## Deployment status

This is an undeployed candidate. `npm run dry-run` creates a local upload bundle;
it does not publish it. No README uses a Worker hostname yet.

Before deployment, confirm Ryan's chosen Cloudflare account, public Workers
hostname, provider usage conditions, account quota and an account-wide request
limit. Local tests cover a 256-badge cold burst at concurrency 32; they do not
prove production CPU limits or capacity across all 2,947 identities. A public
canary must verify HTTPS, freshness, failure responses and resource usage before
any bulk README migration. Deployment needs Ryan's approval.

Licensed under Apache-2.0.

Verify Cache API writes and hits on the exact approved public hostname.
Cloudflare documents functional caching on custom domains and no effect in
Dashboard or Playground previews. Local Miniflare cache tests do not establish
the behaviour of a public workers.dev address.
