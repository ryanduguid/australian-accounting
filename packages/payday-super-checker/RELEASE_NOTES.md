# v0.1.7

- **Breaking for hand-built files.** A `fund_received_date` on a row with neither `matched_amount` nor `remitted_amount` no longer reads as a receipt of the whole `sg_amount`. The date evidences timing only, so where it could be on time the row is left `UNKNOWN` between `ON_TIME` and the partial-receipt outcome (`UNPAID`, `NOT_YET_DUE` or `LATE`), the run exits 2, and the caveat names the amount the row needs. A late receipt with no amount stays `LATE`, but the s 18D reduction of the final shortfall is not applied and the notional earnings run to the as-at date as a maximum. Files written by `import` already carry `matched_amount` and are unaffected.
- Migration: add `remitted_amount` and `matched_amount` columns and fill `matched_amount` with the amount the fund received for each row that has a `fund_received_date`; a full receipt states the whole `sg_amount`. The shipped examples and evaluation fixtures now do so. Older report CSVs, exit codes, column names and the canonical column order are unchanged.
- The Excel workbook now withholds notional earnings and charge estimates beyond the shipped [GIC table](paydaysuper/data/gic_rates.json), while retaining verdicts, days late and shortfalls. The formula changes are in [the workbook source](tools/build_workbook.py); the workbook has no stale-rate opt-in.
- The Excel workbook applies the same rule (branch codes `U1` to `U3`, lateness basis "as-at date (fund receipt amount not evidenced)") and was rebuilt through desktop Excel.

# v0.1.6

- Withhold the notional earnings, administrative uplift and SG-charge exposure for a row whose period runs past the last quarter in `gic_rates.json`, and say so in a caveat naming the day, the last quarter on record and the file to update. The verdict, days late and shortfall are still reported. `--allow-stale-gic` restores the estimate at the last known rate, and the report then says it did.
- Name the financial year a `rates.json` lookup is missing or unreadable for, so a stale rates file is told apart from a run that never needed the year.
- Carry a caveat on a row that has a fund-receipt date but no matched or remitted amount, naming the whole-liability reading the checker applies to it.

# v0.1.5

- Carry the join's structural matching warnings into the canonical CSV's new `join_caveats` column, so they travel through the checker into the report's caveats column and the evidence pack instead of being console-only.
- Keep genuine ambiguous joins refused: identical competing paydays still stop the import with no file written.
- The canonical format gains one optional trailing column; files from earlier releases parse unchanged.

# v0.1.4

- Keep payments awaiting clearance pending, with a warning and no remitted or received date.
- Match duplicate payroll rows only when the payment covers all competing unmet balances.
- Retain refusal for insufficient payments and align evidence-pack documentation.
