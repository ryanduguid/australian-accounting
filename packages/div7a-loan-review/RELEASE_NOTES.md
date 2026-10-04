# v0.2.0

- Require Python 3.11 or later. CPython 3.10 reached end of life on 1 October 2026; 0.1.6 remains the last release that installs on Python 3.10 ([#351](https://github.com/ryanduguid/australian-accounting/pull/351)).
- Read the loan register as UTF-8. A register in another encoding, or with a field longer than the CSV module's field size limit, ends `gate`, `myr` and `review` with one `error:` line and exit 1 instead of a traceback ([#344](https://github.com/ryanduguid/australian-accounting/pull/344)).
- Finish a review whose remaining term is astronomically large, such as `1e999999999` years. The s 109E(6) calculation raises the discount factor to the already whole `Decimal` term instead of first converting it to an integer, so the minimum yearly repayment reaches the interest-only limit instead of the command hanging ([#344](https://github.com/ryanduguid/australian-accounting/pull/344)).

# v0.1.6

- Refuse malformed or conflicting review modes before reading input: both Python entry points take only literal booleans for the mode flags and refuse `gate_only=True` with `myr_only=True`. In 0.1.5 that pair produced a reviewed row with neither result and no attention flag, even when the normal review reported a shortfall, and a textual `"false"` silently selected a restricted mode. The three valid modes and the command line commands behave as before ([#329](https://github.com/ryanduguid/australian-accounting/pull/329)).
- The README compares the engine with the Division 7A worksheet in Xero Workpapers Plus ([#325](https://github.com/ryanduguid/australian-accounting/pull/325)).

# v0.1.5

- Refuse a fractional bare remaining term in the minimum yearly repayment: `remaining_term` must be a whole number of years, and a part year goes through `statutory_remaining_term` first.
- Accept an amount written with trailing zeros past the cent, such as `25000.000` from a ledger or payroll export, while still refusing a real sub-cent amount.
- Refuse a rate override that gives a year or rate as a JSON number with a message asking for a quoted string, instead of ending in a traceback.
- Record each benchmark rate's RBA workbook cell, the value read there and the date the row was compared, in the new `workbook_cell`, `workbook_value` and `row_verified_on` columns. No rate value changed, but the table's SHA-256 in every result `manifest` did.
- Cite the 1 July 2026 compilation of the *Income Tax Assessment Act 1936* instead of the moving latest version, in the README, the evaluation and the workbook.
- Workbook: the benchmark-year selector reads the Rates table through the `RateYears` name, so a year added to the table can be chosen.
- Ship the `NOTICE` file in the distribution, and state in the README and `DISCLAIMER.md` that the author is not a registered tax or BAS agent.

# v0.1.4

- Name the primary source behind every benchmark rate: each rate-table manifest now carries `primary_url`, the RBA F5 historical workbook it was read from, `retrieved_on`, the date that workbook was downloaded, and `snapshot_sha256`, the digest of those bytes. `verify_at` stays a convenience link for a human rather than the source of the figure, and an operator's override carries no such claim. A table whose rows do not all make the same claim names none, instead of attributing every row to the last one read.
- Record the read that produced those columns in `docs/primary-source-review-2026-09-20.md`, which checked all 8 reviewed years against the RBA workbook. No rate value changed.

# v0.1.3

- Name the rate table behind every figure: each JSON result carries a `manifest` whose `rate_table_uris` list the frozen benchmark table with its SHA-256, produced by the lookup that read it, and any reviewed override as `override:<file name>`. An `UNKNOWN` rate still names the table it was looked for in.
- Carry `reason_codes` beside `reasons` on every `REFUSED` and `UNKNOWN` result, one stable token per reason in the same order, so a caller can branch on `REFUSED_*` (outside s 109E) separately from `*_UNKNOWN` (a fact the operator can still establish). A result built with a reason and no code fails at construction.
- Resolve the rate table once per register review instead of once per row.
- Workbook: refused repayment rows drive `REVIEW`; duplicate benchmark-year labels, and blank, text, negative or above-one rates, block the workbook; dates are read as whole calendar days, money is compared in cents, and placeholder dates and out-of-range amounts are refused; the unresolved interest-floor interpretation is stated beside the result.

# v0.1.2

- Reject padded required headers, duplicate override years and gates from another loan year.
- Withhold minimum-yearly-repayment figures when the loan year is missing and retain gate caveats in every result.
- Align the workbook formulas and cached values with the reviewed year checks.

## Correction to the v0.1.1 note

The v0.1.1 release note said that release preserved the v0.1.0 review
calculations and refusal boundaries. The calculations and the s 109N gate were
preserved, but the refusal boundaries were not: v0.1.1 added the two-decimal
place limit and the $1 trillion maximum in `div7aloan/money.py`, and it began
preserving quoted newlines when reading the register so a malformed numeric
cell is refused instead of being joined into a different amount. Version 0.1.0
accepts a principal of `100000.001`, a repayment of `25556.001`, a principal of
`1000000000000.01` and a repayment split across a quoted newline; v0.1.1
refuses each of them. Those safeguards stay in force in this release.
