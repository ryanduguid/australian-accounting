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
