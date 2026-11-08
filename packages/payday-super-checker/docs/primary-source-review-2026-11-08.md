# Primary-source review: receipt evidence and the first-year note, 8 November 2026

This note records the sources for the receipt evidence fields (`receipt_evidence`,
`mvr_status`, `fund_error_ref`) and the first-year note on an `AT_RISK` row, added
on 8 November 2026. It adds to [the 31 October review](primary-source-review-2026-10-31.md).
Every source was read on 8 November 2026. The output remains an experimental review
aid, not an assessment or compliance determination.

## Sources and what each established

| Source | What it established | Runtime consequence |
| --- | --- | --- |
| [ATO, Payday Super and SuperStream for super funds](https://www.ato.gov.au/businesses-and-organisations/super-for-employers/managing-superstream/payday-super-and-superstream-for-super-funds), updated 10 August 2026, 'Member verification requests' | From 1 July 2026 employers must submit a member verification request before a first-time SG contribution to a fund, 'provided their software supports this functionality'. The response confirms whether the fund 'can match the employee information to an active account and accept a contribution' or 'can't find the member or can't accept the contribution' | `mvr_status` takes `matched`, `not_matched` or `not_sent`. `not_matched` adds a caveat and keeps the first-year note off the row. Whether a request was needed is not tested |
| Same, 'Rejecting contributions' | A fund that rejects a contribution 'must send a contribution transaction error response (CTER) message explaining why' | `fund_error_ref` holds that message's reference and adds a caveat |
| [ATO, Getting it right: compliance in the first year of Payday Super](https://www.ato.gov.au/businesses-and-organisations/super-for-employers/missed-or-late-payday-super-payments/getting-it-right-compliance-in-the-first-year-of-payday-super), published 23 June 2026, 'Common scenarios and what to do' | For 'I pay super guarantee on payday but I'm not sure it is received on time': 'Check that no errors have been received for this contribution. If no errors have been received, you can assume the amount was received by the fund on time.' The first year is 1 July 2026 to 30 June 2027 | The note on an `AT_RISK` row whose `remitted_date` is the QE day, with `fund_error_ref=none`. The row stays `AT_RISK`: the page sets a compliance approach and states no rule of receipt. Its scenarios treat paying as the employer's payment and expect clearing-house time within the deadline ('I pay super guarantee on day 7, and don’t take into consideration clearing house processing timeframes') |
| [PCG 2026/1](https://www.ato.gov.au/law/view/print?DocID=COG/PCG20261/NAT/ATO/00001&PiT=99991231235958), 28 January 2026, paragraphs 3, 11 and 13 | The guideline does not alter the law (paragraph 3). With definitive information of a shortfall the Commissioner must apply the law, even in the low-risk zone (paragraph 11). It applies only to QE days from 1 July 2026 to 30 June 2027 (paragraph 13) | The note is limited to QE days to 30 June 2027 and says the ATO still applies the law |
| [LCR 2026/2](https://www.ato.gov.au/law/view/document?DocID=COG%2FLCR20262%2FNAT%2FATO%2F00001), paragraphs 24 and 25, as read on 31 October 2026 | A rejected attempt is not an eligible contribution | The caveat for a recorded fund error says so and points to `returned_by_fund` and `--events` |
| CCH Superannuation Guarantee Scheme ¶12-002, local library, reviewed 30 June 2026, currency not verified | The starting point for the search | None |

The ATO's legal database refused the document view of PCG 2026/1 to an automated
fetch; its print view was read instead.

## Rules implemented

1. The 3 fields annotate a line. No verdict, figure, outer outcome or exit code
   depends on them.
2. A row is refused when its evidence contradicts it: `fund` or `clearing_house`
   with no fund receipt recorded, `sent` with one, or `sent` with no remittance date.
   A fund receipt is recorded by `fund_received_date` or by any event naming one: a
   `received` event, an `allocated` event (its `receipt_date` is required) or a
   `returned` event with a `receipt_date`. `mvr_status=not_matched` is never a
   contradiction, even on an `ON_TIME` row: it adds a caveat of its own, distinct from
   a fund error, and neither caveat says an amount came back.
3. The first-year note needs: verdict `AT_RISK`, QE day on or before 30 June 2027,
   `remitted_date` on the QE day itself and `fund_error_ref=none`. `remitted_date` is
   'the day you sent the money' (README, Input columns). The importers write a vendor
   date there only once the money has left the employer (README, 'The supported export
   profiles do not establish a fund receipt date'; `docs/design.md`, `importers.rs`), and a clearing house's
   onward date (Employment Hero's Beam) can fall after payday, so the equality reads the
   ATO's 'pay on payday' and errs towards withholding the note. `receipt_evidence` is
   not a condition: the ATO scenario turns on paying on payday and finding no error,
   not on a record of sending. The checker also leaves the note off a row with a
   `not_matched` verification, a return or a `fund_received_date`; these are its
   conservative additions, not ATO conditions. A receipt dated after the as-at date
   leaves a row `AT_RISK`, and the row's own record may show it late, which is outside
   the ATO's 'not sure it is received on time'.
4. The report keeps its 18-column header and the evidence pack its 4 files. Without
   evidence the pack is byte for byte the same; with it the notes
   and caveats travel in the pack's `report.csv` and `exceptions.json`.
   `practitioner-review.md` repeats caveats, not notes.
