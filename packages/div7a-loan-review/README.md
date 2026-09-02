# div7a-loan-review

```
+----------------------------------------------------------------------+
|                          div7a-loan-review                           |
+----------------------------------------------------------------------+
|          ITAA 1936 Div 7A: s 109N terms, s 109E repayments           |
+----------------------------------+-----------------------------------+
| DR  what it gives you            | CR  what it needs                 |
+----------------------------------+-----------------------------------+
| s 109N(2) benchmark rate         | loan register CSV                 |
| s 109N(1) verdict, limb by limb  | the year of income to review      |
| s 109E minimum yearly repayment  | operator-asserted statutory facts |
| shortfall and exposure flag      | a reviewed rate for that year     |
| statutory trace to re-perform    | a human to decide the rest        |
+----------------------------------+-----------------------------------+
```

[![License: MIT](https://img.shields.io/badge/License-MIT-4F485E.svg?labelColor=04001F)](LICENSE)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-5C2D91.svg?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/downloads/)

**Experimental review aid. Not a Division 7A determination.**

v1 reviews the s 109N terms of a private-company loan and the s 109E minimum
yearly repayment for an **operator-supplied amalgamated loan**. It does not
form amalgamated loans, does not classify payments, does not do unpaid present
entitlements, and does not apply s 109RB.

**A rising benchmark rate raises the minimum yearly repayment on existing
complying loans, not just on new ones.** s 109E(6) uses the *current* year's
benchmark rate, not the rate written into the agreement. A seven-year loan
signed in 2021-22 at 4.52 per cent is measured for 2026-27 against 8.77 per
cent, and the repayment that was right when it was written is no longer
enough. Nothing in the process tells you. A loan that has met its repayments
for four years can fall short in the fifth on unchanged terms.

No install, just the rates and the reasoning:
<https://duguid.com.au/rates/div7a-benchmark-rate/>

---

## What it does

Three questions, one job.

1. **`rate`** - the s 109N(2) benchmark interest rate for a year of income,
   with provenance: RBA table, series, month, and the date this repository
   last reviewed the figure.
2. **`gate`** - whether a loan meets every limb of s 109N(1), on facts the
   operator asserts. Written agreement, terms in place before the lodgment
   day, interest rate at or above the benchmark, term within the s 109N(3)
   maximum.
3. **`myr`** - the s 109E(5) minimum yearly repayment for a later year of
   income, the payments applied against it, and any shortfall.

`review` runs the gate and the repayment together over a register.

## What it refuses

These are features. The engine returns `UNKNOWN` or `REFUSED` with a reason
rather than a number it cannot stand behind.

| Refused | Why |
| --- | --- |
| Forming the amalgamated loan from constituent loans | s 109E(3). v1 takes the unpaid balance at the end of the previous year from the operator. A list of component loans gets `UNKNOWN`. |
| A minimum yearly repayment for the year the loan was made | s 109E(1)(a) reaches a loan made "in an earlier year of income"; s 109P puts an amalgamated loan outside s 109D in the year it is made. |
| A repayment figure for a loan that is not on s 109N terms | A constituent loan is one that would be a s 109D dividend apart from s 109N (s 109E(3)(b)), so a loan that fails s 109N is not one. Absent another Subdivision D exclusion, s 109D(1) treats it as a dividend in the year it was made (s 109D(1AA)). A schedule does not save it. |
| Whether a payment is a genuine repayment | s 109R turns on what a reasonable person would conclude about intention. The operator asserts the amount applied. Bank credits are not a substitute. |
| Computing the lodgment day | s 109D(6) is the earlier of the return's due date and the date of lodgment. That needs a lodgment-program date this engine does not hold. |
| s 109C payments | This engine is loans, not payments. |
| s 109F forgiven debts | Out of scope. |
| s 109T, s 109W, s 109C(3A) interposed entities | Out of scope. |
| s 109XA unpaid present entitlements, sub-trusts, PCG 2017/13, TD 2022/11 | Out of scope. |
| s 109RB Commissioner's discretion, s 109Q hardship, s 109RD extensions | Out of scope, and each can change the answer. |
| s 109Y distributable surplus | Not modelled. The shortfall is not the dividend; see below. |
| Loans made before 4 December 1997 | s 109D(5). `SKIPPED`, including the 1997-98 year that straddles the date. |
| Public companies | Division 7A is a private-company provision. |
| "This looks like a dividend, call it s 109C or s 109D for me" | Characterisation is not arithmetic. |
| Interest accrued but not paid, offset accounts, set-off, journal-only repayments | Not payments this engine will count for you. |
| Any rate year not in the frozen table and not in a reviewed override | Fail closed. No extrapolation, no runtime scraping. |

An `UNKNOWN` is never softened into a best-effort guess, and never coerced to
`false`. A limb that has not been established is a different finding from a
limb that has failed, and only one of those is a breach.

## The shortfall is not the dividend

Where the repayment falls short the engine reports an experimental
deemed-dividend exposure equal to the shortfall. Read it as a review aid.

s 109E(2) makes the dividend the shortfall **subject to s 109Y**, which caps
Division 7A dividends at the company's distributable surplus, and s 109E(1)(d)
removes the dividend entirely where s 109Q applies. Neither is modelled here,
and neither is s 109RB. This repository does not write "the ATO will assess
$X", and neither should anything built on it.

## Install

```bash
pip install .
```

Python 3.10 or later. No runtime dependencies.

## Use

```bash
div7a-loan-review rate --year 2026-27
```

```bash
div7a-loan-review gate --input examples/sample_loans_myr_met.csv
```

```bash
div7a-loan-review myr --input examples/sample_loans_myr_met.csv --year 2026-27
```

```bash
div7a-loan-review review --input examples/sample_loans_mixed.csv --year 2026-27
```

Add `--format json` for machine-readable output, `--trace` on `review` to
print the statutory trace for every row, and `--rates-override FILE` to supply
a reviewed benchmark rate for a year outside the frozen table.

### Which year the gate reads its benchmark from

`gate` without `--year` anchors each row to its own `year_loan_made`. That is
the s 109N(1)(b) test: the rate payable for later years is measured against
the benchmark rate for the year the loan was made.

`gate --year 2026-27` measures every row against that year's benchmark
instead. That is a practice check on a risen benchmark, not the s 109N(1)(b)
test, and the result says so. The divergence is written up in
[evaluation/div7a_myr/README.md](evaluation/div7a_myr/README.md).

`review` always anchors the gate to `year_loan_made` and works the repayment
for `--year`.

### Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Nothing exposed, nothing undecided |
| 2 | A shortfall, an undecided row, or a loan not on s 109N terms |
| 1 | The input, the rate table or the override could not be read |

## The register CSV

One amalgamated loan per row. The run is refused if a required column is
absent: reading a missing column as blank would turn every absent fact into
`unknown` and produce a file of `UNKNOWN` verdicts that looks like a
considered review rather than a mis-shaped file.

Booleans are `true`, `false` or `unknown`. A blank cell is `unknown`. Amounts
and rates are decimal strings. Rates are fractions, so 8.77 per cent is
`0.0877`, not `8.77`. Years of income are written `2026-27`.

**Required by `gate`, `myr` and `review`:**

| Column | What it asserts |
| --- | --- |
| `loan_id` | Your reference for the loan |
| `year_loan_made` | The year of income the loan was made in |
| `written_agreement` | s 109N(1)(a): the agreement is in writing |
| `terms_in_place_before_lodgment_day` | s 109N(1) chapeau, with s 109D(6). You assert this; the engine does not compute the lodgment day |
| `maximum_term_years` | The **term of the loan** under the written agreement, in years. It is tested against the maximum term worked out under s 109N(3). The column name follows the published API surface, not the Act's use of "maximum term" for the statutory cap |
| `secured_by_registered_mortgage_over_real_property` | s 109N(3)(a)(i) |
| `security_coverage_at_first_made` | s 109N(3)(a)(ii): the property's market value less liabilities secured over it in priority to the loan, as a ratio to the loan, at the time the loan is first made. `1.10` is the 110 per cent the paragraph requires |
| `interest_rate_for_years_after_year_loan_made` | s 109N(1)(b), as a decimal fraction |

**Also required by `myr` and `review`:**

| Column | What it asserts |
| --- | --- |
| `amalgamated_loan_unpaid_at_end_of_previous_year` | The balance the s 109E(6) formula works on |
| `remaining_term_years` | s 109E(6): the longest constituent-loan term, less the years from the end of the loan year to the end of the year before the one being reviewed. Rounded up by the engine if it is not whole |
| `payments_applied_during_the_year` | The amount **you** are treating as applied. The engine does not apply s 109R. If you cannot assert it, leave it `unknown` |

**Optional:**

| Column | Effect |
| --- | --- |
| `borrower_reference` | Printed with the row |
| `out_of_scope_reason` | Any non-empty value marks the row `SKIPPED`, unreviewed, with your reason |
| `year_of_income_being_tested` | Nominates the benchmark floor year for that row's s 109N(1)(b) limb. `--year` on `gate` overrides it |

Two samples ship with the repository:

- [`examples/sample_loans_myr_met.csv`](examples/sample_loans_myr_met.csv) -
  three complying loans that run to a verdict with no confirmation flags.
- [`examples/sample_loans_mixed.csv`](examples/sample_loans_mixed.csv) -
  `UNKNOWN` rows, a loan whose rate is below the benchmark, a 25-year loan
  with 109 per cent cover, a nil remaining term, and two skipped rows.

Both are fabricated. Every identifier is synthetic.

## Benchmark rates

s 109N(2) sets the benchmark interest rate as the RBA Indicator Lending
Rates - Bank variable housing loans figure **last published before the start
of the year of income**. A year of income starts on 1 July. The RBA publishes
statistical table F5 in arrears, in the first week of the following month, so
the last figure published before 1 July is the **May** figure.

The June figure is published in early July, after the year has already begun,
and is the usual wrong rate. For 2025-26, May 2025 was 8.37 per cent and June
2025 was 8.27 per cent. **8.37 is the benchmark rate.**

| Year of income | Rate | RBA F5 FILRHLBVS figure |
| --- | --- | --- |
| 2026-27 | 8.77% | May 2026 |
| 2025-26 | 8.37% | May 2025 |
| 2024-25 | 8.77% | May 2024 |
| 2023-24 | 8.27% | May 2023 |
| 2022-23 | 4.77% | May 2022 |
| 2021-22 | 4.52% | May 2021 |
| 2020-21 | 4.52% | May 2020 |
| 2019-20 | 5.37% | May 2019 |

The table lives in
[`div7aloan/data/benchmark_rates.csv`](div7aloan/data/benchmark_rates.csv)
with its own `reviewed_until` and `reviewed_on` header, last reviewed
28 August 2026. Nothing in this repository reads the network, in tests or at
runtime.

A year outside that coverage is `UNKNOWN`. To review one, supply a reviewed
override:

```json
{
  "verified_until": "2027-28",
  "citation": "RBA table F5 series FILRHLBVS, May 2027 figure, read 2027-07-02",
  "rates": [
    { "year_of_income": "2027-28", "rate": "0.0850", "rba_month": "2027-05" }
  ]
}
```

The file is refused without both `verified_until` and a non-empty `citation`,
and refused if it lists a year past its own `verified_until`. The point of an
override is that a human went and read the figure; a file that does not say
who checked what, and how far, is not a review.

## Arithmetic

`decimal.Decimal` throughout, built from strings. No float, no numpy, no
pandas. Intermediate arithmetic in the s 109E(6) formula runs at 60
significant digits in a local decimal context, so a result never depends on
the ambient context a caller happens to have set, and is quantised once at the
end.

Money is quantised to cents with **`ROUND_HALF_UP`**. The Act prescribes no
rounding for the s 109E(6) amount, so this is the engine's documented choice
rather than a statutory rule, and it is not the ATO calculator's rounding
adopted silently. Every result says so, in its statutory trace and in the JSON
`rounding` field.

JSON amounts are quoted decimal strings, never JSON numbers. Dates are
ISO 8601. Verdicts are enums. The emitter refuses to write a float, and the
test suite fails if one ever appears.

## Library API

The surface the MCP adapter will import. Stable; the modules behind it are
not.

```python
from decimal import Decimal
from div7aloan import (
    GateFacts, MyrFacts, benchmark_rate, complying_loan_gate,
    minimum_yearly_repayment, parse_year, review_register,
)

year = parse_year("2026-27")

rate = benchmark_rate(year)
# rate.verdict, rate.rate (Decimal), rate.rba_month, rate.reason

gate = complying_loan_gate(GateFacts(
    written_agreement=True,
    terms_in_place_before_lodgment_day=True,
    maximum_term_years=Decimal("7"),
    secured_by_registered_mortgage_over_real_property=False,
    interest_rate_for_years_after_year_loan_made=Decimal("0.0827"),
    year_loan_made=parse_year("2023-24"),
))
# gate.verdict, gate.limbs, gate.reasons, gate.caveats

result = minimum_yearly_repayment(MyrFacts(
    year_of_income=year,
    amalgamated_loan_unpaid_at_end_of_previous_year=Decimal("100000.00"),
    remaining_term_years=Decimal("5"),
    payments_applied_during_the_year=Decimal("25556.00"),
    gate_result=gate,
    year_loan_made=parse_year("2023-24"),
))
# result.verdict, result.myr_required, result.shortfall,
# result.experimental_deemed_dividend_exposure, result.statutory_trace

report = review_register(rows, year)
# report.summary, report.lines, report.total_exposure
```

Every result carries a verdict, amounts as `Decimal`, provenance, and a
refusal or unknown reason where one applies.

## Evaluation pack

[`evaluation/div7a_myr/`](evaluation/div7a_myr/) holds fabricated fixtures,
pinned expectations, and three minimum yearly repayments worked by hand from
the statutory formula with the arithmetic shown. A reviewer who knows s 109N
and s 109E can re-perform every one on paper without running Python.

It also records where this repository's reading of s 109N(1)(b) differs from
common practice, and why the Act was followed.

## Tests

```bash
python -m pytest -q
```

Every frozen rate year, every verdict, every refusal, the 2025-26 May-vs-June
trap, the formula against exact rational arithmetic, and a check that no
amount is ever emitted as a JSON number.

[GATES.md](GATES.md) is the completion ledger: one gate per acceptance
criterion, each naming the command that decides it and carrying the recorded
evidence of its last run.

## Provenance

Written against the
[Income Tax Assessment Act 1936](https://www.legislation.gov.au/C1936A00027/latest/text)
as compiled on the Federal Register of Legislation, `C1936A00027`, in force
1 July 2026, Part III Division 7A. Sections 109D, 109E, 109N, 109P and 109R
were read in full from the compilation, and the s 109E(6) formula from the
formula image in that text. It is not a port of any existing Division 7A
calculator.

Where the compiled Act and a secondary source disagree, this repository
follows the Act, cites the section, and records the disagreement.

The full source trail is in
[docs/primary-source-review-2026-08-31.md](docs/primary-source-review-2026-08-31.md):
which sections were read, how the s 109E(6) formula was recovered from the
compilation's own image, the reviewed position on s 109N(1)(b), and what this
engine deliberately does not establish.

## Related

- [DISCLAIMER.md](DISCLAIMER.md) - read it.
- [SECURITY.md](SECURITY.md)
- Benchmark rate explainer: <https://duguid.com.au/rates/div7a-benchmark-rate/>
- Australian tax AI agents: <https://duguid.com.au/tools/australian-tax-ai-agents/>

Later, from an AI coding agent, the same engine should run through
`aus-accounting-mcp`, which currently refuses Division 7A until a reviewed
repayment engine exists. **That adapter is not implemented here.** Pinning
this engine in place of `refuse_div7a` is a separate reviewed change in that
repository.

## Author and licence

Ryan Duguid. Written independently, in his own time, on his own equipment.
Provisional member of Chartered Accountants Australia and New Zealand; that
membership is not an endorsement of this software by CA ANZ.

MIT. See [LICENSE](LICENSE).
