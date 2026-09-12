# v0.1.1

- Reject malformed numeric grouping and conflicting signs.
- Parse and hash the same captured source bytes for each review pack.
- Preserve separate contract assets and liabilities in schedule checks.

## Previous release

### v0.1.0

This is the first PyPI release of `the-wip-tally`.
The monorepo's [filtered GitHub Releases page](https://github.com/ryanduguid/australian-accounting/releases?q=the-wip-tally)
is the canonical release history.

- Move the maintained source to `packages/the-wip-tally` in the
  `australian-accounting` monorepo.
- Publish through `release-the-wip-tally.yml`, using the monorepo's namespaced,
  attested release workflow.
- Preserve the imported v0.1.0 AASB 15 schedule behaviour and CLI contract.
