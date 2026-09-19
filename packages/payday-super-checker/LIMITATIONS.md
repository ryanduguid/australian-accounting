# Known limitations

Behaviour this tool ships with that produces a figure you should not read at
face value. A limitation is not a refusal: a refusal returns the question to a
person with the reason attached, and nothing here refuses. Each entry names the
condition that triggers it, what it affects, **what stays correct**, and the
step that settles it.

For what the tool declines to answer, see the refusal cases in
[README.md](README.md). For scope the tool never claimed, see
[DISCLAIMER.md](DISCLAIMER.md) and
[docs/primary-source-review-2026-08-15.md](docs/primary-source-review-2026-08-15.md).

Applies to payday-super-checker 0.1.4.

## PSC-1 An SG-charge estimate dated past the GIC table extrapolates the rate

**Trigger.** Any day in the notional earnings period falls after the last
quarter recorded in `paydaysuper/data/gic_rates.json`.

**Effect.** `GicTable.daily_rate()` returns the last known quarter's rate for
those days rather than refusing. The notional earnings component under SGAA
s 19A is then compounded at a rate the ATO has not published for that quarter.
The administrative uplift under s 19B(1) is a percentage of shortfalls plus
notional earnings, so it moves with the extrapolated figure wherever the
percentage is non-zero. That percentage is not fixed: `uplift_scenarios()`
models the reg 13C and reg 13D reductions, and `exposure_range()` spans 0% for
the low estimate (clean 24-month history with a voluntary disclosure inside 30
days) to 60% for the high estimate (prior history, no disclosure).

**Both exposure totals move, including the low one.** `exposure_range()` adds
the notional earnings into each total before the uplift, so an extrapolated
rate changes the low estimate too. Only the low estimate's *uplift component*
is unmoved, because 0% of a larger base is still nil. Do not read the low
estimate as a rate-independent floor.

**What stays correct.** Every verdict. All 6 of them, `ON_TIME`, `AT_RISK`,
`LATE`, `UNPAID`, `UNKNOWN` and `SKIPPED`, are decided by the deadline and
fund-receipt tests, which never read the GIC table. Deadlines, business-day
arithmetic, the matched and remitted amounts, and the SG shortfall itself are
all unaffected. `GicTable.daily_rate` is called from exactly one place, the
notional earnings loop in `paydaysuper/sgc.py`, so nothing outside the exposure
estimate depends on it.

The verdict being sound does not make the row's figures sound. `LATE` and
`UNPAID` are the 2 verdicts in `EXPOSED`, so a row carrying either also carries
the exposure estimate this entry qualifies. A `SKIPPED` row has no estimate to
qualify.

**Where it surfaces at runtime.** `GicTable.staleness()` adds a caveat to the
result naming the table's end date, the rate carried forward, and the file to
update. Caveats reach the console, unlike notes.

**Operator step.** Update `paydaysuper/data/gic_rates.json` from the ATO
general interest charge rates page before relying on an exposure figure whose
period runs past the table. Re-run and confirm the caveat is gone.

**Status.** Open, by design. Refusing would withhold a shortfall figure that is
itself correct. The estimate is offered with the extrapolation declared rather
than withheld or presented as settled.

## PSC-2 The payroll-to-super join degrades in four declared ways

**Trigger.** Any of four column conditions in the supplied exports.

| Condition | Effect on matching |
| --- | --- |
| Either file has a blank `employee_id` on any row | Matching falls back to employee name. Two employees sharing a name are merged. |
| The payroll file has no pay-period-end column | Matching falls back to the payday. A super payment recorded against the pay period rather than the payday can be missed. |
| The super file has only one of pay-period start or end | A payment's coverage collapses to a single day and can miss the payday it actually settled. |
| The super file has no pay-period columns at all | A payment is treated as covering every payday for that employee. Where it cannot cover every competing balance, `_check_defensible` refuses only if 2 or more of the still-competing payroll rows are identical in payday, effective period end and amount. Rows differing in any of those 3 are sorted and apportioned, not refused. |

**Effect.** Each condition weakens the join that every downstream verdict rests
on. A merged pair of employees or a missed payment changes which contribution
is tested against which deadline, so a verdict can be wrong without any
arithmetic being wrong.

**What stays correct.** The arithmetic on whatever was matched. Deadline
computation, business-day handling, receipt testing and the exposure estimate
all operate correctly on the rows they were given. The limitation is in the
join, not the assessment.

**Where it surfaces at runtime.** `paydaysuper/join.py` emits one warning per
condition. These are structural warnings: `paydaysuper/cli.py` prints them
ahead of every row-level warning and never truncates them, because they govern
whether the whole join can be trusted. The report header also states the key in
use as `employee matching: by id` or `by name`.

**Since 0.1.5 the warnings also travel with the files.** The import writes
them into the canonical CSV's `join_caveats` column on every row, so the
checker's report carries them in its caveats column and the evidence pack
carries them in `report.csv`: a reviewer holding only an exported file sees
the qualification that governs its verdicts, including an `ON_TIME` produced
from a degraded join. In 0.1.4 this declaration was console-only, so a file
from that release still needs its console transcript beside it before a
verdict can be relied on. Re-exporting with complete columns removes the
warning at the source; the column itself never clears an ambiguous join.

**Operator step.** Re-export with an employee id column present on both files,
a pay-period-end column on the payroll file, and pay-period start and end
columns on the super file. All three are needed: fixing only the ids still
leaves the payday fallback and the coverage collapse in place. Where a
re-export is not available, reconcile the affected employees by hand before
relying on their verdicts.

**Status.** Open, by design. The tool reports what it matched on rather than
refusing an export that a firm cannot re-run.

## Not limitations

Recorded here so they are not re-audited:

- **Fund receipt dates are blank on every import.** No payroll or clearing
  house export carries one, and the s 18C test is receipt by the fund. This is
  the tool's central boundary, not a defect, and it is published on the
  [tool page](https://duguid.com.au/tools/payday-super/) and stated by the CLI
  on every import.
- **Components not modelled in the SG charge.** Choice loading under s 20A, the
  post-assessment late payment penalty, and GIC on an unpaid assessment are out
  of scope and recorded in README.md and the `paydaysuper.sgc` module
  docstring.
