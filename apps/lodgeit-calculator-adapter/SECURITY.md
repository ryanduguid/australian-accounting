# Security

This component makes outbound HTTPS requests to a third-party service when it
is explicitly enabled. Everything below is about that.

## What it does and does not hold

No credentials, no tokens, no API keys, no account. The provider's calculator
surface is open access. If a provider later requires a key, adding one is a
change to review here, not a configuration tweak: this package has no code for
reading a secret and none should be added without deciding where the secret
lives and who can read it.

## Outbound request controls

- Off by default. Three separate conditions must hold before a request is sent:
  explicit enablement, a configured base URL, and that host on the allowlist.
- Only HTTPS off loopback. Plaintext HTTP is refused except on loopback, which
  needs its own switch and exists for a local test stub.
- Only the routes in `ALLOWED_PATH_PREFIXES`. A URL taken from a response body
  is data, not a destination, and cannot become a request.
- Redirects are never followed. A 3xx is a refusal, because the new destination
  was never checked against the allowlist.
- Credentials in a URL are refused.
- Bounded timeouts, a response byte ceiling read chunk by chunk, and retries
  only for a 5xx or a transport failure, never for a 4xx.

## Data handling

Send fabricated figures. This component is built for synthetic trials, and an
evidence file records whether its input was synthetic.

Nothing is logged by the package itself: no request body, no response body, no
amount. The CLI prints what you asked it for, to your terminal.

Do not send client payroll, client loans or any identifier through this
component. A third-party service is outside any engagement, privacy or
retention arrangement this repository knows about.

## Reporting

Open an issue on the repository. For anything sensitive, follow the
[root SECURITY.md](../../SECURITY.md).
