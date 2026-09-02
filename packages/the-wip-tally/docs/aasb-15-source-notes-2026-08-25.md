# AASB 15 source notes (25 August 2026)

Review aid. Confirm the operative compilation at
[standards.aasb.gov.au](https://standards.aasb.gov.au) before citing a
paragraph number. Compilations are remade and renumbered. This file records
which paragraphs the engine *implements as arithmetic*, and which it only
*flags for a person*.

Checked against the public AASB 15 text as cited in
[hardhat-ledger](https://github.com/ryanduguid/hardhat-ledger)
`.claude/skills/wip-over-under-billing/SKILL.md` (source review dated
15 August 2026). This engine does not replace that skill.

## Implemented as arithmetic

| Topic | Paragraphs (compilation then current) | What the engine does |
| --- | --- | --- |
| Transaction price, including constrained variable consideration | 47, 50-59 | Approved variations are added in full. Unapproved estimates are multiplied by `constraint_include_ratio`. The excluded slice is flagged, not booked. |
| Cost-to-cost input method | 39-40, B18 | Progress = progress cost / progress EAC. |
| Exclude wasted cost from the progress measure | B19(a), 98(b) | `inefficiency_rework_wastage` is stripped from numerator and denominator and left in EAC / margin. |
| Uninstalled materials at cost | B19(b) | `uninstalled_materials` recognised at cost (zero margin); remainder of transaction price follows POC. |
| Outcome not reasonably measurable | 44-45 | Revenue limited to `recoverable_costs` (defaulting to progress cost). A `recoverable_costs` above `costs_incurred` is refused with a `ScheduleError` naming the row, not silently reduced to it, because para 45 recognises revenue only to the extent of the costs incurred and an excess is a mapping error for the operator to resolve. Percent complete is reported as zero. |
| Contract asset vs contract liability | 105-107 | Per contract, revenue minus certified billings. Positive = asset. Negative = liability. No cross-contract offset. |
| Change in estimate | 43, AASB 108 | Period revenue = current revenue to date minus prior revenue to date. Comparatives are not restated, because this tool has no comparatives. |
| Right-to-invoice expedient | B16 | If selected, revenue equals certified billings. Flagged as not cost-to-cost. |
| Output method | 41, B14-B15 | If selected, revenue = transaction price x `output_percent`. Flagged as not cost-to-cost. |

## Flagged, not decided

| Topic | Paragraphs | Why it is a flag |
| --- | --- | --- |
| Combining contracts | 17 | Unit of account is an operator fact. Combine before the CSV. |
| Over time vs point in time | 35-38, B9-B13 | Legal characterisation. hardhat-ledger puts it to the engagement lead. |
| Whether a modification is a separate contract | 18-21 | Legal characterisation. |
| Whether variable consideration will reverse | 56-57 | Judgement. The ratio is an input, not a conclusion. |
| Retention as receivable vs contract balance | 105-108 | AASB 15 has no retention-specific rule. Classification is an interpretation. |
| Onerous contracts | AASB 137 66-69, 68A | Provision measurement is outside this engine. Negative EAC margin raises a review flag. Para 69 (impair assets used on the contract first) is flagged when `assets_used_carrying` is supplied. |
| Impairment of contract cost assets | AASB 15 101-103 | Separate, narrower test. Not substituted for AASB 137. |
| Expected credit losses on contract assets | AASB 9 via 15.107 | Not modelled. |
| Disclosure tier | AASB 1060 157-159, AASB 1053 | Not modelled. |

## Australian tax and GST notes (not computations)

- GST on certified progress claims, including retention, is *shown* at `gst_rate` as a BAS review prompt. It is not a BAS engine. Confirm GSTR treatment and the entity's tax-invoice practice.
- Do not carry this progress measure into income tax. Confirm the ATO position on long-term construction contracts at ato.gov.au for the year.

## Worked examples pinned in tests

Fabricated. No real contracts.

1. Clean cost-to-cost underbilling: $1,000,000 price, $400,000 cost, $400,000 ETC, $450,000 certified. Revenue $500,000. Contract asset $50,000.
2. B19 wastage plus stale ETC and profit fade: waste $80,000 stripped from progress; EAC still includes it; margin fades from 18.75% to nil.
3. Constrained mining-services claim: $400,000 unapproved x 25% constraint. Transaction price $2,200,000. $300,000 stays off-schedule.
4. Uninstalled materials on an underwater power-station civil package: $70,000 at cost; remainder at 70% POC; AASB 137 flag.
5. Para 45 early works: outcome not measurable; revenue equals recoverable cost $100,000. No prior period, so period revenue stays blank and no fade is measured.

## Residual limits

- No multi-performance-obligation split.
- No significant financing component.
- No expected-credit-loss matrix.
- No Security of Payment reference-date logic.
- No fuel tax credits, Coal LSL, payroll tax or TPAR.
- Committed-cost tests are reasonableness flags, not a purchase-order subledger.
