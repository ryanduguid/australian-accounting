# Core client catalogue preview

`lodgeitadapter.core` reads one page of non-archived client identities from the
public LodgeiT Core preview contract captured on 9 October 2026. It is a separate
opt-in library interface in the existing network application. Calculator config,
CLI, MCP, trials and evidence writers do not call it.

```python
from lodgeitadapter.core import CoreClient, CoreConfig

client = CoreClient(CoreConfig(enabled=True))
page = client.list_clients(bearer_token=host_token, page_size=100)
# Inspect page.items in the authorised local consumer.
# Pass page.continuation to a later explicit call to read the next page.
```

The host obtains and manages `host_token`. This module reads no environment
variable, file or secret store, acquires no OAuth token and refreshes nothing.
Use a service application authorised by the workspace owner, with the least
privilege scope `read:core.clients`. Token scope and audience are enforced by the
provider; the adapter cannot attest them. Personal unlock scopes are unnecessary.

The sole destination is
`https://api.lodgeit.com/core/clients/v1-preview/list`, using POST. It admits no
base URL override, loopback route, alternate host, prefix, query or redirect.
Every call fixes `archived` to false and `expansion` to null. It makes one HTTP
attempt, never retries and never follows a continuation automatically. Page size
must be an integer from 1 to 500. Continuations remain JSON data sent to the same
route. There is no mutation, profile, group, relation or assignment operation.

Tokens are supplied per call and used only for the Authorization header. The
shared transport rejects header injection, invalid bearer syntax, more than
8,192 characters and conflicting Authorization headers. It does not retain a
token on config or client state. HTTPS uses the system trust store; cookies,
environment proxies and redirects are disabled. Socket timeout bounds each
blocking operation, not the total elapsed time of a slow stream.

Success requires HTTP 200 and JSON with the pinned page structure. The complete
page is checked before any result is returned. UUIDs must be canonical and unique;
booleans must be exact; profiles must be null and expanded data absent or null.
Unknown fields, duplicate JSON keys, non-standard numbers and malformed values
cause a sanitised `CoreError`. Error bodies and redirect locations are discarded.
Categories separate authentication, authorisation, rejected requests, throttling,
upstream failure, transport failure, disabled access and protocol mismatch.
Custom redirect refusals and malformed redirect locations produce transport
errors without an HTTP status. A redirect scheme refused by the standard handler
as an HTTP error produces a protocol mismatch with its status, as does an
unexpected non-redirect status. None exposes a redirect destination.
Core clears its bearer, continuation, request body and
raw response bindings on every exit, including early input refusals. Caller
frames and external telemetry still require the host's own sensitive-data policy.

The returned immutable objects contain client UUID, optional custom code,
archived/private flags and an opaque continuation. These values can identify
people or organisations and must be handled as sensitive. Their ordinary
representations are redacted; callers can deliberately inspect or serialise the
fields. The package logs and persists none of them and exposes no Core evidence
writer. An unexpected profile may transiently arrive in memory, but it is refused
and never returned. This is not an in-process sandbox or a memory-erasure guarantee.

Adapter safety limits are a 1 MiB response, 1,024 UTF-8 bytes for a code, 8,192
UTF-8 bytes for a continuation and at most a 60-second socket timeout. Defaults
use 20 seconds. These are local policies, not vendor guarantees. A fabricated
500-item maximum-code page passes; larger or differently encoded responses may
be refused. No live tenant, token issuance, audience, scope or preview conformance
has been tested. The captured reference is automated evidence for human review,
not a claim of provider stability or professional approval. Contract drift fails
closed; no command downloads or accepts a new contract.
