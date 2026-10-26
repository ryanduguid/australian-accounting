# Australian tax calculators

Australian Tax Calculators 1.0.0 is a standalone Rust binary. It replaces the
Python `australian-tax-calculators` 0.2.0 package and its `austaxcalc` import
API, with the same fourteen worksheets, results and refusals. Python releases
through 0.1.9 contain ten worksheets. Until the MCP server links this crate,
it uses the 0.2.0 Python release from PyPI; the
[release handoff](../../apps/aus-accounting-mcp/docs/worksheet-release.md)
keeps the engine release and MCP integration separate.

Ryan Duguid is not a registered tax agent or BAS agent. Project support is limited to software issues reproduced with fabricated data. Do not send taxpayer information or request advice, return preparation, tax-treatment confirmation, or lodgement.

<div align="center">

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.99-66023C?logo=rust&logoColor=E5E4E2&labelColor=000000&style=flat&color=66023C)](https://www.rust-lang.org/)
[![License: FSL-1.1-ALv2](https://img.shields.io/badge/License-FSL--1.1--ALv2-66023C?labelColor=000000&style=flat&color=66023C)](LICENSE)

</div>

Fourteen calculation worksheets for established facts: ordinary GST, resident basic income
tax, CGT losses and discount, ordinary employer FBT, first-year depreciation,
quarterly super guarantee, Schedule 1 PAYG withholding, super contribution caps,
account-based pension minimums, study and training loan repayments, Schedule 8 study
loan withholding, a single person's Medicare levy, the low income tax offset and
Division 293 tax. These are experimental
review aids, not advice or return preparation. Callers must establish every scope condition before calculating.

Each worksheet is a subcommand that reads one JSON object on stdin and writes
one JSON result on stdout. Results include amounts, scope exclusions, official
source URLs, the source-check date and engine version. The binary makes no
network calls and writes no files. Amounts must be non-negative, finite, at most
2 decimal places and no more than AUD 1 trillion. Rounding uses half-up to cents
at output; individual worksheets state additional conventions.

## Build and check

Install Rust with rustup, then run from this directory:

```sh
cargo build --locked --release
cargo fmt --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
```

The toolchain is pinned in `rust-toolchain.toml`. The executable is
`target/release/australian-tax-calculators` (`.exe` on Windows). No Python
runtime is needed. The native version is unreleased. PyPI still provides the
historical Python release; `pip install` does not install this binary.

## Use

Every worksheet takes its amounts as decimal strings, an explicit period, and a
`scope_confirmed` flag the caller sets only once it has established the scope
conditions the worksheet states. Splitting $1,100.00 GST-inclusive into its
exclusive amount and its GST:

```sh
echo '{"amount": "1100.00", "gst_inclusive": true, "scope_confirmed": true, "year": "2025-26"}' \
  | australian-tax-calculators gst
```

```json
{"ok":true,"engine":"australian-tax-calculators","engine_version":"1.0.0","calculation":"gst","period":"2025-26","amounts":{"gst":"100.00","exclusive":"1000.00","inclusive":"1100.00"},"rates":{"gst_rate":"0.10"},"scope":"One ordinary taxable supply at 10%, already classified by the operator. Excludes mixed supplies, exemptions, margin schemes, adjustments, tax invoice rounding across line items, input-credit entitlement and BAS preparation.","sources":["https://smallbusiness.taxsuperandyou.gov.au/goods-and-services-tax/fast-facts"],"source_checked":"2026-09-10","warnings":["Operator-confirmed facts and scope; eligibility is not independently verified.","Worksheet only, not advice, an assessment or a lodgment. Obtain human review."]}
```

Amounts come back as quoted decimal strings rather than numbers, so an amount
that leaves a worksheet cannot arrive at a JSON reader as a float. The result
also carries `scope`, the official `sources` the figures came from, the engine
version and the warnings that travel with every worksheet. A refusal writes one
`error:` line to stderr and exits 2, with the original worksheet's message.

Full boundary statement: [DISCLAIMER.md](DISCLAIMER.md).

## Commands

Each command takes the original function's keyword arguments, in this order, as
one JSON object. Amounts and ratios are decimal strings; a JSON number, `null`
or text that is not a decimal is refused, as the Python function refused a
value that was not a `Decimal`. Flags are JSON booleans and counts, ages, days,
quarters and scales are JSON integers. `australian-tax-calculators --api-help`
lists the commands.

| Command | Arguments |
| --- | --- |
| `gst` | `amount`, `gst_inclusive`, `scope_confirmed`, `year` |
| `resident-tax` | `taxable_income`, `year`, `scope_confirmed` |
| `capital-gains` | `other_gains`, `discount_gains`, `current_losses`, `prior_losses`, `scope_confirmed`, `year` |
| `fbt` | `type_one_value`, `type_two_value`, `year_ended`, `scope_confirmed` |
| `depreciation` | `cost`, `effective_life`, `days`, `taxable_use`, `method`, `scope_confirmed`, `year` |
| `quarterly-sg` | `ordinary_time_earnings`, `qualifying_contributions`, `year`, `quarter`, `scope_confirmed` |
| `payg-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed` |
| `study-loan-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed` |
| `medicare-levy` | `taxable_income`, `section_9a_adjustment_applies`, `year`, `scope_confirmed` |
| `low-income-tax-offset` | `taxable_income`, `year`, `scope_confirmed` |
| `division-293` | `division_293_income`, `low_tax_contributions`, `year`, `scope_confirmed` |
| `study-loan-repayment` | `repayment_income`, `year`, `scope_confirmed` |
| `contribution-caps` | `total_super_balance`, `concessional_contributions`, `unused_concessional_cap`, `non_concessional_contributions`, `under_75_in_year`, `year`, `scope_confirmed` |
| `pension-minimum` | `account_balance`, `age`, `days`, `year`, `scope_confirmed` |
| `worksheet-catalogue` | none: send `{}` |
| `periods` | `kind`, the worksheet name, such as `"payg_withholding"` |

Every argument is required, as the Python functions had no defaults. Input over 8 MiB,
or a decimal string over 8,192 bytes, is refused.

## Supported periods

| Worksheet | Period and boundary |
| --- | --- |
| GST | 2025-26, one ordinary taxable supply at 10%; no classification or BAS |
| Resident tax | 2024-25, 2025-26 and 2026-27; full-year resident ordinary income, before offsets and levies |
| CGT | 2025-26, resident individual's established gains and losses; losses before the 50% discount |
| FBT | Year ended 31 March 2026; established type 1 and type 2 taxable values, ordinary employer |
| Depreciation | 2025-26, first year of an ordinary tangible Division 40 asset; established life and use |
| SG | Complete quarters of 2025-26; established OTE, eligibility and qualifying contributions |
| Contribution caps | 2024-25 to 2026-27; one individual's classified contributions, no bring-forward period started in the 2 previous years |
| Pension minimum | 2024-25 to 2026-27; one account-based pension paying under SISR Schedule 7 |
| Study loan repayment | 2025-26 and 2026-27; one individual's established repayment income at the marginal rates; the formula amount before the loan-balance limit |
| Study loan withholding | Regular weekly, fortnightly or monthly pays from 1 July 2026 on scale 1, 2, 3, 5 or 6; the Schedule 1 amount plus the Schedule 8 component |
| Medicare levy | 2024-25 and 2025-26; full-year resident single without SAPTO, spouse, dependants or exemption; the 2% rate and the low-income reduction |
| Low income tax offset | 2024-25, 2025-26 and 2026-27; resident individual's established taxable income; the offset amount, not applied against tax |
| Division 293 | 2024-25, 2025-26 and 2026-27; established Division 293 income and low tax contributions; no defined benefit interests |

Unsupported periods fail. The SG worksheet does not implement post-June 2026
Payday entitlement rules. The separate Payday engine reviews timing on a supplied
liability. SMSF fund tax, Division 296 tax, the transfer balance cap, trusts, payroll
tax, the Medicare levy surcharge and Medicare levy reductions other than the single
low-income reduction remain outside these worksheets.

## PAYG withholding

`payg_withholding` applies the Schedule 1 (NAT 1004) formulas to one regular
weekly, fortnightly or monthly pay made from 1 July 2026, on scale 1, 2, 3, 5
or 6 as the operator established from the payee's declarations. Earnings
include allowances subject to withholding. The result gives the weekly
earnings used, the weekly amount and the amount for the pay period, with the
coefficients applied. Scale 4, tax offsets, Medicare levy adjustments, study
and training support loans, extra amounts for 53 or 27 pays, quarterly pays
and the other schedules are outside it. The tests reproduce every amount in
the ATO's published sample data; see
[calculation evidence](docs/calculation-evidence.md#payg-withholding).
This worksheet is published in the 0.1.5 wheel.

## Super contribution caps and pension minimums

`contribution_caps` tests one year's classified contributions against the caps.
Unused concessional cap carries forward only when the total super balance at the
previous 30 June is below $500,000. The non-concessional cap is 4 times the
concessional cap, nil at or above the general transfer balance cap, and otherwise
1, 2 or 3 times the annual cap by the balance bands in ITAA 1997 s 292-85. Bring-forward
needs the person to be under 75 at any time in the year. The result gives room and
excess under each cap, the multiple of the annual cap available and the
bring-forward period the contributions started. A period started in an earlier
year is outside the worksheet.

`pension_minimum` applies SISR Schedule 7 to one account-based pension: the age
factor on the balance at 1 July, pro-rated by days in the first year, nil when the
pension starts on or after 1 June, and rounded to the nearest $10 with an exact $5
rounding up. The tests check every band boundary against the ATO's published
tables; see [calculation evidence](docs/calculation-evidence.md#contribution-caps).
These worksheets are published in the 0.1.6 wheel.

## Study and training loan repayment

`study_loan_repayment` applies the marginal repayment rates that apply from
2025-26 to one individual's established whole-dollar repayment income: nil at or
below the minimum repayment income, 15% of the excess up to the top of the first
band, then the 15% band in full plus 17% of the excess, and 10% of the whole
repayment income once that is the smaller amount. The result gives the marginal
amount, 10% of income, the compulsory repayment and the rate that applied, with
the thresholds. Repayment income (taxable income excluding assessable FHSS
released amounts, plus reportable fringe benefits, total net investment loss,
reportable super contributions and exempt foreign employment income) is the
operator's established fact. The formula amount is not limited to the loan
balance, and whether a debt exists, voluntary repayments, the overseas levy,
allocation between loan types, indexation and the Schedule 8 withholding
component are outside it. The tests hold the worksheet to the ATO's worked
examples, the Gazette that notified the 2026-27 thresholds and every band
boundary; see [calculation evidence](docs/calculation-evidence.md#study-loan-repayment).

## Study loan withholding

`study_loan_withholding` adds the Schedule 8 (NAT 3539) study and training
support loan component to the Schedule 1 amount for one regular weekly,
fortnightly or monthly pay made from 1 July 2026 to a payee who declared a
study and training support loan debt. Scale 1 takes the component for a payee
who did not claim the tax-free threshold; scales 2, 3, 5 and 6 take the one for
a payee who claimed it or is a foreign resident. The result gives the Schedule
1 amount, the component and their total. The ATO also publishes combined
coefficients and accepts either method; the tests hold the worksheet within $1
of them for scales 1 and 2, and to the schedule's three worked examples. See
[calculation evidence](docs/calculation-evidence.md#study-loan-withholding).

## Medicare levy and low income tax offset

`medicare_levy` applies the 2% rate and the low-income reduction in s 7 of the
Medicare Levy Act 1986 to a full-year resident single who is not entitled to
the seniors and pensioners tax offset and has no exemption: nil at or below the
threshold amount, and no more than 10% of the excess up to the phase-in limit.
The caller states whether s 9A (lump sum payments in arrears) applies, and the
worksheet refuses when it does. It covers 2024-25 and 2025-26; 2026-27 is refused until its thresholds are
published. `low_income_tax_offset` gives the ITAA 1997 s 61-115 offset from
established taxable income for 2024-25 to 2026-27. Neither applies the result
against tax payable. See
[calculation evidence](docs/calculation-evidence.md#medicare-levy).

## Division 293

`division_293` applies ITAA 1997 s 293-20 to one individual's established
Division 293 income and low tax contributions: when their sum exceeds $250,000,
taxable contributions are the lesser of the contributions and the excess, taxed
at 15%. It covers 2024-25 to 2026-27 and holds the ATO's worked example. Working
out either input, defined benefit interests and deferred debt accounts are
outside it. See [calculation evidence](docs/calculation-evidence.md#division-293).

These four worksheets are included from release 0.2.0.

## Worksheet discovery

The `worksheet-catalogue` command, `worksheet_catalogue()` from the 0.1.4 wheel,
returns supported periods with inclusive dates, required inputs and units,
available methods, scope exclusions and a fabricated example for each worksheet.
Money in the catalogue uses decimal strings, as the commands take them; MCP
clients can send `example.facts` to `calculate_tax_worksheet`.

The engine owns the supported periods for both discovery and calculation.
Each call builds its own result, so a caller cannot change later calls or engine rules.

## Calculation evidence

[Calculation evidence](docs/calculation-evidence.md) records Library paragraphs,
document review dates, independent arithmetic and the limits of each example.
`example_evidence` in the catalogue points to these records. It requires no local
Library installation or network access at runtime.

Source URLs and exact exclusions remain in `src/texts.rs` and every
result. The existing 10 September 2026 source-check baseline is now stored per
worksheet. A later check of one worksheet cannot refresh another's date. The
15 September Library example check does not extend that baseline to other periods
or establish a taxpayer's circumstances.
