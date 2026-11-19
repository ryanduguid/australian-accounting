# Australian tax calculators

Australian Tax Calculators 1.0.0 is a standalone Rust binary. It replaces the
Python `australian-tax-calculators` 0.2.0 package and its `austaxcalc` import
API, with the same 14 worksheets, results and refusals, and adds 2 FBT car
worksheets, FBT loan, meal entertainment, car parking, debt waiver, expense payment, property, residual, housing, living-away-from-home allowance, board and tax-exempt body entertainment worksheets, an entertainment classifier, a main residence worksheet, a borrowing expenses worksheet, a Division 296 estimate, a termination payment worksheet, an ETP tax free component worksheet, an unused leave payment worksheet, 3 Schedule 5 withholding worksheets, a Schedule 15 working holiday maker worksheet, a Schedule 9 seniors and pensioners worksheet, a seniors and pensioners tax offset worksheet, a Medicare levy surcharge worksheet, a foreign resident tax worksheet, a PAYG gross-up worksheet, a hire purchase schedule and a business capital expenditure worksheet that no Python release contains. Python releases through 0.1.9
contain 10 worksheets. MCP server releases through 0.9.0 use the 0.2.0 Python
release from PyPI; later ones run this executable, requiring the exact version
they were tested against, for the same 14 worksheets. The unreleased native
build in this repository links this crate. The
[release handoff](../../apps/aus-accounting-mcp/docs/worksheet-release.md)
keeps the engine release and MCP integration separate.

