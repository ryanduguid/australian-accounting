# v0.1.3

First published release. Changes since the repository's first commit:

- MCP facade over the reviewed engines payday-super-checker and ato-benchmark-compare, with Division 7A refused by design (`refuse_div7a`) and synthetic SBR fixtures only.
- Repository renamed from its personal codename to `au-tax-mcp-server`; README, client configs and engine dependency pins all point at live, reachable history.
- `labour_to_turnover` no longer presents a partial labour picture as a definite ratio: the ratio requires salary and wages (or W1, which substitutes under the ATO rule), contractor and commission, and cost-of-sales labour to all be supplied.
- The MCP handshake reports the package version.
- CodeQL, CI job timeouts, and the shared release-policy workflow pinned to reachable history.

Tags v0.1.0 to v0.1.2 exist but produced no release: each predates one requirement of the shared release gates (a live workflow pin, RELEASE_NOTES.md, the build tool in the locked dev extras) and the repository ruleset does not allow deleting a pushed tag. This tag supersedes them.

Not advice. Outputs are preparation aids for a qualified professional, not compliance determinations.
