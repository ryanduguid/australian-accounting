# Excel workbook

`div7a-loan-review.xlsx` is the same review as the command line tool, written in
ordinary worksheet formulas for accountants who work in Excel and do not have
Python. It is macro-free and needs desktop Excel for Microsoft 365 or Excel 2024.
Nothing leaves the workbook.

**Experimental review aid. Not a Division 7A determination.** Everything the
engine refuses, the workbook refuses too, and every caveat the engine attaches is
printed on Review Checks.

## Using it

1. **Register.** Paste one amalgamated loan per row over the example rows. The
   fourteen input columns are the CSV columns the command line tool reads, in the
   same order and with the same meanings (see the package README). Booleans are
   `true`, `false` or `unknown`; a blank is unknown. Rates are fractions, so
   8.77 per cent is `0.0877`. Years of income are written `2026-27` in Text cells,
   so Excel does not turn them into dates. Rows pasted below the example pick up the
   calculated columns on their own.
2. **Summary.** Choose the year of income. The s 109N gate reads each loan's
   benchmark rate from the year the loan was made (or `year_of_income_being_tested`
   where you nominate one); the s 109E repayment uses the chosen year's rate.
3. **Register, calculated columns.** Each s 109N(1) limb as PASS, FAIL or UNKNOWN;
   the gate verdict; the maximum term allowed under s 109N(3); the remaining term
   rounded up; the s 109E(6) minimum yearly repayment; the shortfall; the
   experimental exposure; and a reason wherever the answer is REFUSED or UNKNOWN.
4. **Review Checks.** `BLOCKED` means the register cannot be read: a bad year
   label, a value that is not true, false or unknown, text where a number belongs, a
   formula pasted into an input, or no reviewed rate for the year. `REVIEW` means a
   loan is not on s 109N terms, a repayment is short, or a row is undecided. `PASS`
   means nothing exposed and nothing undecided, which is the command line tool's
   exit code 0.
5. **Rates.** The frozen table of May RBA F5 figures. To review a year outside it,
   add a row and cite where you read the figure. The engine refuses an override
   without a citation; the workbook cannot enforce that, so the citation column is
   your record.

The rules are the engine's: a loan that fails any limb is NOT_COMPLYING and gets no
repayment figure; an unestablished limb is UNKNOWN and never coerced to a verdict;
a term of 7 years or less passes s 109N(1)(c) even when the security facts are
unknown; the year the loan was made, a year before it, a nil remaining term and a
nil rate are REFUSED; a missing balance, payments, remaining term or rate is
UNKNOWN. A loan year before 1998-99 or any `out_of_scope_reason` skips the row.

## How it is kept honest

The workbook is a second implementation of the rules in `div7aloan`, so it can
drift. `tests/test_workbook.py` reads the cached values desktop Excel wrote and
holds every row of the mixed sample, the summary counts and the Rates sheet to the
engine. A change to the engine, the rate table or the workbook that is not rebuilt
fails the test.

Rebuild on a machine with desktop Excel, from the package directory:

```powershell
uv run --locked --extra dev python tools/build_workbook.py
```

The build writes the formulas with openpyxl, opens the file once in Excel over COM
to calculate and save real cached values, records the Excel build on Sources &
Version and strips the machine path Excel stamps into the file. Pass `--no-excel`
to skip the Excel pass; the file then ships without cached values and the parity
test fails, which is the point.

The s 109E(6) amount is worked in binary floating point and rounded with Excel's
ROUND, where the engine works in 60-digit decimal and rounds half up. Both were
fuzzed against each other, including amounts landing on a half cent, before the
workbook shipped; rerun that check after any change to the formula.
