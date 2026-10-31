# Primary-source review: returned contributions, 31 October 2026

This note records the sources for the returned-contribution rules added on
31 October 2026. It adds to [the 15 August review](primary-source-review-2026-08-15.md)
and does not change its deadline, rate or calendar findings. Every source was read
on 31 October 2026. The output remains an experimental review aid, not an
assessment or compliance determination.

## Sources and what each established

| Source | What it established | Runtime consequence |
| --- | --- | --- |
| [SGAA 1992, compilation 78, C2026C00272](https://www.legislation.gov.au/C2026C00272), in force from 1 July 2026, s 18A(1)(a)(ii) | An eligible contribution must be 'able to be allocated within the fund for the benefit of the employee' | A receipt counts only for what the fund kept |
| Same, s 18B | Limits the complying-fund presumption and the benefit certificate statement. It says nothing about returned money | No runtime change |
| Same, s 18C(1) | Contributions are applied in the order received, are received in the usual period, the prior 12 months or a longer period, and cannot take the shortfall below nil | Receipts are walked in date order and capped at the SG amount |
| Same, s 18C(2) items 1 and 4 | Item 1 covers 'the first eligible contribution' to a fund, by the end of the extended usual period. Item 4 turns on an earlier eligible contribution applied to an earlier QE day. Neither restarts on a rejection | The 20 business days run from the QE day. A returned amount confirms no item 4 extension |
| Same, s 6(1) | 'Usual period', 'extended usual period', 'late period' and 'business day' | The late period starts after the latest day an eligible contribution could be applied |
| Same, ss 18D and 19A | Late contributions reduce the final shortfall. Notional earnings accrue on the base shortfall for each late-period day the final shortfall is above nil | Figures for `LATE` rows |
| [SIS Regulations 1994, compilation 159, F2026C00541](https://www.legislation.gov.au/F2026C00541), rectification versions 0 and 1 (identical text), reg 7.07G; made by [F2026L00133](https://www.legislation.gov.au/F2026L00133) item 40 | A trustee of a fund other than an SMSF must allocate or refund within 3 business days of receipt. Reg 7.07G(4): a contribution refunded under subreg (3) 'is taken not to have been made' | A return inside those days is not credit and not a 'first' contribution. A later return is undecided |
| Same, reg 7.07H | An SMSF has 28 days after the end of the month to allocate. There is no refund subregulation | The input does not name the fund type, so the late-return test uses the same 3 business days. It only ever moves a line to `UNKNOWN`. LCR 2026/2 paragraph 24 is relied on for an SMSF return inside them |
| [LCR 2026/2](https://www.ato.gov.au/law/view/document?DocID=COG%2FLCR20262%2FNAT%2FATO%2F00001), 5 August 2026, paragraphs 18 to 25 | Paragraph 19: the employer 'may not have visibility' of when the amount becomes allocable. Paragraph 20: the receipt date and the 'able to be allocated' date can differ. Paragraphs 22 and 23: a receipt with the requisite information can meet the test as soon as it is received. Paragraphs 24 and 25: a rejected attempt, whether unallocable or rejected by mistake, 'is not an eligible contribution' | A return shown to be a rejection is counted whatever the as-at date. A receipt does not show the amount is able to be allocated, so on a line with a return only an `allocated` event decides it. Paragraphs 22 and 23 mean allocation can come after the amount is able to be allocated, and paragraph 20 that it can become so after receipt, so an allocation after the deadline leaves a timely receipt undecided |
| Same, paragraphs 32 and 33 and Example 2 | Contributions apply automatically to the earliest QE day with a shortfall | Recorded as a limit: the checker does not move money between rows |
| Same, paragraph 54 | 'Only one contribution (the first) has the allowable longer period.' If it does not clear the shortfall, others are on time only in the usual period or another longer period | Only the first kept receipt has the 20 business days |
| [ATO, Getting it right: compliance in the first year of Payday Super](https://www.ato.gov.au/businesses-and-organisations/super-for-employers/missed-or-late-payday-super-payments/getting-it-right-compliance-in-the-first-year-of-payday-super), published 23 June 2026 | 'This timeframe does not change if there is an error or delay processing the contribution.' Also: with no errors received, 'you can assume the amount was received by the fund on time' | The deadline never moves. The assumption is not implemented on a line with a return, so its first-year scope does not arise there |
| [PCG 2026/1](https://www.ato.gov.au/law/view/document?DocID=COG%2FPCG20261%2FNAT%2FATO%2F00001), 28 January 2026, paragraphs 3, 17, 23, 26 and 27 | A compliance-resource posture for QE days from 1 July 2026 to 30 June 2027 that does not alter the law. A re-made contribution received late can be low risk if corrected as soon as reasonably practicable | A caveat on late notice, with no change to a figure |
| [ATO, About Payday Super](https://www.ato.gov.au/businesses-and-organisations/super-for-employers/about-payday-super), updated 10 August 2026 | Funds have 3 business days to allocate or return, not counting the day of receipt | Consistent with reg 7.07G |
| [ATO, Payday Super and SuperStream for super funds](https://www.ato.gov.au/businesses-and-organisations/super-for-employers/managing-superstream/payday-super-and-superstream-for-super-funds), updated 10 August 2026 | A rejecting fund must send a CTER explaining why. For the fund's 3 business days, a business day excludes a public holiday 'in the fund's location', and the employer's timeframe 'uses a different definition of business day' | The CTER is the natural source for `notified_date`. The fund's period is not used to reach `ON_TIME`, because the checker cannot compute it for a fund's location |
| [MicrOpay Beam batch statuses](https://help-micropay.theaccessgroup.com/en/articles/14297383-beam-super-clearing-batch-status-in-beam-super-clearing) (vendor, not law) | 'Sent to Fund with Refund': one or more contributions were rejected and refunded. 'Refunded': Beam refunded the entire batch | 'with refund' writes `returned_by_fund=yes`. 'Refunded' stays not sent |
| CCH Superannuation Guarantee Scheme ¶12-260, local library, reviewed 30 June 2026, currency not verified | The starting point for the search. It predates the final ruling, so no rule rests on it | None |

The Federal Register API was read for the texts; pages under
`www.legislation.gov.au` are cited as the public location. The yourpayroll Super
Contributions Report returned 403 and was not reread. Its status list is
recorded in the 26 October sweep and the profile did not need it.

## Rules implemented

1. A return inside the fund's 3 business days is not credit, whatever the as-at
   date.
2. The deadline does not restart. Late notice changes no figure.
3. A line with a return is decided only when the kept receipts cover the SG amount,
   every return is inside its period and an `allocated` event from the fund's records
   covers each counted receipt by the as-at date. `ON_TIME` needs every kept amount
   received and allocated inside its window; `LATE` needs a dated eligible receipt
   after the deadline, and a late receipt counted against an assessment needs its
   allocation before that assessment, or the line is `UNKNOWN` with the s 18D figures
   withheld. An item 4 extension resting on such a line needs the same
   evidence by the earlier QE day's deadline. A line with no return event keeps its
   existing reading.
4. Otherwise the line is attention-driving `UNKNOWN` with no exposure, including
   after the deadline. This is a deliberate choice: the checker has no evidence of
   whether a later receipt exists, and an unresolved outcome is not called `LATE`.
   Where the events show a shortfall after the deadline, a caveat states the base
   shortfall that applies if they are complete and the fund allocated what it kept,
   citing ss 18C and 19A. It calculates no SG charge figure.
6. The 3 business day test remains only for a return after that period, where it
   errs towards `UNKNOWN`. It no longer holds or releases a receipt, because the
   national calendar and a single period cannot compute a Victorian fund's period
   across Melbourne Cup Day or an SMSF's under reg 7.07H. No `ON_TIME` rests on it.
5. A wholly returned first attempt was never made, so the resubmission can be the
   first eligible contribution and keep the 20 business days from the QE day.

## Left unresolved

These are documented, and the checker does not guess them:

- **A later contribution inside the 20 business days.** Where the fund kept part
  of the first contribution, a later receipt between the end of the usual period
  and the 20th business day is outside LCR 2026/2 paragraph 54's on-time reading,
  but the s 6(1) late period starts only after the 20th business day. Neither
  source says what it is. The row is `UNKNOWN` between `LATE` and `ON_TIME`.
- **Whether a partly kept amount is 'the first' contribution.** No source read
  addresses a partial return. Treating the kept part as first is an inference
  from reg 7.07G(4) and paragraph 54.
- **A return more than 3 business days after receipt.** Reg 7.07G(4) speaks of a
  refund 'under subregulation (3)', and LCR 2026/2 paragraph 24 of a rejected
  attempt. Neither says what a later refund of a possibly allocated amount is, so
  the line is `UNKNOWN`, its caveat says why, and the checker does not say the
  credit is gone or that it stands.
- **A payment that becomes allocable after receipt** (paragraph 20). No source read
  gives the date for a given payment. On a line with a return, an allocation inside
  the deadline settles it, since an amount allocated was able to be allocated by
  then. An allocation after the deadline leaves a timely receipt `UNKNOWN`. A `LATE`
  row is measured to the receipt and says so. A line with no return event tests the
  date the operator supplies.
- **The fund's business days.** The ATO counts holidays in the fund's location, and
  the input holds no fund location or fund type. The late-return test uses the
  national calendar and the same 3 days for all funds, so a genuine rejection can
  read `UNKNOWN`.
- **Retirement savings accounts.** The RSA Regulations were not read. LCR 2026/2
  says its paragraphs 18 to 25 apply equally to an RSA.
- **Reallocation between QE days.** Paragraph 33 can apply a later payday's money
  to an earlier returned QE day. The operator's row association is relied on.
- **The first-year 'no error received' assumption.** It remains an ATO compliance
  statement. On a line with a return it produces no `ON_TIME`.
- **Fund-confirmation fields in OZEDI or ClickSuper exports.** No sample schema
  was inspected, so the wording is 'no supported export', not 'no export'.
