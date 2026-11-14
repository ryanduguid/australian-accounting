# Australian tax calculators

Australian Tax Calculators 1.0.0 is a standalone Rust binary. It replaces the
Python `australian-tax-calculators` 0.2.0 package and its `austaxcalc` import
API, with the same 14 worksheets, results and refusals, and adds 2 FBT car
worksheets, a main residence worksheet, a borrowing expenses worksheet, a Division 296 estimate, a termination payment worksheet, 3 Schedule 5 withholding worksheets, a Schedule 15 working holiday maker worksheet and a foreign resident tax worksheet that no Python release contains. Python releases through 0.1.9
contain 10 worksheets. Until the MCP server links this crate,
it uses the 0.2.0 Python release from PyPI; the
[release handoff](../../apps/aus-accounting-mcp/docs/worksheet-release.md)
keeps the engine release and MCP integration separate.

<div align="center">

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.99-66023C?logo=rust&logoColor=E5E4E2&labelColor=000000&style=flat&color=66023C)](https://www.rust-lang.org/)
[![License: Ryan Duguid Public Attribution Licence 1.0](https://img.shields.io/badge/License-Public%20Attribution%201.0-66023C?labelColor=000000&style=flat&color=66023C)](LICENSE)

</div>

Twenty-five calculation worksheets for established facts: ordinary GST, resident and foreign
resident basic income tax, CGT losses and discount, the main residence exemption's taxable share of a gain or loss, one loan's borrowing expenses deduction, ordinary employer FBT, car fringe benefit taxable values
under the statutory formula and operating cost methods, depreciation in the start year and later years,
quarterly super guarantee, Schedule 1 PAYG withholding, Schedule 5 withholding from back payments, commissions and bonuses by Methods A, B(i) and B(ii), Schedule 15 withholding for working holiday makers, super contribution caps,
account-based pension minimums, study and training loan repayments, Schedule 8 study
loan withholding, a single person's Medicare levy, the low income tax offset,
Division 293 tax, an estimate of Division 296 tax on large super balances and the tax-free part of a genuine redundancy payment with Schedule 11 withholding from an employment termination payment. Callers must establish every scope condition before calculating.

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

Every worksheet takes its amounts as decimal strings, an explicit period and a
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

Add `--refusal-json` after the command to read the refusal as data as well: stdout
then carries one JSON line, `{"ok":false,"code":"scope.not_confirmed","message":"..."}`,
beside the same `error:` line and exit 2. A code names the failure, stays when a
message is reworded and never moves to a different failure. `REFUSAL_CODES` in
`src/texts.rs` is the catalogue, and a test holds every refusal to it. Without the
flag, stdout stays empty on a refusal.

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
| `foreign-resident-tax` | `taxable_income`, `year`, `scope_confirmed` |
| `capital-gains` | `other_gains`, `discount_gains`, `current_losses`, `prior_losses`, `scope_confirmed`, `year` |
| `main-residence` | `capital_gain`, `capital_loss`, `market_value_basis`, `ownership_start`, `ownership_end`, `lived_periods`, `income_use_periods`, `absence_choice_periods`, `new_home_acquired`, `year`, `scope_confirmed` |
| `borrowing-expenses` | `borrowing_expenses`, `other_borrowing_expenses_in_year`, `income_use`, `loan_start`, `loan_term_end`, `repaid_on`, `section_26_50_applies`, `year`, `scope_confirmed` |
| `fbt` | `type_one_value`, `type_two_value`, `year_ended`, `scope_confirmed` |
| `fbt-car-statutory` | `cost_price`, `later_accessories`, `one_third_reduction`, `days_available`, `employee_contribution`, `electric_car_exempt`, `year_ended`, `scope_confirmed` |
| `fbt-car-operating-cost` | `running_costs`, `lease_charges`, `depreciated_value`, `effective_life_years`, `days_held`, `business_use`, `employee_contribution`, `statutory_formula_taxable_value`, `electric_car_exempt`, `year_ended`, `scope_confirmed` |
| `depreciation` | `cost`, `effective_life`, `days`, `taxable_use`, `method`, `scope_confirmed`, `year`; optional `opening_adjustable_value` |
| `quarterly-sg` | `ordinary_time_earnings`, `qualifying_contributions`, `year`, `quarter`, `scope_confirmed` |
| `payg-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed`; optional `foreign_resident` (scale 4), `tax_offsets`, `medicare_adjustment_children`, `extra_pay_withholding` |
| `study-loan-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed` |
| `medicare-levy` | `taxable_income`, `section_9a_adjustment_applies`, `year`, `scope_confirmed` |
| `low-income-tax-offset` | `taxable_income`, `year`, `scope_confirmed` |
| `division-293` | `division_293_income`, `low_tax_contributions`, `year`, `scope_confirmed` |
| `division-296` | `total_super_balance_end_of_year`, `total_super_earnings`, `child_recipient`, `structured_settlement_contribution`, `died_in_year`, `year`, `scope_confirmed` |
| `termination-payment` | `redundancy_payment`, `completed_years_of_service`, `excluded_payment`, `non_excluded_payment`, `other_taxable_payments`, `date_of_birth`, `year`, `scope_confirmed` |
| `schedule-5-method-a` | `earnings`, `additional_payment`, `pay_periods`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed` |
| `schedule-5-method-b-i` | `earnings`, `back_payment_periods`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed` |
| `schedule-5-method-b-ii` | `earnings`, `additional_payment`, `year_to_date_earnings`, `pay_periods_to_date`, `earlier_additional_payments`, `earlier_additional_withholding`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed` |
| `working-holiday-maker-withholding` | `payment`, `payments_before`, `employer_registered`, `tfn_quoted`, `pay_period`, `year`, `scope_confirmed` |
| `study-loan-repayment` | `repayment_income`, `year`, `scope_confirmed` |
| `contribution-caps` | `total_super_balance`, `concessional_contributions`, `unused_concessional_cap`, `non_concessional_contributions`, `under_75_in_year`, `year`, `scope_confirmed` |
| `pension-minimum` | `account_balance`, `age`, `days`, `year`, `scope_confirmed` |
| `worksheet-catalogue` | none: send `{}` |
| `periods` | `kind`, the worksheet name, such as `"payg_withholding"` |

The 2 car commands, `main-residence`, `borrowing-expenses`, `division-296`, `termination-payment`, the 3 `schedule-5` commands, `working-holiday-maker-withholding` and `foreign-resident-tax` have no Python original; their arguments follow the same
rules. `division-296` alone takes a negative amount, in `total_super_earnings`. `main-residence` also takes dates as `YYYY-MM-DD` strings, lists of
`{"start", "end"}` periods (with `"floor_area"` for income use) and `null`
for no new home. `borrowing-expenses` takes `loan_start` as a `YYYY-MM-DD`
string and `loan_term_end` and `repaid_on` as dates or `null`. `termination-payment` takes `completed_years_of_service` as a JSON integer and `date_of_birth` as a `YYYY-MM-DD` string. `schedule-5-method-b-i` takes `back_payment_periods` as a list of `{"earnings", "back_payment", "withheld"}` decimal strings, one for each earlier pay period. Every argument is required, as the Python functions had no defaults, except `depreciation`'s optional `opening_adjustable_value`: give it, as a decimal string, for a year after the asset's start time, as cost less the earlier years' decline in value. `payg-withholding`'s optional inputs are described under [PAYG withholding](#payg-withholding). Input over 8 MiB,
or a decimal string over 8,192 bytes, is refused.

## Supported periods

| Worksheet | Period and boundary |
| --- | --- |
| GST | 2025-26 and 2026-27, one ordinary taxable supply at 10%; no classification or BAS |
| Resident tax | 2024-25, 2025-26 and 2026-27; full-year resident ordinary income, before offsets and levies |
| Foreign resident tax | 2024-25, 2025-26 and 2026-27; full-year foreign resident ordinary income, no tax-free threshold; no Medicare levy or low income tax offset applies |
| CGT | 2025-26 and 2026-27, resident individual's established gains and losses; losses before the 50% discount |
| Main residence | CGT events in 2024-25 to 2026-27; one dwelling disposed of by a resident individual, before the discount |
| Borrowing expenses | 2024-25 to 2026-27, before s 26-155 from 2027-28; one loan's established expenditure, use and dates |
| FBT | Years ended 31 March 2026 and 2027; established type 1 and type 2 taxable values, ordinary employer |
| FBT car benefits | Years ended 31 March 2026 and 2027; one car's established base value or operating costs, the 20% statutory percentage and a deemed depreciation rate of 200% over the established effective life |
| Depreciation | 2025-26 and 2026-27, an ordinary tangible Division 40 asset's start year, or a later year from its established opening adjustable value; established life and use |
| SG | Complete quarters of 2025-26; established OTE, eligibility and qualifying contributions |
| Contribution caps | 2024-25 to 2026-27; one individual's classified contributions, no bring-forward period started in the 2 previous years |
| Pension minimum | 2024-25 to 2026-27; one account-based pension paying under SISR Schedule 7 |
| Study loan repayment | 2025-26 and 2026-27; one individual's established repayment income at the marginal rates; the formula amount before the loan-balance limit |
| Study loan withholding | Regular weekly, fortnightly or monthly pays from 1 July 2026 on scale 1, 2, 3, 5 or 6; the Schedule 1 amount plus the Schedule 8 component |
| Medicare levy | 2024-25, 2025-26 and 2026-27; full-year resident single without SAPTO, spouse, dependants or exemption; the 2% rate and the low-income reduction |
| Low income tax offset | 2024-25, 2025-26 and 2026-27; resident individual's established taxable income; the offset amount, not applied against tax |
| Division 293 | 2024-25, 2025-26 and 2026-27; established Division 293 income and low tax contributions; no defined benefit interests |
| Division 296 | 2026-27; one individual's established balance at 30 June 2027 and total superannuation earnings; no defined benefit or other s 296-70 interests; an estimate, as the Commissioner assesses |
| Termination payment | 2026-27; one life benefit ETP to a resident employee who gave a TFN, paid within 12 months; established genuine redundancy and classified ETP parts; Schedule 11 withholding, not the tax assessed |
| Schedule 5 withholding | Weekly, fortnightly or monthly pays made from 1 July 2024 to 30 June 2027 that include back payments, commissions or bonuses, on scale 1, 2, 3, 5 or 6; the study and training support loan component only from 1 July 2026 |
| Working holiday makers | 2024-25 to 2026-27; one weekly, fortnightly or monthly payment to an established working holiday maker; no TFN exemption claimants or labour mobility schemes |

Unsupported periods fail. The SG worksheet does not implement post-June 2026
Payday entitlement rules. The separate Payday engine reviews timing on a supplied
liability. SMSF fund tax, Division 296 fund earnings and their attribution, the transfer balance cap, trusts, payroll
tax, the Medicare levy surcharge and Medicare levy reductions other than the single
low-income reduction remain outside these worksheets.

## Foreign resident tax

`foreign_resident_tax` applies Schedule 7 Part II of the Income Tax Rates Act
1986 to a person who was a foreign resident for the whole income year: 30% to
$135,000, 37% to $190,000 and 45% above, with no tax-free threshold. It covers
2024-25 to 2026-27. The ATO publishes the 2024-25 and 2025-26 tables; 2026-27
takes the same rates from the Act, which ties them to that year's second and
third resident rates, and its result says the ATO had not yet published them.
2027-28 is refused until the ATO publishes rates for it. A foreign resident
pays no Medicare levy and is not entitled to the low income tax offset, so the
result is basic tax only and its scope says why. Like the other worksheets
without a Python original, it refuses a field it does not take. See
[calculation evidence](docs/calculation-evidence.md#foreign-resident-tax).

## PAYG withholding

`payg_withholding` applies the Schedule 1 (NAT 1004) formulas to one regular
weekly, fortnightly, monthly or quarterly pay made from 1 July 2024, on the
scale the operator established from the payee's declarations. The income year
selects the coefficients and rules: 2024-25 and 2025-26 use the schedule for
payments from 1 July 2024 to 30 June 2026, and 2026-27 the one for payments
from 1 July 2026. Earnings include allowances subject to withholding. The
result gives the weekly earnings used, the weekly amount and the amount for the
pay period, with the coefficients applied.

- Scale 4, for a payee who gave no TFN, needs `foreign_resident` and applies
  47% (resident) or 45% (foreign resident) to the pay's earnings, ignoring the
  cents in the earnings and in the result.
- `tax_offsets`, the yearly total claimed on the payee's Withholding
  declaration, reduces scales 2, 5 and 6 by 1.9% a week, 3.8% a fortnight,
  8.3% a month or 25% a quarter, rounded to the dollar.
- `medicare_adjustment_children` gives the Medicare levy adjustment on scales 2
  and 6 for a payee whose Medicare levy variation declaration claims it: the
  dependent children shown at question 12, or 0 for a spouse and no children.
- `extra_pay_withholding: true` adds the schedule's additional amount for a
  year of 53 weekly or 27 fortnightly pays, when the payee has asked for it.

When any of the last 3 is given, the result also shows the amount before
them and each adjustment. The schedule does not say what happens when offsets
and the adjustment exceed the amount withheld, so that is refused with a
request to check the payee's declarations and the inputs and, if they are
right, to ask the ATO how much to withhold. An optional input on a scale or
pay period the schedule does not allow it for is refused too.
Bi-monthly, daily and casual pays, a Medicare levy variation that increases
withholding, study and training support loans and the other schedules are
outside it. The tests reproduce every amount and Medicare levy adjustment in
the ATO's published sample data for both schedules, and each schedule's worked
examples; see [calculation evidence](docs/calculation-evidence.md#payg-withholding).
This worksheet was first published in the 0.1.5 wheel. The 0.2.0 wheel, which
the MCP server uses, covers weekly, fortnightly and monthly 2026-27 pays on
scales 1, 2, 3, 5 and 6 only; no release publishes the rest yet.

## Super contribution caps and pension minimums

`contribution_caps` tests one year's classified contributions against the caps.
Unused concessional cap can be applied only when the total super balance at the
previous 30 June is below $500,000. `carry_forward_applied` is only the part that
covers contributions over the year's cap (ITAA 1997 s 291-20(3)-(4)); unapplied
amounts remain available subject to the 5-year expiry rule. The caller supplies
the established eligible unused total. `concessional_available` is the maximum
potential contribution capacity, including that total, rather than the statutory
cap after the amount actually applied. The non-concessional cap is 4 times the
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
of them for scales 1 and 2, and to the schedule's 3 worked examples. See
[calculation evidence](docs/calculation-evidence.md#study-loan-withholding).

## Medicare levy and low income tax offset

`medicare_levy` applies the 2% rate and the low-income reduction in s 7 of the
Medicare Levy Act 1986 to a full-year resident single who is not entitled to
the seniors and pensioners tax offset and has no exemption: nil at or below the
threshold amount, and no more than 10% of the excess up to the phase-in limit.
The caller states whether s 9A (lump sum payments in arrears) applies, and the
worksheet refuses when it does. It covers 2024-25 to 2026-27; 2026-27 uses the
2025-26 thresholds, which the law applies to later years until amended, and its
result says so. `low_income_tax_offset` gives the ITAA 1997 s 61-115 offset from
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

These 4 worksheets are included from release 0.2.0.

## FBT car benefits

`fbt_car_statutory` applies FBTAA s 9 to one car: the base value times 20%,
times the days the car was used or available for private use, divided by the
days in the FBT year, less the recipient's payment. `cost_price` is the part of
the base value the one-third reduction applies to: the cost price of a car the
provider or an associate owned when first held, or otherwise its leased car
value, the lessor's cost price only if the lease began then and the lessor
bought the car at or about that time. `later_accessories`, the non-business
accessories fitted after the earliest holding time, are never reduced. `fbt_car_operating_cost` applies s 10: car expenses for the
holding period, lease charges for a leased car, and for an owned car (including
one under hire purchase) deemed depreciation at 200% over `effective_life_years`
(s 11(1AA): 25% for the 8-year life of cars generally, 50% for a taxi's 4 years,
a life under 2 refused, and 0 for a leased car or an owned car with no depreciated
value left)
and deemed interest at the year's benchmark rate (8.62% for 2026, 8.27% for 2027)
on its s 12 depreciated value, apportioned to the days held, times the private
use percentage, less the
recipient's payment. Under s 10(5) its taxable value is the lower of that value
and `statutory_formula_taxable_value`, the same car's
`taxable_value_before_exemption` from `fbt-car-statutory`. Its gross value and
contribution split stay those of the operating cost method; where the statutory
value is lower, the statutory worksheet's split applies.

Both cover one car held by one provider for a single holding period in the FBT
years ended 31 March 2026 and 2027, and take every valuation
fact as established: holding and private use, the base value, the business use
percentage and whether the electric cars exemption applies. A recipient's
payment cannot take the taxable value below nil, and any excess is reported,
not applied to another benefit. An exempt electric car's `taxable_value` is nil;
`taxable_value_before_exemption` is the notional value its reportable fringe
benefits amount is worked out from, which neither worksheet grosses up. The
results are taxable values before gross-up: pass them to `fbt` for the same
year as type 1 or type 2 values once that classification is established. Only the gross taxable value is rounded to
cents, so the displayed base value or deemed amounts can differ by a cent from
the figures behind it. The tests hold both
worksheets to the ATO's and the Library's worked examples; see
[calculation evidence](docs/calculation-evidence.md#fbt-car-benefits).

Each result ends its amounts with `reportable_taxable_value`, the car benefit's
contribution to the employee's reportable fringe benefits: its taxable value
before any exemption, so for an exempt electric car the notional value. It is
not a reportable fringe benefits amount. That amount is the total taxable value
of all the employee's reportable fringe benefits for the FBT year, grossed up at
the type 2 rate, and is reported only when the total exceeds $2,000; a pooled or
shared car is not reportable. One car cannot know either, so neither worksheet
grosses up or applies the threshold, and each adds a third warning saying so.

`tests/verdict_cases.rs` runs the edge cases from a practitioner's published
review of another FBT car calculator, as fabricated cars
([`tests/data/lodgeit-verdict-cases.json`](tests/data/lodgeit-verdict-cases.json),
attributed in [NOTICE](NOTICE)). Each has a case, a stated reason or both.

## Main residence

`main_residence` works out how much of an individual's capital gain or loss on
one dwelling the main residence exemption leaves taxable (ITAA 1997
Subdiv 118-B). The caller establishes the gain or loss, the ownership period,
the days the dwelling was their main residence, the days it or part of it
produced assessable income (with the floor area used), any absence choice
periods under s 118-145 and when a new home was acquired. Each day, counted
inclusively, is taxable in full, in part or not at all, and the taxable share is
the gain or loss times the taxable days over the days in the ownership period
(s 118-185).

The worksheet applies the 6-year limit on renting a former home, to the sixth
anniversary inclusive as the ATO's Roya example counts it; keeps a part used for
income before the move taxable through the absence, as the ATO's Helen example
does; gives the last 6 months before sale to both homes under s 118-140 when its
conditions hold; and under s 118-192 restarts the ownership period on the day
income use first began, refusing a gain worked out from cost when the market
value rule applies. Results keep cents; the ATO and the Library print whole
dollars. Pass `assessable_capital_gain` to `capital-gains` as a discount or
other gain once the 12-month test is established from `ownership_period_from`.
The tests hold it to the ATO's and the Library's worked examples; see
[calculation evidence](docs/calculation-evidence.md#main-residence).

## Borrowing expenses

`borrowing_expenses` works out one income year's deduction for one loan's
borrowing expenses under ITAA 1997 s 25-25. The caller establishes the
expenditure, the extent the money is used to produce assessable income, the
first day it was borrowed, the end of the original contract's period, any
repayment day and the income-producing share of other borrowing expenses
incurred that year. A total of $100 or less is deducted in the year incurred
(s 25-25(6)). Otherwise the period of the loan is the shortest of the
contract's period, the period to repayment and 5 years from the first
borrowing, every day counted, both ends included, as the ATO counts it; each
year's maximum is the remaining expenditure over the days left times the
year's days in the period, so the year of repayment takes the rest; and the
deduction is the maximum times the income-producing share.

Each maximum is rounded to cents and carried forward at that amount, as the
ATO's examples are; the Library prints whole dollars. The worksheet refuses a
holiday home or other leisure facility unless s 26-50(3) applies at all times
in the year, a loan first drawn on 29 February, and income years from 2027-28,
when s 26-155 quarantines residential rental deductions. The tests hold it to
the ATO's and the Library's worked examples; see
[calculation evidence](docs/calculation-evidence.md#borrowing-expenses).

## Division 296

`division_296` estimates one individual's Division 296 tax for 2026-27 from
their established total superannuation balance at 30 June 2027 (disregarding
LRBA amounts) and their total superannuation earnings for the year, which may be
negative. When the balance exceeds $3 million and the earnings exceed nil, it
works out the share of the balance above $3 million and the share above
$10 million, each as a percentage rounded to 2 decimal places, applies each to
the earnings, and taxes the first result at 15% and the second at a further 10%
(ITAA 1997 ss 296-40 and 296-45; Imposition Act 2026 s 5). A child recipient, a
structured settlement contribution or death by 30 June 2027 means no tax;
`liability` gives the reason for any nil result. Unlike the other worksheets,
it refuses a field it does not take, such as an opening balance. Every result
is labelled an estimate, because the Commissioner assesses the tax from what
funds report. Working out the balance or the earnings, defined benefit and
other s 296-70 interests, departing temporary residents and later years are
outside it. The tests hold it to the Explanatory Memorandum's examples; see
[calculation evidence](docs/calculation-evidence.md#division-296).

## Termination payment

`termination_payment` works out, for one life benefit employment termination
payment made in 2026-27, the tax-free part of a genuine redundancy or early
retirement scheme payment and the PAYG withholding from the rest under
Schedule 11. The tax-free limit is $13,598 plus $6,801 for each completed year
of service (ITAA 1997 s 83-170); the redundancy payment above it, and any other
excluded payment, such as compensation for unfair dismissal, is withheld from
within the $270,000 ETP cap alone. A non-excluded payment, such as a golden
handshake or payment in lieu of notice, comes second, within the smaller of
the ETP cap left and the whole-of-income cap, $180,000 less the payer's other
taxable payments to the employee in the year (s 82-10(4) and (7)). Within the
cap, 17% is withheld if the employee is their preservation age (60 for anyone
born after 30 June 1964) or older on 30 June 2027, otherwise 32%; above it,
47%; each part rounds to the nearest dollar. The caller classifies the
payments; death benefit ETPs, ETPs with a tax-free component, earlier ETPs in
the year or for the same termination, payees without a TFN, foreign residents
and the income tax assessed on the ETP are outside it. It refuses a field it
does not take. The tests hold it to the ATO's Schedule 11 examples; see
[calculation evidence](docs/calculation-evidence.md#termination-payment).

## Schedule 5 withholding

The 3 `schedule-5` commands work out withholding from a regular weekly,
fortnightly or monthly pay that includes back payments, commissions, bonuses or
similar payments, by the methods of Schedule 5 (NAT 3348), with the Schedule 1
amounts `payg-withholding` gives for the year. Method A spreads the additional
payments over 52, 26 or 12 pay periods, or for a commission or bonus for a
defined period of less than 12 months over the periods it relates to, and
multiplies the extra withholding back. Method B(i) recalculates each earlier
pay period this financial year that a back payment applies to, from the
earnings, back payment and withholding the operator gives for each. Method
B(ii) applies the year's additional payments to the average earnings to date,
for back payments for an earlier year and payments that relate to no single
pay period. Methods A and B(ii) limit the withholding from the additional
payment to 47% of it. With `study_loan`, each table amount includes the
Schedule 8 component, which this crate has for payments from 1 July 2026 only,
so earlier years refuse it. The result gives each step of the method, the
withholding on the additional payment and the pay's total. Like
`division-296`, each command refuses a field it does not take. Working holiday
makers (Schedule 15), payees without a TFN, withholding variations and a bonus
for a single pay period, which belongs in that pay's `payg-withholding`
earnings, are outside it. The tests hold the commands to the ATO's 4 worked examples in each year's
schedule; see [calculation evidence](docs/calculation-evidence.md#schedule-5).

## Working holiday makers

`working_holiday_maker_withholding` applies Schedule 15 to one weekly,
fortnightly or monthly payment made from 1 July 2024 to a working holiday
maker the operator has established (subclass 417 or 462, or a bridging visa the
schedule covers). A registered employer whose payee gave a TFN withholds the
payment, cents ignored, times the Table A rate for the column that the payments
made before it in the income year select: 15% below $45,001, 30% from $45,001
to $135,000, 37% from $135,001 to $190,000 and 45% above $190,000. The result
is rounded to the dollar, 50 cents up, except at 45%, where cents are ignored.
Without a TFN any employer withholds 45%, cents ignored. An unregistered
employer withholds at foreign resident rates, the Schedule 1 scale 3 amount
`payg-withholding` gives. Payments before of $135,000.01 to $135,000.99, which
no column covers, are refused.
`withholding_basis` names the column or scale used. Like `division-296`, it
refuses a field it does not take. A payee who claimed an exemption from
quoting a TFN, labour mobility schemes, withholding variations and checking
the visa or registration are outside it. The tests hold it to the ATO's lookup
tool example and examples 1 and 2; see
[calculation evidence](docs/calculation-evidence.md#working-holiday-makers).

## Worksheet discovery

The `worksheet-catalogue` command, `worksheet_catalogue()` from the 0.1.4 wheel,
returns supported periods with inclusive dates, required and optional inputs with units,
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

<sub>Experimental review aids, not tax, legal or financial advice or return preparation. Ryan Duguid is not a registered tax agent or BAS agent. Support covers software issues reproduced with fabricated data: do not send taxpayer information or ask for advice, return preparation, tax-treatment confirmation or lodgement. See [DISCLAIMER.md](DISCLAIMER.md).</sub>
