# v0.1.9

- Publish the benchmark evidence corrections and MCP improvements prepared for
  0.1.8, using a release-policy commit GitHub Actions can resolve.
- The 0.1.8 tag failed before any release job ran. No 0.1.8 package was published;
  the original tag is preserved.

# v0.1.8

- Use the benchmark engine's evidenced serialiser so omitted expense buckets
  cannot become definite zeroes in notes, checks or the selected key ratio.
- Expose structured MCP output schemas and bundled reference resources, add
  industry pagination, and preserve unknown Division 7A results.
- Keep the published engine pins unchanged: `ato-benchmark-compare` 0.1.6,
  `payday-super-checker` 0.1.3 and `div7a-loan-review` 0.1.1.

Outputs remain preparation aids for human review, not advice or compliance
certification. This release makes no change to statutory calculations.

# v0.1.7

- update the exactly pinned delegated engines to `ato-benchmark-compare` 0.1.6,
  `payday-super-checker` 0.1.3 and `div7a-loan-review` 0.1.1 after independent
  public-install verification;
- point the compatibility record and public documentation at the maintained
  monorepo source and namespaced component releases;
- preserve the existing MCP tool surface, fail-closed boundaries, synthetic
  demonstration and compact Division 7A responses; and
- publish the attested GitHub release through `release-aus-accounting-mcp.yml`,
  then its exact distributions and registry record through the approval-gated
  `publish-pypi.yml` and `publish-mcp.yml` workflows.

All six leaf packages were installed and checked from PyPI before this release.
Only the three engines used by registered MCP tools are runtime dependencies.

Not advice. Outputs are preparation aids for a qualified professional, not compliance determinations.

# v0.1.6

The repository's [GitHub Releases](https://github.com/ryanduguid/australian-accounting/releases) page is the canonical release history. A separate changelog is intentionally not maintained.

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
