# Security boundary

This component serves public badge images. It accepts only exact paths from its
compiled public Glama allowlist, with no query string. It constructs requests
against `https://glama.ai/mcp/`; callers cannot choose an upstream hostname,
scheme, port, credentials, headers or arbitrary URL. Certificate validation is
enabled, redirects are disabled and environment proxy settings are ignored.

The upstream response must be a successful SVG response within the byte and
time limits documented in the README. The XML transformer rejects DTDs,
processing instructions, foreign namespaces, unknown elements or attributes,
external references and active content. Embedded SVG logos must match its
bounded path-only grammar. Errors never become a passing badge.

The native listener is an application backend. Its loopback default is for
local verification. A deployment must put it behind an explicitly authorised
HTTPS ingress with connection and request-rate limits and a header-read
deadline. There is no admin endpoint, client authentication or accounting
capability. Do not add confidential identities to the allowlist.

Use the repository's private vulnerability reporting process for suspected
security issues.

The offline MCPRush example reads at most 32 KiB of source SVG from standard
input and writes a validated restyled card to standard output. It fetches
nothing and exposes no HTTP route. Its fixed-identity grammar rejects active
content, foreign namespaces, external references and unsupported layouts.
