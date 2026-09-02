# v0.1.3

This is the first PyPI release of `the-exchequer-tally`.

- Move the maintained source to `packages/the-exchequer-tally` in the
  `australian-accounting` monorepo.
- Publish through `release-the-exchequer-tally.yml`, using the monorepo's
  namespaced, attested release workflow.
- Preserve the v0.1.2 corporate-tax, franking-ledger and Division 203 behavior.

# v0.1.2

The repository's [GitHub Releases](https://github.com/ryanduguid/TheExchequerTally/releases) page is the canonical release history. A separate changelog is intentionally not maintained.

Prepared 0.1.2 notes. No PyPI distribution has been published; this source-only
project currently installs from a clone. The statutory corrections below were
found by an adversarially verified audit and checked against the legislation
text on legislation.gov.au:

- The maximum franking rate now applies the CURRENT year's rate scale to the prior year's amounts, per the s 995-1 ITAA 1997 definition of corporate tax rate for imputation purposes. The old code applied the prior year's scale, over-stating the rate by 1.0 to 1.5 percentage points across the 2020-21 and 2021-22 transitions.
- The FY2018 Base Rate Entity turnover threshold is $25 million (Enterprise Tax Plan Act 2017 Sch 1 Pt 2); $50 million applies from FY2019.
- The s 205-70 FDT offset reduction no longer switches off when zero franking credits arose in the year: with zero credits, any deficit exceeds 10 percent of them.
- The franking ledger's item 3 debit is the UNDER-franking debit (s 203-50(2) shortfall); over-franking tax is a tax liability, not a franking debit. New helpers record the under-franking debit and the s 205-15 FDT-liability credit so multi-year balances reconcile.
- The BREPI 80 percent test compares exactly instead of on a 2dp-rounded ratio; the franking percentage caps at 100; the benchmark comparison works in dollars at each event's own rate; the stated franking credit rounds down so it never exceeds the s 202-60 maximum; negative and out-of-range inputs are refused.

Also: one version source (pyproject) with `__version__` read from package
metadata, project URLs and classifiers, mypy enforced in CI (was advisory) with
Python 3.13 added, CodeQL and deduplicated Dependabot config. The source project
did not yet have the named publisher and compatibility contract required for a
release workflow; v0.1.3 supplies them from the monorepo.

Not advice. Outputs are review aids for a qualified professional, not determinations.
