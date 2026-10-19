# Trust distribution checks

Solomon's Sword 1.0.0 is a standalone Rust binary. It replaces the Python
`solomons-sword` 0.3.2 package and its `louisgoldberg` import API.

Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited
to software issues reproduced with fabricated data. Do not send taxpayer
information or request advice, return preparation, tax-treatment confirmation or
lodgement.

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat-square)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.99-66023C?logo=rust&logoColor=white&labelColor=000000&style=flat-square)](https://www.rust-lang.org/)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache--2.0-66023C?labelColor=000000&style=flat-square)](https://www.apache.org/licenses/LICENSE-2.0)
[![ITAA 1936](https://img.shields.io/badge/Legislation-ITAA%201936%20Division%206-66023C?labelColor=000000&style=flat-square)](https://www.legislation.gov.au/C1936A00027/latest/text)

**Trust distribution allocation, Section 100A / 99B risk evaluation, and Division 6 ITAA 1936 review helpers for Australian trusts.**

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

Full boundary statement: [DISCLAIMER.md](DISCLAIMER.md).

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

Every fact is `True`, `False` or unstated. Amounts are exact decimals, and the
reports write them out in full. `s100a-check` refuses an amount of `1E+1000000`
or more, and the s 99B assessment refuses a figure its basis would show at that
size. On the command line each refusal is one `error:` line with exit 2, as is an
`s99b-check` whose combined amounts leave the decimal arithmetic range.

- **Division 6 Proportionate Allocation (*Commissioner of Taxation v Bamford* [2010] HCA 10)**: Calculates present entitlement proportions and allocates *s 95(1) ITAA 1936* taxable net income and the franking credits that ride with it. Trust net capital gains and franked dividends are **refused**, not allocated, whether or not they are streamed: the Division 6E carve-out with *Subdivision 115-C* (including s 115-220) and *Subdivision 207-B* is not implemented, so a proportionate answer would be wrong. Non-resident beneficiaries (*s 98(2A)/(3)*), nil income of the trust estate and no presently entitled beneficiary (*s 99 / s 99A*) are refused for the same reason. Each beneficiary's residency is a required input and their legal disability is stated as `True` or `False`: those 2 facts select the refused paths, so an unstated one is refused rather than defaulted.
- **Section 100A Reimbursement Agreement Matrix**: Classifies supplied facts against **ATO PCG 2022/2** as Green, Red, or outside those zones. The final guideline has white, green and red; the draft blue zone did not survive. White zone (income years ending before 1 July 2014) is out of scope because the function does not take an income year. The green zone turns on all 11 facts: while any is unstated the result is `FACTS_NOT_ESTABLISHED`, naming the facts that are missing, because an arrangement nobody has described is not one the ATO has said it will leave alone. An established red-zone trigger still returns `RED`. The red zone is the guideline's own scenarios, of which the engine models 2 (paragraph 34(a) and (b), and paragraph 36); parental retention in general and a corporate unpaid present entitlement without a loan keep an arrangement out of the green zone but are not red, and after *Commissioner of Taxation v Bendel* [2026] HCA 18 an unpaid entitlement is not by itself a Division 7A loan.
- **Section 99B Foreign Trust Receipt Assessment**: Computes assessable amounts under *s 99B(1)* after corpus exemptions (*s 99B(2)(a)*) and prior-taxed income. Residency during the year of income is a required input, stated with `--resident-during-year` or `--not-resident-during-year`, because *s 99B(1)* turns on it: an unstated residency is refused and a non-resident receipt is refused rather than assessed. The 3 exemption amounts default to nil, which gives the largest assessable amount, and the *s 99B(2)(a)* corpus add-back defaults to nil in the other direction, leaving the whole corpus exempt. The result carries one caveat naming each nil it relied on, in both directions, because a nil default cannot be told apart from a figure nobody supplied.
- **Trust Resolution Schedule Verifier**: Checks timing against 30 June or the deed's earlier deadline, deed-power and percentage-completeness facts the caller supplies. The deed and execution facts, including whether the resolution streams specific income and the deed's own deadline, are stated or unstated; an unstated one returns `null` for validity, naming the fact, rather than reporting compliance or a breach. Missing streaming powers are a defect only for a resolution that streams.

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
| `proportionate-share` | `trust_accounting_income`, `section95_net_taxable_income`, optional `net_capital_gains`, `franked_dividends`, `franking_credits` (each `0.00` when omitted), `beneficiaries` array | One object per beneficiary with every `BeneficiaryTaxShare` field |
| `s100a-risk` | `beneficiary_name`, `distribution_amount` and the 11 facts by their Python parameter names | Every `Section100AAssessment` field; `risk_zone` is the zone value |
| `s99b-liability` | `beneficiary_name`, `gross_amount_received_aud`, `beneficiary_was_resident_during_year`, optional `corpus_amount_aud`, `corpus_attributable_to_notional_assessable_income_aud`, `not_assessable_to_resident_aud`, `already_assessed_under_div6_aud` | Every `Section99BAssessment` field |
| `trust-resolution` | `financial_year`, `resolution_date`, `is_signed_by_trustee`, `streaming_powers_in_deed`, `default_beneficiary_clause_exists`, `allocated_percentages_total`, `uses_specific_streaming`, `deed_resolution_deadline` (date or `null`) | `result` (`true`, `false` or `null`), `issues`, and the schedule's `resolution_deadline`, `is_effective_by_year_end` and `is_within_income_year` |
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
UTF-8 bytes. Decimal exponents are not capped: the engine's own range checks
and overflow refusals bound what it writes out. JSON nesting follows serde_json's
recursion limit.

| Statutory Domain | Primary Authority | What the code actually does |
| :--- | :--- | :--- |
| **Proportionate Entitlement** | *ITAA 1936* s 95, s 97 (*Commissioner of Taxation v Bamford* [2010] HCA 10) | `Beneficiary Share = (Accounting Entitlement / Total Accounting Income) * s95 Net Income`. |
| **Section 100A Risk Matrix** | *ITAA 1936* s 100A, *ATO PCG 2022/2* | Returns GREEN, RED, OUTSIDE_GREEN, or FACTS_NOT_ESTABLISHED where a fact the zone turns on was not stated. It does not decide the white zone. |
| **Foreign Trust Distributions** | *ITAA 1936* s 99B(1), s 99B(2)(a) | Subtracts settled corpus and previously taxed income before assessable inclusion. Requires a stated residency and caveats every nil exemption. |
| **Trust Resolution Timing** | Caller-supplied deed and execution facts | Refuses incomplete percentages, reports stated deed defects, and returns not-established where a deed fact was never stated. This is not a substitute for current ATO guidance. |

## Integration and releases

No other component in this repository imports this engine. Root `just test` and
`just check` include Cargo. CI builds and tests on Ubuntu and Windows with
read-only repository permissions; CodeQL includes Rust and Dependabot monitors
this Cargo lock. The former Python/PyPI publisher is retired. Native signing and
publication need a separate authorised release change; no publisher is
configured for 1.0.0.

The frozen Python corpus in `tests/data/` is replayed by Rust on both platforms.
[tests/BEHAVIOURS.md](tests/BEHAVIOURS.md) maps the original tests to these checks.
See [CHANGELOG.md](CHANGELOG.md) for the documented divergences.

## Licence

Apache-2.0 licensed. Created by Ryan Duguid. If you redistribute this work or build on it, keep the attribution in [NOTICE](NOTICE).
