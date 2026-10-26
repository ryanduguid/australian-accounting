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

Licensed under the Functional Source License, Version 1.1, ALv2 Future License (FSL-1.1-ALv2). You may use, modify and redistribute it for any purpose except a Competing Use as defined in LICENSE (making it available to others in a competing commercial product or service), and each version becomes available under Apache-2.0 two years after it is made available. See [LICENSE](../../LICENSE) and [NOTICE](../../NOTICE).

Verify Cache API writes and hits on the exact approved public hostname.
Cloudflare documents functional caching on custom domains and no effect in
Dashboard or Playground previews. Local Miniflare cache tests do not establish
the behaviour of a public workers.dev address.

## Synthetic hosting trial

The separate `synthetic-trial` build excludes the live provider handler. It runs
the same Rust renderer on a captured test fixture, with a centred gold
`SYNTHETIC TEST DATA` label. It makes no outbound requests and serves no live
ratings. Its routes are `/synthetic.svg` and `/failure.svg` (a deliberate 503).
Both accept GET and HEAD without queries. Other routes return 404 whilst active.

```sh
cargo clippy --locked --all-targets --features synthetic-trial -- -D warnings
cargo clippy --locked --target wasm32-unknown-unknown --lib --features synthetic-trial -- -D warnings
cargo test --locked --features synthetic-trial
worker-build --release --out-dir build-synthetic -- --features synthetic-trial
npm run test:synthetic
npm run dry-run:synthetic
```

`synthetic.wrangler.toml` uses separate build output and a separate service name,
with a 10 ms CPU limit, preview URLs disabled and observability disabled.
`SYNTHETIC_START_MS` and `SYNTHETIC_END_MS` are public epoch millisecond values.
The default values disable the trial. A missing, invalid, reversed, future,
expired or longer than 1-hour window returns 410 with `no-store` before accessing
the cache. Successful responses cannot advertise freshness beyond that window.
The trial reports `x-synthetic-cache: HIT` or `MISS`; a hit measures this Worker
Cache API namespace, rather than any outer CDN cache. The default build retains
its existing live badge routes and HTTP contract.

Deployment is a separate authorised action. Use a new trial service name, verify
the existing account and hostname and set a finite window. Never overwrite the
live service or register a new account, plan or hostname to make the trial work.
Keep the controlled test to at most 64 requests, collect HTTPS, cache, expiry and
failure evidence and retire the trial afterwards. Expiry disables successful
responses; it does not prevent requests from consuming account quota. Confirm
account quota and request admission before public exposure. A local pass does
not prove production CPU usage, global admission or catalogue capacity. No README
should point at this trial.
