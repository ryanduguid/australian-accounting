# Company tax and franking checks

| Install distribution | Python import | Command |
| --- | --- | --- |
| `the-exchequer-tally` | `edwinnixon` | `the-exchequer-tally` |

[![Python](https://img.shields.io/badge/Python-3.10+-5C2D91?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/)
[![tests](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/the-exchequer-tally.svg?color=5C2D91&labelColor=04001F)](https://pypi.org/project/the-exchequer-tally/)
[![License: MIT](https://img.shields.io/badge/License-MIT-4F485E.svg?labelColor=04001F)](https://opensource.org/licenses/MIT)
[![ITAA 1997](https://img.shields.io/badge/Legislation-ITAA%201997%20Part%203--6-5C2D91?labelColor=04001F)](https://www.legislation.gov.au/C2004A05138/latest/text)

Project name The Exchequer Tally. Distribution `the-exchequer-tally`, import package `edwinnixon`, command `the-exchequer-tally`.

**Corporate tax rate verification, franking account ledger tracking, and Division 203 benchmark rule compliance for Australian private and public companies.**

**Package lifecycle:** published. Install `the-exchequer-tally` from PyPI.

The `australian-accounting` repository contains the maintained source. The
`the-exchequer-tally` distribution and command match the project identity. The
`edwinnixon` import package remains unchanged, so internal Python imports do not
need a rename.

Release: [`v0.1.3`](https://github.com/ryanduguid/australian-accounting/releases/tag/the-exchequer-tally/v0.1.3).

---

## What it checks

- **Base Rate Entity (BRE) Testing**: Deterministic assessment under *s 23AA & s 23AB Income Tax Rates Act 1986* (evaluating aggregated turnover thresholds and Base Rate Entity Passive Income ratios).
- **Franking Account Ledger (FAB)**: Tracks PAYG instalments, company tax payments, dividends paid/received, ordinary income tax refunds, under-franking debits and FDT liability credits.
- **Franking Deficit Tax (FDT) & Offset Reduction**: Evaluates FDT liability under *s 205-45* and the offset reduction under *s 205-70(2) and (8)* for the supported debit types. Refund-only deficits receive no reduction.
- **Division 203 Benchmark Rule Engine**: Detects over-franking tax (*s 203-50(1)*) and franking debit shortfalls (*s 203-50(2)*) across distributions in a franking period.
- **Dividend Distribution Statements**: Builds the statement fields required by *s 202-75 / s 202-80*. This is a review helper, not a lodgment and not a compliance certificate.

## Not advice

Nothing this engine produces is tax, legal or financial advice, an assessment
or a determination. Outputs are review aids: confirm every rate, threshold and
consequence against the current law and the entity's facts before acting, and
leave lodgment decisions with a registered practitioner.

Full boundary statement: [DISCLAIMER.md](DISCLAIMER.md).

---

## Quickstart

### Installation
```bash
pip install the-exchequer-tally
```

### CLI Usage
```bash
# Evaluate Base Rate Entity (BRE) status for FY2025
the-exchequer-tally bre-test --fy 2025 --turnover 4500000 --assessable 800000 --passive 120000

# Generate a dividend distribution statement
the-exchequer-tally dist-statement --entity "Acme Pty Ltd" --acn "123456789" --recipient "Jane Doe" --amount 15000 --franking-pct 100 --tax-rate 0.25
```

---

## Statutory sources and tests

### Worked example: two income years, two rates

These fabricated figures are independently prepared inputs for an ordinary
company. Aggregated turnover is a separate tax measure; do not substitute total
assessable income for it.

| Input | FY2026 | FY2027 |
|---|---:|---:|
| Aggregated turnover | $2,000,000 | $3,000,000 |
| Assessable income | $1,000,000 | $1,000,000 |
| Base rate entity passive income | $900,000 | $200,000 |
| Passive share | 90% | 20% |

```python
from decimal import Decimal
from edwinnixon.corporate_tax import (
    BaseRateEntityTest,
    determine_corporate_tax_rate,
    determine_max_franking_rate,
)

prior = BaseRateEntityTest(2026, Decimal("2000000"), Decimal("1000000"), Decimal("900000"))
current = BaseRateEntityTest(2027, Decimal("3000000"), Decimal("1000000"), Decimal("200000"))
assert determine_corporate_tax_rate(current).applicable_rate == Decimal("0.25")
assert determine_max_franking_rate(2027, prior) == Decimal("0.30")
```

The company tax calculation uses FY2027 facts. The imputation calculation uses
FY2026 amounts with the FY2027 rate scale. Passing `current` as the prior-year
evidence now raises `ValueError`, as does using an older or future year.

The Library's *Tax Examples / Companies and distributions*, paragraph 10-000,
prompted this case. Its text and example data are not reproduced. Authority:
*Income Tax Rates Act 1986* ss 23 and 23AA, and *Income Tax Assessment Act 1997*
s 995-1, definition of corporate tax rate for imputation purposes. The
[Rates Act](https://www.legislation.gov.au/C2004A03348/2026-07-01/text) and
[ITAA dictionary](https://www.legislation.gov.au/C2004A05138/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_10/document_10.html)
compilations dated 1 July 2026 were checked on 10 September 2026 against this
FY2027 scenario and the existing rate table. Verify the applicable version
for any actual distribution.

Newly formed entities without a prior income year need separate review; this
helper still requires prior-year evidence and does not implement that statutory
exception. This test proves arithmetic and period validation, not the supplied
income classifications or a dividend's approval.

`BaseRateEntityTest` rejects passive income greater than assessable income,
including a positive passive amount against nil assessable income. Both amounts
may be zero, and passive income may equal assessable income.

### Refund classification and the FDT offset

`record_tax_refund` now requires the keyword `includes_r_and_d_offset`.
Existing callers must confirm the refund's classification and pass `False`
for an ordinary income tax refund. `True` or an unknown classification raises
an error before posting. Descriptions do not determine tax treatment.

This fabricated FY2027 example has only an ordinary refund debit:

```python
from datetime import date
from decimal import Decimal
from edwinnixon.franking_account import FrankingAccount

account = FrankingAccount(2027)
account.record_payg_instalment(date(2026, 9, 1), Decimal("1000"))
account.record_tax_refund(
    date(2027, 3, 1), Decimal("3000"), includes_r_and_d_offset=False,
)
result = account.evaluate_franking_deficit()
assert result.franking_deficit_tax == Decimal("2000")
assert result.allowable_tax_offset == Decimal("2000")
assert result.fdt_offset_reduction_applies is False
```

For supported debits, an item 1 distribution or item 3 under-franking debit
also brings item 2 refunds into the reduction calculation. The threshold uses
credits arising during the year, excluding the opening credit balance.

The evaluator assumes residency and no first-year exception or Commissioner's
discretion. Other debit types, late-balancing rules and prior-year offset
carry-forwards need separate review. The account accepts entries from 1 July
to 30 June of its financial year, inclusive. Recording and construction reject
out-of-year dates; balance calculations also reject entries inserted directly
into the public list. Bring earlier periods forward through the opening balance.

R&D refunds need separate review in full, including mixed refunds. The R&D
portion does not create an immediate debit under s 205-30(2); s 205-15(4)
can reduce later payment credits. This ledger does not track that history.
Do not use it for an account with outstanding deferred R&D debits, or bypass
the refund guard by constructing entries directly.

The Library's *Tax Examples / Companies and distributions* prompted these
cases. No source text or example figures are reproduced. Checked against
[ITAA 1997 ss 205-15, 205-30 and 205-70](https://www.legislation.gov.au/C2004A05138/2026-07-01/2026-07-01/text/original/epub/OEBPS/document_5/document_5.html)
on 10 September 2026.

All mathematical operations execute via `decimal.Decimal` fixed-point arithmetic to guarantee zero floating-point drift across corporate tax and franking schedules.

| Statutory Domain | Primary Authority | Verification Invariant |
| :--- | :--- | :--- |
| **Base Rate Entity Status** | *Income Tax Rates Act 1986* s 23AA | BRE rate bounded by the year's aggregated-turnover threshold ($25M for FY2018, $50M from FY2019) and BREPI <= 80% compared exactly. |
| **Franking Credits & Debits** | *ITAA 1997* s 205-15, s 205-30 | Cent-exact, order-independent sums of entries within the account's July-to-June financial year. |
| **FDT Offset Reduction** | *ITAA 1997* s 205-45, s 205-70(2) and (8) | Supported debit types determine whether the 30% reduction applies; see the scope above. |
| **Benchmark Rule** | *ITAA 1997* ss 203-25 to 203-55 | Benchmark set by the first frankable distribution in the franking period (*s 203-30*), then one deterministic shortfall or over-franking result per later distribution. |
| **Distribution Statements** | *ITAA 1997* ss 202-75, 202-80 | Precise franking credit formula: `Distribution * (Rate / (1 - Rate)) * Franking%`. |

### Automated Test Suite
- Run the full suite: `uv run --locked --extra dev pytest` (or `pip install .[dev]` then `pytest`; the configured coverage add-on needs the dev extras)
- The suite covers BRE eligibility, FDT penalty triggers, benchmark-rule checks, and distribution-statement generation. Do not treat a static badge as live coverage.

---

## Licence
MIT License. Created by Ryan Duguid.
