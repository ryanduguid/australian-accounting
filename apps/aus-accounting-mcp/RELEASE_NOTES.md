# v0.1.6

The repository's [GitHub Releases](https://github.com/ryanduguid/aus-accounting-mcp/releases) page is the canonical release history. A separate changelog is intentionally not maintained.

- add the exactly pinned `div7a-loan-review` 0.1.0 engine and registered tools
  for reviewed s 109N benchmark rates and a fail-closed s 109N/s 109E loan review;
- default those tools to compact results, with `response_detail="full"` retaining
  the complete engine audit payload;
- retain explicit refusal for Division 7A matters outside that reviewed scope;
- update `aus-accounting-mcp-demo` to make real registered MCP calls for a
  synthetic BAS fixture and a fabricated Division 7A loan-review outcome;
- check the text transcript and static WebP proof, with the transcript as the
  accessible source of truth;
- record the repository, distribution, executable and MCP Registry identity
  mapping plus the compatibility boundary; and
- retain the stdio server, synthetic-fixture, no-advice and human-review boundaries.

Publication uses the approval-gated GitHub release, PyPI and MCP Registry workflows.

Not advice. Outputs are preparation aids for a qualified professional, not compliance determinations.
