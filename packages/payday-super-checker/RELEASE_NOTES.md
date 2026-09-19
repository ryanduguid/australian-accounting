# v0.1.5

- Carry the join's structural matching warnings into the canonical CSV's new `join_caveats` column, so they travel through the checker into the report's caveats column and the evidence pack instead of being console-only.
- Keep genuine ambiguous joins refused: identical competing paydays still stop the import with no file written.
- The canonical format gains one optional trailing column; files from earlier releases parse unchanged.

# v0.1.4

- Keep payments awaiting clearance pending, with a warning and no remitted or received date.
- Match duplicate payroll rows only when the payment covers all competing unmet balances.
- Retain refusal for insufficient payments and align evidence-pack documentation.
