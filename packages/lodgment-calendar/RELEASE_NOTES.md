# v0.1.0

Not yet released.

- `due_dates` returns lodgement, payment and concession dates for monthly and quarterly
  BAS and IAS periods in 2026-27 and for 2025-26 income tax returns of individuals,
  trusts, partnerships, companies (including head companies of consolidated groups) and
  SMSFs, lodged by the taxpayer or through a registered tax or BAS agent.
- Applies the 2026–27 registered agent and BAS agent lodgment program dates and
  concessions, the 2-week online concession for quarterly statements and the December
  BAS concession, each with its ATO source, provision and review date.
- Moves dates to the next business day using the ATO's 2026–27 public holiday tables.
- Returns `UNKNOWN`, naming the missing condition, for any date that depends on a fact
  not supplied, a date the ATO pages leave open or a day outside the holiday tables.
