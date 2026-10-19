# v0.1.0

This is the first PyPI release of `gst-review`, published through
`release-gst-review.yml`, the monorepo's namespaced, attested release workflow.

- `review_coding` gives each coded bank line one review state with a reason:
  `unknown_gst_code`, `gst_differs_from_registration`, `between_entities`, `coded`,
  `suggested` or `nothing_suggested`. Registration conflicts cite GST Act ss 9-5 and 11-5.
- `reconcile_bas` compares lodged G1, G2, G3, G10, G11, 1A and 1B figures with the
  ledger for each period on the cash or accruals basis (GST Act ss 29-5 and 29-10),
  rounding ledger totals down to whole dollars and listing the lines behind each
  difference and timing differences. It refuses a period's line that it cannot place
  on a label rather than comparing totals without it.
- Both reviews do their arithmetic in a fixed Decimal context, so a caller's
  precision or rounding setting cannot change a total, a pairing or the amount limit.
