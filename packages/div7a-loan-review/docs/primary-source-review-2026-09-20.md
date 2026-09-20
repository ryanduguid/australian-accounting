# Primary-source review, 20 September 2026: the benchmark rate table

The earlier review, [primary-source-review-2026-08-31.md](primary-source-review-2026-08-31.md),
read the compiled Act for the Division 7A sections this engine applies, and
cross-checked the frozen benchmark rates against the published explainer at
<https://duguid.com.au/rates/div7a-benchmark-rate/>. That explainer is a
secondary source. Every row of `div7aloan/data/benchmark_rates.csv` cited RBA
statistical table F5 series FILRHLBVS, but nothing in the repository recorded a
read of the RBA publication itself.

This review closes that gap. It changes no rate.

## What was fetched

The RBA lists its statistical tables at <https://www.rba.gov.au/statistics/tables/>.
The F5 entry, "Indicator Lending Rates - F5", links the historical workbook:

- URL: <https://www.rba.gov.au/statistics/tables/xls/f05hist.xlsx>
- Retrieved: 20 September 2026
- Size: 862,341 bytes
- SHA-256: `40da2e5b7b74ca5d96a32688a6aea133c7e722c32701f13d25a32fef77e077b0`

The workbook's `Data` sheet carries series FILRHLBVS in column D, described as
"Lending rates; Housing loans; Banks; Variable; Standard; Owner-occupier",
monthly, original, per cent per annum, source RBA, publication date
7 September 2026. That is the series and description s 109N(2) points at.

The file was parsed with `openpyxl` in a scratch directory outside the
checkout. Nothing in the package reads the network, in tests or at runtime, and
the workbook is not vendored into the repository; the digest above is what ties
the frozen table to the bytes that were read.

## What was observed

The May figure is the one s 109N(2) selects, because the RBA publishes F5 in
arrears in the first week of the following month, so May is the last figure
published before a year of income starts on 1 July.

| Year of income | F5 FILRHLBVS observed | Frozen table | Agrees |
| --- | --- | --- | --- |
| 2019-20 | May 2019, 5.37 | 0.0537 | yes |
| 2020-21 | May 2020, 4.52 | 0.0452 | yes |
| 2021-22 | May 2021, 4.52 | 0.0452 | yes |
| 2022-23 | May 2022, 4.77 | 0.0477 | yes |
| 2023-24 | May 2023, 8.27 | 0.0827 | yes |
| 2024-25 | May 2024, 8.77 | 0.0877 | yes |
| 2025-26 | May 2025, 8.37 | 0.0837 | yes |
| 2026-27 | May 2026, 8.77 | 0.0877 | yes |

All 8 reviewed years match the primary source. No discrepancy was found, so no
rate was touched.

The May versus June trap holds in the observed data as the table describes it:
the workbook gives 8.37 for May 2025, and 2025-26 takes 8.37.

## What changed in the table

Three columns were added to `div7aloan/data/benchmark_rates.csv`, and the
existing `verify_at` column was left in place:

- `primary_url`, the RBA workbook URL above.
- `retrieved_on`, `2026-09-20`.
- `snapshot_sha256`, the digest above.

`verify_at` is now plainly a convenience link for a human re-checking a figure,
not the source of it. The loader reads all 3 columns into each entry, so they
reach the `provenance` block of every rate result, and, where every row makes
the same claim, the `rate_table_uris` manifest entry as well. A table whose
rows disagreed about their primary source would name none at the table level.

## What this review does not establish

It reviews the rate table only. It re-reads no section of the Act, revisits no
reviewed position, and does not extend `reviewed_until`, which stays 2026-27.
The 2026-08-31 review remains the source trail for s 109D, s 109E, s 109N,
s 109P, s 109R and s 109ZD, and for what this engine deliberately does not
establish.

The RBA attaches its own caveat to table F5: the rates are indicative, are
likely to be regularly revised, and the RBA does not comment on whether they
suit any particular purpose. A revision to a past May figure would move the
benchmark rate for that year of income, which is why the digest and the
retrieval date are recorded beside the figures rather than in a commit message.

This review is not tax, legal or financial advice, and is not an endorsement of
this software or this review by CA ANZ.
