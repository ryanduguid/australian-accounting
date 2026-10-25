# v1.0.0

Unreleased standalone Rust major version. Python imports are removed; former API
functions are JSON subcommands. See [CHANGELOG.md](CHANGELOG.md) for migration,
verification and documented divergences, and [README.md](README.md) to build it.

- Measurable right-to-invoice contracts now use cent-quantised certified billings
  for revenue and the contract balance. GST still uses supplied billings. Rust and
  JSON inputs with fractional cents no longer produce unquantised revenue or a
  rounding-only asset or liability. Other revenue methods are unchanged.
- Amounts outside the decimal context's range for cents now produce an input
  refusal with the field context. CSV errors retain their row and contract and
  are collected with other unreadable cells; JSON refusals retain `where`.

- The `review-pack --as-at` help now describes the existing default: the schedule
  date, or today if unavailable. Date selection and calculations are unchanged.

# v0.2.0

- The package moves from MIT to the Apache-2.0 licence, with a `NOTICE` file whose attribution anyone redistributing the package or a derivative must keep ([#373](https://github.com/ryanduguid/australian-accounting/pull/373)). The package metadata declares Apache-2.0. Releases up to v0.1.4 stay under MIT.
- Requires Python 3.14 or later; Python 3.11 to 3.13 are no longer supported, and 0.1.4 remains the last release that installs on them ([#355](https://github.com/ryanduguid/australian-accounting/pull/355)).
- The build backend moves from hatchling 1.32.4 to setuptools 84.0.0 ([#410](https://github.com/ryanduguid/australian-accounting/pull/410)).
- When a contract's outcome is not reasonably measurable, AASB 15 para 45 now limits revenue to recoverable cost before any measure of progress is applied, so the output and right to invoice methods no longer recognise revenue that para 45 does not allow. Recoverable cost above the transaction price is refused, as recoverable cost above costs incurred already was, and a supplied `output_percent` is still range-checked. Contracts whose outcome is reasonably measurable are unchanged ([#396](https://github.com/ryanduguid/australian-accounting/pull/396)).

# v0.1.4

- Requires Python 3.11 or newer; Python 3.10 is no longer supported ([#351](https://github.com/ryanduguid/australian-accounting/pull/351)).
- The build backend moves from hatchling 1.32.3 to 1.32.4 ([#337](https://github.com/ryanduguid/australian-accounting/pull/337)).

No calculation changed.

# v0.1.3

- The public WIP calculation API applies the ratio contract the CSV reader already enforced: each active ratio must be a finite `Decimal` from 0 to 1. In 0.1.2 a direct caller could pass a constraint ratio of 2 for an unapproved estimate of 100 and receive included consideration of 200 and excluded consideration of -100, and a `NaN` GST ratio could reach the returned amount. Formulas, output diagnostics and CSV and command line behaviour are unchanged ([#318](https://github.com/ryanduguid/australian-accounting/pull/318)).
- Tests now pin two money-path boundaries that one-operator mutants survived: a nil cost to complete recognises the full contract price, and nil gross profit carries no onerous or negative-margin flag ([#307](https://github.com/ryanduguid/australian-accounting/pull/307)). No calculation changed.

# v0.1.2

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

Other changes:

- `review-pack` rebuilds the schedule for the `as_at` date the schedule
  records when `--as-at` is omitted, and uses today only when the schedule
  records none. A schedule made without `--as-at` could not previously be
  reviewed on a later day. An explicit `--as-at` still wins, and an empty
  `--as-at ""` is refused.
- `measure` refuses an `output_percent` outside 0 to 1, or one that is not
  finite, with `ScheduleError`. CSV input already refused these; a direct
  caller could reach revenue above the transaction price or below zero.
- A contract file with more than 20 unreadable rows now ends its error list
  with `... and more`. Collection previously stopped at exactly 20, so the
  suffix never appeared.
- `examples/JOB-TO-CASH.md` and `examples/job_to_cash.py` trace WIP evidence
  into dated project cash assumptions.
- `README.md` and `DISCLAIMER.md` state that Ryan Duguid is not a registered
  tax agent or BAS agent, and limit project support to software issues
  reproduced with fabricated data.
- The build backend pin moves from hatchling 1.32.0 to 1.32.3.

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
