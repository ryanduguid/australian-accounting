# Division 7A and FBT car differential, 6 November 2026

This repository's engines against the LodgeiT Labs calculator gateway, on
fabricated inputs, with each case's figure also worked out separately from the
statute. 12 cases were compared and 11 agreed to the cent. The 1 difference is a
wrong term this repository's trial case supplies on purpose, and it traces to
ITAA 1936 s 109E(6).

| | |
| --- | --- |
| Run | 6 November 2026 AEDT; 2026-11-05 18:12:53 to 18:13:32 UTC, the first and last `observed_at` in the record |
| Gateway | `https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app` |
| Gateway state | All 12 requests in the record answered 200. No error, throttle, block or terms notice. The listing read at 17:53:21 UTC showed 23 calculators, and the OpenAPI document read at 17:53:25 UTC was version `0.1.0a0`, both about 19 minutes before the pass |
| Provider's stability wording | 'Not stable yet. Response shapes may change. Nothing here is versioned for production integration.' ([calculators page](https://lodgeit.org/calculators.html), read the same day) |
| Local engines | `div7a-loan-review` and `australian-tax-calculators` at this repository's commit 2384216a, in the adapter's trials |
| Inputs | Fabricated: [`fixtures/div7a-cases.json`](../fixtures/div7a-cases.json), [`fixtures/fbt-car-cases.json`](../fixtures/fbt-car-cases.json) |
| Requests | 24 POSTs and 3 GETs in all: 2 passes, and GETs for the listing, the OpenAPI document and the drift check. One POST a case, 3 seconds apart, no retries |
| Full record | [`live-trials-2026-11-06.json`](live-trials-2026-11-06.json): the second pass's POSTs, each with its response. It does not hold the first pass or the GETs |

The pass ran twice. An earlier run at 18:07 UTC sent the same requests, but a bug
in this repository's record writer lost its record after the last request, so
nothing from it is used here. The bug is fixed and tested.

The JSON is what the run wrote, with 1 reason on D7A-4 reworded afterwards, because
it called that case's deliberately wrong figure 'independently derived'. It has no
`error` field because the writer gained one later. Its `edits_after_the_run` field
states both.

A match says 2 implementations and a separate calculation agreed on a number. It
does not say either implementation is right about the law, and it approves
nothing. No practitioner has reviewed this record.

## Division 7A minimum yearly repayment

Rates: 8.37% for 2025-26 (`fy2026`) and 8.77% for 2024-25 (`fy2025`), both sides.
The term is the remaining term under s 109E(6).

| Case | Base | Term | Worked out separately | This repository | LodgeiT | Outcome |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| D7A-1, repayment met | 100,000.00 | 7 | 19,450.97 | 19,450.97 | 19,450.97 | MATCH |
| D7A-2, short repayment | 100,000.00 | 7 | 19,450.97 | 19,450.97 | 19,450.97 | MATCH |
| D7A-3, excess repayment | 100,000.00 | 7 | 19,450.97 | 19,450.97 | 19,450.97 | MATCH |
| D7A-4, wrong term supplied | 75,000.00 | see below | 18,967.77 at 5 | 29,297.01 at 3 | 18,967.77 at 5 | SCOPE_MISMATCH, as designed |
| D7A-5, 2024-25 rate | 50,000.00 | 7 | 9,857.98 | 9,857.98 | 9,857.98 | MATCH |
| D7A-6, 1-year term | 100,000.00 | 1 | 108,370.00 | 108,370.00 | 108,370.00 | MATCH |
| D7A-7, 25-year term | 250,000.00 | 25 | 24,164.27 | 24,164.27 | 24,164.27 | MATCH |
| D7A-8, nil remaining term | 10,000.00 | 0 | none | Refused | Not sent | LOCAL_REFUSED |
| D7A-9, `fy2024` | 100,000.00 | 7 | none | Unmapped | Not sent | UNSUPPORTED_PERIOD |

Shortfalls agreed on every case that was sent: 0.00, except D7A-2 at 4,450.97 and
D7A-5 at 857.98.

