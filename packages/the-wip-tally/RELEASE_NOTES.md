# Unreleased

Breaking: `outcome_reasonably_measurable` is required on every contract row.

- A blank cell, or a missing column, used to read as `yes` and book full
  percentage-of-completion revenue. It is now the not-stated state, and the
  contract is refused by row number, the way a blank `retention_classification`
  falls to `review` rather than to `receivable`. AASB 15 paras 44-45 decide
  between percentage-of-completion revenue and revenue limited to recoverable
  cost, so the engine states neither until the row answers the question.
- Migration: add `outcome_reasonably_measurable` to the contract CSV with `yes`
  or `no` on every row, or map an existing column to it with `--mapping-file`.
  Explicit `yes` and `no` values, and every figure they produce, are unchanged.
- `ContractInput.outcome_reasonably_measurable` is now `bool | None` for
  callers using the engine directly; `None` raises `ScheduleError` from
  `measure` whichever `progress_method` the row uses.

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
