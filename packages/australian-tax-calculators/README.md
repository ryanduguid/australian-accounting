# Australian tax calculators

Six calculation worksheets for established facts: ordinary GST, resident basic income
tax, CGT losses and discount, ordinary employer FBT, first-year depreciation and
quarterly super guarantee. These are experimental review aids, not advice or return
preparation. Callers must establish every scope condition before calculating.

The Python functions in `austaxcalc.calculations` accept `Decimal` amounts and an
explicit period. Results include amounts, scope exclusions, official source URLs,
the source-check date and engine version. They perform no network calls or writes.
Amounts must be non-negative, finite, at most two decimal places and no more than
AUD 1 trillion. Rounding uses half-up to cents at output; individual worksheets
state additional conventions.

## Supported periods

| Worksheet | Period and boundary |
| --- | --- |
| GST | 2025-26, one ordinary taxable supply at 10%; no classification or BAS |
| Resident tax | 2024-25, 2025-26 and 2026-27; full-year resident ordinary income, before offsets and levies |
| CGT | 2025-26, resident individual's established gains and losses; losses before the 50% discount |
| FBT | Year ended 31 March 2026; established type 1 and type 2 taxable values, ordinary employer |
| Depreciation | 2025-26, first year of an ordinary tangible Division 40 asset; established life and use |
| SG | Complete quarters of 2025-26; established OTE, eligibility and qualifying contributions |

Unsupported periods fail. The SG worksheet does not implement post-June 2026
Payday entitlement rules. The separate Payday engine reviews timing on a supplied
liability. Contribution caps, SMSF tax, trusts, payroll tax, HELP and Medicare
calculations remain outside these worksheets.

Source URLs and exact exclusions are in `austaxcalc/calculations.py` and every result.
Sources were checked on 10 September 2026. A source-check date is not an assurance
that every tax rule or taxpayer circumstance has been reviewed.
