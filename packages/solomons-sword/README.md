# Trust distribution checks

Solomon's Sword 1.0.0 is a standalone Rust binary. It replaces the Python
`solomons-sword` 0.3.2 package and its `louisgoldberg` import API.

<div align="center">

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.99-66023C?logo=rust&logoColor=E5E4E2&labelColor=000000&style=flat&color=66023C)](https://www.rust-lang.org/)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/License-AGPL--3.0--or--later-66023C?labelColor=000000&style=flat&color=66023C)](LICENSE)
[![ITAA 1936](https://img.shields.io/badge/Legislation-ITAA%201936%20Division%206-66023C?labelColor=000000&style=flat&color=66023C)](https://www.legislation.gov.au/C1936A00027/latest/text)

</div>

**Trust distribution allocation, Section 100A / 99B risk evaluation and Division 6 ITAA 1936 review helpers for Australian trusts.**

Named for the judgement of Solomon, where the threat of dividing the child in proportion is what reveals who the true claimant is. Division 6 allocates trust income by proportionate entitlement following *Bamford*; Section 100A asks who actually ended up with the benefit. The name is a tribute only.

## Build and check

Install Rust with rustup, then run from this directory:

```sh
cargo build --locked --release
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The toolchain is pinned in `rust-toolchain.toml`. The executable is
`target/release/solomons-sword` (`.exe` on Windows). Add that directory to PATH
or copy the executable to a directory on PATH. No Python runtime is needed. The
native version is unreleased. PyPI still provides the historical Python release;
`pip install` does not install this binary.

## What it checks

Confirm every allocation, zone and exemption against the current law, the trust
deed and the entity's facts before acting, and leave lodgement decisions with a
registered practitioner.

Where the model does not implement a rule it refuses the input rather than
returning a number it cannot stand behind.

Every fact is `True`, `False` or unstated. Amounts are exact decimals, and the
reports write them out in full. `s100a-check` refuses an amount of `1E+1000000`
or more, and the s 99B assessment refuses a figure its basis would show at that
size. On the command line each refusal is one `error:` line with exit 2, as is an
`s99b-check` whose combined amounts leave the decimal arithmetic range.

- **Division 6 Proportionate Allocation (*Commissioner of Taxation v Bamford* [2010] HCA 10)**: Calculates present entitlement proportions and allocates *s 95(1) ITAA 1936* taxable net income and the franking credits that ride with it. Trust net capital gains and franked dividends are **refused**, not allocated, whether or not they are streamed: the Division 6E carve-out with *Subdivision 115-C* (including s 115-220) and *Subdivision 207-B* is not implemented, so a proportionate answer would be wrong. Non-resident beneficiaries (*s 98(2A)/(3)*), nil income of the trust estate and no presently entitled beneficiary (*s 99 / s 99A*) are refused for the same reason. Each beneficiary's residency is a required input and their legal disability is stated as `True` or `False`: those 2 facts select the refused paths, so an unstated one is refused rather than defaulted.
- **Section 100A Reimbursement Agreement Matrix**: Classifies supplied facts against **ATO PCG 2022/2** as Green, Red or outside those zones. The final guideline has white, green and red; the draft blue zone did not survive. White zone (income years ending before 1 July 2014) is out of scope because the function does not take an income year. The green zone turns on all 11 facts: whilst any is unstated the result is `FACTS_NOT_ESTABLISHED`, naming the facts that are missing, because an arrangement nobody has described is not one the ATO has said it will leave alone. An established red-zone trigger still returns `RED`. The red zone is the guideline's own scenarios, of which the engine models 2 (paragraph 34(a) and (b), and paragraph 36); parental retention in general and a corporate unpaid present entitlement without a loan keep an arrangement out of the green zone but are not red, and after *Commissioner of Taxation v Bendel* [2026] HCA 18 an unpaid entitlement is not by itself a Division 7A loan.
- **Section 99B Foreign Trust Receipt Assessment**: Computes assessable amounts under *s 99B(1)* after corpus exemptions (*s 99B(2)(a)*) and prior-taxed income. Residency during the year of income is a required input, stated with `--resident-during-year` or `--not-resident-during-year`, because *s 99B(1)* turns on it: an unstated residency is refused and a non-resident receipt is refused rather than assessed. The 3 exemption amounts default to nil, which gives the largest assessable amount, and the *s 99B(2)(a)* corpus add-back defaults to nil in the other direction, leaving the whole corpus exempt. The result carries one caveat naming each nil it relied on, in both directions, because a nil default cannot be told apart from a figure nobody supplied.
- **Trust Resolution Schedule Verifier**: Checks timing against 30 June or the deed's earlier deadline, deed-power and percentage-completeness facts the caller supplies. The deed and execution facts, including whether the resolution streams specific income and the deed's own deadline, are stated or unstated; an unstated one returns `null` for validity, naming the fact, rather than reporting compliance or a breach. Missing streaming powers are a defect only for a resolution that streams.

## Partnerships, trustee assessment and trust losses

3 JSON commands added after the Python port cover income years 2024-25 to
2026-27 (`financial_year` 2025 to 2027) and refuse other years, because their
provisions were compared only in the compilations in force from 1 July 2024.
Amounts are non-negative strings with at most 2 decimal places; facts are
`true`, `false` or `null`. Each refusal reads `refused: <reason> (<provision>)`.

- **`partnership-net-income`** (*ITAA 1936* ss 90, 91, 92): net income or
  partnership loss, leaving out deductions under s 290-150 or Division 36 of the
  *ITAA 1997*, and each partner's share of it and of exempt and non-assessable
  non-exempt income by their stated interest (the interests must total 100%).
  Shares are rounded half up to the cent and cents left over go to the largest
  shares first. A non-resident partner is decided only when all the income is
  from Australian sources. Refused: part-year residents, corporate limited
  partnerships (Div 5A), venture capital partnerships (s 92(2AA)), foreign
  hybrids (s 830-45), uncontrolled partnership income (s 94), segregated or
  pension asset interests (s 92(2A)) and salary or interest allocated to
  partners.
- **`trustee-assessment`** (*ITAA 1936* ss 95, 97, 98, 99, 99A; *Income Tax
  Rates Act 1986*): splits a resident trust's net income by Division 6
  percentage and says who is assessed on each share. The trustee is taxed on an
  adult resident beneficiary under a legal disability (s 98(1)(a), resident
  scale with the tax-free threshold), a non-resident individual (non-resident
  scale), a non-resident company (25% for a base rate entity, otherwise 30%), a
  non-resident trustee beneficiary (45%, s 98(4)), and income to which no
  beneficiary is presently entitled: 45% under s 99A, or under s 99 when the
  trust is a s 99A(2) kind and the Commissioner's opinion is stated (the
  resident scale with the threshold for an estate whose deceased died less than
  3 years before year end, otherwise without it and limited by Rates Act s 14).
  Tax is worked on each amount in dollars and cents, as the Rates Act sets no
  whole-dollar rounding, and rounded half up to the cent. Medicare levy,
  offsets and credits are not computed. Refused: non-resident
  trust estates, part-year residents, beneficiaries under 18 (Div 6AA), s 95A(2)
  deemed entitlements, s 97A and s 97(3) beneficiaries, capital gains and
  franked distributions (Div 6E), special disability trusts, managed investment
  trusts, public trading and corporate unit trusts and s 94 income.
- **`trust-loss-tests`** (*ITAA 1936* Sch 2F): for deducting a prior-year tax
  loss, which of the 50% stake, pattern of distributions, control and income
  injection tests apply to a fixed or non-fixed trust or an excepted trust
  (family trust elections taken as stated; complying superannuation and
  approved deposit funds and pooled superannuation trusts are excepted under
  s 272-100(b)), and whether each is passed on the
  stated facts. The pattern of distributions test is worked out from stated
  test year distributions when given: each individual counts the smallest
  percentage they received (s 269-70), and the totals must exceed 50% for income
  and for capital. `deductible` is `true`, `false` or `null` with the tests not
  established named. Refused: widely held trusts (Subdivs 266-C to 266-E), bad
  debt deductions and current-year losses (Div 268). Not decided: part-year
  deductions (ss 266-50, 267-50) and family trust distribution tax.

```sh
echo '{"financial_year":2025,"assessable_income":"1000","deductions":"0","partners":[{"name":"A","residency":"resident","interest_percentage":"50"},{"name":"B","residency":"resident","interest_percentage":"50"}]}' | solomons-sword partnership-net-income
```

## Original commands

```sh
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

`--help` and each command's `--help` preserve the Python help text, flags,
abbreviations, exit codes and error lines. Both output streams write a lone
surrogate from the command line as its `\udcXX` escape.

Command output carries the beneficiary name you supply, because a workpaper line
item is unusable without it. Treat that output as client data: write it to the
firm's approved secure location, never to a path inside a repository.

## Former Python API

Each API command reads one UTF-8 JSON object from stdin and prints one compact
JSON result followed by a newline. Amounts are decimal strings throughout,
preserving Python Decimal values and scale. Facts are `true`, `false` or `null`
(unstated); an omitted fact is unstated, and any other JSON type is refused with
the original `ValueError` text naming the field. Dates use `YYYY-MM-DD`. An
invalid request exits 2, writes `error: <reason>` to stderr and writes no stdout.
`--api-help` lists these commands.

| Command | Input fields | Result |
| --- | --- | --- |
| `partnership-net-income` | `financial_year`, `assessable_income`, `deductions`, optional `excluded_deductions`, `exempt_income`, `non_assessable_non_exempt_income`, `all_income_from_australian_sources`, `partners` array (`name`, `residency`, `interest_percentage`) and the refusal flags | Net income or partnership loss and each partner's `assessable_income`, `deductible_loss`, `exempt_income` and `non_assessable_non_exempt_income` |
| `proportionate-share` | `trust_accounting_income`, `section95_net_taxable_income`, optional `net_capital_gains`, `franked_dividends`, `franking_credits` (each `0.00` when omitted), `beneficiaries` array | One object per beneficiary with every `BeneficiaryTaxShare` field |
| `s100a-risk` | `beneficiary_name`, `distribution_amount` and the 11 facts by their Python parameter names | Every `Section100AAssessment` field; `risk_zone` is the zone value |
| `s99b-liability` | `beneficiary_name`, `gross_amount_received_aud`, `beneficiary_was_resident_during_year`, optional `corpus_amount_aud`, `corpus_attributable_to_notional_assessable_income_aud`, `not_assessable_to_resident_aud`, `already_assessed_under_div6_aud` | Every `Section99BAssessment` field |
| `trust-resolution` | `financial_year`, `resolution_date`, `is_signed_by_trustee`, `streaming_powers_in_deed`, `default_beneficiary_clause_exists`, `allocated_percentages_total`, `uses_specific_streaming`, `deed_resolution_deadline` (date or `null`) | `result` (`true`, `false` or `null`), `issues`, and the schedule's `resolution_deadline`, `is_effective_by_year_end` and `is_within_income_year` |
| `trust-loss-tests` | `financial_year`, `loss_year`, `trust_kind`, `excepted_trust`, `deduction`, the stated test facts and optional `test_year_distributions` (`income` and `capital` arrays of name-to-percentage objects) | Each test's `applies`, `passed` and provision, the pattern of distributions working, `deductible`, `not_established` and `not_decided` |
| `trustee-assessment` | `financial_year`, `trust_residency`, `net_income`, `income_of_trust_estate`, `beneficiaries` array (`name`, `kind`, `residency`, `entitlement`, `under_18`, `under_legal_disability`, `base_rate_entity`), `all_net_income_from_australian_sources`, `section_99a_2_kind`, `commissioner_opinion_99a_unreasonable`, `deceased_estate_less_than_3_years` and the refusal flags | Each beneficiary's Division 6 percentage, share, provision and trustee tax; the part no beneficiary is entitled to; `total_trustee_tax` |
| `validate-amount` | `value` string | The command line's decimal parser result, or its refusal |

A beneficiary object has `beneficiary_name`, `is_resident`,
`is_under_legal_disability`, optional `percentage_entitlement` or
`fixed_entitlement_amount` (a percentage wins when both are given), and optional
`specifically_streamed_capital_gains` and `specifically_streamed_franked_dividends`.
`trust_name`, `source_country` and the assessment's `financial_year` are accepted
and ignored, as the Python functions ignored them.

For example, in a POSIX shell:

```sh
echo '{"beneficiary_name":"Adult Child","distribution_amount":"40000","beneficiary_is_adult_child":true,"entitlement_applied_to_pre_18_expenses":true}' | solomons-sword s100a-risk
echo '{"trust_accounting_income":"10.00","section95_net_taxable_income":"10.00","beneficiaries":[{"beneficiary_name":"A","is_resident":true,"is_under_legal_disability":false,"percentage_entitlement":"33.33"},{"beneficiary_name":"B","is_resident":true,"is_under_legal_disability":false,"percentage_entitlement":"66.67"}]}' | solomons-sword proportionate-share
```

In PowerShell use a single-quoted JSON string piped directly to the executable,
without `echo`. Non-ASCII input needs a UTF-8 pipe encoding.

## Arithmetic and limits

Arithmetic follows Python's default Decimal context: 28 significant digits,
half even for intermediate operations and display, and half up where the engine
explicitly rounds to cents. NaN, Infinity, Overflow and DivisionByZero behave as
the Python engine did; through the API a trapped signal is refused as
`error: decimal.<Signal>`. Unrounded entitlement ratios allocate each pool, and
rounding residuals go to the largest shares first, as before.

The JSON boundary accepts at most 8 MiB, and each decimal field at most 8,192
UTF-8 bytes (`validate-amount` excepted, like the command-line parser it mirrors).
An amount field whose default is `0.00` uses that default when omitted or `null`;
required amounts must be strings, and optional fixed or percentage entitlements
stay unset. Unreadable strings in engine-command amount fields are refused as
`error: decimal.InvalidOperation`; `validate-amount` keeps the command-line
validator's messages. Decimal exponents are not capped: the engine's own range checks
and overflow refusals bound what it writes out. JSON nesting follows serde_json's
recursion limit.

| Statutory Domain | Primary Authority | What the code actually does |
| :--- | :--- | :--- |
| **Proportionate Entitlement** | *ITAA 1936* s 95, s 97 (*Commissioner of Taxation v Bamford* [2010] HCA 10) | `Beneficiary Share = (Accounting Entitlement / Total Accounting Income) * s95 Net Income`. |
| **Section 100A Risk Matrix** | *ITAA 1936* s 100A, *ATO PCG 2022/2* | Returns GREEN, RED, OUTSIDE_GREEN or FACTS_NOT_ESTABLISHED where a fact the zone turns on was not stated. It does not decide the white zone. |
| **Foreign Trust Distributions** | *ITAA 1936* s 99B(1), s 99B(2)(a) | Subtracts settled corpus and previously taxed income before assessable inclusion. Requires a stated residency and caveats every nil exemption. |
| **Trust Resolution Timing** | Caller-supplied deed and execution facts | Refuses incomplete percentages, reports stated deed defects, and returns not-established where a deed fact was never stated. This is not a substitute for current ATO guidance. |

## Integration and releases

No other component in this repository imports this engine. Root `just test` and
`just check` include Cargo. CI builds and tests on Ubuntu, Windows and macOS
with read-only repository permissions; CodeQL includes Rust and Dependabot monitors
this Cargo lock. The former Python/PyPI publisher is retired.
`.github/workflows/release-solomons-sword.yml` builds and publishes attested
x86-64 Linux (static musl) and Windows executables through `release-policy` on
GitHub Releases. `--version` prints `solomons-sword 1.0.0`.

The frozen Python corpus in `tests/data/` is replayed by Rust on all 3 platforms.
[tests/BEHAVIOURS.md](tests/BEHAVIOURS.md) maps the original tests to these checks.
See [CHANGELOG.md](CHANGELOG.md) for the documented divergences.

## Licence

Ryan Duguid's code is licensed under [AGPL-3.0-or-later](LICENSE) with [section 7(b) attribution terms](NOTICE). Created by Ryan Duguid. If you redistribute this work or build on it, keep the copyright notices in [NOTICE](NOTICE).

Versions released before this change keep the licences they were released with, including any future grants under those terms.

<sub>Review aid, not tax, legal or financial advice, an assessment or a determination. Ryan Duguid is not a registered tax agent or BAS agent. Support covers software issues reproduced with fabricated data: do not send taxpayer information or ask for advice, return preparation, tax-treatment confirmation or lodgement. See [DISCLAIMER.md](DISCLAIMER.md).</sub>
