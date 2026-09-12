# Australian tax calculators

[![tests](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/australian-tax-calculators.svg?color=5C2D91&labelColor=04001F)](https://pypi.org/project/australian-tax-calculators/)
[![License: MIT](https://img.shields.io/badge/License-MIT-4F485E.svg?labelColor=04001F)](https://opensource.org/licenses/MIT)
[![Python 3.10+](https://img.shields.io/badge/python-3.10%2B-5C2D91.svg?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/downloads/)

Distribution `australian-tax-calculators`, import package `austaxcalc`. Library
only: this component ships no command.

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

## Install

Python 3.10 or later. No runtime dependencies.

```bash
pip install australian-tax-calculators
```

## Use

Every worksheet takes `Decimal` amounts, an explicit period, and a
`scope_confirmed` flag the caller sets only once it has established the scope
conditions the worksheet states. Splitting $1,100.00 GST-inclusive into its
exclusive amount and its GST:

```python
from decimal import Decimal

from austaxcalc.calculations import gst

result = gst(Decimal("1100.00"), gst_inclusive=True, scope_confirmed=True, year="2025-26")
print(result["amounts"], result["source_checked"])
```

```
{'gst': '100.00', 'exclusive': '1000.00', 'inclusive': '1100.00'} 2026-09-10
```

Amounts come back as quoted decimal strings rather than numbers, so an amount
that leaves a worksheet cannot arrive at a JSON reader as a float.

The full result also carries `scope`, the official `sources` the figures came
from, the engine version and the warnings that travel with every worksheet:

```python
>>> result["scope"]
'One ordinary taxable supply at 10%, already classified by the operator. Excludes mixed supplies, exemptions, margin schemes, adjustments, tax invoice rounding across line items, input-credit entitlement and BAS preparation.'
>>> result["warnings"]
['Operator-confirmed facts and scope; eligibility is not independently verified.', 'Worksheet only, not advice, an assessment or a lodgment. Obtain human review.']
```

Full boundary statement: [DISCLAIMER.md](DISCLAIMER.md).

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
