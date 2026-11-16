# Australian tax calculators

Australian Tax Calculators 1.0.0 is a standalone Rust binary. It replaces the
Python `australian-tax-calculators` 0.2.0 package and its `austaxcalc` import
API, with the same 14 worksheets, results and refusals, and adds 2 FBT car
worksheets, FBT loan, meal entertainment, car parking, debt waiver, expense payment, property, residual, housing, living-away-from-home allowance, board and tax-exempt body entertainment worksheets, a main residence worksheet, a borrowing expenses worksheet, a Division 296 estimate, a termination payment worksheet, 3 Schedule 5 withholding worksheets, a Schedule 15 working holiday maker worksheet, a Schedule 9 seniors and pensioners worksheet, a seniors and pensioners tax offset worksheet, a Medicare levy surcharge worksheet, a foreign resident tax worksheet, a PAYG gross-up worksheet and a hire purchase schedule that no Python release contains. Python releases through 0.1.9
contain 10 worksheets. Until the MCP server links this crate,
it uses the 0.2.0 Python release from PyPI; the
[release handoff](../../apps/aus-accounting-mcp/docs/worksheet-release.md)
keeps the engine release and MCP integration separate.

<div align="center">

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.99-66023C?logo=rust&logoColor=E5E4E2&labelColor=000000&style=flat&color=66023C)](https://www.rust-lang.org/)
[![License: Ryan Duguid Public Attribution Licence 1.0](https://img.shields.io/badge/License-Public%20Attribution%201.0-66023C?labelColor=000000&style=flat&color=66023C)](LICENSE)

</div>

Forty-one calculation worksheets for established facts: ordinary GST, resident and foreign
resident basic income tax, CGT losses and discount, the main residence exemption's taxable share of a gain or loss, one loan's borrowing expenses deduction, a hire purchase agreement's monthly notional interest, ordinary employer FBT, car fringe benefit taxable values
under the statutory formula and operating cost methods, loan, meal entertainment, car parking, debt waiver, expense payment, property, residual, housing, living-away-from-home allowance, board and tax-exempt body entertainment fringe benefit taxable values, depreciation in the start year and later years,
quarterly super guarantee, Schedule 1 PAYG withholding and the gross pay that leaves a stated net, Schedule 5 withholding from back payments, commissions and bonuses by Methods A, B(i) and B(ii), Schedule 15 withholding for working holiday makers, Schedule 9 withholding for seniors and pensioners, super contribution caps,
account-based pension minimums, study and training loan repayments, Schedule 8 study
loan withholding, the Medicare levy with its low-income, SAPTO and family reductions and full or half exemptions, the Medicare levy surcharge, the low income tax offset, the seniors and pensioners tax offset,
Division 293 tax, an estimate of Division 296 tax on large super balances and the tax-free part of a genuine redundancy payment with Schedule 11 withholding from an employment termination payment. Callers must establish every scope condition before calculating. `personal-tax-estimate` composes the personal ones into one year's tax and levies.

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
| `hire-purchase-schedule` | `amount_financed`, `instalment`, `instalments`, `balloon`, `timing`, `start_date`, `year`, `scope_confirmed` |
| `fbt` | `type_one_value`, `type_two_value`, `year_ended`, `scope_confirmed` |
| `fbt-car-statutory` | `cost_price`, `later_accessories`, `one_third_reduction`, `days_available`, `employee_contribution`, `electric_car_exempt`, `year_ended`, `scope_confirmed` |
| `fbt-car-operating-cost` | `running_costs`, `lease_charges`, `depreciated_value`, `effective_life_years`, `days_held`, `business_use`, `employee_contribution`, `statutory_formula_taxable_value`, `electric_car_exempt`, `year_ended`, `scope_confirmed` |
| `fbt-loan` | `daily_balance_total`, `interest_accrued`, `income_use`, `accrued_interest_deduction`, `year_ended`, `scope_confirmed` |
| `fbt-meal-entertainment` | `method`, `meal_entertainment_expenditure`, `year_ended`, `scope_confirmed`; with `12_week_register`, `register_fringe_benefits_value`, `register_meal_entertainment_value`, `register_first_year` and `register_years_expenditure` |
| `fbt-car-parking` | `method`, `year_ended`, `scope_confirmed`; with `commercial_parking_station`, `first_business_day_fee`, `benefits`, `lowest_fee` and `employee_contribution`; with `market_value`, `first_business_day_fee`, `benefits`, `market_value` and `employee_contribution`; with `average_cost`, `first_business_day_fee`, `benefits`, `first_day_fee`, `last_day_fee` and `employee_contribution`; with `spaces`, `space_groups`, `employees_at_start`, `employees_at_end`, `eligible_spaces_at_start`, `eligible_spaces_at_end` and `employee_contribution`; with `12_week_register`, `first_business_day_fee`, `register_value`, `availability_days` and `register_first_year` |
| `fbt-debt-waiver` | `amount_waived`, `year_ended`, `scope_confirmed` |
| `fbt-expense-payment` | `method`, `recipients_expenditure`, `amount_paid`, `employee_contribution`, `year_ended`, `scope_confirmed`; with `external`, `income_use` and `recipients_portion_deduction`; with an in-house property method, that method's inputs as for `fbt-property`; with `in_house_residual_salary_packaging` or `in_house_residual_other`, `notional_value`; with `in_house_residual_public_price`, `lowest_price`; and with every in-house method but the 2 salary packaging ones, `in_house_reduction_available` |
| `fbt-property` | `method`, `employee_contribution`, `income_use`, `contribution_deduction`, `year_ended`, `scope_confirmed`; with `in_house_trade_price` or `in_house_public_price`, `lowest_price`; with `in_house_acquired`, `arms_length_price` and `notional_value`; with `external_cost_price`, `cost_price`; with `external_expenditure`, `employer_expenditure`; with another method, `notional_value`; with an in-house method other than `in_house_salary_packaging`, also `in_house_reduction_available` |
| `fbt-residual` | `method`, `employee_contribution`, `income_use`, `contribution_deduction`, `year_ended`, `scope_confirmed`; with `in_house_salary_packaging`, `in_house_other` or `external_notional_value`, `notional_value`; with `in_house_public_price`, `lowest_price`; with `external_cost_price`, `cost_price`; with `external_expenditure`, `employer_expenditure`; with `in_house_public_price` or `in_house_other`, also `in_house_reduction_available` |
| `fbt-housing` | `method`, `employee_contribution`, `year_ended`, `scope_confirmed`; with `outside_australia`, `market_value`; with `accommodation_business`, `market_value` and `in_house`; with `base_year`, `market_value` and `tenancy_days`; with `indexed`, `prior_statutory_annual_value`, `state`, `base_year_ended` and `tenancy_days` |
| `fbt-lafha` | `method`, `allowance`, `year_ended`, `scope_confirmed`; with `maintains_home` or `fly_in_fly_out`, `accommodation_component`, `accommodation_expenses`, `food_component`, `food_expenses`, `normal_food_amount`, `adult_days` and `child_days` |
| `fbt-board` | `adult_meals`, `child_meals`, `employee_contribution`, `income_use`, `contribution_deduction`, `year_ended`, `scope_confirmed` |
| `fbt-tax-exempt-body-entertainment` | `method`, `entertainment_expenditure`, `year_ended`, `scope_confirmed`; with `per_head`, `employee_heads` and `total_heads` |
| `depreciation` | `cost`, `effective_life`, `days`, `taxable_use`, `method`, `scope_confirmed`, `year`; optional `opening_adjustable_value` |
| `quarterly-sg` | `ordinary_time_earnings`, `qualifying_contributions`, `year`, `quarter`, `scope_confirmed` |
| `payg-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed`; optional `foreign_resident` (scale 4), `tax_offsets`, `medicare_adjustment_children`, `extra_pay_withholding` |
| `payg-gross-up` | `net_pay`, `pay_period`, `scale`, `year`, `scope_confirmed`; optional `foreign_resident` (scale 4), `study_loan`; `pay_date` with `study_loan`, required for 2025-26 |
| `study-loan-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed`; `pay_date`, required for 2025-26 |
| `medicare-levy` | `taxable_income`, `section_9a_adjustment_applies`, `year`, `scope_confirmed`; optional `sapto_entitled`, `spouse_taxable_income`, `spouse_sapto_entitled`, `dependant_offset_entitled`, `dependent_children`, `medicare_levy_exemption_full_days`, `medicare_levy_exemption_half_days` |
| `medicare-levy-surcharge` | `taxable_income`, `reportable_fringe_benefits`, `income_for_surcharge_purposes`, `days_without_cover`, `year`, `scope_confirmed`; optional `spouse_income_for_surcharge_purposes`, `dependent_children` |
| `low-income-tax-offset` | `taxable_income`, `year`, `scope_confirmed` |
| `seniors-pensioners-tax-offset` | `rebate_income`, `status`, `year`, `scope_confirmed`; `spouse_rebate_income`, required unless `status` is `single`; optional `spouse_taxable_income` |
| `division-293` | `division_293_income`, `low_tax_contributions`, `year`, `scope_confirmed` |
| `division-296` | `total_super_balance_end_of_year`, `total_super_earnings`, `child_recipient`, `structured_settlement_contribution`, `died_in_year`, `year`, `scope_confirmed` |
| `termination-payment` | `redundancy_payment`, `completed_years_of_service`, `excluded_payment`, `non_excluded_payment`, `other_taxable_payments`, `date_of_birth`, `year`, `scope_confirmed` |
| `schedule-5-method-a` | `earnings`, `additional_payment`, `pay_periods`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed`; `pay_date`, with `study_loan` only and required with it for 2025-26 |
| `schedule-5-method-b-i` | `earnings`, `back_payment_periods`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed`; `pay_date`, as for Method A |
| `schedule-5-method-b-ii` | `earnings`, `additional_payment`, `year_to_date_earnings`, `pay_periods_to_date`, `earlier_additional_payments`, `earlier_additional_withholding`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed`; `pay_date`, as for Method A |
| `working-holiday-maker-withholding` | `payment`, `payments_before`, `employer_registered`, `tfn_quoted`, `pay_period`, `year`, `scope_confirmed` |
| `seniors-pensioners-withholding` | `earnings`, `pay_period`, `status`, `medicare_levy_exemption`, `year`, `scope_confirmed`; optional `study_loan`; `pay_date`, with `study_loan` only and required with it for 2025-26 |
| `study-loan-repayment` | `repayment_income`, `year`, `scope_confirmed` |
| `contribution-caps` | `total_super_balance`, `concessional_contributions`, `unused_concessional_cap`, `non_concessional_contributions`, `under_75_in_year`, `year`, `scope_confirmed` |
| `pension-minimum` | `account_balance`, `age`, `days`, `year`, `scope_confirmed` |
| `worksheet-catalogue` | none: send `{}` |
| `periods` | `kind`, the worksheet name, such as `"payg_withholding"` |
| `dgr-check` | `donations`, a list of 1 to 10,000 objects, each with `abn` or `name` or both, and optional `date` and `amount`; see [DGR check](#dgr-check) |
| `personal-tax-estimate` | `year`, `scope_confirmed`, `residency`, `taxable_income` or `assessable_labour_income` (then optional `other_assessable_income`, `work_related_deductions`, `other_deductions`), optional `tax_withheld` or, to work it out, one of `payg-withholding`, `study-loan-withholding` and `seniors-pensioners-withholding` with `pays_in_year` and optional `schedule-5-method-a`, for a resident from 2027-28 `net_labour_income` (required beside `taxable_income`), and for a full-year resident `medicare-levy` and `medicare-levy-surcharge`; optional `seniors-pensioners-tax-offset`, `study-loan-repayment` and `division-293`; see [Personal tax estimate](#personal-tax-estimate) |
| `derive` | `command`, one of `fbt`, `fbt-car-statutory`, `fbt-car-operating-cost`, `fbt-loan`, `fbt-meal-entertainment`, `fbt-car-parking`, `fbt-debt-waiver`, `fbt-expense-payment`, `fbt-property`, `division-293`, `pension-minimum`, `payg-withholding`, `schedule-5-method-a`, `schedule-5-method-b-i`, `schedule-5-method-b-ii`, `termination-payment`, `working-holiday-maker-withholding`, `study-loan-withholding`, `seniors-pensioners-withholding`, `quarterly-sg`, `borrowing-expenses`, `resident-tax`, `foreign-resident-tax`, `low-income-tax-offset`, `seniors-pensioners-tax-offset`, `medicare-levy`, `medicare-levy-surcharge`, `division-296`, `contribution-caps`, `study-loan-repayment`, `capital-gains`, `main-residence`, `fbt-residual`, `fbt-housing`, `fbt-lafha`, `fbt-board`, `fbt-tax-exempt-body-entertainment`, `gst`, `depreciation` and `hire-purchase-schedule`, and `input`, that command's arguments; see [Derivations](#derivations) |

The 2 car commands, the 11 FBT benefit commands (`fbt-loan`, `fbt-meal-entertainment`, `fbt-car-parking`, `fbt-debt-waiver`, `fbt-expense-payment`, `fbt-property`, `fbt-residual`, `fbt-housing`, `fbt-lafha`, `fbt-board` and `fbt-tax-exempt-body-entertainment`), `main-residence`, `borrowing-expenses`, `division-296`, `termination-payment`, the 3 `schedule-5` commands, `working-holiday-maker-withholding`, `seniors-pensioners-withholding`, `seniors-pensioners-tax-offset`, `medicare-levy-surcharge`, `foreign-resident-tax`, `payg-gross-up` and `hire-purchase-schedule` have no Python original; their arguments follow the same
rules. `division-296` alone takes a negative amount, in `total_super_earnings`. `main-residence` also takes dates as `YYYY-MM-DD` strings, lists of
`{"start", "end"}` periods (with `"floor_area"` for income use) and `null`
for no new home. `borrowing-expenses` takes `loan_start` as a `YYYY-MM-DD`
string and `loan_term_end` and `repaid_on` as dates or `null`. `hire-purchase-schedule` takes `instalments` as a JSON integer, `timing` as `in_arrears` or `in_advance` and `start_date` as a `YYYY-MM-DD` string, and refuses a field it does not take. `termination-payment` takes `completed_years_of_service` as a JSON integer and `date_of_birth` as a `YYYY-MM-DD` string. `schedule-5-method-b-i` takes `back_payment_periods` as a list of `{"earnings", "back_payment", "withheld"}` decimal strings, one for each earlier pay period. Every argument is required, as the Python functions had no defaults, except `depreciation`'s optional `opening_adjustable_value`: give it, as a decimal string, for a year after the asset's start time, as cost less the earlier years' decline in value; and `medicare-levy`'s optional family inputs, described under [Medicare levy](#medicare-levy-and-low-income-tax-offset). `medicare-levy` refuses a field it does not take, as does `medicare-levy-surcharge`, whose optional inputs are described under [Medicare levy surcharge](#medicare-levy-surcharge). `payg-withholding`'s optional inputs are described under [PAYG withholding](#payg-withholding), `payg-gross-up` under [PAYG gross-up](#payg-gross-up), `study-loan-withholding`'s `pay_date` under [Study loan withholding](#study-loan-withholding), `seniors-pensioners-withholding`'s `study_loan` and `pay_date` under [Seniors and pensioners](#seniors-and-pensioners) and `seniors-pensioners-tax-offset`'s spouse inputs under [Seniors and pensioners tax offset](#seniors-and-pensioners-tax-offset). The FBT benefit commands take the inputs their `method` needs and refuse any other, as [FBT loans, meal entertainment and car parking](#fbt-loans-meal-entertainment-and-car-parking), [FBT debt waivers, expense payments and property](#fbt-debt-waivers-expense-payments-and-property) and [FBT residual, housing, LAFHA, board and entertainment benefits](#fbt-residual-housing-lafha-board-and-entertainment-benefits) describe. Input over 8 MiB,
or a decimal string over 8,192 bytes, is refused.

## Derivations

`derive` shows how each amount of a worksheet in the `derive` row of the table
above was worked out. It takes `{"command": "fbt", "input": {...}}`, runs
the command on that input and returns
`{"format": "australian-tax-calculators/derivation/1", "engine", "engine_version",
"calculation", "input", "steps", "result"}`. `result` is the command's own output,
byte for byte. `steps` holds one step per amount, in the amounts' order:

```json
{"id":"return_item_16","authority":"FBT return 2026 instructions, item 16","working":"return_item_15 x fbt_rate = 45644 x 0.47 = 21452.68","value":"21452.68"}
```

`authority` is the provision or ATO page that sets the amount: a section of the
Fringe Benefits Tax Assessment Act 1986 (FBTAA), of the Income Tax Assessment
Act 1997 (ITAA 1997) or of the Superannuation (Sustaining the Superannuation
Contribution Concession) Imposition Act 2013, a clause of Schedule 7 to the
Superannuation Industry (Supervision) Regulations 1994 (SISR), a section of the
Income Tax Rates Act 1986, the Income Tax Assessment Act 1936 (ITAA 1936), the
Medicare Levy Act 1986, the A New Tax System (Medicare Levy Surcharge—Fringe
Benefits) Act 1999 or the A New Tax System (Goods and Services Tax) Act 1999
(GST Act), or of the Income Tax Assessment (1936 Act) Regulation 2015
or Regulations 2025 for the seniors and pensioners tax offset. `working` is the
formula, the same formula on the values it used and its unrounded result. Each
name in a formula is an input, a rate in `result.rates` or an earlier step,
whose unrounded result it carries; `x` multiplies, `min` takes the lower, `max`
the higher, `cents` and `dollars` round half up and `dollars_up` rounds up to
the dollar. When Medicare Levy Act s 8(3) shares the family reduction, the
reductions are worked times family income and divided by it once, so the
working carries `x family_income` and `/ family_income`. `value` is the amount, the
result rounded half up to cents. The worksheet records each step beside its
arithmetic and renders its amounts from the steps, so a step cannot differ from
its amount; `tests/derivation.rs` recomputes every amount from its working
alone. A refused input gets the
command's own refusal, any other command is refused as
`derivation.unsupported`, and a key other than `command` and `input` as
`input.field_unknown`. `payg-gross-up` is not derived: its gross pay is the one
whose own withholding leaves the net, found by search, so no step can work it
out before the withholding worked on it. `derive` `payg-withholding`, or
`study-loan-withholding` for a payee with a study loan, on its `gross_pay` to
see that withholding worked.
The plain commands' output does not change.

The PAYG withholding, Schedule 5 and termination payment steps cite the
schedules of the Taxation Administration (Withholding Schedules) Instrument for
the year, made under Schedule 1 to the Taxation Administration Act 1953,
s 15-25; the termination payment steps also cite ITAA 1997 s 82-10 and 83-170.
Schedule 1 names its parts by heading, such as 'Using a formula', and the
steps cite those headings. `floor` drops the cents where a schedule says to
ignore them. A Schedule 5 step writes in the coefficients of the
Schedule 1 (and, for a study loan, Schedule 8) row it used, and Method B(i)
the figures of each earlier pay period; `cents` wraps a tax-table amount,
which the method uses as rounded.

The working holiday maker, study loan and seniors and pensioners steps cite
the headings of Schedules 15, 8 and 9 of the same instruments, and Schedule 1
where they apply its formula or scales. With `study_loan`, a seniors and
pensioners pay adds a step for Schedule 8's component and a total that cites
the study and training support loans weekly tax table, to which the ATO's
Schedule 9 pages refer and the instruments' Schedule 9 does not. The
quarterly SG steps cite the
Superannuation Guarantee (Administration) Act 1992 as in force before 1 July
2026 and the ATO's maximum contribution base, and the borrowing expenses steps
ITAA 1997 s 25-25. A borrowing expenses step for a later year writes in each
earlier year's maximum deduction, worked on the figures that year used.
The Division 296 steps cite ITAA 1997 ss 296-15 to 296-50, the Income Tax
(Transitional Provisions) Act 1997 s 296-1 and the Superannuation (Building a
Stronger and Fairer Super System) Imposition Act 2026 s 5; a step whose
amount is nil cites the reason, and a negative very large superannuation
balance earnings component has the value null, as its amount does. The
contribution caps steps cite ITAA 1997 s 291-20, s 292-85 and s 292-90, the
study loan repayment steps the Higher Education Support Act 2003 (HESA)
ss 154-1 to 154-25, the capital gains steps ITAA 1997 s 102-5, s 102-10,
s 102-15 and s 115-100, and the main residence steps ss 118-110, 118-185,
118-190 and 118-192. `non_main_residence_days` in the main residence rates is
the sum of each day's taxable weight, which the worksheet works out from the
periods given. The GST steps cite the GST Act ss 9-70 and 9-75, the
depreciation steps ITAA 1997 ss 40-25, 40-70, 40-72, 40-75 and 40-85, and the
hire purchase steps ss 240-25 and 240-60. A hire purchase schedule's monthly
rows, their interest by income year and its implicit rate are not amounts, so
they have no step.

## Supported periods

| Worksheet | Period and boundary |
| --- | --- |
| GST | 2025-26 to 2027-28, one ordinary taxable supply at 10%; no classification or BAS |
| Resident tax | 2024-25 to 2027-28; full-year resident ordinary income, before offsets and levies |
| Foreign resident tax | 2024-25 to 2027-28; full-year foreign resident ordinary income, no tax-free threshold; no Medicare levy or low income tax offset applies |
| CGT | 2025-26 and 2026-27, resident individual's established gains and losses; losses before the 50% discount |
| Main residence | CGT events in 2024-25 to 2026-27; one dwelling disposed of by a resident individual, before the discount |
| Borrowing expenses | 2024-25 to 2026-27, before s 26-155 from 2027-28; one loan's established expenditure, use and dates |
| FBT | Years ended 31 March 2026 and 2027; established type 1 and type 2 taxable values, ordinary employer |
| FBT car benefits | Years ended 31 March 2026 and 2027; one car's established base value or operating costs, the 20% statutory percentage and a deemed depreciation rate of 200% over the established effective life |
| FBT loans | Years ended 31 March 2026 and 2027 at the statutory interest rate (8.62% and 8.27%); one loan to a current employee, its established daily balances, interest accrued and deductible share |
| FBT meal entertainment | Years ended 31 March 2026 and 2027; the established year's expenditure, by the 50/50 split or a 12 week register checked against its age and the 20% rule |
| FBT car parking | Years ended 31 March 2026 and 2027 at the car parking threshold ($11.03 and $11.48); the commercial parking station, market value, average cost, spaces or 12 week register method; the 12 week register's 10% rule is the operator's |
| FBT debt waivers | Years ended 31 March 2026 and 2027; the established amount waived |
| FBT expense payments | Years ended 31 March 2026 and 2027; one external payment or reimbursement with the otherwise deductible rule, or one in-house property or residual expense payment with the $1,000 in-house reduction; no otherwise deductible rule for in-house payments |
| FBT property | Years ended 31 March 2026 and 2027; one benefit valued by an in-house (s 42) or external (s 43) rule, with the otherwise deductible rule and the $1,000 in-house reduction; no airline transport or car property rules |
| FBT residual benefits | Years ended 31 March 2026 and 2027; one benefit's established in-house or external value, contribution and deductible share; no car, airline transport or jointly provided benefit |
| FBT housing | Years ended 31 March 2026 and 2027 at the ATO's housing indexation factors; a unit outside Australia or in an accommodation business at market value, or a statutory annual value from its base year; no remote area housing |
| FBT living-away-from-home allowances | Years ended 31 March 2026 and 2027; an allowance's established components, expenses and family members' days, at the statutory food amounts of $42 and $21 a week |
| FBT board | Years ended 31 March 2026 and 2027; the established meals at $2.00 and $1.00, contributions and deductible share |
| FBT tax-exempt body entertainment | Years ended 31 March 2026 and 2027; established non-deductible entertainment expenditure, actual or per head |
| Depreciation | 2025-26 and 2026-27, an ordinary tangible Division 40 asset's start year, or a later year from its established opening adjustable value; established life and use |
| SG | Complete quarters of 2025-26; established OTE, eligibility and qualifying contributions |
| Contribution caps | 2024-25 to 2026-27; one individual's classified contributions, no bring-forward period started in the 2 previous years |
| Pension minimum | 2024-25 to 2027-28; one account-based pension paying under SISR Schedule 7 |
| Study loan repayment | 2025-26 and 2026-27; one individual's established repayment income at the marginal rates; the formula amount before the loan-balance limit |
| Study loan withholding | Regular weekly, fortnightly, monthly or quarterly pays from 24 September 2025 on scale 1, 2, 3, 5 or 6; the Schedule 1 amount plus the Schedule 8 component |
| Medicare levy | 2024-25 to 2027-28; full-year resident; the 2% rate, the low-income reduction at the single or SAPTO thresholds, the family reduction from established spouse, offset and dependent child facts, and established full and half exemption days |
| Medicare levy surcharge | 2024-25, 2025-26 and 2026-27, and 2027-28 with no day without hospital cover (nil); full-year resident in no exemption category, single without dependants or married for the whole year; established income for surcharge purposes and days without hospital cover |
| Low income tax offset | 2024-25 to 2027-28; resident individual's established taxable income; the offset amount, not applied against tax |
| Seniors and pensioners tax offset | 2024-25 to 2027-28; an established eligible individual's rebate income and whole-year status, and a spouse's rebate income; the offset amount with any transfer from a resident spouse, not applied against tax |
| Division 293 | 2024-25 to 2027-28; established Division 293 income and low tax contributions; no defined benefit interests |
| Division 296 | 2026-27; one individual's established balance at 30 June 2027 and total superannuation earnings; no defined benefit or other s 296-70 interests; an estimate, as the Commissioner assesses |
| Termination payment | 2026-27; one life benefit ETP to a resident employee who gave a TFN, paid within 12 months; established genuine redundancy and classified ETP parts; Schedule 11 withholding, not the tax assessed |
| Schedule 5 withholding | Weekly, fortnightly or monthly pays made from 1 July 2024 to 30 June 2027 that include back payments, commissions or bonuses, on scale 1, 2, 3, 5 or 6; the study and training support loan component for pays from 24 September 2025, and from 1 July 2026 in Method B(i) |
| Working holiday makers | 2024-25 to 2026-27; one weekly, fortnightly or monthly payment to an established working holiday maker; no TFN exemption claimants or labour mobility schemes |
| Seniors and pensioners | 2024-25 to 2026-27; one weekly, fortnightly or monthly payment to an established senior or pensioner who gave a TFN, by status and Medicare levy exemption; no other tax offsets or Medicare levy adjustment |
| Personal tax estimate | Each part's own periods: 2024-25 to 2026-27, or 2025-26 and 2026-27 with a study loan, and 2027-28 without a study loan or a day without hospital cover; a full-year resident's or full-year foreign resident's tax and levies before credits, and the refund or amount payable against tax withheld when given |

Unsupported periods fail. The 2027-28 rows use rates and thresholds the law
already sets for that year; the ATO had published none of them when they were
read on 16 November 2026, and each such result but GST, the low income tax
offset, Division 293 and the pension minimum says so. A worksheet that refuses
2027-28 for one of these reasons says which: its figures were not yet published (withholding schedules,
the Medicare levy surcharge thresholds, study loan thresholds, contribution
caps, ETP caps and the Division 296 thresholds), or the law changes from 1 July
2027 in ways it does not apply (CGT, residential rental deductions under
s 26-155), or 2027-28's 366 days exceed what depreciation takes. The Medicare
levy surcharge takes 2027-28 only with no day without hospital cover, when no
surcharge applies whatever the thresholds. For a resident in 2027-28 the
personal tax estimate applies the working Australians tax offset (ITAA 1997
ss 61-155 and 61-160); it does not work out the extra tax on a minimum tax
capital gain (s 119-10). The SG worksheet does not implement post-June 2026
Payday entitlement rules. The separate Payday engine reviews timing on a supplied
liability. SMSF fund tax, Division 296 fund earnings and their attribution, the transfer balance cap, trusts and payroll
tax remain outside these worksheets.

## Foreign resident tax

`foreign_resident_tax` applies Schedule 7 Part II of the Income Tax Rates Act
1986 to a person who was a foreign resident for the whole income year: 30% to
$135,000, 37% to $190,000 and 45% above, with no tax-free threshold. It covers
2024-25 to 2027-28. The ATO publishes the 2024-25 and 2025-26 tables; 2026-27
and 2027-28 take the same rates from the Act, which ties them to that year's
second and third resident rates, and their results say the ATO had not yet
published them. A foreign resident
pays no Medicare levy and is not entitled to the low income tax offset, so the
result is basic tax only and its scope says why. Like `division-296`, it
refuses a field it does not take. See
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

## PAYG gross-up

`payg_gross_up` finds the gross pay whose Schedule 1 withholding leaves the
`net_pay` given, as for pay agreed net of tax, on the same `pay_period`,
`scale`, `year` and, for scale 4, `foreign_resident` as `payg_withholding`.
It does not invert the formulas: it searches `payg_withholding`'s own results,
so every rule and refusal of that worksheet applies, and the result gives
`net_pay`, `gross_pay` and that worksheet's amounts and coefficients for the
gross. Schedule 1 withholds whole dollars and never less from a higher pay, so
a gross leaves the net exactly when it is the net plus the withholding on it;
the search finds every such gross, of which there is always at least one.

For a payee with a study and training support loan, `study_loan` true searches
`study_loan_withholding`'s results instead, so the gross carries Schedule 8's
component and that worksheet's `pay_date` rule: required for 2025-26, whose
coefficients apply from 24 September 2025, and no 2024-25 component. Scale 4
has no component, so `study_loan` does not take it or `foreign_resident`.

Where a gross a dollar higher (for a weekly pay; more for longer pay periods)
withholds a dollar more, both leave the same net, and the worksheet refuses
with `payg.gross_up_not_unique`, listing every gross: Schedule 1 does not say
which one a net-agreed pay means, so take it from the pay agreement. Such ties
are common: each dollar of withholding added as pay rises makes one. Tax offsets,
the Medicare levy adjustment, the additional amount for 53 or 27 pays, super
and other deductions are not inputs. A net above $499,999,999,974.00, or
$333,333,333,300.00 with a study loan, is refused, so that every gross the
search tries is an amount the withholding worksheet takes. The tests gross up the net of every row of
the ATO's sample data for both schedules to the row's earnings or a tie that
names them, and the net of each of Schedule 8's examples 1 to 3 to its
earnings or a tie; see [calculation evidence](docs/calculation-evidence.md#payg-gross-up).

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
fortnightly, monthly or quarterly pay made from 24 September 2025 to a payee who declared
a study and training support loan debt. The year selects both schedules'
coefficients. A 2025-26 pay needs `pay_date` (`YYYY-MM-DD`): that year's
component coefficients are those of the Taxation Administration (Withholding
Schedules) Instrument (No. 2) 2025 (F2025L01016), which apply from 24 September
2025, so an earlier 2025-26 pay, whose component was a percentage of weekly
earnings, is refused. A pay date outside the income year is refused, and `pay_date` may be
given for 2026-27. Scale 1 takes the component for a payee
who did not claim the tax-free threshold; scales 2, 3, 5 and 6 take the one for
a payee who claimed it or is a foreign resident. The result gives the Schedule
1 amount, the component and their total. The ATO also publishes combined
coefficients and accepts either method; the tests hold the worksheet within $1
of them for scales 1 and 2, and to each schedule's 3 worked examples. A
quarterly pay follows both schedules' rule: a thirteenth of the earnings is the
weekly equivalent, and 13 times each weekly amount is withheld. Neither
schedule works a quarterly example, so the tests check hand-worked figures and
the quarterly starting earnings. See
[calculation evidence](docs/calculation-evidence.md#study-loan-withholding).
The 0.2.0 wheel, which the MCP server uses, covers 2026-27 only; no release
publishes 2025-26 yet.

## Medicare levy and low income tax offset

`medicare_levy` applies the 2% rate and the low-income reduction in s 7 of the
Medicare Levy Act 1986 to a full-year resident: nil at or
below the threshold amount, and no more than 10% of the excess up to the
phase-in limit, at the higher thresholds for a person entitled to the seniors
and pensioners tax offset (`sapto_entitled`). For a person with a spouse on the
last day of the year (`spouse_taxable_income` and `spouse_sapto_entitled`), or
one entitled to the invalid and invalid carer tax offset for their child or the
sole parent notional tax offset (`dependant_offset_entitled`), it then applies the s 8 family reduction: no levy at or below the family income
threshold, which rises for each dependent child (`dependent_children`), and
above it a reduction that shades out by 8% of the excess, shared between
spouses who are both liable, with any excess of one spouse's share passing to
the other. Last, s 9 reduces that levy by the share of the year's days for
which the person qualified for a full exemption
(`medicare_levy_exemption_full_days`) and half the share for a half exemption
(`medicare_levy_exemption_half_days`), the 2 counts the tax return asks for: a
full exemption for the whole year leaves no levy, and a half one halves it.
Foreign residents are outside it. The caller establishes each of those facts;
omitting them means a person without SAPTO, spouse, dependants or exemption. The caller also states whether s 9A
(lump sum payments in arrears) applies, and the worksheet refuses when it does.
It covers 2024-25 to 2027-28; 2026-27 and 2027-28 use the 2025-26 thresholds,
which the law applies to later years until amended, and their results say so. The tests
hold it to the ATO's Ashton and Priya examples and every threshold.
`low_income_tax_offset` gives the ITAA 1997 s 61-115 offset from
established taxable income for 2024-25 to 2027-28. Neither applies the result
against tax payable. See
[calculation evidence](docs/calculation-evidence.md#medicare-levy).

## Medicare levy surcharge

`medicare_levy_surcharge` applies the surcharge in ss 8B and 8D of the
Medicare Levy Act 1986 and the A New Tax System (Medicare Levy Surcharge -
Fringe Benefits) Act 1999 for 2024-25 to 2027-28 to a full-year resident who
was in no exemption category and was either single without dependants all
year or married all year. The tier comes from `income_for_surcharge_purposes`,
added to `spouse_income_for_surcharge_purposes` for a married person, against
the ATO's singles or family thresholds; the family thresholds rise by $1,500
for each `dependent_children` after the first. The 1%, 1.25% or 1.5% rate then
applies only to `taxable_income` plus `reportable_fringe_benefits`, not to the
net investment losses or reportable super contributions that count towards
the tier, for the `days_without_cover` as a share of the year. A married person
whose own income for surcharge purposes is at or below the s 3(1) threshold
amount pays none. A person with dependants but no spouse, a change of status
during the year, exemption categories and trustees are outside it, and it
refuses a field it does not take. The ATO had not published the 2027-28
thresholds when they were checked on 16 November 2026, so 2027-28 is taken only
with `days_without_cover` 0: ss 8B, 8C and 8D and the Fringe Benefits Act
ss 12 to 14 apply only to a period without private patient hospital cover, so
the surcharge is then nil, and any day without cover is refused as
`period.unsupported`. The tests hold it to the ATO's Tom, Josh,
Jacinta and Jill and Kevin examples and every tier boundary; see
[calculation evidence](docs/calculation-evidence.md#medicare-levy-surcharge).

## Division 293

`division_293` applies ITAA 1997 s 293-20 to one individual's established
Division 293 income and low tax contributions: when their sum exceeds $250,000,
taxable contributions are the lesser of the contributions and the excess, taxed
at 15%. It covers 2024-25 to 2027-28 and holds the ATO's worked example. Working
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

## FBT loans, meal entertainment and car parking

Each worksheet gives one category's taxable value before gross-up, for `fbt`
to gross up with the others. They take their inputs by name: each method needs
its own, and an input the method does not use, or one missing, is refused with
the method named (a string that is not a readable decimal is refused as
unreadable first). Each derives its amounts (see [Derivations](#derivations)).

`fbt_loan` applies FBTAA ss 18 and 19. `daily_balance_total` is the loan's
balance at the end of each day the employee owed it, added together, so
repayments and drawdowns are counted by the day; the notional interest is that
total times the year's statutory interest rate over the days in the year (s
136(1)), less the interest accrued. The otherwise deductible rule takes off the
gross deduction on the notional interest (`income_use` of it) less the
deduction for the interest accrued, never below nil. A loan benefit is type 2,
and the result's warning says so.

`fbt_meal_entertainment` takes the year's meal entertainment expenses, net of
employees' unreimbursed contributions, and applies 50% (`50_50_split`, s 37BA)
or the register percentage (`12_week_register`, s 37CB), kept exact and shown to
2 places. A register must be in its first valid year or one of the 4 after it,
and it ceases to be valid at the end of a year whose meal entertainment expenses
were more than 20% higher than its first valid year's (s 37CD).
`register_years_expenditure` lists each year's expenses from the first valid
year to the year before this one, oldest first, and the worksheet refuses a
register that has ceased. The operator establishes that the register was kept
as the Act requires and that no later register replaced it.

`fbt_car_parking` gives a nil taxable value unless `first_business_day_fee`,
the highest of the nearby stations' lowest all-day fees on the first business
day, is more than the year's car parking threshold (s 39A(1)(a)(iii)); a nil
result holds that taxable value alone, with no gross value or payment. It
takes one premises and the stations within 1 km of it; run it once for each.
`commercial_parking_station` values each benefit at the
day's lowest fee (s 39C), the same on every day, and `average_cost` at the
average of the fees on the days benefits were first and last provided (s 39DA);
the recipients' payments come off in total, each counted only up to its
benefit's value, never below nil. `market_value` values each benefit at the
amount a suitably qualified valuer at arm's length reported (s 39D), with the
payments taken off the same way; the operator establishes that the report came
before the declaration date in the approved form. `spaces` (s 39FA) takes
`space_groups`, each a group of spaces sharing a daily rate amount (a benefit's
value by the commercial parking station, market value or average cost method,
before any payment; s 39FC) and an availability period, with its own first
business day fee above the threshold. For each space it multiplies the rate by
the period's days over 366 and by 228, adds them, divides by 366 once and rounds
to cents. Where the election's employees average fewer than its eligible spaces
(each average from the start and end of the parking period), it scales that
total by the one over the other (s 39FB); every recipient's payment then comes
off the total, never below nil. `12_week_register`
takes the register's net value times 52 / 12 times the availability period's
days over 366 (s 39GB), the divisor the Act sets for every year. It checks only
the register's age (s 39GF); the operator establishes the record keeping, that
the number of benefits rose by no more than 10% on any day of an earlier year,
and that no later register replaced it: s 39GF(3) does not say what the 10%
rise is measured from.

Each worksheet reproduces, to the cent, the published examples for the years it
supports (FBT return 2026 instructions examples 8 and 18, and the Library's
¶3-100 and ¶3-220) and the employer's guide's car parking examples 15, 17 and 18,
and reruns the guide's earlier-year loan examples at the supported rates; see
[calculation evidence](docs/calculation-evidence.md#fbt-loan).

## FBT debt waivers, expense payments and property

These worksheets also give one category's taxable value before gross-up, for
`fbt` to gross up, take their inputs by name and derive each amount.

`fbt_debt_waiver` gives the amount waived, principal and interest together
(FBTAA s 15). The Act sets no contribution or otherwise deductible rule
against it, so the worksheet takes neither, and a debt waiver is type 2.

`fbt_property` values one benefit under the rule its `method` names. The 6
in-house rules follow s 42(1): `in_house_salary_packaging` the notional value,
`in_house_trade_price` the lowest arm's length price to manufacturers,
wholesalers or retailers, `in_house_public_price` 75% of the lowest price to
the public, `in_house_produced_other` and `in_house_other` 75% of the notional
value, and `in_house_acquired` the lesser of the provider's arm's length price
and the notional value. The 3 external rules follow s 43: `external_cost_price`,
`external_expenditure` and `external_notional_value`. 75% is rounded to cents.
The recipient's contribution comes off, never below nil; the otherwise
deductible rule (s 44) then takes off the gross deduction on the value before
the contribution (`income_use` of it, rounded to cents) less
`contribution_deduction`, never below nil. An in-house benefit not salary
packaged is then reduced by `in_house_reduction_available`, the part of the
employee's $1,000 for the year (s 62) their other in-house benefits have left,
so a run for each benefit carries the rest to the next.

The Act states no rounding. The worksheets in this section and the next, like
the car and loan worksheets, round half up to cents each amount they show (a 75% value, a gross
deduction, a statutory annual value) before the next step uses it, so the
amounts shown add up; where an amount falls on a half cent, the taxable value
can differ by a cent from unrounded arithmetic. See
[calculation evidence](docs/calculation-evidence.md#fbt-debt-waiver).

`fbt_expense_payment` takes the recipient's expenditure, the amount paid or
reimbursed and the recipient's contribution, which arises only for a payment
to a third party (s 20(a)) and is 0.00 for a reimbursement. With `external`,
the value is the payment less the contribution (s 23), and the otherwise
deductible rule (s 24) takes off the gross deduction on the expenditure less
`recipients_portion_deduction`, the deduction for the part the employee bore.
An in-house method values an in-house property expense payment (s 22A(1)): the
property's value under that `fbt-property` rule less the recipient's portion,
the expenditure less the payment net of the contribution, then the s 62
reduction. An in-house residual expense payment (s 22A(2)), such as a service
of a kind the employer sells to the public, takes the in-house residual rules
of `fbt-residual` the same way: `in_house_residual_salary_packaging` the
notional value, `in_house_residual_public_price` 75% of the lowest price to the
public and `in_house_residual_other` 75% of the notional value (ss 48 and 49,
read with s 22A(3) for a period benefit). The otherwise deductible rule for
in-house expense payments (s 24(1)(b)(i)) is not covered, and an `income_use`
or `recipients_portion_deduction` given with an in-house method is refused as
`fbt_expense_payment.in_house_deductible_not_covered`.

Which gross-up an expense payment or property benefit takes depends on
whether the provider is entitled to a GST credit for providing it; the result's
warning says so. The worksheets reproduce, to the cent, FBT return 2026
instructions examples 9, 10 and 14, the employer's guide's examples in 8.2,
9.3, 9.4, 17.3 and 17.5, and the Library's ¶3-240, ¶3-260 and ¶3-340; see
[calculation evidence](docs/calculation-evidence.md#fbt-debt-waiver).

## FBT residual, housing, LAFHA, board and entertainment benefits

Like the worksheets above, each gives one category's taxable value before
gross-up, takes its inputs by name and derives its amounts.

`fbt_residual` applies Division 12. An in-house benefit under a salary
packaging arrangement takes its notional value (`in_house_salary_packaging`, s
48(aa)); another takes 75% of the lowest price charged to the public
(`in_house_public_price`) or of its notional value (`in_house_other`). An
external benefit takes the provider's cost (`external_cost_price`), the
employer's expenditure (`external_expenditure`) or its notional value
(`external_notional_value`). The recipient's contribution comes off, then the
otherwise deductible rule (s 52): the gross deduction is `income_use` of the
value, less `contribution_deduction`, the deduction the recipient could claim
for the contribution. An in-house benefit outside salary packaging then takes
up to `in_house_reduction_available`, what is left of the employee's $1,000
reduction for the year (s 62). Car residual benefits, airline transport and
benefits provided jointly to an employee and an associate (s 138(3) and
s 52(5)) are left out.

`fbt_housing` applies Division 6. A unit outside Australia, or one in a
business that lets identical accommodation to the public, is valued at market
value (s 26(1)(a) and (b)), the latter at 75% when `in_house` says it would be
an in-house benefit. Otherwise the statutory annual value applies for the
tenancy days (s 26(1)(c)): in a base year, the tenancy's market value
annualised (s 26(2)(a)); in a later year, the previous year's value times the
State or Territory's indexation factor from the ATO's rates page (s 26(2)(b)).
`state` is a string, `NSW`, `VIC`, `QLD`, `SA`, `WA`, `TAS`, `NT` or `ACT`; the
Jervis Bay Territory counts as NSW, and Christmas Island and the Cocos
(Keeling) Islands as NT (s 28(5)). Rent comes off, never below nil. The
operator establishes the market value and whether the year is a base year (s
26(3)). Each statutory annual value is rounded to cents, the amount the next
year's `prior_statutory_annual_value` takes. Remote area housing is exempt (s 58ZC), so the worksheet does not value
it.

`fbt_lafha` applies Division 7. Where s 31 (an employee who keeps an Australian
home, for up to 12 months at one work location) or s 31A (fly-in fly-out or
drive-in drive-out) applies, the allowance is reduced by its exempt
accommodation component, the accommodation component up to the expenses, and
its exempt food component (s 31H): the food component less the applicable
statutory food total, up to the food expenses. The statutory food total is $42
a week for each eligible family member aged 12 or more before the year began
and $21 for each younger one (s 136(1)), counted by the day, less the normal
food amount taken into account. Otherwise (`other`) the taxable value is the
allowance (s 31B). The operator establishes the declarations and substantiation
(s 31G), the Commissioner's reasonable food amount and the days the employee
lived at home (s 31(3)).

`fbt_board` values each meal at $2.00, or $1.00 for a person under 12 before
the year began (s 36), less the recipients' contributions, then applies the
otherwise deductible rule (s 37). Board benefits are type 2, and the result's
warning says so.

`fbt_tax_exempt_body_entertainment` takes the non-deductible entertainment
expenditure attributable to employees and their associates (s 39): the actual
amount or, where that cannot easily be worked out, a per head share (the
employer's guide, 15.9). No contribution or reduction applies.

Each worksheet reproduces, to the cent, the FBT return 2026 instructions'
examples 11, 12, 13, 15 and 16, the employer's guide's examples in 10.5, 11.5,
11.6, 13.2, 15.9, 18.4 and 18.7, and the Library's ¶3-120, ¶3-160 and ¶3-180.
Where the guide misprints 2 figures, the worksheets give the correct amounts:
$15,685.95 under s 26(2)(a), not 10.5's single-employee $15,687.70, and
$2,008, 10% of $20,080, where 18.6 prints $2,080; see
[calculation evidence](docs/calculation-evidence.md#fbt-residual).

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

## Hire purchase schedule

`hire_purchase_schedule` sets out a hire purchase agreement month by month
for the hirer, who is the notional buyer under ITAA 1997 s 240-17(2). The
caller establishes the amount financed (the price less any deposit or
trade-in), the level monthly instalment, the number of instalments (1 to
360), any balloon, whether instalments fall at each month's end
(`in_arrears`) or start (`in_advance`), and the start date, which must fall in
the income year given (2024-25 to 2026-27). The worksheet does not take an
interest rate: it works out the implicit interest rate, the monthly compound
rate at which the payments repay the amount financed, as step 3 of
s 240-60(1) does, and each month's notional interest is the outstanding
notional loan principal, less an instalment due at the month's start, times
that rate (steps 1, 2 and 4). The payments themselves are not deductible
(s 240-55).

The result gives the amount financed, the total payments and the total
interest, the implicit monthly rate and 12 times it as `nominal_annual_rate`
(both to 16 places),
`balloon_date` (the end of the term) and `schedule`, one row a month with its
`period_start`, `period_end`, `payment_date`, `opening`, `interest`,
`payment` (the instalment, paid on `payment_date`), `balloon` (nil until the
last month, paid on `balloon_date`: with the last instalment in arrears, a
month after it in advance), `principal` and `closing`. A month runs to the same day of the next
month, or that month's last day when it is shorter. Each month shows the
notional interest to date rounded half up to cents, less what the months
before show, so a cent of rounding never compounds: the months' interest adds
up to the total charges, each is within a cent of its amount at the rate
found, and the last balance is nil. That rate is found to 64 halvings and
leaves less than half a cent after the last payment, so a month can differ
from its amount at the exact implicit rate by slightly more than a cent at
the largest amounts. The worksheet refuses payments that imply a rate below
nil or of 100% a month or more, and a rate it cannot find closely enough to
leave a nil balance to the cent.

`interest_by_income_year` totals the interest of the months wholly in each
income year, one entry a year with its `income_year`, `first_instalment`,
`last_instalment` and `interest`, each the sum of those rows' `interest`. A
month runs from `period_start` up to `period_end`, so a month ending on
1 July falls wholly in the year before. A month that spans 30 June is listed
in `interest_across_30_june` instead, with its `instalment`, `interest`, the 2
`income_years` and the `days` it runs in each: s 240-60(2) relates the part
of a month in an income year to it in accordance with accounting principles,
which the caller must apply to that month. The years and the spanning months
add up to the total interest; from a start on the 1st of a month no month
spans 30 June. Fees, GST, irregular payments, termination amounts other than
the balloon and the notional seller are out of scope; see
[calculation evidence](docs/calculation-evidence.md#hire-purchase-schedule).

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
Schedule 8 component for the pay, as `study-loan-withholding` gives it: a
2025-26 pay needs `pay_date`, on or after 24 September 2025, when that year's
coefficients apply, and 2024-25, with no Schedule 8 here, refuses it.
`pay_date` without `study_loan` is refused, as it selects nothing else. Method
B(i) refuses a 2025-26 study loan: it recalculates earlier periods, and the
schedule does not say which coefficients apply to periods before 24 September
2025; Method A takes it. The result gives each step of the method, the
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

## Seniors and pensioners

`seniors_pensioners_withholding` applies Schedule 9 (NAT 4466), the tax table
for seniors and pensioners, to one weekly, fortnightly or monthly payment made
from 1 July 2024 to a payee the operator has established it covers: 67 or
older at the end of the financial year, or a veteran or war widow or widower
of at least 60 with a Department of Veterans' Affairs pension or supplement,
who claimed the seniors and pensioners tax offset on a Withholding declaration
as an Australian resident. The year selects the schedule: 2024-25 and 2025-26
use the one for payments from 1 July 2024 to 30 June 2026, and 2026-27 the one
from 1 July 2026. `status` (`single`, `illness_separated` or `couple`) picks
column 2, 3 or 4 of the tax table, and `medicare_levy_exemption` (`none`,
`full` or `half`) the schedule's scales for a payee who claimed an exemption on
a Medicare levy variation declaration, so the coefficients are scale 1 to 9,
which the result names. The weekly earnings, formula, rounding and fortnightly
and monthly conversions are Schedule 1's, as the schedule says. Other tax
offsets, which the schedule values only by combining its ready reckoner's
weekly amounts, and the Medicare levy adjustment, which it leaves to another
table, are not inputs, and quarterly pays, for which it has no rule, are
refused. Like `division-296`, it refuses a field it does not take. Payees who
gave no TFN (`payg-withholding` scale 4), super income streams (Schedule 13
too) and lump sum leave loading (Schedule 5) are outside it. The tests
reproduce the worked example and all 6,360 amounts in the weekly tax table of
each schedule, and check the exemption scales by hand; see
[calculation evidence](docs/calculation-evidence.md#seniors-and-pensioners).

For a payee who declared a study and training support loan debt, Schedule 9
says to refer to the study and training support loans weekly tax table, which
adds the component for a payee claiming the tax-free threshold, as every
Schedule 9 payee does, to the withholding amount. With `study_loan` true the
result adds that component from Schedule 8's formula on the same weekly
earnings, converted to the pay period as Schedule 8 says, and shows the
Schedule 9 amount, the component and their total. As in `study-loan-withholding`,
2025-26 needs the `pay_date`, on or after 24 September 2025, and 2024-25 is
refused, because Schedule 8 then used a percentage of weekly earnings that
this crate does not hold. A full or half Medicare levy exemption keeps the
component; a payee who claimed a Medicare levy reduction for a spouse or
dependants and low family income has none, so omit `study_loan` for them.
Without `study_loan`, or with it false, the amounts and rates are as before. The tests
hold the component to column 2 of every row of the weekly table from 1 July
2026 ($985 to $3,600), its example and the 2025-26 one, and Schedule 8's
examples.

## Seniors and pensioners tax offset

`seniors_pensioners_tax_offset` works out the seniors and pensioners tax offset
(ITAA 1936 s 160AAAA) for 2024-25 to 2027-28 from the regulations' formulas,
for a person the caller has established meets the pension or age condition.
`rebate_income` is the person's whole-dollar rebate income and `status`
(`single`, `illness_separated` or `couple`) gives the base amount: $2,230,
$2,040 or $1,602. The shading-out threshold and cut-out follow from the base
amount and the year's lowest marginal tax rate, so 2026-27, at 15%, and
2027-28, at 14%, have higher thresholds than the ATO has yet published, and
their results say so. A member of
a couple also gives `spouse_rebate_income`: entitlement then tests half the
couple's rebate income, and the amount the person's own. The offset falls by
12.5 cents a dollar above the threshold and is rounded up to the dollar, as the
ATO rounds it. With `spouse_taxable_income`, a spouse entitled to the offset
whose whole taxable income is ordinary taxable income at the resident scale
passes their unused amount (regulations s 12),
which raises this person's base amount and threshold once their own offset is
below its base amount (s 12(1)(e)). A person whose spouse status changed in
the year, trustees, a person also entitled to the beneficiary tax offset and
applying the non-refundable offset against tax are outside it, and like `division-296` it refuses a field it does
not take. An offset of at least $1 is the SAPTO entitlement `medicare-levy`
takes as `sapto_entitled`. The tests hold it to the ATO's 9 examples, its
thresholds and every cut-out; see
[calculation evidence](docs/calculation-evidence.md#seniors-and-pensioners-tax-offset).

## Personal tax estimate

`personal-tax-estimate` composes one full-year resident's or full-year foreign
resident's income tax and levies for a year from the worksheets above, without working any part out itself. It
takes `year`, `scope_confirmed`, `residency` and whole-dollar `taxable_income`
once, and one object of each part's own inputs under the part's command name:
`medicare-levy` and `medicare-levy-surcharge` for a full-year resident, and
`seniors-pensioners-tax-offset`, `study-loan-repayment` and `division-293` when
they apply. It adds the shared inputs to each part, runs `resident-tax` and
`low-income-tax-offset` on them alone, and gives `medicare-levy` the SAPTO
entitlement the offset worked out (at least $1). Confirming scope confirms the
scope each part states.

With `"residency": "foreign_resident"` it runs `foreign-resident-tax` in place
of `resident-tax`, and `division-293` when given. A foreign resident has no low
income tax offset (ITAA 1997 s 61-110(1)(a)) and pays no Medicare levy or
surcharge, which ITAA 1936 s 251S(1)(a) levies only on a person resident at
some time in the year, so those lines are nil and their inputs are refused as
`input.field_unknown`. It refuses `seniors-pensioners-tax-offset` for a foreign
resident, and `study-loan-repayment`, because a foreign resident's repayment
includes the overseas levy on worldwide income, which that worksheet excludes;
both as `estimate.residency_unsupported`.

```json
{"year": "2025-26", "scope_confirmed": true, "residency": "full_year_resident",
 "taxable_income": "60000",
 "medicare-levy": {"section_9a_adjustment_applies": false},
 "medicare-levy-surcharge": {"reportable_fringe_benefits": "0",
   "income_for_surcharge_purposes": "60000", "days_without_cover": 0}}
```

The result's `amounts` are `basic_income_tax`, the seniors and pensioners tax
offset and then the low income tax offset each applied up to the tax left
(ITAA 1997 s 63-10(1) items 5 and 20), `income_tax_after_offsets`,
`medicare_levy`, `medicare_levy_surcharge`, `study_loan_repayment`,
`total_before_credits` and `division_293_tax`, which stays out of the total
because the ATO assesses it on its own notice. Neither offset reduces the levy
or the surcharge. `working` names the worksheet and amount behind each line,
`components` holds each part's own result unchanged, with its scope, sources
and warnings, and `not_composed` lists the parts not given or not applying, each nil.
The total is before PAYG withholding and other credits, refundable and other
offsets, the transfer of unused SAPTO to a spouse and any refund.

In place of `taxable_income` it can take whole-dollar `assessable_labour_income`
(salary, wages and the other payments ITAA 1997 s 25-130(4) lists) with
optional `other_assessable_income`, `work_related_deductions` (the kinds
s 25-130(2)(c) to (g) count) and `other_deductions` (the rest, including the
insurance premiums and association fees s 25-130(3) leaves out), and work out taxable
income as assessable income less deductions, nil where they equal or exceed it
(s 4-15(1)). For a resident from 2026-27 the deductions include the standard
deduction for work-related expenses: the lesser of $1,000 and assessable labour
income, reduced, but not below nil, by the work-related deductions (s 25-130,
inserted by Act No. 49 of 2026, Schedule 4). The amounts then start with
`total_assessable_income`, `standard_deduction`, `total_deductions` and
`taxable_income`, and the parts run on that taxable income. Classifying income
and deductions and any tax loss are the caller's. With `tax_withheld`, the
year's PAYG withholding credit (TAA 1953 Sch 1 s 18-15(1)), the amounts end
with `tax_withheld`, `refund` and `amount_payable`, comparing it with
`total_before_credits`; other credits and refundable offsets are not in it.

In place of `tax_withheld` it can work out the tax one employer withholds
from regular pays of the same amount. Give one regular pay's inputs under
`payg-withholding`, `study-loan-withholding` (with a study loan) or
`seniors-pensioners-withholding` (Schedule 9), without `year` and
`scope_confirmed`, and `pays_in_year`, the number of those pays in the income
year: at most 53 weekly, 27 fortnightly, 12 monthly or 4 quarterly. Add
`schedule-5-method-a` with `additional_payment` and `pay_periods` for one
bonus, commission or back payment paid with one of them; it takes the pay's
earnings, period, scale and study loan. Method A is refused beside
`seniors-pensioners-withholding`, a quarterly pay (the Schedule 5 and 9
commands take none) and a pay that declares `tax_offsets`,
`medicare_adjustment_children` or `extra_pay_withholding`, which Method A here
does not take. The tax withheld is the pay's
withholding times `pays_in_year`, plus Method A's withholding on the
additional payment, and the amounts add `withholding_per_pay` and
`additional_payment_withholding` before `tax_withheld`; each part's result is
under `components`. A pay on a study loan needs `study-loan-repayment`, which
its component goes towards; a scale that does not match the residency (3, or
4 with `foreign_resident`, for a foreign resident) is refused as
`estimate.scale_mismatch`, and Schedule 9 or a study loan for a foreign
resident as `estimate.residency_unsupported`. A study loan pay in 2025-26 is
refused as `period.unsupported`: that year's Schedule 8 coefficients changed
on 24 September 2025, and the estimate takes no year whose coefficients
changed after 1 July. `extra_pay_withholding` needs `pays_in_year` of 53 weekly or 27 fortnightly.
Pays of other amounts, other payers' withholding and withholding variations
are not in it: give `tax_withheld` for those.

```json
{"year": "2026-27", "scope_confirmed": true, "residency": "full_year_resident",
 "assessable_labour_income": "85000",
 "payg-withholding": {"earnings": "3269.23", "pay_period": "fortnightly", "scale": 2},
 "pays_in_year": 26,
 "medicare-levy": {"section_9a_adjustment_applies": false},
 "medicare-levy-surcharge": {"reportable_fringe_benefits": "0",
   "income_for_surcharge_purposes": "84000", "days_without_cover": 0}}
```

For a resident from 2027-28 it applies the working Australians tax offset
(ITAA 1997 Subdivision 61-E, inserted by Act No. 49 of 2026, Schedule 3)
before every other offset (s 63-10(1) item 3): the lesser of $250 and the
basic income tax the resident rates give on net labour income, which is nil
unless net labour income exceeds the $18,200 tax-free threshold. Net labour
income is labour amounts less labour deductions (s 61-155(2)). Give it as
whole-dollar `net_labour_income`, which is required beside `taxable_income`;
with `assessable_labour_income` it is worked out as that income less
`work_related_deductions` and the standard deduction, which takes
`other_assessable_income` and `other_deductions` to hold no labour amounts or
labour deductions, such as business or personal services income, or the union
fees and income protection premiums s 25-130(3) leaves out of work-related
deductions: give `net_labour_income` where they do. Worked out, it can be
negative; given, it is at least 0, so give 0 for a labour loss, which also
gives no offset. The amounts add
`net_labour_income`, `working_australians_tax_offset` and
`working_australians_tax_offset_applied` after `basic_income_tax`.
`net_labour_income` is refused for an earlier year or a foreign resident, who
has no such offset (s 61-155(1)(a)). The estimate does not work out the extra
income tax on a minimum tax capital gain (s 119-10, for CGT events from
1 July 2027), so its 2027-28 scope excludes one. The ATO had not
published the 2027-28 withholding schedules or the surcharge and study loan
thresholds on 16 November 2026, so give `tax_withheld`, hospital cover for the
whole year and no study loan.

A refusal from a part keeps the part's code, and its message starts with the
part's command, such as a sole parent's `medicare-levy-surcharge:
dependent_children is not an input ...`: there is no partial total. The
estimate itself refuses a part-year resident and a working holiday maker, whose
rates and apportionment no worksheet here works out, as
`estimate.residency_unsupported`, an unknown residency as `input.residency_invalid`, Medicare levy
exemption days beside the surcharge, whose scope excludes them, as
`estimate.surcharge_with_exemption`, parts that describe different families
(a spouse in one part but not another, 2 spouse taxable incomes, or dependants
without a spouse, whom the surcharge worksheet excludes) as
`estimate.family_mismatch`, and a shared input or `sapto_entitled` given inside
a part, or a field the part does not take, as `input.field_unknown`. It does
not compare the counts of dependent children the levy and the surcharge each
take, which the law defines differently: the levy counts those covered by ITAA
1997 s 961-5(1)(c), and the surcharge a person's dependants. `derive` does not explain it.
The tests hold it to the ATO's figures for tax with the levy, the surcharge,
SAPTO and the foreign resident rates, and to the sum of its parts called alone; see
[calculation evidence](docs/calculation-evidence.md#personal-tax-estimate).

## DGR check

`dgr-check` says whether the recipient of a donation is shown as a deductible
gift recipient (DGR), from what date and as at what data date, and lists the
ITAA 1997 Division 30 conditions that DGR status leaves open. It never says a
payment is deductible. It takes `donations`, a list of 1 to 10,000 objects, each
with an `abn` (11 digits, spaces allowed, passing ABN Lookup's check digit
test), a `name` from the ledger, or both, and optionally the payment `date` as
`YYYY-MM-DD` and the `amount` as a decimal string. The first donation below
uses ABN Lookup's worked example ABN, which is not in the data:

```json
{"donations": [
  {"abn": "51 824 753 556", "date": "2025-03-01", "amount": "250.00"},
  {"name": "Arthritis Foundation"}
]}
```

Each result keeps its `index` and `input` and gives `register`:

- `shown_as_dgr`: the ABN has DGR entries in the data. `dgr_entries` gives each
  entry's start date (`from`) and, in `entry_for`, whether it is for the entity
  itself (endorsed, or a DGR the law names) or only for the operation of a
  named fund, authority or institution (or one the extract gives no name for,
  shown as such); `on_payment_date` reads them for `date` as `shown`,
  `shown_for_a_fund`, `not_shown` (entries start after the payment),
  `abn_cancelled` (the ABN was cancelled on or before it), `after_data_date` or
  `no_date`. A `name` beside the ABN gives `name_matches_register`.
- `not_shown_as_dgr`: the ABN is not in the data. The finding says it 'is not
  shown as a deductible gift recipient in the data of' the data date, never
  that it is not a DGR.
- `abn_needed`: a name alone gives up to 10 `candidates`, each `unconfirmed`,
  whose names hold every word of it, and `candidates_total`. The ABN on the
  receipt settles which entity was paid.

`conditions` lists what remains: that the payment is a gift, the special
conditions of the s 30-15 item and of the recipient's DGR item, a record of the
gift, the gift being made to the fund where the data shows the entity endorsed
only for one on the payment date, or now when there is no date (s 30-17,
s 30-229(2)), and, for a gift made before 1 July 2024 or undated, the $2 minimum
that Act No. 58, 2026, Sch 1 removed from that date. Each is `open` except the
minimum, which is `met` or `fails` when the amount and date settle it. Each
donation's result also carries the `data_date`, the extract's `source_url` and
the limits below, so it can be read alone; the response carries the data's
source, extract time and licence, the scope, the limits and the sources.

It refuses a list that is empty, too long or not a list as
`input.donations_invalid`, an ABN that is malformed or fails the check digit
test as `input.abn_invalid`, a name that is not 1 to 200 characters with an
ASCII letter or digit as `input.name_invalid`, a donation with neither as
`input.missing_field`, and any other field as `input.field_unknown`; dates and
amounts are refused as in the
other commands. One refused donation refuses the list. `derive` does not
explain it.

The data, `data/dgr-register.jsonl`, comes from the
[ABN Bulk Extract](https://data.gov.au/data/dataset/abn-bulk-extract) of
11 November 2026 (extract time 2026-11-11T12:20:05), published by the
Australian Business Register under Creative Commons Attribution 3.0 Australia.
Contains deductible gift recipient entries from the ABN Bulk Extract,
Australian Business Register, licensed under CC BY 3.0 AU; changes: only ABNs
with DGR entries are kept, with their status, names and DGR entries. Of
20,658,780 ABN records, 31,876 have DGR entries; no individual (sole trader)
record had one. The extract also holds entity types, individuals' names,
addresses' state and postcode, ASIC numbers and GST status, none of which is
kept. Its limits:

- It holds current details only. An endorsement that ended before the extract
  is not in it, and no entry has an end date, so a payment before an entry's
  start date is `not_shown` rather than read as no endorsement.
- It gives no DGR item, so the gift conditions of the recipient's category stay
  open. ABN Lookup shows the item.
- A DGR named in the law need not be endorsed or have an ABN, and an entry in
  the register, or its absence, does not decide deductibility (s 30-229(1),
  note 1).
- ABN Lookup holds each ABN's history and the current position; confirm a
  result there before relying on it.

The engine takes the data date from the data's first line and nowhere else.
Stale data shows itself in every result: `data.data_date` and each finding name
the extract date, and a payment after it reads `after_data_date`.

To refresh it:

1. Download `public_split_1_10.zip` and `public_split_11_20.zip` from the
   [ABN Bulk Extract](https://data.gov.au/data/dataset/abn-bulk-extract) page,
   both of one week's extract.
2. Run the loop in `examples/dgr_register.rs`'s comment from this directory. It
   streams the 20 XML files into `data/dgr-register.jsonl`, keeps only records
   with DGR entries and stops on anything listed there, including a file whose
   record count differs from its `RecordCount` or files from 2 extracts. Its
   own tests run with `cargo test`.
3. Run `cargo test`. `tests/dgr_check.rs` pins the extract time and the
   readings of named ABNs, so it fails until updated: recheck those ABNs at ABN
   Lookup, then update the tests, this section, `NOTICE`, the changelog and
   the [calculation evidence](docs/calculation-evidence.md#dgr-check) with the
   new date and counts.

Do not commit the raw extract. The cross-check against ABN Lookup's DGR listing
and the sources read are in the calculation evidence.

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
