# v0.1.0

First release of the hosted calculators Worker: a stateless Streamable HTTP MCP
endpoint for 6 Aus Accounting MCP tools, built from this commit to
WebAssembly. The tarball holds the `worker-build` output,
`wrangler.deploy.toml` for `duguid.com.au/mcp*` and `wrangler.test.toml` for
the test Worker on `duguid.com.au/mcp-test*`. Every tool call is refused from
the `serve_until` date in each reply's `x-connector-release` header.

Licensed under `AGPL-3.0-or-later` with section 7(b) attribution terms (see
NOTICE).
