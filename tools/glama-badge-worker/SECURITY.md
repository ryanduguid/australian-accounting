# Security boundary

This Worker serves only public Glama badge images. Its fixed upstream, exact
allowlist and shared strict SVG parser are documented in
[the renderer security policy](../glama-badge-server/SECURITY.md).

The Worker has no accounting capability, secret binding or client-data input.
Caller headers are not copied upstream. Query strings and unknown routes are
rejected. Redirects, unsupported content, oversized streams and deadlines fail
closed. Error responses are never cached as successful SVGs.

Cache keys and source timestamps are created internally. Source freshness is
bounded to 5 minutes, including downstream freshness. The per-isolate fetch
ceiling bounds local work; it is not an account-wide rate limit. Public exposure
requires an authorised account and request admission policy, quota checks and a
canary. Keep preview URLs and observability disabled unless separately approved.

Report suspected security issues through the repository's private vulnerability
reporting process.

The Cache API namespace is trusted internal storage. Only this Worker may write
its versioned cache keys; a cache write uses validated, transformed SVG. A public
canary must verify hostname and namespace isolation as well as actual reuse.
The freshness window starts when the Worker successfully acquires and transforms
the response; it does not attest when Glama generated its underlying scores.
