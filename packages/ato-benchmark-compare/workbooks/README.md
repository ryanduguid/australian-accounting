# Excel workbook

`ato-benchmark-compare.xlsx` is the same comparison as the command line tool, written
in ordinary worksheet formulas for accountants who work in Excel and do not have
Python. It is macro-free and needs desktop Excel for Microsoft 365 or Excel 2024.
Nothing leaves the workbook.

## Using it

1. **P&L Import.** Paste account names and amounts over the example rows. The
   Account column is formatted as Text, so anything pasted there stays inert, and a
   formula or a non-numeric amount trips the Guard column and blocks the result.
2. **Mapping.** Give every account a bucket from the drop-down and set Source to
   `reviewed`. The buckets are the same eleven the engine uses, and a `suggested` or
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
   a finding.

Every rule is visible on the Calculation sheet: turnover falls back to total business
income when sales are not positive or are less than half of it, payments to
associated persons are deducted once, cost of sales excludes wages, W1 replaces the
salary and wages label only when it is larger, ratios are rounded to four places
before comparison, and band and benchmark boundaries are inclusive.

## How it is kept honest

The workbook is a second implementation of the rules in `atobenchmark`, so it can
drift. `tests/test_workbook.py` reads the cached values desktop Excel wrote and holds
them to the engine's answer for the bakery example, and holds the Benchmarks sheet to
the shipped JSON. A change to the engine, the data or the workbook that is not
rebuilt fails the test.

Rebuild on a machine with desktop Excel, from the package directory:

```powershell
uv run --locked --extra dev python tools/build_workbook.py
```

The build writes the formulas with openpyxl, opens the file once in Excel over COM to
calculate and save real cached values, records the Excel build on Sources & Version
and strips the machine path Excel stamps into the file. Pass `--no-excel` to skip the
Excel pass; the file then opens fine in Excel but ships without cached values and the
parity test fails, which is the point.

Account identity in the workbook is the trimmed, lower-cased account name rather than
the engine's SHA-256 account key. Both resolve the same account to the same mapping
row.
