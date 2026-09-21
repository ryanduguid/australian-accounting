# Release and workbook verification, 22 September 2026

Dates use Australia/Sydney (AEST, UTC+10). The ATO browser capture at
21 September 2026, 17:49 UTC was 22 September, 3:49 am locally. The package
provenance capture at 17:51 UTC was 3:51 am locally. These are completed checks,
not a future review date.

This review distinguishes published checker 0.1.6, published MCP 0.2.4 and the
development 0.1.7 source. It does not publish a release or renew the historical
15 August legal review or the public 0.1.3 evaluation.

## Artefacts and sources

- Source baseline: `19a5824af78cf281ccfc6b8f07f7e21b860ae1a7`.
- Published tag `payday-super-checker/v0.1.6`: `8c23e10ff87e4f30ee289e4809a1b9074791a0c2`.
- [PyPI checker metadata](https://pypi.org/pypi/payday-super-checker/json) and
  [GitHub release](https://github.com/ryanduguid/australian-accounting/releases/tag/payday-super-checker/v0.1.6)
  both identify 0.1.6 as published. No 0.1.7 publication was found.
- Checker wheel SHA-256: `5260ef43231eeee3ea7b5896e93cde9cb5b31b2b8ecea4baafe87ac502bcf377`.
- [MCP PyPI metadata](https://pypi.org/pypi/aus-accounting-mcp/json) identifies 0.2.4.
  Wheel SHA-256: `fa0b9130473590f46a453d7a92f6d568f611943a3501ee09e314d7bfa2f1a40c`. A clean installation
  resolves checker 0.1.6. All Python modules in both installed distributions
  matched the downloaded wheels byte for byte.
- Tagged workbook SHA-256: `8d19ef499457a5ae2202dbfc293cb40646ac996fdddf0b0323430b68a386a35a`.
- Corrected development workbook SHA-256: `412f15a8024c84339897ec46032550e1239756a4422b34e8dee9b63918538b67`.
- The [ATO GIC table](https://www.ato.gov.au/tax-rates-and-codes/general-interest-charge-rates)
  returned HTTP 200 in Camofox on 22 September 2026. July to September is 11.43%
  and October to December is 11.51%; the bundled table ends on 31 December 2026.
  Rates were not changed. The January boundary below is a synthetic future test.
- The authoritative local Library's *Superannuation Guarantee Scheme*, paragraphs
  12-110 and 12-260, distinguishes fund receipt, allocation and the amount that
  reduces a shortfall. Its 30 June review date is preserved; its draft-ruling
  status is historical and was not used to replace the later source review.

## Method and results

All records are fabricated. The base row uses employee `SYN001`, payday
6 August 2026 and SG amount $120, with no first-fund, out-of-cycle or defined-benefit
flags. The usual deadline is 17 August. Timely receipt is 17 August, late receipt
18 August, remittance 14 August, partial receipt $60 and full receipt $120.
The contradictory case remits on 18 August but records receipt on 17 August.
Missing amount means both `matched_amount` and `remitted_amount` are blank.

Each CLI case ran through the installed command's `paydaysuper.cli.main` entry
with `input.csv --as-at DATE -o report.csv`. Only the acknowledged-remittance case
adds `--confirm-remittance-only`; only the opt-in case adds `--allow-stale-gic`.
Input CSVs, their hashes, arguments, complete output, exit status and report CSVs
were retained with the delivery evidence. Published MCP was tested separately
through its `calc_payday_super_deadline` tool function, with the equivalent
`qe_day`, `sg_amount`, `as_at`, `received`, `remitted` and amount arguments.
The workspace run reports checker 0.1.7; the clean published run reports 0.1.6.
These calls test tool behaviour, not an AI host or transport integration.

| Case | As-at date | Published 0.1.6 | Development 0.1.7 |
| --- | --- | --- | --- |
| timely missing amount | 2026-08-20 | ON_TIME, exit 0 | UNKNOWN, exit 2 |
| partial receipt | 2026-08-20 | UNPAID, exit 2 | UNPAID, exit 2 |
| full receipt | 2026-08-20 | ON_TIME, exit 0 | ON_TIME, exit 0 |
| remittance only | 2026-08-20 | AT_RISK, exit 2 | AT_RISK, exit 2 |
| remittance acknowledged | 2026-08-20 | AT_RISK, exit 0 | AT_RISK, exit 0 |
| late missing amount | 2026-08-20 | LATE, exit 2 | LATE, exit 2 |
| late full receipt | 2026-08-20 | LATE, exit 2 | LATE, exit 2 |
| contradictory dates | 2026-08-20 | rejected, exit 1 | rejected, exit 1 |
| last gic day | 2026-12-31 | UNPAID, exit 2 | UNPAID, exit 2 |
| first uncovered gic day | 2027-01-01 | UNPAID, exit 2 | UNPAID, exit 2 |
| stale gic opt in | 2027-01-01 | UNPAID, exit 2 | UNPAID, exit 2 |

For the late date without an amount, 0.1.6 assumes the full $120 arrived,
reports nil shortfall and $0.04 notional earnings. Development retains a $120
shortfall and $0.11 notional earnings to 20 August as a maximum. The verdict
stays `LATE`; missing amount does not erase established lateness.

Disposable workbook copies were recalculated with Excel 16.0 build 20430.0,
using `CalculateFullRebuild`. Macros were disabled; there were no external links
or data connections. Unused sample table rows were deleted as the workbook
instructions require. Inputs were written to Register's 11 canonical columns;
Summary B2 selected the as-at date and B5 the remittance acknowledgement.
The source workbooks remained unchanged during the scenario runs.

The ten non-opt-in cases compared Register verdict, shortfall, notional earnings
and both estimates with the matching CLI. Contradictory dates were rejected by
the CLI and marked `BLOCKED` by Review Checks, even though a row verdict remains
visible. Read that overall status before using row results.

On 31 December, both workbooks and the CLI report $5.25 notional earnings and
a $125.25 low estimate for the unpaid $120 row. On 1 January 2027:

- The tagged 0.1.6 workbook shows $5.28 and $125.28. It extrapolates; the matching
  CLI default withholds both figures. CLI `--allow-stale-gic` reproduces those
  amounts, but the workbook has no corresponding opt-in.
- Development correctly withholds the row estimates. Before this change,
  Summary B18:B20 summed those blanks as zero. They now display `Not assessed`,
  while B17 retains the $120 shortfall.
- A mixed register with an additional fully received late row retains that
  row's $0.04 estimate but keeps the aggregate `Not assessed`. Returning the
  as-at date to 31 December restores the complete $125.29 low total.

The changed Summary was inspected in Excel and as a captured image. Its labels,
status and `Not assessed` values are visible without clipping. These cases do
not prove parity for every possible input or workbook option.

## Corrections and remaining limits

README, workbook guidance and `llms.txt` now separate the published release,
development source, quick trial and historical evaluation. Evidence packs are
available from 0.1.4; 0.1.5 carries exported join warnings. The frozen PyPI 0.1.6
description still calls evidence packs unreleased and links 0.1.4. A source edit
cannot replace that metadata; a separately authorised release is needed.

PSC-1 and PSC-2 now describe what their specific issue leaves unchanged. They
do not promise correctness despite missing receipts, unreliable matches or
uncovered rates. Historical source dates and evaluation outputs are preserved.

The Summary regression failed on the old workbook at B18, then passed after
the formula change and the existing builder's desktop Excel recalculation.
The package suite passed 711 tests with one skip. Ruff,
Mypy, dependency audit and the source distribution/wheel build passed locally.
Hosted checks, review and merge evidence are recorded on the accompanying PR.
No package, tag or release was published by this task.
