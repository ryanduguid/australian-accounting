# Excel workbook

`ato-benchmark-compare.xlsx` is the same comparison as the command line tool, written
in ordinary worksheet formulas for accountants who work in Excel rather than at a
command line. It is macro-free and needs desktop Excel for Microsoft 365 or Excel 2024.
Nothing leaves the workbook.

## Using it

1. **P&L Import.** Paste account names and amounts over the example rows. The
   Account column is formatted as Text, so anything pasted there stays inert, and a
   formula or a non-numeric amount trips the Guard column and blocks the result.
   Delete any example rows you did not overwrite: Review Checks flags a fabricated
   bakery line that is still in the P&L, because it would otherwise count in the
   ratios.
2. **Mapping.** Give every account a bucket from the drop-down and set Source to
   `reviewed`. The buckets are the same 11 the engine uses, and a `suggested` or
   missing mapping blocks the result, exactly as the CLI refuses unreviewed mappings.
   An account name that appears twice in the P&L or the mapping blocks it too, as
   the CLI does. Rows pasted below the example pick up the Bucket and Guard formulas
   on their own, and Review Checks names an example offending account for each block.
3. **Calculation.** Choose the benchmark year and industry. Enter the activity
   statement W1 total if it is higher than the salary and wages label. Set B4 to `Y`
   when the export shows expenses as negatives (the `--flip-expense-signs` switch).
4. **Results and Review Checks.** Five ratios, the published range for each, and a
   status of `within`, `below`, `above`, `no benchmark in this dataset` or
   `no turnover band applies`. The overall status is `BLOCKED`, `REVIEW` or `PASS`.
   Blank or an error is never a pass. Sitting outside a range is a review prompt, not
   a finding. The Range source column says whether a range is the data.gov.au
   workbook's (`ato_dataset`) or comes from the ATO's page for the industry
   (`ato_industry_page`), as labour, rent and motor vehicle ranges do; only the key
   ratio can move the overall status.

Every rule is visible on the Calculation sheet: turnover falls back to total business
income when sales are not positive or are less than half of it, payments to
associated persons are deducted once, cost of sales excludes wages, W1 replaces the
salary and wages label only when it is larger, ratios are rounded to 4 places
before comparison, benchmark range boundaries are inclusive, and a band includes
its upper bound while only the first band includes its lower bound.

## How it is kept honest

The workbook is a second implementation of the engine's rules, so it can drift. The
repository's `tests/test_ato_workbook.py` reads the cached values desktop Excel wrote
and holds them to the engine's answer for the bakery example, frozen in
`tests/data/workbook-sample.json`, which `cargo test` holds to the engine, and holds
the Benchmarks sheet to the shipped JSON. A change to the engine, the data or the
workbook that is not rebuilt fails one of the two tests. The shipped workbook was
built against engine 0.2.2, whose results the Rust 1.0.0 engine reproduces.

Rebuild on a machine with desktop Excel and Rust, from the repository root:

```powershell
uv run python packages/ato-benchmark-compare/tools/build_workbook.py
```

The build reads the benchmark bands through the engine's `load-dataset` command,
building the binary with Cargo if needed.

The build writes the formulas with openpyxl, opens the file once in Excel over COM to
calculate and save real cached values, records the Excel build on Sources & Version
and strips the machine path Excel stamps into the file. Pass `--no-excel` to skip the
Excel pass; the file then opens fine in Excel but ships without cached values and the
parity test fails, which is the point.

Account identity in the workbook is the trimmed, lower-cased account name rather than
the engine's SHA-256 account key. Both resolve the same account to the same mapping
row. Duplicate checks treat `*`, `?` and `~` as literal account-name characters.