<div align="center">

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.99-66023C?logo=rust&logoColor=E5E4E2&labelColor=000000&style=flat&color=66023C)](https://www.rust-lang.org/)
[![License: Ryan Duguid Public Attribution Licence 1.0](https://img.shields.io/badge/License-Public%20Attribution%201.0-66023C?labelColor=000000&style=flat&color=66023C)](LICENSE)

</div>

Fifty-one calculation worksheets for established facts: ordinary GST, resident and foreign
resident basic income tax, CGT losses and discount, the main residence exemption's taxable share of a gain or loss, one loan's borrowing expenses deduction, a hire purchase agreement's notional interest, a chattel mortgage's interest schedule and input tax credit, a loan's statement with daily interest at changing rates, business capital expenditure (black hole and start-up costs) under s 40-880, ordinary employer FBT, car fringe benefit taxable values
under the statutory formula and operating cost methods, loan, meal entertainment, car parking, debt waiver, expense payment, property, residual, housing, living-away-from-home allowance, board and tax-exempt body entertainment fringe benefit taxable values, the FBT, deduction and GST credit treatment of entertainment expense lines, depreciation in the start year and later years, a depreciating asset's balancing adjustment with its reduction for private use and CGT event K7, a trade-in's GST on both sides with the old asset's balancing adjustment and the replacement's cost,
quarterly super guarantee, Schedule 1 PAYG withholding and the gross pay that leaves a stated net, Schedule 5 withholding from back payments, commissions and bonuses by Methods A, B(i) and B(ii), Schedule 15 withholding for working holiday makers, Schedule 9 withholding for seniors and pensioners, super contribution caps,
account-based pension minimums, study and training loan repayments, Schedule 8 study
loan withholding, the Medicare levy with its low-income, SAPTO and family reductions and full or half exemptions, the Medicare levy surcharge, the low income tax offset, the seniors and pensioners tax offset, the lump sum payment in arrears tax offset,
Division 293 tax, an estimate of Division 296 tax on large super balances and the tax-free part of a genuine redundancy payment with Schedule 11 withholding from an employment termination payment, the invalidity and pre-July 83 segments of an employment termination payment's tax free component, the assessable share, Schedule 7 withholding and income statement label of unused leave paid on termination, and the income measures the surcharge, Division 293 tax, the offsets, study loan repayments and the super co-contribution are tested against. Callers must establish every scope condition before calculating. `personal-tax-estimate` composes the personal ones into one year's tax and levies,
and `fbt-return` composes the FBT ones into an employer's FBT return.

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
runtime is needed. PyPI still provides the historical Python release; `pip
install` does not install this binary.

## Install

Each `australian-tax-calculators/v*` release on GitHub Releases carries the
executable for x86_64 Linux (static, musl) and x86_64 Windows, built and
attested by Release Policy's
[`release-binary.yml`](https://github.com/ryanduguid/release-policy/blob/main/docs/binary-consumers.md).
Release Policy's installer, at its v0.5.0 commit, downloads one, verifies it and
installs it in one command. On x86_64 Linux:

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/ryanduguid/release-policy/d7039efe6d8c441f1f213c674902909e80535599/scripts/install.sh | sh -s -- ryanduguid/australian-accounting australian-tax-calculators/v1.0.0 --modify-path
```

In PowerShell on x86_64 Windows:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/ryanduguid/release-policy/d7039efe6d8c441f1f213c674902909e80535599/scripts/install.ps1))) ryanduguid/australian-accounting australian-tax-calculators/v1.0.0 -ModifyPath
```

It needs the [GitHub CLI](https://cli.github.com/) (`gh`), with no sign-in. It
installs nothing unless `gh attestation verify` confirms that `SHA256SUMS` and
the archive were signed by `release-binary.yml` for this repository's tag on a
GitHub-hosted runner, and the archive matches its line in `SHA256SUMS`: a
changed archive or digest is refused. The executable goes into
`${XDG_DATA_HOME:-~/.local/share}/australian-tax-calculators` or
`%LOCALAPPDATA%\Programs\australian-tax-calculators`. `--modify-path` adds a
line putting that folder on `PATH` to the startup file of the shell `$SHELL`
names (for bash on Linux, `~/.bashrc`, which a bash login shell reads only if
`~/.profile` or `~/.bash_profile` sources it), and `-ModifyPath` adds it to your
user `Path`, both for shells started afterwards; leave it off and the installer
prints the folder instead. The command runs the script as
GitHub serves it at that commit, so read it there first. The installer does
not pin the commit the tag names: to check that too, verify by hand with both
digests pinned, as the release-binary page's 'Downloading a release' shows.
Other platforms build from source, as under Build and check. `australian-tax-calculators
--version` prints the version. `.github/workflows/calculators-install.yml`
runs both commands each week, and with the archive or `SHA256SUMS` altered in
transit, which they must refuse.

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
| `hire-purchase-schedule` | `amount_financed`, `balloon`, `timing`, `start_date`, `year`, `scope_confirmed`; `instalment`, `instalments` and optionally `frequency`, or `segments` in their place; optionally `deposit`, `trade_in`, `payout` and `gst`; see [Hire purchase schedule](#hire-purchase-schedule) |
| `chattel-mortgage-schedule` | `amount_financed`, `instalments`, `balloon`, `timing`, `start_date`, `year`, `scope_confirmed`; `instalment` or `annual_rate`; optionally `frequency`, `deposit`, `trade_in` and `gst`; see [Chattel mortgage schedule](#chattel-mortgage-schedule) |
| `loan-transactions` | `opening_balance`, `annual_rate`, `start_date`, `end_date`, `transactions`, `year`, `scope_confirmed`; optionally `rate_changes`; see [Loan transactions](#loan-transactions) |
| `business-capital-expenditure` | `items`, `year`, `scope_confirmed`; see [Business capital expenditure](#business-capital-expenditure) |
| `balancing-adjustment` | `cost`, `termination_value`, `asset`, `years`, `year`, `scope_confirmed`; for a car, `car_limit`; see [Balancing adjustment](#balancing-adjustment) |
| `trade-in` | `cost`, `asset`, `years`, `trade_in_allowance`, `trade_in_taxable`, `replacement_price`, `replacement_asset`, `replacement_creditable_acquisition`, `arms_length`, `involuntary_disposal`, `year`, `scope_confirmed`; `car_limit`, `adjusted_input_tax_credit`, `full_input_tax_credit`, `replacement_luxury_car_tax` and `creditable_purpose` where the facts need them; see [Trade-in](#trade-in) |
| `dasp-tax` | `tax_free_component`, `taxed_element`, `untaxed_element`, `working_holiday_maker_contributions`, `year`, `scope_confirmed`; see [DASP tax](#dasp-tax) |
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
| `fbt-entertainment-classifier` | `lines`, `income_tax_exempt`, `year_ended`, `scope_confirmed`; `method`, required when a line is meal entertainment; with `12_week_register`, `income_tax_register_percentage` |
| `depreciation` | `cost`, `effective_life`, `days`, `taxable_use`, `method`, `scope_confirmed`, `year`; optional `opening_adjustable_value` |
| `quarterly-sg` | `ordinary_time_earnings`, `qualifying_contributions`, `year`, `quarter`, `scope_confirmed` |
| `payg-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed`; optional `foreign_resident` (scale 4), `tax_offsets`, `medicare_adjustment_children`, `extra_pay_withholding` |
| `payg-gross-up` | `net_pay`, `pay_period`, `scale`, `year`, `scope_confirmed`; optional `foreign_resident` (scale 4), `study_loan`; `pay_date` with `study_loan`, required for 2025-26 |
| `study-loan-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed`; `pay_date`, required for 2025-26 |
| `medicare-levy` | `taxable_income`, `section_9a_adjustment_applies`, `year`, `scope_confirmed`; optional `sapto_entitled`, `spouse_taxable_income`, `spouse_sapto_entitled`, `dependant_offset_entitled`, `dependent_children`, `medicare_levy_exemption_full_days`, `medicare_levy_exemption_half_days` |
| `medicare-levy-surcharge` | `taxable_income`, `reportable_fringe_benefits`, `income_for_surcharge_purposes`, `days_without_cover`, `year`, `scope_confirmed`; optional `spouse_income_for_surcharge_purposes`, `dependent_children` |
| `low-income-tax-offset` | `taxable_income`, `year`, `scope_confirmed` |
| `seniors-pensioners-tax-offset` | `rebate_income`, `status`, `year`, `scope_confirmed`; `spouse_rebate_income`, required unless `status` is `single`; optional `spouse_taxable_income` |
| `lump-sum-arrears-tax-offset` | `taxable_income`, `net_capital_gain`, `accrual_years`, `year`, `scope_confirmed`; see [Lump sum payment in arrears tax offset](#lump-sum-payment-in-arrears-tax-offset) |
| `division-293` | `division_293_income`, `low_tax_contributions`, `year`, `scope_confirmed` |
| `income-tests` | `taxable_income`, `assessable_fhss_released_amount`, `family_trust_distribution_amounts`, `reportable_fringe_benefits_section_57a`, `reportable_fringe_benefits_other`, `reportable_employer_super_contributions`, `deductible_personal_super_contributions`, `excess_concessional_contributions`, `net_financial_investment_loss`, `net_rental_property_loss`, `tax_free_government_pensions`, `target_foreign_income`, `exempt_foreign_employment_income`, `child_support_paid`, `assessable_income`, `business_deductions`, `residency`, `year`, `scope_confirmed`; see [Income tests](#income-tests) |
| `division-296` | `total_super_balance_end_of_year`, `total_super_earnings`, `child_recipient`, `structured_settlement_contribution`, `died_in_year`, `year`, `scope_confirmed` |
| `termination-payment` | `redundancy_payment`, `completed_years_of_service`, `excluded_payment`, `non_excluded_payment`, `other_taxable_payments`, `date_of_birth`, `year`, `scope_confirmed` |
| `etp-tax-free-component` | `payment`, `employment_start`, `termination_date`, `last_retirement_day`, `ill_health`, `medical_certification`, `year`, `scope_confirmed`; see [ETP tax free component](#etp-tax-free-component) |
| `unused-leave-payment` | `annual_leave`, `leave_loading`, `long_service_leave`, `leave_accrual_start`, `termination_reason`, `working_holiday_maker`, `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed`; optional `foreign_resident` (scale 4); `annual_leave_before_18_august_1993`, `leave_loading_before_18_august_1993`, `long_service_leave_before_16_august_1978` and `long_service_leave_16_august_1978_to_17_august_1993` where the accrual start and reason need them |
| `schedule-5-method-a` | `earnings`, `additional_payment`, `pay_periods`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed`; `pay_date`, with `study_loan` only and required with it for 2025-26 |
| `schedule-5-method-b-i` | `earnings`, `back_payment_periods`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed`; `pay_date`, as for Method A |
| `schedule-5-method-b-ii` | `earnings`, `additional_payment`, `year_to_date_earnings`, `pay_periods_to_date`, `earlier_additional_payments`, `earlier_additional_withholding`, `pay_period`, `scale`, `study_loan`, `year`, `scope_confirmed`; `pay_date`, as for Method A |
| `working-holiday-maker-withholding` | `payment`, `payments_before`, `employer_registered`, `tfn_quoted`, `pay_period`, `year`, `scope_confirmed` |
| `seniors-pensioners-withholding` | `earnings`, `pay_period`, `status`, `medicare_levy_exemption`, `year`, `scope_confirmed`; optional `study_loan`; `pay_date`, with `study_loan` only and required with it for 2025-26 |
| `study-loan-repayment` | `repayment_income`, `year`, `scope_confirmed` |
| `contribution-caps` | `total_super_balance`, `concessional_contributions`, `non_concessional_contributions`, `under_75_in_year`, `year`, `scope_confirmed`, and either `unused_concessional_cap` or `earlier_concessional_contributions` with `defined_benefit_interest`; with the latter, optional `proposed_concessional_contribution` |
| `pension-minimum` | `account_balance`, `age`, `days`, `year`, `scope_confirmed` |
| `worksheet-catalogue` | none: send `{}` |
| `periods` | `kind`, the worksheet name, such as `"payg_withholding"` |
| `dgr-check` | `donations`, a list of 1 to 10,000 objects, each with `abn` or `name` or both, and optional `date` and `amount`; see [DGR check](#dgr-check) |
| `personal-tax-estimate` | `year`, `scope_confirmed`, `residency`, `taxable_income` or `assessable_labour_income` (then optional `other_assessable_income`, `work_related_deductions`, `other_deductions`), optional `tax_withheld` or, to work it out, one of `payg-withholding`, `study-loan-withholding` and `seniors-pensioners-withholding` with `pays_in_year` and optional `schedule-5-method-a`, for a resident from 2027-28 `net_labour_income` (required beside `taxable_income`), and for a full-year resident `medicare-levy` and `medicare-levy-surcharge`; optional `seniors-pensioners-tax-offset`, `study-loan-repayment` and `division-293`; see [Personal tax estimate](#personal-tax-estimate) |
| `fbt-return` | `employer`, `year_ended`, `scope_confirmed` and `benefits`, a list of objects each with `employee`, `command` (an FBT benefit worksheet), `input` (that worksheet's arguments), `gst_type` (`type_1` or `type_2`) and `excluded`; see [FBT return](#fbt-return) |
| `derive` | `command`, one of `fbt`, `fbt-car-statutory`, `fbt-car-operating-cost`, `fbt-loan`, `fbt-meal-entertainment`, `fbt-car-parking`, `fbt-debt-waiver`, `fbt-expense-payment`, `fbt-property`, `division-293`, `pension-minimum`, `payg-withholding`, `schedule-5-method-a`, `schedule-5-method-b-i`, `schedule-5-method-b-ii`, `termination-payment`, `etp-tax-free-component`, `unused-leave-payment`, `working-holiday-maker-withholding`, `study-loan-withholding`, `seniors-pensioners-withholding`, `quarterly-sg`, `borrowing-expenses`, `resident-tax`, `foreign-resident-tax`, `low-income-tax-offset`, `seniors-pensioners-tax-offset`, `medicare-levy`, `medicare-levy-surcharge`, `division-296`, `contribution-caps`, `study-loan-repayment`, `capital-gains`, `main-residence`, `fbt-residual`, `fbt-housing`, `fbt-lafha`, `fbt-board`, `fbt-tax-exempt-body-entertainment`, `gst`, `depreciation`, `hire-purchase-schedule`, `chattel-mortgage-schedule`, `loan-transactions`, `lump-sum-arrears-tax-offset`, `income-tests`, `business-capital-expenditure`, `balancing-adjustment`, `trade-in` and `dasp-tax`, and `input`, that command's arguments; see [Derivations](#derivations) |

The 2 car commands, the 11 FBT benefit commands (`fbt-loan`, `fbt-meal-entertainment`, `fbt-car-parking`, `fbt-debt-waiver`, `fbt-expense-payment`, `fbt-property`, `fbt-residual`, `fbt-housing`, `fbt-lafha`, `fbt-board` and `fbt-tax-exempt-body-entertainment`), `main-residence`, `borrowing-expenses`, `division-296`, `termination-payment`, `etp-tax-free-component`, `unused-leave-payment`, the 3 `schedule-5` commands, `working-holiday-maker-withholding`, `seniors-pensioners-withholding`, `seniors-pensioners-tax-offset`, `medicare-levy-surcharge`, `foreign-resident-tax`, `payg-gross-up`, `hire-purchase-schedule`, `chattel-mortgage-schedule`, `loan-transactions`, `lump-sum-arrears-tax-offset`, `fbt-entertainment-classifier`, `income-tests`, `business-capital-expenditure`, `balancing-adjustment`, `trade-in` and `dasp-tax` have no Python original; their arguments follow the same
rules. `division-296` alone takes a negative amount, in `total_super_earnings`. `main-residence` also takes dates as `YYYY-MM-DD` strings, lists of
`{"start", "end"}` periods (with `"floor_area"` for income use) and `null`
for no new home. `borrowing-expenses` takes `loan_start` as a `YYYY-MM-DD`
string and `loan_term_end` and `repaid_on` as dates or `null`. `hire-purchase-schedule` takes `instalments` as a JSON integer, `timing` as `in_arrears` or `in_advance`, `start_date` as a `YYYY-MM-DD` string and `segments`, `payout` and `gst` as described under [Hire purchase schedule](#hire-purchase-schedule), and refuses a field it, a segment, the payout or the GST facts do not take. `business-capital-expenditure` takes `items` as a list of objects, and refuses a field it or an item does not take. `balancing-adjustment` takes `years` as a list of objects and a car's `car_limit` as an object or `null`, and refuses a field it, a row or `car_limit` does not take; `trade-in` takes them as it does, and refuses an optional input its other facts do not call for. `termination-payment` takes `completed_years_of_service` as a JSON integer and `date_of_birth` as a `YYYY-MM-DD` string. `unused-leave-payment` takes `leave_accrual_start` as a `YYYY-MM-DD` string, `termination_reason` as `normal`, `genuine_redundancy`, `early_retirement_scheme` or `invalidity`, and its 4 accrual parts as optional decimal strings, described under [Unused leave payment](#unused-leave-payment). `schedule-5-method-b-i` takes `back_payment_periods` as a list of `{"earnings", "back_payment", "withheld"}` decimal strings, one for each earlier pay period. Every argument is required, as the Python functions had no defaults, except `depreciation`'s optional `opening_adjustable_value`: give it, as a decimal string, for a year after the asset's start time, as cost less the earlier years' decline in value; and `medicare-levy`'s optional family inputs, described under [Medicare levy](#medicare-levy-and-low-income-tax-offset). `medicare-levy` refuses a field it does not take, as does `medicare-levy-surcharge`, whose optional inputs are described under [Medicare levy surcharge](#medicare-levy-surcharge). `payg-withholding`'s optional inputs are described under [PAYG withholding](#payg-withholding), `payg-gross-up` under [PAYG gross-up](#payg-gross-up), `study-loan-withholding`'s `pay_date` under [Study loan withholding](#study-loan-withholding), `seniors-pensioners-withholding`'s `study_loan` and `pay_date` under [Seniors and pensioners](#seniors-and-pensioners) and `seniors-pensioners-tax-offset`'s spouse inputs under [Seniors and pensioners tax offset](#seniors-and-pensioners-tax-offset). `income-tests` requires every component and refuses a field it does not take; see [Income tests](#income-tests). The FBT benefit commands take the inputs their `method` needs and refuse any other, as [FBT loans, meal entertainment and car parking](#fbt-loans-meal-entertainment-and-car-parking), [FBT debt waivers, expense payments and property](#fbt-debt-waivers-expense-payments-and-property) and [FBT residual, housing, LAFHA, board and entertainment benefits](#fbt-residual-housing-lafha-board-and-entertainment-benefits) describe. `fbt-entertainment-classifier` takes `lines` as a list of line objects and refuses a field it does not take, as [FBT entertainment classifier](#fbt-entertainment-classifier) describes. Input over 8 MiB,
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
Income Tax Rates Act 1986, the Income Tax (Transitional Provisions) Act 1997, the Income Tax Assessment Act 1936 (ITAA 1936), the
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
see that withholding worked. `fbt-entertainment-classifier` is not derived
either: each line and attendee class carries its own authority, from the row
of the decision table that classified it.
The plain commands' output does not change.

The PAYG withholding, Schedule 5 and termination payment steps cite the
schedules of the Taxation Administration (Withholding Schedules) Instrument for
the year, made under Schedule 1 to the Taxation Administration Act 1953,
s 15-25; the termination payment steps also cite ITAA 1997 s 82-10 and 83-170.
The ETP tax free component steps cite ITAA 1997 ss 82-140 to 82-155.
The unused leave payment steps cite ITAA 1997 ss 83-10 to 83-90, the headings
of the ATO's Schedule 7 for payments from 1 July 2026, its STP Phase 2
employer reporting guidelines, the Superannuation Guarantee (Administration)
Act 1992 ss 6(1) and 10A(1) and the ATO's qualifying earnings table.
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
they have no step. The income tests steps cite ITAA 1997 s 995-1 and
s 293-20, ITAA 1936 s 6(1), HESA s 154-5, Schedule 3 to the A New Tax System
(Family Assistance) Act 1999, FBTAA s 135N and the Superannuation (Government
Co-contribution for Low Income Earners) Act 2003 s 8; `floor` drops the cents
of a section 57A employer's fringe benefits at 53%, as the ATO's worksheets do.
The business capital expenditure steps cite s 40-880(2) to
(4) and write in each item's amount, extent and years, as the items are a
list; each item's row cites its own paragraphs. The balancing adjustment
steps cite ITAA 1997 ss 40-85, 40-285, 40-290, 40-300, 40-325, 40-370 and
104-240, and write in each year's decline and reduction; a step that is nil
because s 40-370(1) Note 2 or s 118-5 applies cites that provision. The
trade-in steps add GST Act ss 9-70, 9-75, 11-30, 69-10 and 132-5 and ITAA
1997 ss 27-80, 27-95, 40-185, 40-230 and 40-305.

## Supported periods

| Worksheet | Period and boundary |
| --- | --- |
| GST | 2025-26 to 2027-28, one ordinary taxable supply at 10%; no classification or BAS |
| Resident tax | 2024-25 to 2027-28; full-year resident ordinary income, before offsets and levies |
| Foreign resident tax | 2024-25 to 2027-28; full-year foreign resident ordinary income, no tax-free threshold; no Medicare levy or low income tax offset applies |
| CGT | 2025-26 and 2026-27, resident individual's established gains and losses; losses before the 50% discount |
| Main residence | CGT events in 2024-25 to 2026-27; one dwelling disposed of by a resident individual, before the discount |
| Borrowing expenses | 2024-25 to 2026-27, before s 26-155 from 2027-28; one loan's established expenditure, use and dates |
| Balancing adjustment | Balancing adjustment events in 2024-25 to 2026-27, before Act No. 49 of 2026's CGT changes from 1 July 2027; one depreciating asset's established cost, termination value and each year's decline and deduction, with s 40-370 for a car from 2015-16; no pooled or written-off assets |
| Trade-in | 2025-26 and 2026-27, at the car limits of $69,674 and $69,883; one depreciating asset traded in at arm's length for a replacement first held and used that year; established GST status, credits, allowance and price |
| DASP tax | Payments in 2025-26 to 2027-28; one departing Australia superannuation payment's established components, at the ordinary or working holiday maker rates; no excess untaxed roll-over amount |
| Business capital expenditure | 2020-21 to 2027-28, items incurred from 1 July 2020 under s 40-880 as amended by Act No 92 of 2020; each item's established capital nature, business, taxable purpose extent, start-up facts and exclusions |
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
| FBT entertainment classifier | Years ended 31 March 2026 and 2027; an ordinary taxable employer's entertainment expense lines with established facts, split per head between attendee classes; no tax-exempt bodies, Subdivision 32-B exceptions, salary packaging or reportable fringe benefits amounts |
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
| Lump sum payment in arrears tax offset | Years of receipt 2013-14 to 2026-27; a full-year resident individual's established eligible lump sums by accrual year, recent accrual years from 2012-13 at resident rates and earlier distant years at their average; the offset amount, not applied against tax, without the Medicare levy and surcharge relief |
| Division 293 | 2024-25 to 2027-28; established Division 293 income and low tax contributions; no defined benefit interests |
| Income tests | 2024-25, 2025-26 and 2026-27; a full-year resident's established and classified components; the measures, not the thresholds and tests they feed |
| Division 296 | 2026-27; one individual's established balance at 30 June 2027 and total superannuation earnings; no defined benefit or other s 296-70 interests; an estimate, as the Commissioner assesses |
| Termination payment | 2026-27; one life benefit ETP to a resident employee who gave a TFN, paid within 12 months; established genuine redundancy and classified ETP parts; Schedule 11 withholding, not the tax assessed |
| ETP tax free component | 2026-27 and 2027-28; one life benefit ETP relating to one continuous period of employment; established ill-health, medical certificates and last retirement day; the invalidity and pre-July 83 segments and the taxable component, not the tax or withholding |
| Unused leave payment | 2026-27; unused leave paid on one termination to an employee who is not a working holiday maker; established reason, accrual start, accrual parts and normal earnings; Schedule 7 withholding and assessable shares, not the tax assessed |
| Schedule 5 withholding | Weekly, fortnightly or monthly pays made from 1 July 2024 to 30 June 2027 that include back payments, commissions or bonuses, on scale 1, 2, 3, 5 or 6; the study and training support loan component for pays from 24 September 2025, and from 1 July 2026 in Method B(i) |
| Working holiday makers | 2024-25 to 2026-27; one weekly, fortnightly or monthly payment to an established working holiday maker; no TFN exemption claimants or labour mobility schemes |
| Seniors and pensioners | 2024-25 to 2026-27; one weekly, fortnightly or monthly payment to an established senior or pensioner who gave a TFN, by status and Medicare levy exemption; no other tax offsets or Medicare levy adjustment |
| Personal tax estimate | Each part's own periods: 2024-25 to 2026-27, or 2025-26 and 2026-27 with a study loan, and 2027-28 without a study loan or a day without hospital cover; a full-year resident's or full-year foreign resident's tax and levies before credits, and the refund or amount payable against tax withheld when given |

Unsupported periods fail. The 2027-28 rows use rates and thresholds the law
already sets for that year; the ATO had published none of them when they were
read on 16 November 2026, and each such result but GST, the low income tax
offset, Division 293, the pension minimum and business capital expenditure
says so. A worksheet that refuses
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

Instead of the unused total, `contribution_caps` can work it out from
`earlier_concessional_contributions`: one `{"year": "2021-22",
"concessional_contributions": "13000.00"}` entry for each year from the 5th
before the year to the one before it, and any earlier years back to 2018-19,
without a gap. A year whose contributions exceed its cap also takes
`total_super_balance`, at the 30 June before it. The result gives each year's
cap (`concessional_cap_2021_22`) and unused cap (`unused_cap_2021_22`, ITAA 1997
s 291-20(6) and (7)), any unused cap an earlier year's increase applied,
earliest year first (`applied_2019_20_in_2021_22`, s 291-20(3) to (5)), what
each of the previous 5 years carries into the year, and their total,
`unused_concessional_cap`, from which the amounts above follow. After the
year's contributions and an optional `proposed_concessional_contribution`, it
gives the amount applied from each year, what is left of each
(`remaining_2021_22`), what lapses at the end of the year
(`unused_cap_expiring`) and the year's own unused cap; `unused_cap_expiry`
lists each with the year at whose end it lapses. At or above $500,000 just
before the year nothing is applied (s 291-20(3)(b)): the general cap is
reported and the amounts still age. `defined_benefit_interest` is required with
the years, and true is refused: Subdivision 291-C and s 291-370 change what
counts for a defined benefit interest. Years before 2018-19, a total across
years, a gap, and a year over its cap without its balance or without the years
its increase drew on are refused, each naming what to give. The worksheet
reports cap arithmetic only; it does not say what anyone should contribute.
The tests hold it to the ATO's Sam example and its carry-forward table.

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

## FBT entertainment classifier

`fbt-entertainment-classifier` takes an ordinary taxable employer's
entertainment expense lines and works out, for each line and each class of
attendee (employees, their associates, and others such as clients), whether it
is entertainment, the fringe benefit and its type or the s 41 exemption, the
income tax deduction and the GST credit, and for each fringe benefit whether it
is an excluded fringe benefit, left out of reporting (`excluded`: meal
entertainment is, under FBTAA s 5E(3)(a); other entertainment is not). Each
answer cites its provision, and
each view taken from TR 97/17 or TR 2007/12 is cited by paragraph as the
Commissioner's view, not the law.

Each line gives `cost` (a decimal string), `gst_status` (`includes_gst` or
`no_gst`), `provided` (`food_or_drink`, `recreation`,
`accommodation_or_travel_for_food_or_drink` or
`accommodation_or_travel_for_recreation`), `venue`
(`business_premises_working_day`, `business_premises_other_day` or
`elsewhere`), `paid_by` (`employer` or `reimbursed`) and `attendees`, such as
`{"employees": 6, "associates": 2, "others": 2}`; a food or drink line, and an
accommodation or travel line to do with food or drink, also gives `occasion` (`social_function`, `business_meal`,
`with_other_entertainment`, `light_refreshments` or `travel_meal`), and any
line may give `infrequent_and_irregular`.

2 tables in `src/engine/fbt_benefits/entertainment.rs` decide, and each row
reads against the ruling it cites. The first gives the line's character from
what was provided and the occasion: meal entertainment, recreation or not
entertainment. The second's first matching row gives each class's outcome
from that character, the employer's valuation method, the class and whether
s 41 can reach it, in the order TR 97/17 Part B asks: an election under
Division 9A first, then who was entertained, then s 41.

The classifier leaves a fact it lacks undetermined and names it, never
guessing: food or drink, or accommodation or travel to do with it, with no
occasion, and an employee's share of something
other than food or drink provided on the business premises on a working day,
where s 41 turns on whether it is property consumed there. Their cost is
`undetermined_cost`, outside the other totals. The minor benefits exemption is
shown only as possible: s 58P(1)(e)'s $300 is the Act's, `infrequent_and_irregular`
is the operator's statement, and the rest of s 58P(1)(f) is the employer's
judgement, so the totals treat the benefit as a fringe benefit.

On a reimbursed line with more than one employee, the employees' share is
`expense_payment_and_property`: the reimbursed employee's is an expense payment
benefit and each other employee's a benefit to that employee (TR 97/17 para
98); give the reimbursed employee's share as its own line to separate them.
With the number of employees unknown, the type is `undetermined`.

Under the 50/50 split or 12 week register method, the meal entertainment
lines' cost is `meal_entertainment_expenditure`, repeated in
`fbt_meal_entertainment_input` for `fbt-meal-entertainment`, which values it;
the deduction and GST credit follow ITAA 1936 s 51AEA or 51AEB and GST Act
s 69-5(3A). `income_tax_register_percentage` is the register percentage of
s 51AEB(2).

It refuses an employer exempt from income tax
(`fbt_entertainment_classifier.tax_exempt_body`; use
`fbt-tax-exempt-body-entertainment`), a line spanning classes without a head
count for each (`line_not_split`), a cost without its GST status
(`gst_status_invalid`), a meal entertainment line with no `method`
(`method_required`), the 12 week register without its percentage
(`register_percentage_invalid`), and a negative or non-finite cost
(`input.amount_invalid`). It classifies facts; it does not advise on how to
arrange an event. See [calculation evidence](docs/calculation-evidence.md#fbt-entertainment-classifier).

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

`hire_purchase_schedule` sets out a hire purchase agreement period by period
for the hirer, who is the notional buyer under ITAA 1997 s 240-17(2). The
caller establishes the amount financed (the notional loan principal: the
consideration less any deposit or trade-in, s 240-25(3)), the instalments,
any balloon, whether instalments fall at each period's end (`in_arrears`) or
start (`in_advance`), and the start date, which must fall in the income year
given (2024-25 to 2026-27). The worksheet does not take an interest rate: it
works out the implicit interest rate, the compound rate at which the payments
repay the amount financed, as step 3 of s 240-60(1) does, and each period's
notional interest is the outstanding notional loan principal, less an
instalment due at the period's start, times that rate for the period (steps
1, 2 and 4). The payments themselves are not deductible (s 240-55).

The instalments are either a level `instalment`, their number (`instalments`)
and a `frequency` (`weekly`, `fortnightly`, `monthly`, `quarterly` or
`annual`; monthly when omitted), or `segments`: a list of runs of level
instalments, each an object with `start` (`YYYY-MM-DD`), `count`, `amount`
(`0.00` for a payment holiday) and `frequency`. The first segment starts on
`start_date` and each later one where the one before it ends; segments are
all weekly or fortnightly, or all monthly, quarterly or annual. The term is
at most 30 years: 360 months or 1,565 weeks.

Periods are counted in months from the start date's day of the month (or the
month's last day when it is shorter), or in weeks of 7 days. Each
instalment's period is an arrangement payment period (s 240-70(1)); an annual
period is 2 periods of 6 months (s 240-70(2)). The rate found is a monthly
rate (weekly for weekly and fortnightly instalments), and a period of n months
or weeks has the rate (1 + r)^n - 1, so a quarter compounds 3 months. A month
with no instalment, in a payment holiday or the first half of a year, is its
own period with nil payment.

The result gives the amount financed, the total payments and the total
interest, `implicit_monthly_rate` (or `implicit_weekly_rate`) and 12 (or 52)
times it as `nominal_annual_rate` (both to 16 places), `balloon_date` (the
end of the term) and `schedule`, one row a period with its `period_start`,
`period_end`, `payment_date` (`null` when nothing is paid), `opening`,
`interest`, `payment` (the instalment), `balloon` (nil until the last period,
paid on `balloon_date`: with the last instalment in arrears, at the end of
its period in advance), `principal` and `closing`; `instalment` numbers the rows.
Each period shows the notional interest to date rounded half up to cents,
less what the periods before show, so a cent of rounding never compounds:
the periods' interest adds up to the total charges, each is within a cent of
its amount at the rate found, and the last balance is nil. That rate is found
to 64 halvings and leaves less than half a cent after the last payment, so a
period can differ from its amount at the exact implicit rate by slightly more
than a cent at the largest amounts. The worksheet refuses payments that imply
a rate below nil or of 100% a month (or week) or more, and a rate it cannot
find closely enough to leave a nil balance to the cent.

`interest_by_income_year` totals the interest of the periods wholly in each
income year, one entry a year with its `income_year`, `first_instalment`,
`last_instalment` and `interest`, each the sum of those rows' `interest`. A
period runs from `period_start` up to `period_end`, so a period ending on
1 July falls wholly in the year before. A period that spans 30 June is listed
in `interest_across_30_june` instead, with its `instalment`, `interest`, the
2 `income_years` and the `days` it runs in each: s 240-60(2) relates the part
of a period in an income year to it in accordance with accounting principles,
which the caller must apply to that period. The years and the spanning
periods add up to the total interest.

`deposit` and `trade_in` (an amount the financier credits as paid, such as a
trade-in allowance) are paid at or before the start for the cost of the
goods, and taken as paid or credited on the start date, which
`gst.first_consideration_date` and the start journal use. They do not change
the loan; with either, or `gst`, the result adds `deposit`, `trade_in` and
`consideration`, the amount financed plus both.

`journals` gives balanced journal lines, each `{"account", "debit",
"credit"}` in cents: `start` (the asset at the consideration, less any input
tax credit on the price, against the liability at the amount financed, the
deposit from `bank` and the `trade_in_allowance`), one entry in `periods` for
each schedule row's interest (to `interest_expense`, against
`hire_purchase_liability`), dated at the period's end, one entry in
`payments` for each payment on the day it is made (from `bank`, off the
liability), a `payout` entry, and the same lines totalled by `income_year` in
`by_income_year`: the interest by its period, with the periods spanning
30 June in `across_30_june`, and each payment by its day, so an instalment
paid on 1 July counts in the year it starts though the interest it pays is
the year before's. The liability is carried at the notional loan principal
outstanding, so it closes at nil. `gst_booked` says whether an input tax
credit is in the lines.

`payout`, an object with `date`, `amount` and `known_at_start`, is an early
payout that replaces every payment due on or after its date, which must be
the end of a period before the end of the term: Division 240, as read on
18 November 2026, gives no notional interest for part of a period a payout
cuts short. `known_at_start` must be `false`, stating that the payout was
neither known nor expected at the start, so the reasonable estimate of
termination amounts then was nil and the scheduled payments set the implicit
rate (s 240-60(1) step 3 and (3)). A payout known or expected at the start,
or one no reasonable estimate could then be made of (s 240-60(4) re-estimates
at the end of each of the notional seller's income years), is refused.
`total_payments` and `total_interest` stay the scheduled amounts to the end
of the term. The schedule then runs to the payout, and the result adds `payout`,
`interest_to_payout`, `payments_before_payout`, `balance_at_payout` and
`payout_excess` or `payout_shortfall`, with `payout.consequence` stating which
way Subdivision 240-G moves the difference (s 240-105 for the notional
seller, s 240-110 for the hirer, to the extent s 240-110(3) allows) and
`payout.review` what a reviewer must establish. A payout with `gst` is
refused.

`gst` holds the hirer's GST facts: `registered`; when true, `accounting_basis`
(`cash` or `accruals`), `creditable_purpose` (a fraction from 0 to 1) and
`asset` (`passenger_car`, a car GST Act s 69-10 applies to, or `other`); and
for a passenger car `first_used_year`, `gst_inclusive_market_value`,
`luxury_car_tax` and `quotes_abn_for_luxury_car_tax`. With every fact a
figure turns on, the result adds `total_consideration` (every component of
the supply, the notional interest included, for an agreement entered into
from 1 July 2012), `gst_payable` (1/11 of it, less luxury car tax for a car,
rounded to the nearest cent, 0.5 up, as GST Act s 9-90 rounds GST, with
the goods and the credit component taken as recorded on one tax invoice),
`full_input_tax_credit` (that GST, s 11-25), `input_tax_credit` (times the
creditable purpose, rendered to cents half up) and `credit_on_interest`
(1/11 of the notional interest times the creditable purpose), the journals'
allocation of the credit to the interest, booked through
`deferred_interest_gst`; it is not a credit of its own.
`gst.attributable` states the tax period: the whole credit up front, on a
cash basis too (GST Act ss 29-10(1) and 158-5). A missing fact gives no
figure, and `gst.missing` names it; an unregistered hirer's credit is nil.

`first_used_year` may come before the agreement's year, as for a car first
used under an earlier hire. A passenger car worth more than the car limit for
the year it is first used
($69,674 for 2024-25 and 2025-26, $69,883 for 2026-27), where the hirer may
not quote an ABN for luxury car tax, gives no figure either: `gst.review`
says why. GST Act s 69-10(1) caps the credit on the car at 1/11 of that limit,
and the ATO's GST issues register 'Financial services - questions and
answers' (issue 1.17) applies the cap under a hire purchase agreement, but the
sources read do not settle whether the cap also takes in the GST on the
credit component, which the same register (issue 1.1) treats as a separate
supply of finance.

The ATO's Albert example ('GST – Hire purchase and leasing') prints a credit
of $3,654.54, one-eleventh of $40,200; the worksheet gives $3,654.55, as
s 9-90 rounds $3,654.5454... to the nearest cent, 0.5 up, whether the price
and the credit component are one taxable supply on the invoice (s 9-90(1)) or
2 (s 9-90(2)).

Fees, charges and stamp duty, substituted accounting periods, the car's cost
for Division 40, agreements entered into before 1 July 2012, GST adjustments
after the start and the notional seller are out of scope; see
[calculation evidence](docs/calculation-evidence.md#hire-purchase-schedule).

## Chattel mortgage schedule

`chattel_mortgage_schedule` sets out a loan to buy goods, secured by a
chattel mortgage over them, for the purchaser, who owns the goods from the
purchase: it is not a hire purchase agreement (ITAA 1997 s 995-1), so
Division 240 does not apply. The caller establishes the amount financed, the
number of `instalments` and their `frequency` (as for `hire_purchase_schedule`,
monthly when omitted), the `balloon`, `timing`, the `start_date` (the day of
the purchase and the loan, in the income year given, 2024-25 to 2026-27) and
either the `instalment` or a stated `annual_rate`. Given the instalment, the
rate is the one the payments imply, found as `hire_purchase_schedule` finds
it. Given `annual_rate`, for monthly or weekly instalments in arrears only,
the rate a period is `annual_rate / 12` (or `/ 52`), the result adds
`instalment` (the level instalment at that rate, rounded half up to the cent)
and `final_adjustment` (the balance that rounding leaves, added to the last
instalment), and the rates are `monthly_rate` (or `weekly_rate`) and
`nominal_annual_rate`.

Each period's interest is the balance owing times the rate for the period,
the actuarial method TR 93/16 accepts, shown as `hire_purchase_schedule`
shows it, in `schedule` and as a printable Markdown table in `statement`.
`interest_by_income_year` and `interest_across_30_june` total and set apart
the periods as `hire_purchase_schedule` does. The interest is deductible
under s 8-1 to the extent the money is used to gain or produce assessable
income, which the caller establishes; where the lender's statements give the
interest, that figure is used (TR 93/16 para 2).

`deposit`, `trade_in` and `gst` take the facts `hire_purchase_schedule` takes;
with any of them the result adds `deposit`, `trade_in` and `price`. A
registered purchaser's credit is on the price alone, as the loan is an input
taxed financial supply (GST Act s 40-5(1)): `gst_payable` (1/11 of the price,
less luxury car tax for a car, rounded to the cent), `full_input_tax_credit`
(for a passenger car worth more than the car limit and not quoted for, no
more than 1/11 of the limit, s 69-10(1)) and `input_tax_credit` (times the
creditable purpose). `gst.attributable` states the tax period, from the ATO's
GST issues register (issue 1.21). A rate below nil or of 100% a month or
more, both or neither of `instalment` and `annual_rate`, `annual_rate` with
other instalments, a balloon that alone repays the loan at the stated rate
and a rounded instalment that moves the last instalment by more than a whole
instalment (below nil or above twice the instalment) are refused.
Fees, stamp duty, changing rates (see [Loan transactions](#loan-transactions)),
daily interest, payouts, substituted accounting periods and the goods'
decline in value are out of scope; see
[calculation evidence](docs/calculation-evidence.md#chattel-mortgage-schedule).

## Loan transactions

`loan_transactions` gives a loan's statement from `start_date` (in the income
year given) to `end_date` (less than 30 years later), from the
`opening_balance`, the `annual_rate` from the start, any `rate_changes` (a
list of `{"date", "annual_rate"}` in date order, after the start, one a day)
and `transactions` (a list of `{"date", "kind", "amount"}` in date order, with
`kind` `drawing` or `repayment`; `[]` for none). Each day's interest is the
balance at the end of the day, when above nil, times the rate in force over
365; it is debited on the last day of each month and on `end_date`, rounded
half up to the cent. The result gives `opening_balance`, `total_drawings`,
`total_repayments`, `total_interest` and `closing_balance`,
`interest_by_income_year` (the interest debited in each year, which is the
interest for its days), `statement_rows` (each line's `date`, `details`,
`debit`, `credit` and `balance`) and `statement`, the same lines as a
printable Markdown table. To pay a loan out, end the statement on the payout
day with a repayment of the balance and the interest to that day; a balance
below nil at the end of a day is refused. Interest is deductible under
ITAA 1997 s 8-1 to the extent the money is used to gain or produce assessable
income, which the caller establishes. Other day counts, fees, offset accounts
and Division 7A loans are out of scope; see
[calculation evidence](docs/calculation-evidence.md#loan-transactions).

## Business capital expenditure

`business_capital_expenditure` works out the deduction for business capital
expenditure under ITAA 1997 s 40-880 (black hole and start-up costs) for one
income year from 2020-21 to 2027-28, and each item's deductions year by
year. It takes `items`, a list with one object for each item of expenditure
incurred from 1 July 2020, when s 40-880 as amended by Act No 92 of 2020
began to apply, to the end of the year asked about. Each item states:

- `amount` (a decimal string), `incurred_in` (an income year) and `capital`
  (`true`: the caller establishes that the outlay is capital expenditure);
- `business`: `current` (your business, s 40-880(2)(a)), `former` (one that
  used to be carried on, (2)(b)), `proposed` ((2)(c)) or `wind_up`
  (liquidating, deregistering or winding up a company, partnership or trust
  you were a member of, partner in or beneficiary of, (2)(d));
- `taxable_purpose`, the extent from 0 to 1 the business is, was or is
  proposed to be carried on for a taxable purpose (s 40-880(3) and (4)(a));
- `exclusions`, the paragraphs of s 40-880(5), (8) and (9) that exclude it,
  such as `["5(c)"]`, or `[]`.

A former or proposed business also states `carried_on_by` (`you` or
`another_entity`). Another entity's business, and a wind-up, state
`assessable_income_connection` (s 40-880(4)(b)). A proposed business states
`within_reasonable_time` (s 40-880(7)) and `start_up_expense`
(`advice_or_services`, `government_fees` or `neither`, s 40-880(2A)(b)),
and, unless `neither`, `entity_test` for the income year incurred
(`small_business_entity`, `covered_by_2b`, `not_in_business_and_unconnected`
or `none`, s 40-880(2A)(c) and (2B)). An item listing `5(d)` or `5(f)` states
`goodwill_right` (s 40-880(6)). A fact an item needs and does not state is
refused with its name, as is a fact it does not take.

An item that s 40-880(4)(b), (5), (7), (8) or (9) excludes has nothing to
deduct, and its row cites each paragraph. Otherwise its deductible
expenditure is its amount times the taxable purpose extent. A start-up cost
that meets s 40-880(2A) is deducted in full in the year incurred; the rest is
deducted one fifth a year over 5 income years starting in the year incurred,
with no apportionment for part of a year (s 40-880(2); TR 2011/6 para 10).
Eligibility is fixed when an item is incurred, so its later fifths continue
if the business ceases (TR 2011/6 para 10). The result's amounts are the expenditure, the deductible expenditure, the
year's deduction and what remains for later years. `items` gives each item's
paragraph of s 40-880(2), its limit, `excluded_by`, the method with its
citation, the year's deduction, `schedule` (each income year and its
deduction) and `notes`, each worked exactly; the amounts are shown to the
cent, half up.

It refuses expenditure that forms part of the cost of a depreciating asset
(s 40-880(5)(a)), which Division 40 deducts, and expenditure that could be
taken into account in working out a capital gain or loss (s 40-880(5)(f)),
which the CGT provisions take, unless s 40-880(6) applies; a revenue outlay;
and an item incurred before 1 July 2020 or after the year asked about.
Whether an outlay is capital or business related, the taxable purpose extent,
the entity tests and the exclusions are the caller's to establish, and
non-arm's length amounts (s 40-885), Division 35's deferral of an
individual's non-commercial losses, personal services income (s 85-10),
s 26-155's limit on residential rental deductions from 2027-28, a taxpayer
that ceases to exist before the 5 years end, and website and software costs
are out of scope. The tests
hold it to the Commissioner's examples in TR 2011/6 and the ATO's start-up
examples; see
[calculation evidence](docs/calculation-evidence.md#business-capital-expenditure).

## Balancing adjustment

`balancing_adjustment` works out the balancing adjustment for one depreciating
asset when a balancing adjustment event occurs for it in 2024-25, 2025-26 or
2026-27 under ITAA 1997 Subdivision 40-D, with its reduction for use other than
for a taxable purpose (s 40-290) and the capital gain or loss of CGT event K7
(s 104-240). It takes:

- `cost` and `termination_value` (decimal strings), each after Subdivision
  27-B: the cost after the car limit for a car (s 40-230), and the termination
  value under ss 40-300 to 40-320, before s 40-325;
- `asset`: `car` (as ITAA 1997 s 995-1 defines it), `motor_cycle_or_similar`
  or `other`;
- `years`, one object for each income year held from the start time to the
  event, the event's year included (a year with no decline gives `0.00`),
  each with `income_year`, `decline_in_value` and `deduction`, the
  decline less the reductions s 40-25(2) to (4) required. A car's rows also
  state `method` (`log_book` or `cents_per_kilometre`) and `days_held` (at
  most the days in that income year, 366 when it includes 29 February); a
  cents per kilometre row takes no `deduction`, and its decline is the notional
  decline s 40-370(3) assumes;
- for a car only, and then required, `car_limit`: `null`, or where s 40-230
  reduced its cost, an object with `limit` (the car limit for the financial
  year the car was first used), `second_element_cost` and
  `cost_ignoring_car_limit`; `cost` is then `limit` plus
  `second_element_cost`.

Each year's decline and deduction are inputs; `depreciation` works out a
year's decline. The adjustable value is cost less the years' declines
(s 40-85). The balancing adjustment amount is the difference between the
termination value and the adjustable value (s 40-285); it is reduced by itself
times the sum of the reductions over the total decline (s 40-290(2)), and the
rest is assessable where the termination value is more and deductible where it
is less. A car whose expenses were claimed by the log book method in some years
and the cents per kilometre method in others is worked under s 40-370 instead:
a cents per kilometre year counts 80% of its notional decline as a reduction,
and the reduced amount is scaled by the days the decline was deducted over the
days held, a log book year with a nil deduction adding no days. A car claimed
only by the cents per kilometre method has nothing assessable or deductible
(s 40-370(1) Note 2). Where s 40-230 reduced a car's cost, its termination
value is first multiplied by the car limit plus the
second element over its cost ignoring the car limit (s 40-325). The K7 gain or
loss is the termination value less cost, or cost less termination value, times
the same fraction of reductions (s 104-240), and is nil for a car, motor cycle
or similar vehicle (s 118-5). Each amount is worked exactly with its fraction
divided once, last, and shown to the cent, half up. `rates` gives the years
held, the provision applied and, under s 40-370, the days deducted and held.

It refuses a year after the event's year, a year before 2001-02, a car year
before 2015-16 (when the one-third of actual expenses and 12% of original value
methods could apply, which s 40-370 as compiled now does not count), a year
given twice, years that skip one or stop before the event's, a deduction above
its year's decline, declines totalling more than cost or nil, car limit facts
that show s 40-230 did not reduce the cost, a car's cost other than the car
limit plus the second element (so a car first used in a later financial year
than the one it was acquired in, whose 2 car limits differ, and a car held
with another entity, s 40-230(4)), and 2027-28, when Schedule 1
to Act No. 49 of 2026 changes capital gains. Whether an event occurred, the
termination value, cost, decline, use and method are the caller's to establish.
Out of scope are assets in a low-value, software development, project or small
business pool and assets written off under s 328-180 or temporary full
expensing; the fixed 50% reduction chosen under s 40-291A from 2026-27;
second-hand assets in residential rental property (s 40-291); R&D use; split or
merged assets; roll-over relief (s 40-340); an involuntary disposal whose
assessable amount reduces a replacement asset's cost (s 40-365); a vessel's
deferral; a car acquired at a discount (s 40-225);
pre-CGT assets and other CGT exemptions, such as collectables and personal use
assets; and the tax payable. The tests hold it to the Guide to depreciating
assets 2026's Bridget, Andrew and Louise examples and the example in
s 118-10(1); see
[calculation evidence](docs/calculation-evidence.md#balancing-adjustment).

## Trade-in

`trade_in` works out trading in a depreciating asset for a replacement in
2025-26 or 2026-27, at arm's length, with the replacement started to be held
and first used in the same income year: the GST on both sides, the old asset's
termination value and balancing adjustment, and the replacement's cost. It
takes `balancing-adjustment`'s inputs for the traded-in asset (`cost`, `asset`,
`years` and, for a car, `car_limit`) without `termination_value`, which it
works out, and:

- `trade_in_allowance`, the amount by which the replacement's price is reduced
  for the old asset, including any GST, and `trade_in_taxable`, whether trading
  it in is a taxable supply (GST Act s 9-5); when it is,
  `adjusted_input_tax_credit` (the old asset's input tax credit as adjusted)
  and `full_input_tax_credit` (the credit had it been acquired solely for a
  creditable purpose), as GST Act s 132-5(2) defines them;
- `replacement_price`, the price before the trade-in including GST and any
  luxury car tax; `replacement_asset`, `passenger_car` (a car designed mainly
  for carrying passengers, to which ITAA 1997 s 40-230 and GST Act s 69-10
  apply) or `other`; for a passenger car, `replacement_luxury_car_tax`; and
  `replacement_creditable_acquisition` (GST Act s 11-5), with
  `creditable_purpose`, the extent from above 0 to 1, when it is one;
- `arms_length` and `involuntary_disposal`, flags that must be `true` and
  `false`.

The old asset's termination value is the trade-in allowance, the liability to
pay the replacement's price that it terminates (ITAA 1997 s 40-305(1)(b)
item 2), less the GST on a taxable trade-in, 1/11 of the allowance (GST Act
ss 9-70 and 9-75(1); ITAA 1997 s 27-95(1)), plus the decreasing adjustment for
an asset acquired partly for a private purpose or for making financial
supplies, 1/11 of the allowance times (1 - adjusted / full input tax credit),
capped at their difference (GST Act s 132-5; ITAA 1997 s 27-95(3)). Its
balancing adjustment, reduction for non-taxable use and CGT event K7 amount
are then worked as [Balancing adjustment](#balancing-adjustment) describes,
s 40-325 included. The replacement's input tax credit is the GST payable,
1/11 of its price less any luxury car tax, capped for a passenger car at 1/11
of the car limit, times the extent of creditable purpose (GST Act ss 11-25,
11-30 and 69-10); its cost is its price less that credit (ITAA 1997 ss 40-180,
40-185 and 27-80), and a passenger car's is then reduced to the car limit
(s 40-230): $69,674 for 2025-26 and $69,883 for 2026-27, as the ATO publishes
them. `replacement_cost_ignoring_car_limit` is what `balancing-adjustment`'s
`car_limit` will need when the replacement is disposed of.

It refuses a trade-in not at arm's length, where s 40-300(2) item 6 makes the
termination value its market value, s 27-95(2) then takes no GST off it and
s 40-225 can increase a car's cost by a discount portion (the Guide's Kristine
example). It refuses an involuntary disposal: where the old asset was lost,
destroyed or compulsorily acquired, s 40-365 lets the taxpayer exclude some or
all of an assessable balancing adjustment amount and reduce a replacement's
cost by it instead, if the replacement is acquired from one year before the
event to one year after the end of its income year and used wholly for a
taxable purpose; the worksheet does not make that choice. It also refuses an
adjusted input tax credit above the full one, a creditable acquisition with a
nil creditable purpose, and luxury car tax above the price. A nil full input
tax credit, as for an asset bought without GST, gives a nil decreasing
adjustment (GST Act s 132-5(3)). A car whose purchaser may quote an ABN for luxury car tax, one that is
not a luxury car under s 25-1(2) of the Luxury Car Tax Act, a leased car, a car
excluded from the car limit by s 40-230(2), a trade-in on which luxury car tax
is payable, a decreasing adjustment arising in a later year (s 27-95(5)) and
increasing adjustments are out of scope, as is everything `balancing-adjustment`
leaves out. The tests hold it to the ATO's examples for Darren's car and Ian's
vehicle; see [calculation evidence](docs/calculation-evidence.md#trade-in).

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
payments; death benefit ETPs, working out a tax-free component
([`etp-tax-free-component`](#etp-tax-free-component) does, and gives the
taxable component to classify here), earlier ETPs in
the year or for the same termination, payees without a TFN, foreign residents
and the income tax assessed on the ETP are outside it. It refuses a field it
does not take. The tests hold it to the ATO's Schedule 11 examples; see
[calculation evidence](docs/calculation-evidence.md#termination-payment).

## DASP tax

`dasp-tax` works out the tax on one departing Australia superannuation
payment (ITAA 1997 s 301-170) paid in 2025-26 to 2027-28, by component, under
the Superannuation (Departing Australia Superannuation Payments Tax) Act 2007
s 5:

```text
tax_free_component_tax = 0                                   s 5(1)(a)
taxed_element_tax      = taxed_element x 35%, or 65%         s 5(1)(b), s 5(3)
untaxed_element_tax    = untaxed_element x 45%, or 65%       s 5(1)(c), s 5(3)
dasp_tax               = the sum; net_payment = payment - dasp_tax
```

The 65% rates apply when `working_holiday_maker_contributions` is true: the
payment includes amounts attributable to contributions made whilst the person
held a subclass 417 or 462 visa or an associated bridging visa, and they then
apply to the whole taxable component. Each element's tax is rounded to the
cent, 0.5 cents up, so the tax and `net_payment` add back to the payment. The
payment is not assessable income and not exempt income (s 301-175), so the
tax is final and nothing goes in a tax return; a payment under $200 is taxed
like any other. It refuses a nil payment. The caller establishes that the
payment is a DASP and its components; a roll-over under s 20H of the
Superannuation (Unclaimed Money and Lost Members) Act 1999 with an excess
untaxed roll-over amount (s 5(2)), the 2014-15 to 2016-17 budget repair levy
rates (s 6) and the fund's withholding and reporting are outside it. See
[calculation evidence](docs/calculation-evidence.md#dasp-tax).
## ETP tax free component

`etp_tax_free_component` works out the tax free component of one life benefit
employment termination payment made in 2026-27 or 2027-28, and the taxable
component left (ITAA 1997 ss 82-140 and 82-145). The invalidity segment
(s 82-150) is the payment times the days to retirement over the employment
days plus the days to retirement. It arises only where the payment was made
because the employee stopped being gainfully employed through ill-health,
before their last retirement day (the day their employment would have ended at
a set age or period of service, otherwise their 65th birthday, s 995-1(1)),
and 2 legally qualified medical practitioners have certified that they are
unlikely ever to be gainfully employed in a capacity they are reasonably
qualified for; otherwise it is nil, and the step names the condition not met.
The pre-July 83 segment (s 82-155) is the payment less the invalidity segment,
times the employment days before 1 July 1983 over all of them. Employment days
count both the first day and the termination day, as the ATO's worksheet
NAT 71949 counts them; the days to retirement run from the day after the
termination to the last retirement day. Each segment rounds half up to the
cent and the taxable component is the rest, so the 2 components add up to the
payment.

`invalidity_excluded_payment` says whether the payment includes an invalidity
segment, or would but for its timing: such a payment is an excluded payment
(s 82-10(6)(c)), so give its taxable component to `termination-payment` as
`excluded_payment`. Death benefit ETPs, payments that relate to only part of
the employment or to broken periods of it, unused annual and long service leave,
superannuation disability benefits (s 307-145) and the tax on the taxable
component are outside it. The tests hold it to the ATO's Jackson example and
the Library's Jim and Jon examples; see
[calculation evidence](docs/calculation-evidence.md#etp-tax-free-component).

## Unused leave payment

`unused_leave_payment` works out, for unused annual leave, leave loading and
unused long service leave paid in 2026-27 on the termination of an
employee's employment, each part's assessable share and tax offset under
ITAA 1997 Subdivisions 83-A and 83-B, the PAYG withholding under Schedule 7
and the income statement label. Annual leave and loading paid on a genuine
redundancy, an early retirement scheme or invalidity, and the parts accrued
before 18 August 1993 on any other termination, have a tax offset that limits
their rate to 30% (s 83-15); so do long service leave from 16 August 1978 to
17 August 1993, and all of it from 16 August 1978 on those 3 reasons (s 83-85).
Schedule 7 withholds 32% from them, rounded to the nearest dollar, and reports
them as lump sum A, type R for those 3 reasons and T otherwise. Only 5% of long
service leave from before 16 August 1978 is assessable (s 83-80); it is lump
sum B in full. That 5% and the rest, leave from after 17 August 1993 on a normal
termination, which is reported as unused leave on termination (paid leave
type U), go through Schedule 7's marginal rate calculation once, on the
employee's normal gross earnings and Schedule 1 scale (scale 3 for a foreign
resident); a post-17 August 1993 payment under $300 has at most 32% of it
withheld. Without a TFN (scale 4) Schedule 7 withholds 47% of the whole
payment instead, or 45% for a foreign resident, cents ignored. The caller gives the day the
leave's accrual started and, where that day comes before 16 August 1978 or
18 August 1993 and the reason makes it matter, the part of each total accrued
before it, worked out by days under ss 83-95 to 83-115; a missing part is
refused, as is one the accrual start rules out. `qualifying_earnings` is nil:
the Superannuation Guarantee (Administration) Act 1992 leaves unused annual
and long service leave payments on termination out of ordinary time earnings
(s 6(1)) and so out of qualifying earnings (s 10A(1)), and the ATO's table of
qualifying earnings says the same of all unused leave on termination, whatever
the reason. Leave paid after death, working holiday makers (Schedule 15), a
payment under $300 beside pre-16 August 1978 long
service leave, which Schedule 7 does not say how to limit, and other years are
refused, and it refuses a field it does not take. The tests hold it to Schedule
7's example and the ATO's unused leave examples; see
[calculation evidence](docs/calculation-evidence.md#unused-leave-payment).

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

## Lump sum payment in arrears tax offset

`lump_sum_arrears_tax_offset` works out the tax offset for lump sum payments
in arrears (ITAA 1936 ss 159ZR to 159ZRD) for a full-year resident individual
whose year of receipt is 2013-14 to 2026-27. `taxable_income` is that year's
taxable income, the arrears included, and `net_capital_gain` the net capital
gain in it, `"0"` for none. `accrual_years` lists each earlier income year in
which part of the arrears accrued, once, as `{"year": "2023-24", "arrears": "3000"}`
with every payer's arrears for that year added together. The 2 latest accrual
years, or each one when there are fewer than 3, are recent and also take
`taxable_income`; with 3 or more, each recent year may take `net_capital_gain`,
and a recent year that was itself a rebate year takes `rebate_year_arrears`, or
`earlier_rebate_arrears` for what accrued in it of an earlier rebate year's
arrears. Earlier, distant years take `year` and `arrears` only.

The offset is due when the total arrears are at least 10% of normal taxable
income less them (s 159ZRA(1)(b)). It is the tax the arrears add this year
(s 159ZRB) less the tax they would have added in each recent year (s 159ZRC),
and in the distant years at the average of the recent years' rates, each to 3
decimal places and raised by 0.001 when the fourth would be more than 4
(s 159ZRD). Each year's tax is at the ATO's published resident rates, with the
2% temporary budget repair levy above $180,000 in 2014-15 to 2016-17, which
Income Tax (Transitional Provisions) Act 1997 s 4-11 makes extra income tax. It
refuses a recent accrual year before 2012-13, for which it holds no rates, and
an accrual year that is not before the year of receipt, given twice or with
nil arrears. Foreign and part-year residents, employment termination payments,
superannuation lump sums, arrears the caller has not classified as eligible and
applying the offset against tax are outside it. The Medicare levy relief
(Medicare Levy Act 1986 s 9A) and the surcharge offset (ITAA 1997
Subdivision 61-L) are separate questions its result names. Like `division-296`
it refuses a field it does not take. The tests hold it to the ATO's Sudeep and
Anthony results, work its Heather and Yan facts by hand with assumed taxable
incomes and check its prior-year rates; see
[calculation evidence](docs/calculation-evidence.md#lump-sum-arrears-tax-offset).

## Income tests

`income_tests` works out 6 income measures for one full-year resident from
taxable income and the components each definition adds back, each in a
`derive` step that cites the definition. Every component is required, as a
decimal string, nil where the person has none; `taxable_income` is whole
dollars, and a negative component is refused. For each measure, `not_used`
lists the components it leaves out. The fringe benefits come in 2 parts, as
return item IT1 splits them: a section 57A employer's amounts count at 53%,
rounded down to the dollar, in rebate income and adjusted taxable income, and
in full elsewhere. The command refuses a part-year resident, a working holiday
maker or a foreign resident (`income_tests.residency_unsupported`), a component
larger than the amount that includes it
(`income_tests.components_inconsistent`) and a year other than 2024-25 to
2026-27.

| Measure | Definition | Command that takes it |
| --- | --- | --- |
| `income_for_surcharge_purposes` | ITAA 1997 s 995-1 | `medicare-levy-surcharge`, as `income_for_surcharge_purposes`, with `reportable_fringe_benefits_total` as `reportable_fringe_benefits` |
| `division_293_income` | ITAA 1997 s 293-20(1)(a) | `division-293`, as `division_293_income` |
| `rebate_income` | ITAA 1936 s 6(1) | `seniors-pensioners-tax-offset`, as `rebate_income`, in whole dollars |
| `repayment_income` | HESA s 154-5 | `study-loan-repayment`, as `repayment_income`, in whole dollars |
| `adjusted_taxable_income` | ITAA 1997 s 995-1 'adjusted taxable income for offsets' | None; the dependant offsets test it |
| `co_contribution_total_income` | Superannuation (Government Co-contribution for Low Income Earners) Act 2003 s 8 | None; the super co-contribution tests it |

Each command's input has the measure's name and definition, with one
difference. ITAA 1997 s 995-1 does not add exempt foreign employment income to
income for surcharge purposes; the ATO's surcharge page adds it where taxable
income is $1 or more, and `medicare-levy-surcharge` describes its input as that
page does. `income-tests` follows the Act and warns when the two differ.
Measures another agency applies are outside it: the Commonwealth Seniors Health
Card income test and family assistance adjusted taxable income are Services
Australia's ([Income test for Commonwealth Seniors Health
Card](https://www.servicesaustralia.gov.au/income-test-for-commonwealth-seniors-health-card)).
The tests hold it to the ATO's Christina and Tom examples and pass each measure
to the command that takes it; see
[calculation evidence](docs/calculation-evidence.md#income-tests).

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
deductions: give `net_labour_income` where they do. Given or worked out, it
can be negative, which gives no offset. The amounts add
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

## FBT return

`fbt-return` composes one ordinary taxable employer's fringe benefits for an FBT
year ended 31 March 2026 or 2027 into the amounts the FBT return asks for,
without valuing any benefit itself. It takes `employer`, a label, `year_ended`
and `scope_confirmed` once, and `benefits`, a list. Each benefit names the
worksheet that values it (one of the 2 car commands or the 11 FBT benefit
commands above) and that worksheet's own arguments under `input`, without
`year_ended` and `scope_confirmed`, which the return adds. The operator states
the rest: `employee`, a label for the employee the benefit is provided in
respect of, an associate's benefits included (the employer's guide, 5.3); `gst_type`,
`type_1` for a GST-creditable benefit or `type_2` (s 5C(3) and (4)); and
`excluded`, whether it is an excluded fringe benefit under s 5E(3).

```json
{"employer": "Example Pty Ltd", "year_ended": 2026, "scope_confirmed": true,
 "benefits": [
   {"employee": "E1", "command": "fbt-debt-waiver", "input": {"amount_waived": "2545.00"},
    "gst_type": "type_2", "excluded": false}]}
```

Each benefit's worksheet runs as it does alone, and `components` keeps every
result, with its scope, sources and warnings. The return then adds each type's
taxable values, excluded benefits included, and runs `fbt` once on the 2
totals, so the gross-up and the tax are `fbt`'s. The result gives:

- `amounts`: the type 1 and type 2 aggregate amounts, each grossed up, the
  fringe benefits taxable amount (item 15), the FBT payable (item 16) and the
  employee contributions applied to the benefits; `working` names the source of
  each.
- `employees`: for each employee label, the individual fringe benefits amount,
  the sum of their benefits' taxable values that are not excluded (s 5E(2)),
  with an exempt electric car's notional value (s 135P(3)), the employee
  contributions applied, and the reportable fringe benefits amount. That amount
  is nil unless the individual amount is more than $2,000 (s 135P(1)); above
  it, the individual amount times 1 / (1 - the FBT rate), as s 135P(2) states
  it, in whole dollars with the cents dropped. No provision states that
  rounding; the ATO's examples drop the cents. The ATO's pages multiply by
  1.8868 instead, which can give a dollar more: $15,000 gives $28,301 here and
  $28,302 in the employer's guide, 5.8.
- `return_labels`: items 14A, 14B, 15, 16 and 19, and item 23 by benefit
  category (A to G and J to P), with the gross taxable value, employee
  contribution, value of reductions and taxable value, and at 23G the number
  of employee labels that received a living-away-from-home allowance. A column
  the instructions say to leave blank, such as 23D's Number, employee
  contribution and value of reductions, is left out. An exempt electric car is
  not a fringe benefit and is left out of item 23 and the aggregates.
- `left_to_preparer`: the labels it does not fill and why: the employer's
  details (items 1 to 9 and 11 to 13); item 10's count of employees and
  associates; the number of cars, loans and units of accommodation at 23A,
  23B, 23C and 23F, as one can be given as several benefits; items 14C, 17
  and 18, which a taxable employer leaves blank;
  items 20 to 22, which need the instalments reported on activity statements;
  and the declarations.
- `reconciliation`: the benefits' taxable values added one by one beside the 2
  aggregate amounts, and the difference, which is nil.

The FBT return 2026 instructions were the latest the ATO had published on
16 November 2026, so a 2027 result follows their labels and says so in its
first warning; the 2027 form may differ. A benefit must belong wholly to one employee label: a benefit shared
by employees or valued with others in a class (s 5F(3) and (4)), or partly
GST-creditable, is given as separate benefits. It is not for a rebatable
employer, a public or not-for-profit hospital, a public ambulance service or an
employer exempt under s 57A or s 58 (items 14C, 17 and 18, and s 135Q).

It refuses a benefit without a `gst_type` of `type_1` or `type_2` as
`fbt_return.gst_type_missing`, a year without `fbt`'s rates as
`period.unsupported`, a benefit whose `input.year_ended` is not the return's as
`fbt_return.mixed_years`, more than one employer as
`fbt_return.multiple_employers`, and any input whose name starts with `payroll_tax` as
`fbt_return.payroll_tax_unsupported`: payroll tax is imposed by each state and
territory under its own Act, and the return does not work out a payroll tax
gross-up. A car parking or meal entertainment benefit stated not to be excluded
is refused as `fbt_return.exclusion_conflict` (s 5E(3)(a) and (b); the meal
entertainment worksheet covers no salary packaging). An operating cost car
whose statutory formula value is lower takes its s 9 value under s 10(5), so
it is refused as `fbt_return.car_statutory_lower`: give it to
`fbt-car-statutory`, whose gross value and contribution item 23 needs. A
command that is not a benefit worksheet is `fbt_return.benefit_command_unsupported`.
A worksheet's own refusal, such as a negative or non-finite amount, keeps its
code, and its message starts with the benefit and the worksheet, such as
`benefits[0] fbt-debt-waiver: ...`. Choosing between paying FBT and an employee
contribution is planning, outside a worksheet. `derive` does not explain the
return; it explains each worksheet.

The tests hold the return to the ATO's FBT return 2026 instructions (examples
2 and 4 for items 14A to 19, examples 6 to 12 for item 23), its reportable
fringe benefits page's example and the employer's guide's 5.4 example for
reportable amounts, the examples in s 135P(1), and a seeded property that the
aggregates add up to the benefits; see
[calculation evidence](docs/calculation-evidence.md#fbt-return).

Between The Spreadsheets sells a Full FBT Calculator for 31 March 2026, $1,400
plus GST as a bundle with its car and entertainment calculators, whose product
page (read on 16 November 2026) says it tracks reportable fringe benefits,
summarises the benefits into a page that feeds the FBT return, shows a payroll
tax gross-up, summarises employee contributions and links the ATO's employee
declarations. `fbt-return` covers the return, reportable amounts and
contributions for 2026 and 2027, free, with each step's provision cited; it does
not work out a payroll tax gross-up or provide declarations. Nobody here has
run the rival's workbook, so this compares what each says it does, not results.

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
