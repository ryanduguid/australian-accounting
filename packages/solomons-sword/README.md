# Trust distribution checks

Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited to software issues reproduced with fabricated data. Do not send taxpayer information or request advice, return preparation, tax-treatment confirmation, or lodgement.

| Install distribution | Python import | Command |
| --- | --- | --- |
| `solomons-sword` | `louisgoldberg` | `solomons-sword` |

[![Python](https://img.shields.io/badge/Python-3.10+-5C2D91?logo=python&logoColor=white&labelColor=04001F)](https://www.python.org/)
[![tests](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml/badge.svg)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
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

In-repo version: `0.1.9`; see [release notes](RELEASE_NOTES.md).

Named for the judgement of Solomon, where the threat of dividing the child in proportion is what reveals who the true claimant is. Division 6 allocates trust income by proportionate entitlement following *Bamford*; Section 100A asks who actually ended up with the benefit. The name is a tribute only.


## Not advice

Nothing this engine produces is tax, legal or financial advice, an assessment or
a determination. Outputs are review aids: confirm every allocation, zone and
exemption against the current law, the trust deed and the entity's facts before
acting, and leave lodgement decisions with a registered practitioner. Where the
model does not implement a rule it refuses the input rather than returning a
number it cannot stand behind.

Command output carries the beneficiary name you supply, because a workpaper line
item is unusable without it. Treat that output as client data: write it to the
firm's approved secure location, never to a path inside a repository.

Full boundary statement: [DISCLAIMER.md](https://github.com/ryanduguid/australian-accounting/blob/main/packages/solomons-sword/DISCLAIMER.md).

---

## What it checks

- **Division 6 Proportionate Allocation (*Commissioner of Taxation v Bamford* [2010] HCA 10)**: Calculates present entitlement proportions and allocates *s 95(1) ITAA 1936* taxable net income and the franking credits that ride with it. Trust net capital gains and franked dividends are **refused**, not allocated, whether or not they are streamed: the Division 6E carve-out with *Subdivision 115-C* (including s 115-220) and *Subdivision 207-B* is not implemented, so a proportionate answer would be wrong. Non-resident beneficiaries (*s 98(2A)/(3)*), nil income of the trust estate and no presently entitled beneficiary (*s 99 / s 99A*) are refused for the same reason. Each beneficiary's residency is a required input and their legal disability is stated as `True` or `False`: those 2 facts select the refused paths, so an unstated one is refused rather than defaulted.
- **Section 100A Reimbursement Agreement Matrix**: Classifies supplied facts against **ATO PCG 2022/2** as Green, Red, or outside those zones. The final guideline has white, green and red; the draft blue zone did not survive. White zone (income years ending before 1 July 2014) is out of scope because the function does not take an income year. Every fact is `True`, `False` or unstated, and the green zone turns on all 11 of them: while any is unstated the result is `FACTS_NOT_ESTABLISHED`, naming the facts that are missing, because an arrangement nobody has described is not one the ATO has said it will leave alone. An established red-zone trigger still returns `RED`. The red zone is the guideline's own scenarios, of which the engine models 2 (paragraph 34(a) and (b), and paragraph 36); parental retention in general and a corporate unpaid present entitlement without a loan keep an arrangement out of the green zone but are not red, and after *Commissioner of Taxation v Bendel* [2026] HCA 18 an unpaid entitlement is not by itself a Division 7A loan.
- **Section 99B Foreign Trust Receipt Assessment**: Computes assessable amounts under *s 99B(1)* after corpus exemptions (*s 99B(2)(a)*) and prior-taxed income. Residency during the year of income is a required input, stated with `--resident-during-year` or `--not-resident-during-year`, because *s 99B(1)* turns on it: an unstated residency is refused and a non-resident receipt is refused rather than assessed. The 3 exemption amounts default to nil, which gives the largest assessable amount, and the *s 99B(2)(a)* corpus add-back defaults to nil in the other direction, leaving the whole corpus exempt. The result carries one caveat naming each nil it relied on, in both directions, because a nil default cannot be told apart from a figure nobody supplied.
- **Trust Resolution Schedule Verifier**: Checks timing against 30 June or the deed's earlier deadline, deed-power and percentage-completeness facts the caller supplies. The deed and execution facts, including whether the resolution streams specific income and the deed's own deadline, are stated or unstated; an unstated one returns `None` for validity, naming the fact, rather than reporting compliance or a breach. Missing streaming powers are a defect only for a resolution that streams.

---

## Quickstart

### Installation
```bash
pip install solomons-sword
```

### CLI usage
```bash
# Evaluate Section 100A risk zone
solomons-sword s100a-check --beneficiary "Adult Child" --amount 40000 --adult-child --pre-18-expenses

# Every fact stated, which is what a green zone needs. Each fact has a
# --no- form, and a fact left out is reported as not established.
solomons-sword s100a-check --beneficiary "Adult Child" --amount 40000 \
  --no-adult-child --no-retained-by-parents --no-circular --no-corporate-upe \
  --received-funds --within-two-years --no-direct-benefit --no-commercial-loan \
  --no-pre-18-expenses --no-retention-conditions --no-para-32-exclusion

# Assess Section 99B receipt from foreign trust with corpus deduction
solomons-sword s99b-check --beneficiary "Jane Doe" --gross 150000 --corpus 50000 --resident-during-year
```

---

## Statutory sources and tests

All allocation and threshold algorithms use exact `decimal.Decimal` calculations to prevent rounding discrepancies in trust tax schedules.

| Statutory Domain | Primary Authority | What the code actually does |
| :--- | :--- | :--- |
| **Proportionate Entitlement** | *ITAA 1936* s 95, s 97 (*Commissioner of Taxation v Bamford* [2010] HCA 10) | `Beneficiary Share = (Accounting Entitlement / Total Accounting Income) * s95 Net Income`. |
| **Section 100A Risk Matrix** | *ITAA 1936* s 100A, *ATO PCG 2022/2* | Returns GREEN, RED, OUTSIDE_GREEN, or FACTS_NOT_ESTABLISHED where a fact the zone turns on was not stated. It does not decide the white zone. |
| **Foreign Trust Distributions** | *ITAA 1936* s 99B(1), s 99B(2)(a) | Subtracts settled corpus and previously taxed income before assessable inclusion. Requires a stated residency and caveats every nil exemption. |
| **Trust Resolution Timing** | Caller-supplied deed and execution facts | Refuses incomplete percentages, reports stated deed defects, and returns not-established where a deed fact was never stated. This is not a substitute for current ATO guidance. |

### Automated test suite
- Run the suite: `pytest tests/`
- The suite covers proportionate streaming, Section 100A zones the engine implements, Section 99B corpus deductions, and resolution gates. Do not treat a badge as a live coverage certificate.

---

## Licence
MIT License. Created by Ryan Duguid.
