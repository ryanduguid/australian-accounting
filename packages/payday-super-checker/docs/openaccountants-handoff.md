# From an OpenAccountants guide to a Payday Super check

A worked handoff on fabricated data. The OpenAccountants
[`au-super-guarantee` guide](https://github.com/openaccountants/openaccountants/blob/c4c1212ce5dac374ceda90abd7b12a7f1558a0b3/skills/international/australia/au-super-guarantee.md)
(version 3.3, read at commit `c4c1212` on 25 September 2026) sets out the Payday Super
rules and a per-payday working paper. This checker takes the same facts and computes
each contribution's due date, lateness and experimental SG charge estimate. The guide is
linked, not copied: its content carries the OpenAccountants Guide License, separate from
this repository's MIT licence.

The guide's own disclaimer and `review_status: pending_review` apply. Nothing here
reviews or certifies the guide.

## Which copy of the guide

Read the maintained guide under `skills/`. At the same commit the repository also holds
an [Agent Skills export](https://github.com/openaccountants/openaccountants/blob/c4c1212ce5dac374ceda90abd7b12a7f1558a0b3/agent-skills/au-super-guarantee/SKILL.md),
generated on 21 July 2026 and not regenerated since. That copy is version 2.0, written
for 2024-25, and still tests contributions against quarterly due dates. An agent that
loads the export gets the old rule; the maintained guide says never to apply quarterly
due dates to earnings paid from 1 July 2026.

## The fabricated case

EMP101 is paid fortnightly from 2 July 2026, with $480.00 super guarantee on each of 7
paydays in the July to September quarter. The employer pays the whole quarter in one
remittance on 20 October 2026, and the fund receives it on 22 October. Under the old
quarterly rule that would have met the 28 October due date.

| Guide working paper field | Value from the case | Checker column |
| --- | --- | --- |
| Employee | EMP101 (pseudonym) | `employee_id` |
| Payday | each of the 7 paydays | `payment_date` |
| SG contribution | $480.00 per payday | `sg_amount` |
| Fund receipt deadline (QE day + 7 business days) | worked out per payday | `due_date`, derived |
| Fund receipt confirmed | received 22 October, $480.00 per payday | `fund_received_date`, `matched_amount` |

The case is in [`examples/quarterly_remittance_2026_27.csv`](../examples/quarterly_remittance_2026_27.csv).
The guide's bank statement notes flag a quarterly-sized lump after June 2026 as possible
regime confusion; this case is that pattern.

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

The maintained guide and the checker agree that all 7 contributions are late. The guide
gives the rule and the working paper; the checker adds the per-payday dates, the day
counts and the estimate. The outdated Agent Skills export would have marked the quarter
as paid on time.

## What stays with the reviewer

- Whether each `sg_amount` is the right super guarantee amount (qualifying earnings,
  salary sacrifice, employee status). Neither the guide's working paper nor the checker
  decides it.
- Whether any contribution was the first to a new fund, which extends its due date. The
  checker's report says how the last payday changes if it was.
- What to do about the late contributions, including any voluntary disclosure. The
  estimate range depends on facts the checker does not have.

`tests/test_openaccountants_handoff.py` reruns this case against the engine in this
directory and checks the figures above.
