# v0.1.2

- Reject padded required headers, duplicate override years and gates from another loan year.
- Withhold minimum-yearly-repayment figures when the loan year is missing and retain gate caveats in every result.
- Align the workbook formulas and cached values with the reviewed year checks.

## Correction to the v0.1.1 note

The v0.1.1 release note said that release preserved the v0.1.0 review
calculations and refusal boundaries. The calculations and the s 109N gate were
preserved, but the refusal boundaries were not: v0.1.1 added the two-decimal
place limit and the $1 trillion maximum in `div7aloan/money.py`, and it began
preserving quoted newlines when reading the register so a malformed numeric
cell is refused instead of being joined into a different amount. Version 0.1.0
accepts a principal of `100000.001`, a repayment of `25556.001`, a principal of
`1000000000000.01` and a repayment split across a quoted newline; v0.1.1
refuses each of them. Those safeguards stay in force in this release.
