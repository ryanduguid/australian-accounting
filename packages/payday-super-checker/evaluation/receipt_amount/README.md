# Receipt amount evaluation

## Accounting problem

Payday Super is met when the fund receives the contribution, not when the
employer remits it. A fund-receipt date shows when the fund received
something; it does not show how much. This evaluation records what checker
0.1.7 does when the receipt amount is missing, partial or supplied only as a
remitted amount, and when a payday falls before the 1 July 2026 start.

## Intended reviewer

A payroll reviewer or bookkeeper checking whether the checker's result matches
what they would conclude from the same records. It is not a client workpaper.

## Fabricated inputs

Each file in `fixtures/` holds one synthetic contribution: employee `SYN001`,
paid on 6 August 2026 with $120.00 of SG, except the refusal case, which is
paid on 25 June 2026. No file contains client or employee data.

## Expected results

Assessed as at 20 August 2026. The due date for a 6 August payday is
17 August 2026, 7 business days later.

| Scenario | Receipt date | Amount columns | Verdict | Exit | Final shortfall |
| --- | --- | --- | --- | --- | --- |
| `timely_receipt_no_amount` | 17 Aug | none | `UNKNOWN` | 2 | not assessed |
| `late_receipt_no_amount` | 18 Aug | none | `LATE` | 2 | 120.00, a maximum |
| `timely_part_receipt` | 17 Aug | remitted 100.00, matched 100.00 | `UNPAID` | 2 | 20.00 |
| `timely_receipt_remitted_amount_only` | 17 Aug | remitted 120.00, ten-column file | `ON_TIME` | 0 | none |
| `timely_receipt_matched_amount` | 17 Aug | matched 120.00 | `ON_TIME` | 0 | none |

- **No amount, on time.** The checker cannot tell whether the fund received
  $120.00 or less, so the result could be `UNPAID` or `ON_TIME`. It reports
  `UNKNOWN` and asks for `matched_amount`. Checker 0.1.6 and earlier read the
  same row as `ON_TIME`.
- **No amount, late.** The receipt date alone proves lateness. Without an
  amount the checker cannot apply the reduction for a late payment, so the
  final shortfall shown is the whole $120.00 and is a maximum.
- **Part receipt.** $100.00 received on time reduces the shortfall to $20.00.
- **Remitted amount only.** This fixture uses the older ten-column layout,
  which has no `matched_amount` column. There, `remitted_amount` on a row with
  a receipt date is read as the amount that receipt covers. The
  result is `ON_TIME`, which rests on that reading: confirm the fund
  received $120.00 before relying on it.
- **Matched amount.** The explicit association the checker prefers.

### Refused

`payday_before_1_july_2026` exits 1 before any assessment with an error that
names the QE day before 1 Jul 2026. Quarterly SG law applies to that payday and
the checker covers Payday Super only.

## Reproduce the result

These fixtures are newer than the 0.1.7 release tag, but the published 0.1.7
package reproduces every row. From the component directory, with
[uv](https://docs.astral.sh/uv/) installed:

```bash
uvx --from payday-super-checker==0.1.7 payday-super-check evaluation/receipt_amount/fixtures/timely_receipt_no_amount.csv --as-at 2026-08-20 -o timely-no-amount.csv
uv run --locked --extra dev pytest tests/test_receipt_amount_evaluation.py -q
```

The first command runs the published package; the second runs the checked-out
source. The report CSV is written in the component directory, where the
repository ignores CSV output.

## Human decision

A fund-receipt date shows when the fund received something, not how much. A human must confirm the amount received and its allocation from the fund or clearing-house confirmation before relying on ON_TIME.

## Practitioner review

Pending. No external practitioner has yet confirmed these results. Until one
does, treat the table as the checker's recorded behaviour, not a benchmark.

## Product and fixture version

Product release `0.1.7`: every result above was reproduced with the published
0.1.7 package on 25 September 2026 (AEST). Fixture version `1`. The legal content is the
checker's own, current at 15 August 2026. This evaluation adds no legal
interpretation. The [evidence boundary evaluation](../payday_super_evidence/README.md)
stays pinned to release 0.1.3.

## Limitations and non-claims

Notional earnings and SG charge estimates depend on the bundled GIC table and
are not pinned here. The fixtures do not cover new starters, fund switches,
out-of-cycle payments or defined benefit members. This is not tax, legal or
financial advice; the ATO assesses the charge.