## FBT car statutory formula

`fy2026` is the FBT year ended 31 March 2026, 365 days. The separate figure is
FBTAA s 9(1). Grossed-up amounts and FBT payable were compared on each case and
also agreed to the cent, at gross-up rates of 2.0802 and 1.8868 and an FBT rate
of 47%.

| Case | Base value and accessories | Days | Recipient's payment | Worked out separately | This repository | LodgeiT | Outcome |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| FBT-1, full year | 40,000.00 | 365 | 0.00 | 8,000.00 | 8,000.00 | 8,000.00 | MATCH |
| FBT-2, part year | 32,000.00 | 274 | 0.00 | 4,804.38 | 4,804.38 | 4,804.38 | MATCH |
| FBT-3, accessories and payment | 57,500.60 | 200 | 1,000.00 | 5,301.44 | 5,301.44 | 5,301.44 | MATCH |
| FBT-4, payment above value | 10,000.00 | 73 | 900.00 | 0.00 | 0.00 | 0.00 | MATCH |
| FBT-5, large type 1 | 73,595.92 | 365 | 0.00 | 14,719.18 | 14,719.18 | 14,719.18 | MATCH |

## The difference, traced to statute

**D7A-4.** A 7-year loan made in 2022-23, worked out for 2025-26. Section 109E(6)
makes the remaining term the longest term, 7, less the 2 years between the end of
2022-23 and the end of 2024-25: 5. LodgeiT used 5 and returned 18,967.77, which is
the separate figure. The case supplies 3 and expects a figure at the 2024-25 rate, both
wrong on purpose, to test that a term difference is reported as a convention
difference and not as agreement. This repository's engine takes the remaining term
from its operator, so it used the supplied 3 at the 2025-26 rate, for 29,297.01.
The difference of 10,329.24 comes from the term the case supplied.

## Fields one side returns and the other does not

Not compared, and recorded so nothing reads as agreement.

| Field | Provision | Position |
| --- | --- | --- |
| `deemed_dividend`, LodgeiT | ITAA 1936 s 109E(2): the shortfall, subject to s 109Y | LodgeiT returned the shortfall on each case, 4,450.97 on D7A-2 and 857.98 on D7A-5. This repository's result calls the same figure an experimental exposure. Neither trial works out a distributable surplus under s 109Y |
| `is_complying`, LodgeiT | ss 109E and 109N | The provider's label for its own arithmetic. Recorded, never mapped onto a local verdict |
| `interest_accrued`, LodgeiT | s 109E(7) | This repository's engine has no such figure |
| The s 109N gate, this repository | s 109N(1) | Supplied to the trial as an asserted fact, because the provider's route takes no such input |

The responses also carry `rounding_policy`, and the FBT route
`taxable_value_before_statutory` and `verification_required`, which the snapshot of
18 September 2026 does not record. The contract check noted them and accepted the
responses. `verification_required` was null on every case.

## A defect on this repository's side, fixed before this run

On 18 September 2026, 6 Division 7A cases matched, and they matched at the wrong
remaining term. This repository's case file took 1 off the term in the year after
the loan year, giving 6 for a 7-year loan. Section 109E(6) gives 7: the years
between the end of the loan year and the end of the year before are 0. The figure
in the case file was 21,874.92 and the statutory figure is 19,450.97.

LodgeiT's note states: 'CORRECTED 2026-09-28 (D57): before this date
remaining_term_years was one year short for every MYR year'
([`llms.txt`](https://lodgeit.org/llms.txt), read 6 November 2026). The contract
snapshot records the route returning 21,874.92 for a fabricated loan on
18 September 2026, which is consistent with that note. This run did not observe the
provider's behaviour before 28 September.

The case file is corrected, and a test now derives each remaining term from the
dates. The local engine's formula was not affected: it takes the remaining term
from its operator.

## Not covered

- Other gateway calculators, including the FBT car operating cost route. It is not
  in this trial.
- 2026-27 and FBT year 2027: the gateway listed neither on the day.
- Whether either side is right on facts these cases do not exercise.
