# Trust distribution checks

| Install distribution | Python import | Command |
| --- | --- | --- |
| `solomons-sword` | `louisgoldberg` | `solomons-sword` |

Compatibility: install `solomons-sword`, import `louisgoldberg`, and run `solomons-sword`. These remain the supported names; no migration is required.

[![Python](https://img.shields.io/badge/Python-3.10+-5C2D91?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/)
[![tests](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci-solomons-sword.yml/badge.svg)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci-solomons-sword.yml)
[![PyPI](https://img.shields.io/pypi/v/solomons-sword.svg?color=5C2D91&labelColor=04001F)](https://pypi.org/project/solomons-sword/)
[![License: MIT](https://img.shields.io/badge/License-MIT-4F485E.svg?labelColor=04001F)](https://opensource.org/licenses/MIT)
[![ITAA 1936](https://img.shields.io/badge/Legislation-ITAA%201936%20Division%206-5C2D91?labelColor=04001F)](https://www.legislation.gov.au/C1936A00027/latest/text)

Project name Solomon's Sword. Distribution `solomons-sword`, import package `louisgoldberg`, command `solomons-sword`.

**Trust distribution allocation, Section 100A / 99B risk evaluation, and Division 6 ITAA 1936 review helpers for Australian trusts.**

**Package lifecycle:** published. Install `solomons-sword` from PyPI.

The `australian-accounting` repository contains the maintained source. The
`solomons-sword` distribution and command match the project identity. The
`louisgoldberg` import package remains unchanged, so internal Python imports do
not need a rename.

Release: [`v0.1.3`](https://github.com/ryanduguid/australian-accounting/releases/tag/solomons-sword/v0.1.3).

Named for the judgement of Solomon, where the threat of dividing the child in proportion is what reveals who the true claimant is. Division 6 allocates trust income by proportionate entitlement following *Bamford*; Section 100A asks who actually ended up with the benefit. The name is a tribute only.


## Not advice

Nothing this engine produces is tax, legal or financial advice, an assessment or
a determination. Outputs are review aids: confirm every allocation, zone and
exemption against the current law, the trust deed and the entity's facts before
acting, and leave lodgment decisions with a registered practitioner. Where the
model does not implement a rule it refuses the input rather than returning a
number it cannot stand behind.

Command output carries the beneficiary name you supply, because a workpaper line
item is unusable without it. Treat that output as client data: write it to the
firm's approved secure location, never to a path inside a repository.

Full boundary statement: [DISCLAIMER.md](https://github.com/ryanduguid/australian-accounting/blob/main/packages/solomons-sword/DISCLAIMER.md).

---

## Core Features

- **Division 6 Proportionate Allocation (*Commissioner of Taxation v Bamford* [2010] HCA 10)**: Calculates present entitlement proportions and allocates *s 95(1) ITAA 1936* taxable net income and the franking credits that ride with it. Specifically streamed capital gains and franked dividends are **refused**, not allocated: the Division 6E carve-out with *Subdivision 115-C* (including s 115-220) and *Subdivision 207-B* is not implemented, so a proportionate answer would be wrong. Non-resident beneficiaries (*s 98(2A)/(3)*), nil income of the trust estate and no presently entitled beneficiary (*s 99 / s 99A*) are refused for the same reason.
- **Section 100A Reimbursement Agreement Matrix**: Classifies supplied facts against **ATO PCG 2022/2** as Green, Red, or outside those zones. The final guideline has white, green and red; the draft blue zone did not survive. White zone (income years ending before 1 July 2014) is out of scope because the function does not take an income year.
- **Section 99B Foreign Trust Receipt Assessment**: Computes assessable amounts under *s 99B(1)* after corpus exemptions (*s 99B(2)(a)*) and prior-taxed income.
- **Trust Resolution 30 June Schedule Verifier**: Checks timing, deed-power and percentage-completeness facts the caller supplies.

---

## Quickstart

### Installation
```bash
pip install solomons-sword
```

### CLI Usage
```bash
# Evaluate Section 100A risk zone
solomons-sword s100a-check --beneficiary "Adult Child" --amount 40000 --adult-child --retained-by-parents

# Assess Section 99B receipt from foreign trust with corpus deduction
solomons-sword s99b-check --beneficiary "Jane Doe" --gross 150000 --corpus 50000
```

---

## Statutory Ground Truth & Test Harness

All allocation and threshold algorithms use exact `decimal.Decimal` calculations to prevent rounding discrepancies in trust tax schedules.

| Statutory Domain | Primary Authority | What the code actually does |
| :--- | :--- | :--- |
| **Proportionate Entitlement** | *ITAA 1936* s 95, s 97 (*Commissioner of Taxation v Bamford* [2010] HCA 10) | `Beneficiary Share = (Accounting Entitlement / Total Accounting Income) * s95 Net Income`. |
| **Section 100A Risk Matrix** | *ITAA 1936* s 100A, *ATO PCG 2022/2* | Returns GREEN, RED or OUTSIDE_GREEN from the supplied flags. It does not decide the white zone. |
| **Foreign Trust Distributions** | *ITAA 1936* s 99B(1), s 99B(2)(a) | Subtracts settled corpus and previously taxed income prior to assessable inclusion. |
| **Trust Resolution Timing** | Caller-supplied deed and execution facts | Refuses incomplete percentages and missing deed facts. This is not a substitute for current ATO guidance. |

### Automated Test Suite
- Run the suite: `pytest tests/`
- The suite covers proportionate streaming, Section 100A zones the engine implements, Section 99B corpus deductions, and resolution gates. Do not treat a badge as a live coverage certificate.

---

## Licence
MIT License. Created by Ryan Duguid.
