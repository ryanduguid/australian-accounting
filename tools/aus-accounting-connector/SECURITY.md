# Security boundary

This Worker answers tool calls with public tax tables and the inputs in each
request. It has no secret binding, no storage binding (KV, R2, D1, Durable
Objects, Cache API or Queues), no outbound `fetch` and no logging: observability
is off in `wrangler.toml` and the code writes nothing to the console. A request
body is held in memory only for the request that carries it. The harness checks
that the Worker makes no outbound request and that workerd's output never holds
a request body.

There is no sign-in and no session. Anyone who can reach the route can call the
6 tools, so the admission limits are the boundary:

- `cpu_ms = 10`, the Workers Free limit.
- 60 requests a minute per caller address in each Cloudflare location, through
  the Workers rate-limit binding `LIMITER` in its own namespace. Its counters
  are approximate and per location. If the binding is missing or fails, the
  Worker refuses with 503.
- Request bodies over 16 KiB are refused before parsing, whether or not
  `Content-Length` is sent.
- Browser requests are accepted only from the listed origins. A request without
  `Origin` is accepted, as MCP clients outside a browser send none.

The account's daily request allowance is shared by every Worker in the account.
Before any deployment, confirm the account, set the route to fail closed when
that allowance is spent, keep preview URLs and `workers.dev` off, and run
[`live-check.mjs`](live-check.mjs) against the deployed endpoint.

Each release stops answering tool calls on its `serve_until` date, so a release
built with one year's tables cannot keep serving after they may have changed.
An engine refuses periods it holds no table for.

Report suspected security issues through the repository's private vulnerability
reporting process.
