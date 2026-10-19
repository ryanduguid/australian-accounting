# v0.1.0

Not yet released.

- `review_coding` gives each coded bank line one review state with a reason:
  `unknown_gst_code`, `gst_differs_from_registration`, `between_entities`, `coded`,
  `suggested` or `nothing_suggested`. Registration conflicts cite GST Act ss 9-5 and 11-5.
- `reconcile_bas` compares lodged G1, G2, G3, G10, G11, 1A and 1B figures with the
  ledger for each period on the cash or accruals basis (GST Act ss 29-5 and 29-10),
  rounding ledger totals down to whole dollars and listing the lines behind each
  difference, timing differences, and lines it cannot place.
