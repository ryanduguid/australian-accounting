# Company tax and franking checks

| Install distribution | Python import | Command |
| --- | --- | --- |
| `the-exchequer-tally` | `edwinnixon` | `the-exchequer-tally` |

These published identities remain supported. The functional title describes the accounting task; no package rename or import migration is required.

[![Python](https://img.shields.io/badge/Python-3.10+-5C2D91?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/)
[![tests](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci-the-exchequer-tally.yml/badge.svg)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci-the-exchequer-tally.yml)
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

## Core Features

- **Base Rate Entity (BRE) Testing**: Deterministic assessment under *s 23AA & s 23AB Income Tax Rates Act 1986* (evaluating aggregated turnover thresholds and Base Rate Entity Passive Income ratios).
- **Franking Account Ledger (FAB)**: Complete balance management under *Part 3-6 ITAA 1997*, tracking PAYG instalments, company tax payments, dividends paid/received, and tax refunds.
- **Franking Deficit Tax (FDT) & Offset Penalty**: Evaluates FDT liability under *s 205-45* and calculates the 30% tax offset reduction penalty under *s 205-70(6)* where the deficit exceeds 10% of annual credits.
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

## Statutory Ground Truth & Test Harness

All mathematical operations execute via `decimal.Decimal` fixed-point arithmetic to guarantee zero floating-point drift across corporate tax and franking schedules.

| Statutory Domain | Primary Authority | Verification Invariant |
| :--- | :--- | :--- |
| **Base Rate Entity Status** | *Income Tax Rates Act 1986* s 23AA | BRE rate bounded by the year's aggregated-turnover threshold ($25M for FY2018, $50M from FY2019) and BREPI <= 80% compared exactly. |
| **Franking Credits & Debits** | *ITAA 1997* s 205-15, s 205-30 | Cent-exact ledger of credits and debits (dates recorded for the workpaper; balances are order-independent sums). |
| **FDT Offset Reduction** | *ITAA 1997* s 205-45, s 205-70(6) | Exact 30% offset reduction penalty applied when deficit exceeds 10% threshold. |
| **Benchmark Rule** | *ITAA 1997* ss 203-25 to 203-55 | Benchmark set by the first frankable distribution in the franking period (*s 203-30*), then one deterministic shortfall or over-franking result per later distribution. |
| **Distribution Statements** | *ITAA 1997* ss 202-75, 202-80 | Precise franking credit formula: `Distribution * (Rate / (1 - Rate)) * Franking%`. |

### Automated Test Suite
- Run the full suite: `uv run --locked --extra dev pytest` (or `pip install .[dev]` then `pytest`; the configured coverage add-on needs the dev extras)
- The suite covers BRE eligibility, FDT penalty triggers, benchmark-rule checks, and distribution-statement generation. Do not treat a static badge as live coverage.

---

## Licence
MIT License. Created by Ryan Duguid.
