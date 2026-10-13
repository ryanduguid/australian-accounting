# LodgeiT Core Clients API preview

The Core Clients API is separate from the calculator constellation. This application implements an [opt-in client catalogue](core-security.md) through one fixed read-only listing route; broader profile, assignment and workspace operations remain outside it. This note records public documentation inspected on 9 October 2026. It does not establish live compatibility or client synchronisation.

## Public surfaces

| Surface | Purpose | Evidence |
| --- | --- | --- |
| Calculator constellation | Deterministic calculator discovery and calculations | [LodgeiT Labs](https://lodgeit.org/calculators.html) and this component's [reviewed contracts](../contracts/README.md) |
| Core Clients API | Client records, assignments, groups, relations and workspace members and teams | [API documentation](https://api.lodgeit.com/) and [OpenAPI document](https://api.lodgeit.com/core.lodgeit.openapi.json) |
| LodgeiT application | Account-controlled tax and reporting workflows | [Public support documentation](https://help.lodgeit.net.au/support/home) |

The Core schema inspected was OpenAPI 3.1.1 with 34 documented operations. All use POST, including reads. Neither HTTP method nor the shared vendor name establishes whether a call changes data. The documented Core operations do not include tax return lodgement.

The [preview announcement, modified on 8 October 2026](https://help.lodgeit.net.au/support/solutions/articles/60001648439-the-lodgeit-clients-api-is-now-in-public-preview), describes live client data and warns that endpoints and fields may change. The older [API overview](https://help.lodgeit.net.au/support/solutions/articles/60001644001-lodgeit-s-api) still describes client access as planned. Use the preview announcement and current schema for this surface, while preserving their preview status.

## What the schema establishes

The Core API documents OAuth 2.0 client credentials, issued through an owner-created service application at LodgeiT SSO. Read and manage scopes are distinct; personal fields require separate unlock scopes. Client listing returns items and an optional continuation token. The documented invalid-token problem says the listing filter must stay consistent between pages. Problems have a stable type and a trace identifier.

Operation access also depends on caller identity. The `list-assigned` operation requires a user and rejects machine callers, so a service application cannot use it through client credentials. Check each operation's user, role and permission requirements separately from its scopes.

`modify-profile` changes only supplied fields. `replace-profile` replaces the profile as a whole: omitted contact details are removed, the first email address becomes primary, and personal fields without an unlock grant retain their existing values. A replacement assembled from an incomplete read can therefore remove contact details. A future comparison must distinguish unchanged, withheld and intentionally removed fields before proposing replacement.

These are observations about the captured schema. No credentials were requested, no token was obtained and no authenticated operation was tested.

## Requirements for a future integration

The [security policy](../SECURITY.md) separates synthetic calculator trials from the [Core catalogue boundary](core-security.md). The catalogue uses a host-managed bearer token supplied per call, acquires no OAuth token, reads no secret store, returns no expanded profile and performs no mutation. Token acquisition, additional sensitive fields or broader operations would require a separate design and security review; extending the calculator allowlist cannot enable them.

For an extension beyond the current catalogue, first specify its exact operations, approved data location and account authority. Start with a read-only comparison in a separate test organisation. Preserve client identifiers as strings, keep sensitive fields out of logs, bind pagination to its filters and report partial results explicitly. Define duplicate matching and the human review of a proposed change before any write operation is exposed.

Acceptance must cover missing scopes, user-only operations called by a machine, withheld fields, omitted contact details in a replacement, pagination changes, duplicate matches, schema drift, failed transport and sanitised error reporting. A future implementation needs the owning repository's documented checks and consultation before choosing an access-boundary change. It must not introduce network access into the offline engines or map a provider response into accounting approval.
