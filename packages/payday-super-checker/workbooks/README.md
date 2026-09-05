# Excel workbook

`payday-super-checker.xlsx` is the same review as the command line checker,
written in ordinary worksheet formulas for accountants who work in Excel and do
not have Python. It is macro-free and needs desktop Excel for Microsoft 365 or
Excel 2024. Nothing leaves the workbook.

**Experimental review aid. Not a compliance determination.** Everything the
checker refuses or leaves `UNKNOWN`, the workbook refuses or leaves `UNKNOWN`,
and the assumptions the checker prints with every run sit on Review Checks.

## Using it

1. **Register.** Paste one contribution line per row over the example rows in
   the eleven canonical columns the checker reads and `import` writes:
   `employee_id`, `payment_date`, `sg_amount`, `remitted_date`,
   `remitted_amount`, `matched_amount`, `fund_received_date`,
   `first_contribution_to_fund`, `out_of_cycle`, `next_standard_payday`,
   `defined_benefit`. Dates are real Excel dates, amounts are dollars, the
   yes/no columns take `yes`, `no` or blank. The column meanings are in the
   package README. Rows pasted below the example pick up the calculated columns.
   Delete any example rows you did not overwrite: Review Checks flags a
   fabricated example line that is still in the register, because it would
   otherwise count in the totals.
2. **Summary.** Set the as-at date (`--as-at`), an assessment date if the ATO
   has assessed (`--assessment-date`), and the two confirmations the checker
   asks for: LCR 2026/1 transition allocation
   (`--confirm-transition-allocation`) and remittance-only review
   (`--confirm-remittance-only`). Both ship as `N`, as the CLI defaults them,
   so the shipped sample is BLOCKED at the transition check exactly as the CLI
   stops on it without the flag. Do not tick a confirmation mechanically; the
   package README says when each is appropriate.
3. **Register, calculated columns.** The pathway and final deadline (usual 7
   business days, 20 for a first contribution to a fund, the next standard
   payday's period for an out-of-cycle payment, or item 4 alignment to an
   evidenced earlier contribution), the verdict, the conservative outer
   outcomes where the facts cannot decide, days late and what they are measured
   to, the base and final shortfall after any s 18D offset, notional earnings,
   the uplift range and the experimental SG charge estimate range. These are
   the `report.csv` columns.
4. **Review Checks.** `BLOCKED` means the register cannot be read (a date or
   amount that is not one, a negative, an amount invariant the checker rejects,
   a pre-1 July 2026 payday, an out-of-cycle line without its next standard
   payday, a formula in an input) or a transition line is not confirmed.
   `REVIEW` means a line is `LATE` or `UNPAID`, a line is undecided, or no
   assessable line has a fund receipt and remittance-only review is not
   accepted. `PASS` mirrors the checker's exit code 0.
5. **Holidays and GIC.** The whole-of-jurisdiction holiday table and the
   quarterly general interest charge rates the engine ships. To review a
   deadline past the holiday coverage, add the official dates and move the
   coverage date on Summary; to keep notional earnings exact past the last
   known quarter, add the ATO's next rate and mark it known.

## How it is kept honest

The workbook is a second implementation of the rules in `paydaysuper`, so it can
drift. `tests/test_workbook.py` reads the cached values desktop Excel wrote and
holds every line of `examples/sample_payrun.csv`, the summary counts and totals,
the Holidays table and the GIC table to the engine. A change to the engine, the
data or the workbook that is not rebuilt fails the test.

Rebuild on a machine with desktop Excel, from the package directory:

```powershell
uv run --locked --extra dev python tools/build_workbook.py
```

The build writes the formulas with openpyxl, opens the file once in Excel over COM
to calculate and save real cached values, records the Excel build on Sources &
Version and strips the machine path Excel stamps into the file. Pass `--no-excel`
to skip the Excel pass; the file then ships without cached values and the parity
test fails, which is the point.

Two things are worth knowing about how the formulas differ from the code while
giving the same answers. Item 4 is not swept row by row: the confirmed latest
deadline for a line is the largest own deadline of an earlier same-employee line
whose receipt is confirmed against its own deadline, and the possible upper bound
is the same over lines not proved impossible, which is what the sweep converges
to. Notional earnings are not accrued day by day: the daily compounding over each
GIC quarter is one power, so the exact formula is `base x (product of
(1 + rate/days in year)^days) - 1`. Both were fuzzed against the engine before
the workbook shipped.
