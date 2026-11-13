# ATO calculator interest comparison, 13 November 2026

This record compares the engine's `rollforward` (the year's interest and the
amount of the loan not repaid at its end, s 109E(7)) with the ATO's Division 7A
calculator and decision tool. The calculator is a secondary source. s 109E(7)
sets the rate, interest at the benchmark rate 'on the balance of the loan from
time to time in a year of income', but no day count or rounding; the engine
takes the day count from the worked example in the ATO's
[Loans by private companies](https://www.ato.gov.au/businesses-and-organisations/corporate-tax-measures-and-assurance/private-company-benefits-division-7a-dividends/in-detail/division-7a-loans)
(last updated 2 July 2026, read 13 November 2026) and the rest from the
calculator, as the README sets out.

## What was compared

The calculator page,
<https://www.ato.gov.au/single-page-applications/calculatorsandtools?anchor=DIV7A#DIV7A/questions>,
was loaded on 13 November 2026 at about 09:40 AEDT. Its scripts had the same
SHA-256 hashes as on 7 October 2026
([ato-calculator-comparison-2026-10-07.md](ato-calculator-comparison-2026-10-07.md)):

| File | SHA-256 |
| --- | --- |
| <https://www.ato.gov.au/misc/spa/calculators/ui/scripts/ato/subjectArea.IncomeTax.atogov.min.js> | `bf74939d7cc8bfb0de2e84813689282458521d8bcd37c5e96d459e190fdecd70` |
| <https://www.ato.gov.au/misc/spa/calculators/ui/scripts/ato/DIV7A.atogov.min.js> | `dae52ebe13b0eb86a9e7230473149c7e717902e7b223f765b8681093383d1c93` |

The function `calculateDivision7A` in the module
`subjectArea/IncomeTax/rules/calculateDivision7A` was called in the page,
through the page's module loader, with fabricated inputs. Nothing was entered
into the calculator's form and nothing was submitted. Read from that module, the
calculator:

- charges each period between repayments balance x rate x days / days in the
  year, with 366 days where the year of income ends in a leap year;
- rounds each period's interest to cents half up and adds the periods;
- reduces the balance by each repayment from its own date, merging repayments
  made on the same day, and charges nothing on a balance below nil;
- gives the closing balance as the opening balance less the repayments in
  excess of the interest, and never less than nil;
- takes the benchmark rate for the year from its own table.

Nothing in this package reads the network, in tests or at runtime, and none of
these files is vendored here.

## Cases

- The worked example in Loans by private companies: $75,000 for 2014-15,
  repayments of $20,000 on 31 August 2014 and $8,000 on 30 May 2015. 2014-15 is
  outside this engine's rate table, so the unit test `the_atos_worked_example`
  in `src/rollforward.rs` checks it with the example's own 5.95% rate. The
  calculator, run on it, gave the same periods, interest and closing balance.
- 2,000 fabricated loans: 250 for each year of income from 2019-20 to 2026-27,
  each in the second year after the loan year, so the first-year treatment of
  repayments before the lodgment day does not arise. 8 fixed opening balances
  from $0.01 to $99,999,999.99 and the rest pseudo-random up to $5,000,000.00,
  with 0 to 6 repayments. Of the 2,000, 130 have a repayment on 29 February,
  506 two repayments on one day, 536 one on 1 July and 499 one on 30 June.

Each loan's interest, each period's days and interest, the principal repaid,
the closing balance and the rate were compared with the engine's
`interest`, `periods`, `applied_to_principal`,
`amalgamated_loan_unpaid_at_end_of_year` and `benchmark_rate`.

## Results

In 1,997 of the 2,000 loans every figure agreed to the cent. In the other 3
the repayments were more than the opening balance and the year's interest: the
calculator gives a nil closing balance, and `rollforward` refuses, as it is
designed to. The worked example's periods (61, 272 and 32 days, interest
$745.79, $2,438.68 and $245.17) and its $3,430 interest and $50,430 balance,
rounded to the dollar, are reproduced by the unit test.

## What this does not establish

- The law on day count and rounding. s 109E(7) prescribes neither; the
  agreement shows only that the engine works as the ATO's example and
  calculator do.
- The first year after the loan year, which was not compared beyond the worked
  example; the calculator's treatment of repayments before the lodgment day is
  in the 7 October record.
- A substituted accounting period, which neither the calculator nor this engine
  takes.
- The calculator's future behaviour. Its scripts are undocumented and can
  change without notice; these results hold for the files hashed above.
