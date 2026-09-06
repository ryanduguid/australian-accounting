# Sent before the deadline, received when?

Synthetic case study, reproduced on 6 September 2026. Review aid, not professional advice; a reviewer remains responsible for the conclusion.

## The review question

A payroll preparer has a remittance record and wants to mark super as paid on time. The missing fact is when the fund received the contribution. This example keeps the payday and amount fixed so the reviewer can see exactly which evidence changes the result.

## Input and result

The four existing fixtures use synthetic employee SYN001, payday 6 August 2026, an operator-supplied SG amount of $120 and an as-at date of 20 August. The bundled national calendar produces a deadline of 17 August. These examples assume the ordinary timing pathway; they do not establish employee eligibility, SG amount or an extension.

| Existing fixture | Remitted | Fund received | Actual verdict |
| --- | --- | --- | --- |
| timely_remittance_no_receipt.csv | 14 August | Missing | AT_RISK |
| late_remittance_no_receipt.csv | 18 August | Missing | LATE |
| receipt_on_due_date.csv | 14 August | 17 August | ON_TIME |
| receipt_after_due_date.csv | 14 August | 18 August | LATE |

The first row has enough evidence to say the employer remitted before the deadline, but not enough to conclude the fund received it on time. The second row establishes lateness from the late remittance itself. Missing receipt evidence does not make every row unknowable.

## Reproduce

From `packages/payday-super-checker`, with uv installed:

```bash
uv run --locked --extra dev --python 3.12 payday-super-check evaluation/payday_super_evidence/fixtures/timely_remittance_no_receipt.csv --as-at 2026-08-20 --confirm-remittance-only
```

This prints AT_RISK and writes `report.csv`. With `--confirm-remittance-only`, it exits 0 because the operator has acknowledged the evidence gap. Without that flag it exits 2. The flag cannot turn AT_RISK into ON_TIME. Replace the fixture filename with another row above to reproduce it; the receipt-on-deadline case exits 0 and the late cases exit 2 when any required remittance-only acknowledgement is supplied.

[All fixture inputs and expected results](../evaluation/payday_super_evidence/README.md) remain inspectable. The existing `tests/test_evaluation_pack.py` checks the production calculation for all four records. The CLI acknowledgement is a separate operational boundary.

## Interpretation and evidence

Review the payroll date, contribution amount, remittance and per-contribution fund receipt together. Do not substitute an account debit or a clearing-house submission acknowledgement for a receipt. Check whether an extension or another timing pathway applies before accepting the date.

The [primary-source review](primary-source-review-2026-08-15.md) records the legal reading and its review date. The [calendar and runtime documentation](../README.md) records coverage and unknown outcomes. A charge estimate is experimental; it is not an ATO assessment. No live payroll, client information or external user evaluation is represented here.

## Human decision

Do the fund-receipt evidence and applicable timing rules support closing each exception, or must I obtain further evidence or escalate a late contribution for practitioner review?
