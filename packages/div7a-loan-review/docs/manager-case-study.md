# The bank credit exists. Does the repayment count?

Synthetic case study, reproduced on 6 September 2026. Review aid, not professional advice; repayment classification remains a human decision.

## The review question

The loan register shows $25,556 repaid and the arithmetic says the minimum yearly repayment is met. The reviewer then learns that $10,000 of those credits coincides with a further company advance. Does the amount supplied to the calculation still have a defensible basis?

The further advance is a synthetic evidence complication, not an engine finding. It does not automatically establish that a repayment is disregarded under s 109R.

## Input and result

The existing F1 fixture supplies an amalgamated loan made in 2023-24, an unpaid balance of $100,000 at the end of 2025-26 and five years remaining. Its agreement facts are operator assertions. For the 2026-27 scenario, the frozen reviewed rate is 8.77% and the engine rounds its minimum yearly repayment to $25,556.00.

| Conditional scenario | Repayments supplied | MYR | Shortfall | Verdict |
| --- | ---: | ---: | ---: | --- |
| All credits accepted | $25,556.00 | $25,556.00 | $0.00 | MYR_MET |
| Questioned $10,000 excluded | $15,556.00 | $25,556.00 | $10,000.00 | MYR_SHORT |

This is a full-year hypothetical, not a claim that the 2026-27 repayment deadline has passed. The second row is a sensitivity calculation. It shows the consequence of an input decision without making that decision for the reviewer.

## Reproduce

From `packages/div7a-loan-review`, using Python 3.10 or later:

```bash
python -m div7aloan.cli review --input evaluation/div7a_myr/fixtures/myr_met_exact.csv --year 2026-27
python -m div7aloan.cli review --input examples/repayment_review_excluded.csv --year 2026-27
```

The first command exits 0; the second exits 2. Add `--format json` to inspect decimal amounts and the statutory trace. The second CSV copies the existing synthetic F1 facts, changes the scenario identifier and reduces only the asserted repayment amount by $10,000.

[The existing evaluation](../evaluation/div7a_myr/README.md) shows the formula, remaining-term derivation, frozen-rate provenance and hand-worked arithmetic. Its missing-payments fixture returns UNKNOWN, which is appropriate when the reviewer cannot yet supply a supported amount.

## What still needs evidence

The reviewer must establish the loan grouping, agreement facts, repayment dates and source of funds. The engine neither forms amalgamated loans nor classifies repayments under s 109R. A bank credit alone is not that classification.

The shortfall is not a final deemed-dividend conclusion. Distributable surplus and applicable exclusions or relief remain outside the arithmetic. The [primary-source implementation review](primary-source-review-2026-08-31.md) records the source reading and residual limits. This case demonstrates the existing implementation; it is not independent legal validation or an external user trial.

## Human decision

What amount can I substantiate as repayments for Division 7A purposes, including the s 109R assessment, before accepting the calculation and determining any further tax treatment?
