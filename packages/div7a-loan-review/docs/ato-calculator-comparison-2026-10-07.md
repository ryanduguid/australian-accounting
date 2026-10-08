# ATO calculator comparison, 7 October 2026

This record compares the engine's s 109E(6) minimum yearly repayment with the
ATO's Division 7A calculator and decision tool. It changes no rule, rate or
figure. The calculator is a secondary source: where it and the compiled Act
disagree, this repository follows the Act, as
[primary-source-review-2026-08-31.md](primary-source-review-2026-08-31.md)
sets out.

## What was compared

The calculator page,
<https://www.ato.gov.au/single-page-applications/calculatorsandtools?anchor=DIV7A#DIV7A/questions>,
works out its figures in the browser. On 7 October 2026 it loaded these files,
which were also fetched and hashed that day:

| File | Bytes | SHA-256 |
| --- | --- | --- |
| <https://www.ato.gov.au/misc/spa/calculators/ui/scripts/ato/subjectArea.IncomeTax.atogov.min.js> | 570,723 | `bf74939d7cc8bfb0de2e84813689282458521d8bcd37c5e96d459e190fdecd70` |
| <https://www.ato.gov.au/misc/spa/calculators/ui/scripts/ato/DIV7A.atogov.min.js> | 145,700 | `dae52ebe13b0eb86a9e7230473149c7e717902e7b223f765b8681093383d1c93` |
| <https://onlineservices.ato.gov.au/cdn/static-data/codes-tables/TC9GENTAC.json> | 850,612 | `357516f45f895fbcc13ea13b620d91447b780688e894427438845b8a8b125a63` |

The first file holds the module `subjectArea/IncomeTax/rules/calculateDivision7A`.
Its `calculateMinimumYearRepayment` function applies the formula with the
BigNumber library: 20 decimal places for intermediate division, rounding half
up, and a final rounding of the result half up to 2 decimal places. The third
file is the table the calculator reads its benchmark rates from, in rows of
type `DIV7A` and category `BENCHMARK_INTEREST_RATE`.

The function was called in the calculator's own page, through the page's
module loader, with fabricated inputs. Nothing was entered into the
calculator's form and nothing was submitted. Nothing in this package reads the
network, in tests or at runtime, and none of these files is vendored here.

## Cases

- Years of income ending 30 June 2020 to 30 June 2027 (2019-20 to 2026-27).
- Full terms of 7 and 25 years, with the loan made in each year of the term
  before the year of income, giving remaining terms of 1 to 25 years.
- 50 balances: 20 fixed values from $0.01 to $99,999,999.99 and 30
  pseudo-random amounts in cents up to $5,000,000.00.
- No repayments.

That is 12,800 calls. Each was compared with `minimum_yearly_repayment_amount`
in `div7aloan/myr.py`, using this engine's own benchmark rate for the year and
the same remaining term.

## Results

All 12,800 figures agreed to the cent. The calculator's benchmark rates for
2019-20 to 2026-27 equal those in `div7aloan/data/benchmark_rates.csv`; its
table also holds 22 earlier years, which this engine does not cover.

In the first year after the loan year, the function subtracts repayments dated
before the lodgment date entered from the amount not repaid by the end of the
loan year, then applies the formula. A repayment on the lodgment date itself is
not subtracted, and in later years nothing is subtracted. The calculator then
compares all repayments made in the year with that figure. The ATO's guidance,
[Loans by private companies](https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/private-company-benefits-division-7a-dividends/in-detail/division-7a-loans)
(last updated 2 July 2026), describes the same treatment.

For a $100,000 loan made in 2025-26 on a 7-year term, a lodgment date of
15 May 2027 and the year of income 2026-27:

| Repayments in 2026-27 | Calculator | This engine on $100,000 | This engine on $100,000 less repayments before 15 May 2027 |
| --- | --- | --- | --- |
| None | $19,715.97 | $19,715.97 | $19,715.97 |
| $20,000 on 1 October 2026 | $15,772.78 | $19,715.97 | $15,772.78 |
| $20,000 on 15 May 2027 | $19,715.97 | $19,715.97 | $19,715.97 |
| $20,000 on 1 June 2027 | $19,715.97 | $19,715.97 | $19,715.97 |
| $20,000 on 1 October 2026 and $5,000 on 1 June 2027 | $15,772.78 | $19,715.97 | $15,772.78 |

For the same loan made in 2024-25, so that 2026-27 is the second year after the
loan year, a $20,000 repayment on 1 October 2026 left the calculator at
$22,139.33, the figure this engine gives on $100,000.

## What this does not establish

- How the Act treats repayments made before the lodgment day. The comparison
  shows what the calculator and the ATO's guidance do; s 109E(3) and the
  formula's opening balance were not re-read for it.
- The calculator's interest and closing balance, which this engine does not
  compute.
- The calculator's future behaviour. Its scripts and table are undocumented
  and can change without notice; these results hold for the files hashed
  above.
