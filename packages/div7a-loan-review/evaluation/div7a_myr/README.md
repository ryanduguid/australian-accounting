# Division 7A minimum yearly repayment evaluation

## Accounting problem

This pack isolates one question: given an amalgamated loan the operator has
already identified, and a later year of income, does the s 109E(6) formula
produce the amount this engine says it produces, and does the engine refuse
the questions it should refuse?

Six fabricated loans hold the statutory facts steady and vary one thing each.
Three are hand-worked below, so a reviewer who knows s 109N and s 109E can
re-perform them on paper without running Python.

## Intended reviewer

An accountant or tax adviser who can read the compiled Act, decide the facts
the engine will not decide, and form their own view. The pack demonstrates
the arithmetic and the refusal boundary; it does not replace professional
judgement, and it is not an ATO assessment.

## Primary sources and review date

The implementation was written on 31 August 2026 against the
[Income Tax Assessment Act 1936](https://www.legislation.gov.au/C1936A00027/latest/text)
as compiled on the Federal Register of Legislation, compilation
`C1936A00027` in force 1 July 2026, Part III Division 7A. Sections 109D,
109E, 109N, 109P and 109R were read in full from the compilation text. The
s 109E(6) formula was read from the formula image in the compilation
(`document_2/image.017.png`), not from a secondary source or a calculator.

Benchmark rates come from the frozen table in
[`div7aloan/data/benchmark_rates.csv`](../../div7aloan/data/benchmark_rates.csv),
cited to RBA statistical table F5, series `FILRHLBVS`, and cross-checked
against the published table at
<https://duguid.com.au/rates/div7a-benchmark-rate/>, last reviewed
28 August 2026. Nothing in this repository reads the network, in tests or at
runtime.

The repository's [primary-source implementation review](../../docs/primary-source-review-2026-08-31.md) records the wider source
trail and residual limits.

## The formula, as compiled

s 109E(5): "The minimum yearly repayment of an amalgamated loan for a year of
income is the amount worked out using the formula in subsection (6)."

s 109E(6):

```
    Amount of the loan not repaid by         Current year's benchmark
    the end of the previous year of    x         interest rate
                 income
  ---------------------------------------------------------------------
                                                        Remaining term
                  /                       1                 \
          1  -   |  ---------------------------------------  |
                  \  1 + Current year's benchmark interest   /
                                     rate
```

with, from the same subsection:

- **current year's benchmark interest rate** is the benchmark interest rate
  for the year of income for which the minimum yearly repayment is being
  worked out. Under s 109N(2) that is the RBA Indicator Lending Rates - Bank
  variable housing loans rate last published before the start of that year of
  income: the **May** figure, because the June figure is published in early
  July, after the year has begun.
- **remaining term** is the difference between (a) the number of years in the
  longest term of any of the constituent loans the amalgamated loan takes
  account of, and (b) the number of years between the end of the year of
  income in which the loan was made and the end of the year of income before
  the one being worked out; **rounded up** to the next higher whole number if
  the difference is not already a whole number.

## Deriving the remaining term for each fixture

The engine takes the remaining term as an operator input, because computing
it needs the constituent loans and v1 does not form amalgamated loans. It is
derived here so the fixtures can be checked end to end.

| Fixture | Year loan made | (a) longest term | Year of income | (b) years from end of loan year to end of previous year | Remaining term (a) - (b) |
| --- | --- | --- | --- | --- | --- |
| F1 | 2023-24 (ends 30 Jun 2024) | 7 | 2026-27 | 30 Jun 2024 to 30 Jun 2026 = 2 | 5 |
| F2 | 2022-23 (ends 30 Jun 2023) | 7 | 2023-24 | 30 Jun 2023 to 30 Jun 2023 = 0 | 7 |
| F3 | 2021-22 (ends 30 Jun 2022) | 7 | 2026-27 | 30 Jun 2022 to 30 Jun 2026 = 4 | 3 |

## Hand-worked fixture 1: repayment met exactly

`fixtures/myr_met_exact.csv`, reviewed for **2026-27**.

This loan is on complying terms. It was written in 2023-24 at 0.0827, which
met the 2023-24 benchmark of 0.0827, and it has a seven-year term.

Amount unpaid at the end of 2025-26: **100,000.00**.
Benchmark rate for 2026-27 (s 109N(2), RBA F5 FILRHLBVS, May 2026):
**0.0877**. Remaining term: **5**.

```
1 + r                  = 1.087700000000
(1 + r)^5              = 1.522459129015
(1 / (1 + r))^5        = 0.656832082347
1 - (1 / (1 + r))^5    = 0.343167917653

numerator  = 100000.00 x 0.0877 = 8770.000000
MYR        = 8770.000000 / 0.343167917653
           = 25556.002030
           = 25556.00   (quantised to cents, ROUND_HALF_UP)
```

Payments applied: 25,556.00. Shortfall: `max(0, 25556.00 - 25556.00)` =
**0.00**. Verdict **MYR_MET**, exit code 0.

## Hand-worked fixture 2: short by one thousand dollars

`fixtures/myr_short_by_1000.csv`, reviewed for **2023-24**.

This loan is on complying terms. It was written in 2022-23 at 0.0477, which
met the 2022-23 benchmark of 0.0477, and it has a seven-year term.

Amount unpaid at the end of 2022-23: **250,000.00**.
Benchmark rate for 2023-24 (May 2023): **0.0827**. Remaining term: **7**.

```
1 + r                  = 1.082700000000
(1 + r)^7              = 1.744042072513
(1 / (1 + r))^7        = 0.573380663093
1 - (1 / (1 + r))^7    = 0.426619336907

numerator  = 250000.00 x 0.0827 = 20675.000000
MYR        = 20675.000000 / 0.426619336907
           = 48462.407142
           = 48462.41   (quantised to cents, ROUND_HALF_UP)
```

Payments applied: 47,462.41. Shortfall: `max(0, 48462.41 - 47462.41)` =
**1,000.00**. Verdict **MYR_SHORT**, experimental exposure **1,000.00**,
exit code 2.

## Hand-worked fixture 3: a risen benchmark rate

`fixtures/rising_benchmark.csv`, reviewed for **2026-27**.

This loan is on complying terms. It was written in 2021-22 at 0.0452, which
met the 2021-22 benchmark of 0.0452, and it has a seven-year term. Nothing
about the agreement has changed. The benchmark rate has.

Amount unpaid at the end of 2025-26: **48,500.00**.
Benchmark rate for 2026-27 (May 2026): **0.0877**. Remaining term: **3**.

```
1 + r                  = 1.087700000000
(1 + r)^3              = 1.286848396133
(1 / (1 + r))^3        = 0.777092315618
1 - (1 / (1 + r))^3    = 0.222907684382

numerator  = 48500.00 x 0.0877 = 4253.450000
MYR        = 4253.450000 / 0.222907684382
           = 19081.666080
           = 19081.67   (quantised to cents, ROUND_HALF_UP)
```

Worked at the rate the agreement was written under, 0.0452, the same balance
and term give:

```
1 - (1 / 1.0452)^3     = 0.124206340224
MYR                    = (48500.00 x 0.0452) / 0.124206340224
                       = 2192.200000 / 0.124206340224
                       = 17649.662618
                       = 17649.66
```

The borrower repaid 17,649.66. Shortfall:
`max(0, 19081.67 - 17649.66)` = **1,432.01**. Verdict **MYR_SHORT**.

s 109E(6) uses the **current year's** benchmark rate, not the rate in the
agreement. A rising benchmark raises the minimum yearly repayment on loans
that were already complying, and the borrower who repaid the right amount
last year repays too little this year without changing anything.

## Refusal and unknown fixtures

| Fixture | What varies | Gate | MYR | Why |
| --- | --- | --- | --- | --- |
| `refused_year_of_loan.csv` | Loan made in the year being reviewed | COMPLYING | REFUSED | s 109E(1)(a) reaches an amalgamated loan made "in an earlier year of income", and s 109P puts an amalgamated loan outside s 109D in the year it is made. There is no minimum yearly repayment for the year of the loan. |
| `refused_not_complying.csv` | Interest rate 0.0400 against a 0.0477 benchmark | NOT_COMPLYING | REFUSED | s 109N(1)(b) fails. A constituent loan is one that would be a s 109D dividend apart from s 109N (s 109E(3)(b)), so this loan is not one and s 109E produces no repayment for it. Absent another Subdivision D exclusion, s 109D(1) treats it as a dividend in the year it was made (s 109D(1AA)). The engine will not print a schedule as though one might save it. |
| `unknown_missing_payments.csv` | `payments_applied_during_the_year` left unknown | COMPLYING | UNKNOWN | s 109R takes some payments out of the reckoning on a test the engine cannot apply. Bank credits are not a substitute for the operator's assertion. |

## Reproduce the result

The engine needs Python 3.10 or later and has no runtime dependencies. The
[repository README](../../README.md) carries the install step and the
definition of every register column used by the fixtures.

Record the version the figures were reproduced under:

```bash
python -m div7aloan.cli --version
```

Then, from the repository root:

```bash
python -m div7aloan.cli review --input evaluation/div7a_myr/fixtures/myr_met_exact.csv --year 2026-27
```

```bash
python -m div7aloan.cli review --input evaluation/div7a_myr/fixtures/myr_short_by_1000.csv --year 2023-24
```

```bash
python -m div7aloan.cli review --input evaluation/div7a_myr/fixtures/rising_benchmark.csv --year 2026-27
```

```bash
python -m pytest tests/test_evaluation_pack.py -q
```

`--format json` on any of those emits the machine-readable form, with every
amount as a quoted decimal string. The expectations are pinned in
[`expected_results.json`](expected_results.json) and the test re-runs each
fixture against them, so the numbers can be checked without trusting the
pretty-printed output.

## Where the Act and common practice diverge

Two points where this engine follows the compiled Act, and a reader coming
from a commercial calculator may expect something else.

**1. Which year's benchmark rate the s 109N(1)(b) floor uses.**

s 109N(1)(b) requires that "the rate of interest payable on the loan for
years of income after the year in which the loan is made equals or exceeds
the benchmark interest rate **for the year**". Read against the chapeau, "the
year" is the year of income in which the loan was made, and the whole of
s 109N(1) is tested once, before the lodgment day for that year.

So the `gate` command, run without `--year`, anchors each row to its own
`year_loan_made`. That is the statutory test.

It is also common in practice to check, year by year, that the rate being
charged still meets the current benchmark, on the footing that a loan
agreement expressed to carry "the benchmark rate" must keep up with it.
Passing `--year` runs that later-year comparison. The result carries a caveat
saying so, because it is a practice check on a risen benchmark and **not** the
s 109N(1)(b) test. Where the two readings could differ, this repository
follows the Act and labels the other.

The reviewed position, and the date it was confirmed, are recorded in
[docs/primary-source-review-2026-08-31.md](../../docs/primary-source-review-2026-08-31.md),
which also carries the full source trail behind every rule in the engine.

Nothing turns on this for the repayment calculation. s 109E(6) uses the
*current* year's benchmark rate on its own terms, so a risen benchmark raises
the minimum yearly repayment whichever reading of s 109N(1)(b) you prefer.
Fixture 3 is that case.

**2. Rounding.**

The Act prescribes no rounding for the s 109E(6) amount. This engine
quantises to cents with `ROUND_HALF_UP` and says so in every result, in the
statutory trace, and in the JSON `rounding` field. That is an engineering
choice, not a statutory rule, and it is not the ATO calculator's rounding
adopted silently. A reviewer who reaches a figure one cent away from this
one has not necessarily found an error in either.

Intermediate arithmetic runs at 60 significant digits in a local
`decimal` context and is quantised once, at the end. The implementation was
cross-checked against exact rational arithmetic (`fractions.Fraction`) over
the fixture set; the two agree to the cent.

## Controls and refusal boundary

The commands above run the production code against only the six declared
fabricated fixtures. Every identifier is synthetic (`SYN-001` and similar),
and there are no client names, TFNs, ABNs, ACNs or addresses in this
repository.

No result here authorises a payment, a journal, a lodgment, a disclosure or a
compliance conclusion.

## Limitations and non-claims

These fixtures do not establish that a loan exists, that an entity is a
shareholder or associate, that a payment is a genuine repayment under
s 109R, or that a company has a distributable surplus under s 109Y. They do
not form amalgamated loans from constituent loans (s 109E(3)), do not compute
the lodgment day (s 109D(6)), and do not touch s 109C payments, s 109F
forgiven debts, the interposed-entity rules in s 109T to s 109X, unpaid
present entitlements under s 109XA, the hardship discretion in s 109Q, the
extension in s 109RD, or the Commissioner's discretion in s 109RB.

The shortfall figure is an experimental review aid. It is not the dividend:
s 109E(2) makes the dividend the shortfall *subject to s 109Y*, and
s 109E(1)(d) removes the dividend entirely where s 109Q applies. It is not an
ATO assessment, and it is not a s 109E(1) determination.
