# From an OpenAccountants guide to a Payday Super check

A worked handoff on fabricated data. The OpenAccountants
[`au-super-guarantee` guide](https://github.com/openaccountants/openaccountants/blob/c4c1212ce5dac374ceda90abd7b12a7f1558a0b3/agent-skills/au-super-guarantee/SKILL.md)
(read at commit `c4c1212`, 25 September 2026) helps a preparer gather the facts for
super guarantee work. This checker then tests the timing of each contribution against
the Payday Super deadlines. The guide is linked, not copied: its content carries the
OpenAccountants Guide License, separate from this repository's MIT licence.

## Why the handoff matters for 2026-27

At that commit the guide is written for 2024-25. Its quick reference notes that
Payday Super commences on 1 July 2026, but its deadline rule and working paper still
test each quarter's contributions against the quarterly due date (28 October for July
to September). From 1 July 2026 the deadline runs from each payday instead: usually 7
business days to fund receipt. A working paper that follows the quarterly rule can mark
a 2026-27 contribution as paid on time when it is not.

The guide's own disclaimer and "Validated by: Pending" status apply. Nothing here
reviews or certifies the guide.

## The fabricated case

EMP101 is paid fortnightly from 2 July 2026, with $480.00 super guarantee on each of 7
paydays in the July to September quarter. The employer pays the whole quarter in one
remittance on 20 October 2026, and the fund receives it on 22 October, before 28 October.

| Guide working paper field | Value from the case | Checker column |
| --- | --- | --- |
| Employee name | EMP101 (pseudonym) | `employee_id` |
| SG contribution | $480.00 per payday | `sg_amount` |
| Due date | 28 October 2026 (quarterly rule) | derived per payday, not supplied |
| Paid on time | YES under the quarterly rule | `verdict`, derived |
| (evidence behind "paid") | remitted 20 October, received 22 October | `remitted_date`, `fund_received_date`, `matched_amount` |

The case is in [`examples/quarterly_remittance_2026_27.csv`](../examples/quarterly_remittance_2026_27.csv).
The checker needs each payday as its own row, which the guide's per-quarter working
paper does not record. Collecting pay dates is the main extra step in the handoff.

## Run it

```bash
uvx --from payday-super-checker==0.1.7 payday-super-check examples/quarterly_remittance_2026_27.csv --as-at 2026-10-28 -o quarterly-report.csv
```

Run from `packages/payday-super-checker/`. The command exits 2 because it finds late
contributions.

## Result

`ON_TIME: 0  AT_RISK: 0  LATE: 7  UNPAID: 0  UNKNOWN: 0  SKIPPED: 0`

Every payday is late to fund receipt, from 101 days (2 July payday, due 13 July) to 15
days (24 September payday, due 7 October). Because the fund received the full amount
before any assessment, the final shortfall is nil, and what remains is notional earnings
and uplift: `experimental estimated SG charge $62.49 - $99.98` across the 7 lines. The
checker labels these figures experimental estimates; the ATO assesses the charge.

So the same facts give two answers: on time under the guide's quarterly working paper,
late on all 7 paydays under Payday Super.

## What stays with the reviewer

- Whether each `sg_amount` is the right super guarantee amount (qualifying earnings,
  salary sacrifice, employee status). Neither the guide's working paper nor the checker
  decides it.
- Whether any contribution was the first to a new fund, which extends its due date. The
  checker's report says how the last payday changes if it was.
- What to do about the late contributions, including any voluntary disclosure. The
  estimate range depends on facts the checker does not have.
- Whether the guide should be updated for 2026-27. That is a matter for its maintainers.

`tests/test_openaccountants_handoff.py` reruns this case against the engine in this
directory and checks the figures above.
