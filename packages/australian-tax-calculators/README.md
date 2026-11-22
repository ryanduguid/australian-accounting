# Australian tax calculators

Australian Tax Calculators 1.1.0 is a standalone Rust binary with a JSON
stdin interface. It replaces the Python `austaxcalc` import API. The MCP
server through 0.9.0 used the 0.2.0 Python package from PyPI. Later versions
run the calculator executable and require the exact version they were
tested against. The native MCP build in this repository links this crate.
The [release handoff](../../apps/aus-accounting-mcp/docs/worksheet-release.md)
keeps the calculator release and MCP integration separate.

<div align="center">

[![tests](https://img.shields.io/github/actions/workflow/status/ryanduguid/australian-accounting/ci.yml?branch=main&label=tests&labelColor=000000&style=flat)](https://github.com/ryanduguid/australian-accounting/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.99-66023C?logo=rust&logoColor=E5E4E2&labelColor=000000&style=flat&color=66023C)](https://www.rust-lang.org/)
[![License: AGPL-3.0-or-later](https://img.shields.io/badge/License-AGPL--3.0--or--later-66023C?labelColor=000000&style=flat&color=66023C)](LICENSE)

</div>

83 calculation worksheets for established facts: ordinary GST, fuel tax credits at the ATO's rate for each day from 1 July 2022 with the simplified methods for small claimants, resident and foreign
resident basic income tax, CGT losses and discount, the gains and losses from parcels of shares or units, the small business CGT concessions open for a gain, the foreign resident capital gains withholding a purchaser pays on Australian real property, the small business income tax offset, whether Division 35 defers a business activity loss, whether the personal services income rules apply, an employee's working from home, car and self-education deductions, the main residence exemption's taxable share of a gain or loss, one loan's borrowing expenses deduction and a schedule of several loans' deductions over every year, a residential rental property's income, deductions by category and net rent with its effect on tax, a hire purchase agreement's notional interest, a chattel mortgage's interest schedule and input tax credit, a loan's statement with daily interest at changing rates, business capital expenditure (black hole and start-up costs) under s 40-880, ordinary employer FBT, car fringe benefit taxable values
under the statutory formula and operating cost methods, loan, meal entertainment, car parking, debt waiver, expense payment, property, residual, housing, living-away-from-home allowance, board and tax-exempt body entertainment fringe benefit taxable values, the FBT, deduction and GST credit treatment of entertainment expense lines, whether a 12 week meal entertainment register is valid, depreciation in the start year and later years, a depreciating asset's balancing adjustment with its reduction for private use and CGT event K7, a trade-in's GST on both sides with the old asset's balancing adjustment and the replacement's cost, whether an electric car's fringe benefits are exempt, one employer's payroll tax in each state and territory, a company's franking account with franking deficit tax, its offset and the benchmark rule, wine equalisation tax with the producer rebate under its yearly cap and BAS labels 1C and 1D, luxury car tax on sales and importations with its adjustments, refunds and BAS labels 1E and 1F, the general and shortfall interest charges, failure to lodge and statement penalties before remission and the interest the ATO pays on early payments and overpayments,
quarterly super guarantee, Schedule 1 PAYG withholding and the gross pay that leaves a stated net, Schedule 5 withholding from back payments, commissions and bonuses by Methods A, B(i) and B(ii), Schedule 15 withholding for working holiday makers, Schedule 9 withholding for seniors and pensioners, super contribution caps, the super co-contribution, the low income super tax offset and the spouse contribution tax offset,
account-based pension minimums, study and training loan repayments, Schedule 8 study
loan withholding, the Medicare levy with its low-income, SAPTO and family reductions and full or half exemptions, the Medicare levy surcharge, the low income tax offset, the seniors and pensioners tax offset, the lump sum payment in arrears tax offset, the beneficiary, invalid and invalid carer, zone and overseas forces and net medical expenses tax offsets,
Division 293 tax, an estimate of Division 296 tax on large super balances and the tax-free part of a genuine redundancy payment with Schedule 11 withholding from an employment termination payment, the invalidity and pre-July 83 segments of an employment termination payment's tax free component, the assessable share, Schedule 7 withholding and income statement label of unused leave paid on termination, the private health insurance tax offset or excess premium reduction from the statement's lines, and the income measures the surcharge, Division 293 tax, the offsets, study loan repayments and the super co-contribution are tested against. Callers must establish every scope condition before calculating. `personal-tax-estimate` composes the personal ones into one year's tax and levies,
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
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/ryanduguid/release-policy/d7039efe6d8c441f1f213c674902909e80535599/scripts/install.sh | sh -s -- ryanduguid/australian-accounting australian-tax-calculators/v1.1.0 --modify-path
```

In PowerShell on x86_64 Windows:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/ryanduguid/release-policy/d7039efe6d8c441f1f213c674902909e80535599/scripts/install.ps1))) ryanduguid/australian-accounting australian-tax-calculators/v1.1.0 -ModifyPath
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
{"ok":true,"engine":"australian-tax-calculators","engine_version":"1.1.0","calculation":"gst","period":"2025-26","amounts":{"gst":"100.00","exclusive":"1000.00","inclusive":"1100.00"},"rates":{"gst_rate":"0.10"},"scope":"One ordinary taxable supply at 10%, already classified by the operator. Excludes mixed supplies, exemptions, margin schemes, adjustments, tax invoice rounding across line items, input-credit entitlement and BAS preparation.","sources":["https://smallbusiness.taxsuperandyou.gov.au/goods-and-services-tax/fast-facts"],"source_checked":"2026-09-10","warnings":["Operator-confirmed facts and scope; eligibility is not independently verified.","Worksheet only, not advice, an assessment or a lodgment. Obtain human review."]}
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

`contract-register` is the one command that reads a CSV file rather than a
JSON object: a practice's hire purchase and chattel mortgage contracts, one a
line, set out together with cashflow by month, interest by income year and
the amount outstanding split into current and non-current. See
[Contract register](docs/contract-register.md).

## Commands

Each command takes the original function's keyword arguments, in this order, as
one JSON object. For amounts and ratios, pass a string of ASCII digits with an
optional leading minus sign and optional decimal point followed by digits,
such as `"1000.00"` or `"-0.50"`. Surrounding whitespace is allowed. Expand
exponents and remove digit separators before sending a value. Flags are JSON
booleans and counts, ages, days, quarters and scales are JSON integers.
`australian-tax-calculators --api-help` lists the commands.

| Command | Arguments |
| --- | --- |
| `gst` | `amount`, `gst_inclusive`, `scope_confirmed`, `year` |
| `fuel-tax-credits` | `lines`, `year`, `scope_confirmed`; `small_claimant` with a simplified method, `rate_at_period_end` and `return_date` optionally; see [Fuel tax credits](#fuel-tax-credits) |
| `gst-margin-scheme` | `interest`, `agreement`, `acquisition`, `consideration`, `method`, `recipient_associate`, `year`, `scope_confirmed`; with method `consideration`, `acquisition_consideration` and `unpaid_acquisition_consideration`; with method `approved_valuation`, `approved_valuation`, `valuation_day`, `registered_on_1_july_2000`, `government_unimproved_on_1_july_2000` and the applicable date facts below; for part of the land, `subdivision_share` and `subdivision_whole`; see [GST margin scheme](#gst-margin-scheme) |
| `resident-tax` | `taxable_income`, `year`, `scope_confirmed` |
| `foreign-resident-tax` | `taxable_income`, `year`, `scope_confirmed` |
| `capital-gains` | `other_gains`, `discount_gains`, `current_losses`, `prior_losses`, `scope_confirmed`, `year` |
| `main-residence` | `capital_gain`, `capital_loss`, `market_value_basis`, `ownership_start`, `ownership_end`, `lived_periods`, `income_use_periods`, `absence_choice_periods`, `new_home_acquired`, `year`, `scope_confirmed` |
| `borrowing-expenses` | `borrowing_expenses`, `other_borrowing_expenses_in_year`, `income_use`, `loan_start`, `loan_term_end`, `repaid_on`, `section_26_50_applies`, `year`, `scope_confirmed` |
| `borrowing-expenses-schedule` | `loans`, `year`, `scope_confirmed`; optionally `other_borrowing_expenses`; see [Borrowing expenses schedule](#borrowing-expenses-schedule) |
| `hire-purchase-schedule` | `amount_financed`, `balloon`, `timing`, `start_date`, `year`, `scope_confirmed`; `instalment`, `instalments` and optionally `frequency`, or `segments` in their place; optionally `deposit`, `trade_in`, `payout` and `gst`; see [Hire purchase schedule](#hire-purchase-schedule) |
| `chattel-mortgage-schedule` | `amount_financed`, `instalments`, `balloon`, `timing`, `start_date`, `year`, `scope_confirmed`; `instalment` or `annual_rate`; optionally `frequency`, `deposit`, `trade_in` and `gst`; see [Chattel mortgage schedule](#chattel-mortgage-schedule) |
| `loan-transactions` | `opening_balance`, `annual_rate`, `start_date`, `end_date`, `transactions`, `year`, `scope_confirmed`; optionally `rate_changes`; see [Loan transactions](#loan-transactions) |
| `business-capital-expenditure` | `items`, `year`, `scope_confirmed`; see [Business capital expenditure](#business-capital-expenditure) |
| `balancing-adjustment` | `cost`, `termination_value`, `asset`, `years`, `year`, `scope_confirmed`; for a car, `car_limit`; see [Balancing adjustment](#balancing-adjustment) |
| `franking-account` | `private_company`, `benchmark_rule_exempt`, `corporate_tax_rate`, `opening_balance`, `rd_deferred_debit_brought_forward`, `last_benchmark`, `events`, `distributions`, `year`, `scope_confirmed`; optional `income_tax` (required for a private company whose offset reduction would otherwise apply) and `commissioner_determination`; see [Franking account](#franking-account) |
| `cgt-parcels` | `entity`, `identification`, `parcels`, `disposals`, `other_discount_capital_gains`, `other_capital_gains`, `other_capital_losses`, `prior_net_capital_losses`, `year`, `scope_confirmed`; see [CGT parcels](#cgt-parcels) |
| `trade-in` | `cost`, `asset`, `years`, `trade_in_allowance`, `trade_in_taxable`, `replacement_price`, `replacement_asset`, `replacement_creditable_acquisition`, `arms_length`, `involuntary_disposal`, `year`, `scope_confirmed`; `car_limit`, `adjusted_input_tax_credit`, `full_input_tax_credit`, `replacement_luxury_car_tax` and `creditable_purpose` where the facts need them; see [Trade-in](#trade-in) |
| `dasp-tax` | `tax_free_component`, `taxed_element`, `untaxed_element`, `working_holiday_maker_contributions`, `unclaimed_money_rollover`, `year`, `scope_confirmed`; see [DASP tax](#dasp-tax) |
| `super-benefit-tax` | `benefit`, `interest_value`, `interest_tax_free_component`, `untaxed_element`, `recipient`, `payment_date`, `year`, `scope_confirmed`; optionally `date_of_birth`, `terminal_medical_condition`, `disability_superannuation_benefit`, `service_before_july_1983`, `low_rate_cap_used`, `untaxed_plan_cap_used`, `other_lump_sums_this_year`; see [Super benefit tax](#super-benefit-tax) |
| `payroll-tax` | `jurisdiction`, `taxable_wages`, `australian_wages`, `group_member`, `employed_all_year`, `year`, `scope_confirmed`; in Victoria, `regional_employer`; see [Payroll tax](#payroll-tax) |
| `fbt` | `type_one_value`, `type_two_value`, `year_ended`, `scope_confirmed` |
| `fbt-car-statutory` | `cost_price`, `later_accessories`, `one_third_reduction`, `days_available`, `employee_contribution`, `electric_car_exempt`, `year_ended`, `scope_confirmed` |
| `fbt-car-operating-cost` | `running_costs`, `lease_charges`, `depreciated_value`, `effective_life_years`, `days_held`, `business_use`, `employee_contribution`, `statutory_formula_taxable_value`, `electric_car_exempt`, `year_ended`, `scope_confirmed` |
| `fbt-ev-exemption-check` | `vehicle`, `car`, `current_employee`, `first_held_and_used`, `first_retail_sale`, `luxury_car_tax_value`, `luxury_car_tax_on_other_supply`, `year_ended`, `scope_confirmed`; for a plug-in hybrid, `commitment_before_1_april_2025`, `new_commitment_from_1_april_2025` and `exempt_benefit_before_1_april_2025`; see [FBT electric cars exemption](#fbt-electric-cars-exemption) |
| `fbt-loan` | `daily_balance_total`, `interest_accrued`, `income_use`, `accrued_interest_deduction`, `year_ended`, `scope_confirmed` |
| `fbt-meal-entertainment` | `method`, `meal_entertainment_expenditure`, `year_ended`, `scope_confirmed`; with `12_week_register`, `register_fringe_benefits_value`, `register_meal_entertainment_value`, `register_first_year` and `register_years_expenditure` |
| `fbt-car-parking` | `method`, `year_ended`, `scope_confirmed`; with `commercial_parking_station`, `first_business_day_fee`, `benefits`, `lowest_fee` and `employee_contribution`; with `market_value`, `first_business_day_fee`, `benefits`, `market_value` and `employee_contribution`; with `average_cost`, `first_business_day_fee`, `benefits`, `first_day_fee`, `last_day_fee` and `employee_contribution`; with `spaces`, `space_groups`, `employees_at_start`, `employees_at_end`, `eligible_spaces_at_start`, `eligible_spaces_at_end` and `employee_contribution`; with `12_week_register`, `first_business_day_fee`, `register_value`, `availability_days` and `register_first_year` |
| `fbt-debt-waiver` | `amount_waived`, `year_ended`, `scope_confirmed` |
| `luxury-car-tax` | `lines`, `year`, `scope_confirmed`; see [Luxury car tax](#luxury-car-tax) |
| `fbt-expense-payment` | `method`, `recipients_expenditure`, `amount_paid`, `employee_contribution`, `year_ended`, `scope_confirmed`; with `external`, `income_use` and `recipients_portion_deduction`; with an in-house property method, that method's inputs as for `fbt-property`; with `in_house_residual_salary_packaging` or `in_house_residual_other`, `notional_value`; with `in_house_residual_public_price`, `lowest_price`; and with every in-house method but the 2 salary packaging ones, `in_house_reduction_available` |
| `fbt-property` | `method`, `employee_contribution`, `income_use`, `contribution_deduction`, `year_ended`, `scope_confirmed`; with `in_house_trade_price` or `in_house_public_price`, `lowest_price`; with `in_house_acquired`, `arms_length_price` and `notional_value`; with `external_cost_price`, `cost_price`; with `external_expenditure`, `employer_expenditure`; with another method, `notional_value`; with an in-house method other than `in_house_salary_packaging`, also `in_house_reduction_available` |
| `fbt-residual` | `method`, `employee_contribution`, `income_use`, `contribution_deduction`, `year_ended`, `scope_confirmed`; with `in_house_salary_packaging`, `in_house_other` or `external_notional_value`, `notional_value`; with `in_house_public_price`, `lowest_price`; with `external_cost_price`, `cost_price`; with `external_expenditure`, `employer_expenditure`; with `in_house_public_price` or `in_house_other`, also `in_house_reduction_available` |
| `fbt-housing` | `method`, `employee_contribution`, `year_ended`, `scope_confirmed`; with `outside_australia`, `market_value`; with `accommodation_business`, `market_value` and `in_house`; with `base_year`, `market_value` and `tenancy_days`; with `indexed`, `prior_statutory_annual_value`, `state`, `base_year_ended` and `tenancy_days` |
| `fbt-lafha` | `method`, `allowance`, `year_ended`, `scope_confirmed`; with `maintains_home` or `fly_in_fly_out`, `accommodation_component`, `accommodation_expenses`, `food_component`, `food_expenses`, `normal_food_amount`, `adult_days` and `child_days` |
| `fbt-board` | `adult_meals`, `child_meals`, `employee_contribution`, `income_use`, `contribution_deduction`, `year_ended`, `scope_confirmed` |
| `fbt-tax-exempt-body-entertainment` | `method`, `entertainment_expenditure`, `year_ended`, `scope_confirmed`; with `per_head`, `employee_heads` and `total_heads` |
| `wine-equalisation-tax` | `dealings`, `year`, `scope_confirmed`; `rebate_claimed_earlier_in_year` and `associated_producer` with a dealing's `rebate_conditions`; see [Wine equalisation tax](#wine-equalisation-tax) |
| `fbt-meal-entertainment-register` | `register_start`, `register_end`, `entries`, `year_ended`, `scope_confirmed`; `register_years_expenditure`, required for a later year of the register's validity; see [FBT meal entertainment register](#fbt-meal-entertainment-register) |
| `fbt-minor-benefit-check` | `category`, `notional_taxable_value`, `salary_sacrifice`, `frequency`, `similar_benefits_substantial`, `other_associated_benefits_substantial`, `valuation_difficult`, `unexpected_event`, `reward_for_services`, `year_ended`, `scope_confirmed`; `in_house` for an expense payment, property or residual benefit, `tax_exempt_body_exception` for tax-exempt body entertainment and `fifty_fifty_split` for meal entertainment or entertainment facility leasing; see [FBT minor benefits](#fbt-minor-benefits) |
| `fhss` | `determination_date`, `year`, `scope_confirmed`, and either `contributions` with `determination_requested` (optional `personal_deductions`) or `determination`; optional `amount_requested` and `estimated_marginal_rate`; see [First Home Super Saver scheme](#first-home-super-saver-scheme) |
| `ato-interest-penalties` | `question` (`gic`, `sic`, `failure_to_lodge`, `statement_penalty`, `early_payment_interest` or `overpayment_interest`), `year`, `scope_confirmed`, and the question's own inputs; see [ATO penalties and interest](#ato-penalties-and-interest) |
| `ppsr-timing` | `question` (`registration`, `pmsi` or `company_grantor`), `year`, `scope_confirmed`, and the question's own inputs; see [PPSR timing](#ppsr-timing) |
| `fbt-entertainment-classifier` | `lines`, `income_tax_exempt`, `year_ended`, `scope_confirmed`; `method`, required when a line is meal entertainment; with `12_week_register`, `income_tax_register_percentage` |
| `depreciation` | `cost`, `effective_life`, `days`, `taxable_use`, `method`, `scope_confirmed`, `year`; optional `opening_adjustable_value` |
| `depreciation-low-value-pool` | `previous_closing_pool_balance`, `low_cost_assets`, `low_value_assets`, `second_element_costs`, `termination_values`, `year`, `scope_confirmed`; see [Depreciation pools and capital works](#depreciation-pools-and-capital-works) |
| `depreciation-small-business-pool` | `opening_pool_balance`, `additions`, `cost_additions`, `termination_values`, `year`, `scope_confirmed` |
| `capital-works` | `construction_expenditure`, `construction_start`, `four_percent_manner`, `days_used`, `taxable_use`, `undeducted_construction_expenditure`, `year`, `scope_confirmed` |
| `quarterly-sg` | `ordinary_time_earnings`, `qualifying_contributions`, `year`, `quarter`, `scope_confirmed` |
| `payg-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed`; optional `foreign_resident` (scale 4), `tax_offsets`, `medicare_adjustment_children`, `extra_pay_withholding` |
| `payg-gross-up` | `net_pay`, `pay_period`, `scale`, `year`, `scope_confirmed`; optional `foreign_resident` (scale 4), `study_loan`; `pay_date` with `study_loan`, required for 2025-26 |
| `study-loan-withholding` | `earnings`, `pay_period`, `scale`, `year`, `scope_confirmed`; `pay_date`, required for 2025-26 |
| `medicare-levy` | `taxable_income`, `section_9a_adjustment_applies`, `year`, `scope_confirmed`; optional `sapto_entitled`, `spouse_taxable_income`, `spouse_sapto_entitled`, `dependant_offset_entitled`, `dependent_children`, `medicare_levy_exemption_full_days`, `medicare_levy_exemption_half_days` |
| `private-health-insurance-rebate` | `income_for_surcharge_purposes`, `lines`, `year`, `scope_confirmed`; optional `spouse_income_for_surcharge_purposes`, `maintained_dependent_person`, `dependent_children`; see [Private health insurance rebate](#private-health-insurance-rebate) |
| `medicare-levy-surcharge` | `taxable_income`, `reportable_fringe_benefits`, `income_for_surcharge_purposes`, `days_without_cover`, `year`, `scope_confirmed`; optional `spouse_income_for_surcharge_purposes`, `dependent_children` |
| `low-income-tax-offset` | `taxable_income`, `year`, `scope_confirmed` |
| `seniors-pensioners-tax-offset` | `rebate_income`, `status`, `year`, `scope_confirmed`; `spouse_rebate_income`, required unless `status` is `single`; optional `spouse_taxable_income` |
| `beneficiary-tax-offset` | `rebatable_benefit`, `year`, `scope_confirmed`; see [Individual tax offsets](#individual-tax-offsets) |
| `invalid-carer-tax-offset` | `dependant`, `adjusted_taxable_income`, `dependant_adjusted_taxable_income`, `year`, `scope_confirmed`; optional `days`, `spouse_adjusted_taxable_income`, `spouse_days`, `maintained_by_others`, `ftb_b_shared_care`, `different_spouses` |
| `zone-tax-offset` | `area`, `days`, `year`, `scope_confirmed`; optional `prior_year` with `prior_days`, `continuous_residence`, `includes_first_day`, `dependants`, `sole_parent_days`, `invalid_carer_tax_offset`, `remote_area_allowance` |
| `net-medical-expenses-tax-offset` | `net_medical_expenses`, `adjusted_taxable_income`, `year`, `scope_confirmed`; optional `spouse_adjusted_taxable_income`, `dependent_children` |
| `lump-sum-arrears-tax-offset` | `taxable_income`, `net_capital_gain`, `accrual_years`, `year`, `scope_confirmed`; see [Lump sum payment in arrears tax offset](#lump-sum-payment-in-arrears-tax-offset) |
| `division-293` | `division_293_income`, `low_tax_contributions`, `year`, `scope_confirmed` |
| `sb-cgt-concessions` | `entity`, `share_or_trust_interest`, `acquired`, `event_date`, `partnership_asset`, `passively_held`, `business`, `previous_year_turnover`, `year_before_turnover`, `likely_turnover`, `year_turnover`, `net_asset_value`, `active_days`, `business_ceased`, `date_of_birth`, `retirement`, `permanently_incapacitated`, `last_rollover_event`, `earnout_latest_due`, `year`, `scope_confirmed`; the facts some answers call for; see [Small business CGT concessions](#small-business-cgt-concessions) |
| `small-business-income-offset` | `taxable_income`, `basic_income_tax_liability`, `offsets_applied_first`, `net_small_business_income`, `shares`, `prescribed_person`, `year`, `scope_confirmed`; with net small business income of your own, `business` and the 4 turnovers; see [Small business income offset](#small-business-income-offset) |
| `non-commercial-losses` | `activity`, `assessable_income`, `deductions`, `brought_forward`, `part_year`, `earlier_years`, `real_property_value`, `other_assets_value`, `taxable_income`, `business_losses`, `assessable_fhss_released_amount`, `reportable_fringe_benefits`, `reportable_super_contributions`, `total_net_investment_loss`, `net_exempt_income`, `year`, `scope_confirmed`; `assessable_income_estimate` for a part year and `other_income` for an excepted activity; see [Non-commercial losses](#non-commercial-losses) |
| `psi-test` | `earner`, `mainly_personal_efforts`, `determination_in_force`, `payers`, `agent`, `principal_work_by_others`, `apprentice`, `premises`, `year`, `scope_confirmed`; `commission` and `principal_payments` for an agent; see [PSI test](#psi-test) |
| `government-super-amounts` | `income-tests`' 16 components and `residency`, `eligible_income`, `personal_contributions`, `concessional_contributions`, `return_lodged`, `under_71`, `temporary_visa_held`, `non_concessional_within_cap`, `total_super_balance_below_cap`, `year`, `scope_confirmed`; for a spouse contribution all of `spouse_contributions`, `spouse_assessable_income`, `spouse_assessable_fhss_released_amount`, `spouse_reportable_fringe_benefits_total`, `spouse_reportable_employer_super_contributions`, `spouse_excess_concessional_contributions`, `spouse_non_concessional_within_cap` and `spouse_total_super_balance_below_cap`; see [Government super amounts](#government-super-amounts) |
| `work-expenses` | `method` (`home_office`, `car_cents_per_kilometre`, `car_log_book` or `self_education`), `reimbursed`, `allowance_received`, `year`, `scope_confirmed`, and the inputs the method and year take; see [Work expenses](#work-expenses) |
| `rental-property` | `ownership_share`, `acquired`, `disposed`, `days_available`, `section_26_50_applies`, `income`, `expenses`, `rental_only_expenses`, `loans`, `borrowing_loans`, `capital_works`, `depreciating_assets`, `year`, `scope_confirmed`; optional `other_taxable_income`; see [Rental property](#rental-property) |
| `foreign-resident-withholding` | `payment`, `acquisition_date`, `asset`, `market_value`, `cost_base_first_element`, `option_payments`, `vendors`, `exclusion`, `class_variation_to_nil`, `year`, `scope_confirmed`; see [Foreign resident withholding](#foreign-resident-withholding) |
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
| `personal-tax-estimate` | `year`, `scope_confirmed`, `residency`, `taxable_income` or `assessable_labour_income` (then optional `other_assessable_income`, `work_related_deductions`, `other_deductions`), optional `tax_withheld` or, to work it out, one of `payg-withholding`, `study-loan-withholding` and `seniors-pensioners-withholding` with `pays_in_year` and optional `schedule-5-method-a`, for a resident from 2027-28 `net_labour_income` (required beside `taxable_income`), and for a full-year resident `medicare-levy` and `medicare-levy-surcharge`; optional `seniors-pensioners-tax-offset`, `study-loan-repayment`, `division-293` and `private-health-insurance-rebate`; see [Personal tax estimate](#personal-tax-estimate) |
| `fbt-return` | `employer`, `year_ended`, `scope_confirmed` and `benefits`, a list of objects each with `employee`, `command` (an FBT benefit worksheet), `input` (that worksheet's arguments), `gst_type` (`type_1` or `type_2`) and `excluded`; see [FBT return](#fbt-return) |
| `derive` | `command`, one of `fbt`, `fbt-car-statutory`, `fbt-car-operating-cost`, `fbt-loan`, `fbt-meal-entertainment`, `fbt-car-parking`, `fbt-debt-waiver`, `fbt-expense-payment`, `fbt-property`, `division-293`, `pension-minimum`, `payg-withholding`, `schedule-5-method-a`, `schedule-5-method-b-i`, `schedule-5-method-b-ii`, `termination-payment`, `etp-tax-free-component`, `unused-leave-payment`, `working-holiday-maker-withholding`, `study-loan-withholding`, `seniors-pensioners-withholding`, `quarterly-sg`, `borrowing-expenses`, `borrowing-expenses-schedule`, `resident-tax`, `foreign-resident-tax`, `low-income-tax-offset`, `seniors-pensioners-tax-offset`, `medicare-levy`, `medicare-levy-surcharge`, `division-296`, `contribution-caps`, `study-loan-repayment`, `capital-gains`, `main-residence`, `fbt-residual`, `fbt-housing`, `fbt-lafha`, `fbt-board`, `fbt-tax-exempt-body-entertainment`, `gst`, `depreciation`, `depreciation-low-value-pool`, `depreciation-small-business-pool`, `capital-works`, `hire-purchase-schedule`, `chattel-mortgage-schedule`, `loan-transactions`, `lump-sum-arrears-tax-offset`, `income-tests`, `government-super-amounts`, `business-capital-expenditure`, `balancing-adjustment`, `trade-in`, `dasp-tax`, `super-benefit-tax`, `gst-margin-scheme`, `payroll-tax`, `cgt-parcels`, `work-expenses` and `rental-property`, and `input`, that command's arguments; see [Derivations](#derivations) |

The 2 car commands, the 11 FBT benefit commands (`fbt-loan`, `fbt-meal-entertainment`, `fbt-car-parking`, `fbt-debt-waiver`, `fbt-expense-payment`, `fbt-property`, `fbt-residual`, `fbt-housing`, `fbt-lafha`, `fbt-board` and `fbt-tax-exempt-body-entertainment`), `main-residence`, `borrowing-expenses`, `borrowing-expenses-schedule`, `fuel-tax-credits`, `division-296`, `termination-payment`, `etp-tax-free-component`, `unused-leave-payment`, the 3 `schedule-5` commands, `working-holiday-maker-withholding`, `seniors-pensioners-withholding`, `seniors-pensioners-tax-offset`, `medicare-levy-surcharge`, `foreign-resident-tax`, `payg-gross-up`, `hire-purchase-schedule`, `chattel-mortgage-schedule`, `loan-transactions`, `lump-sum-arrears-tax-offset`, `fbt-entertainment-classifier`, `fbt-meal-entertainment-register`, `fbt-minor-benefit-check`, `fhss`, `ppsr-timing`, `income-tests`, `business-capital-expenditure`, `balancing-adjustment`, `trade-in`, `gst-margin-scheme`, `franking-account`, `wine-equalisation-tax`, `luxury-car-tax`, `payroll-tax`, `dasp-tax`, `super-benefit-tax`, `cgt-parcels`, `fbt-ev-exemption-check`, `sb-cgt-concessions`, `ato-interest-penalties`, `government-super-amounts`, `non-commercial-losses`, `small-business-income-offset`, `psi-test` and `rental-property` have no Python original; their arguments follow the same
rules. `division-296` alone takes a negative amount, in `total_super_earnings`. `main-residence` also takes dates as `YYYY-MM-DD` strings, lists of
`{"start", "end"}` periods (with `"floor_area"` for income use) and `null`
for no new home. `borrowing-expenses` takes `loan_start` as a `YYYY-MM-DD`
string and `loan_term_end` and `repaid_on` as dates or `null`. `hire-purchase-schedule` takes `instalments` as a JSON integer, `timing` as `in_arrears` or `in_advance`, `start_date` as a `YYYY-MM-DD` string and `segments`, `payout` and `gst` as described under [Hire purchase schedule](#hire-purchase-schedule), and refuses a field it, a segment, the payout or the GST facts do not take. `business-capital-expenditure` takes `items` as a list of objects, and refuses a field it or an item does not take. `balancing-adjustment` takes `years` as a list of objects and a car's `car_limit` as an object or `null`, and refuses a field it, a row or `car_limit` does not take; `trade-in` takes them as it does, and refuses an optional input its other facts do not call for. `cgt-parcels` takes `parcels` and `disposals` as lists of objects, and refuses a field it, a parcel or a disposal does not take. `termination-payment` takes `completed_years_of_service` as a JSON integer and `date_of_birth` as a `YYYY-MM-DD` string. `unused-leave-payment` takes `leave_accrual_start` as a `YYYY-MM-DD` string, `termination_reason` as `normal`, `genuine_redundancy`, `early_retirement_scheme` or `invalidity`, and its 4 accrual parts as optional decimal strings, described under [Unused leave payment](#unused-leave-payment). `schedule-5-method-b-i` takes `back_payment_periods` as a list of `{"earnings", "back_payment", "withheld"}` decimal strings, one for each earlier pay period. Every argument is required, as the Python functions had no defaults, except `depreciation`'s optional `opening_adjustable_value`: give it, as a decimal string, for a year after the asset's start time, as cost less the earlier years' decline in value; and `medicare-levy`'s optional family inputs, described under [Medicare levy](#medicare-levy-and-low-income-tax-offset). `medicare-levy` refuses a field it does not take, as does `medicare-levy-surcharge`, whose optional inputs are described under [Medicare levy surcharge](#medicare-levy-surcharge). `payg-withholding`'s optional inputs are described under [PAYG withholding](#payg-withholding), `payg-gross-up` under [PAYG gross-up](#payg-gross-up), `study-loan-withholding`'s `pay_date` under [Study loan withholding](#study-loan-withholding), `seniors-pensioners-withholding`'s `study_loan` and `pay_date` under [Seniors and pensioners](#seniors-and-pensioners) and `seniors-pensioners-tax-offset`'s spouse inputs under [Seniors and pensioners tax offset](#seniors-and-pensioners-tax-offset). `income-tests` requires every component and refuses a field it does not take; see [Income tests](#income-tests). `sb-cgt-concessions` takes `previous_year_turnover`, `year_before_turnover`, `business_ceased`, `last_rollover_event` and `earnout_latest_due` as `null` for none, `active_days` as a JSON integer and its flags as booleans, requires the optional facts its other answers call for, and refuses any other field. `small-business-income-offset` takes `shares` as a JSON list of `{amount, deductions}` objects, `previous_year_turnover` and `year_before_turnover` as `null` for none and `prescribed_person` and `small_business_when_stopped` as booleans, takes `business` and the turnovers only with net small business income of your own, and refuses any other field. `non-commercial-losses` takes `earlier_years` as a JSON list of 4 entries, each `null` or `{assessable_income, deductions}`, `part_year` as a boolean and `taxable_income` as a decimal string that may be negative, takes `assessable_income_estimate` only for a part year and `other_income` only for an excepted activity, and refuses any other field. `psi-test` takes `payers` as a JSON list of objects with `amount`, 5 booleans and an optional `source` label, `premises` as an object of 5 booleans and its other answers as booleans, takes `commission` and `principal_payments` only for an agent, and refuses any other field. `government-super-amounts` takes its yes-or-no facts as booleans, `non_concessional_within_cap`, `total_super_balance_below_cap` and the spouse's 2 like it also as `null` for not established, and the 8 spouse inputs all together or not at all, and refuses any other field. `work-expenses` takes the inputs its `method` and year need, `log_book_weeks` as a JSON integer and its flags as booleans, and refuses any other field; see [Work expenses](#work-expenses). `ato-interest-penalties` takes its question's amounts as decimal strings, dates as `YYYY-MM-DD` strings, `payments` and `parts` as lists of objects and its flags as booleans, and refuses a field its question does not take; see [ATO penalties and interest](#ato-penalties-and-interest). The FBT benefit commands take the inputs their `method` needs and refuse any other, as [FBT loans, meal entertainment and car parking](#fbt-loans-meal-entertainment-and-car-parking), [FBT debt waivers, expense payments and property](#fbt-debt-waivers-expense-payments-and-property) and [FBT residual, housing, LAFHA, board and entertainment benefits](#fbt-residual-housing-lafha-board-and-entertainment-benefits) describe. `fbt-entertainment-classifier` takes `lines` as a list of line objects and refuses a field it does not take, as [FBT entertainment classifier](#fbt-entertainment-classifier) describes. `wine-equalisation-tax` takes `dealings` as a list of dealing objects and refuses a field it, a dealing, `average_wholesale` or `rebate_conditions` does not take; see [Wine equalisation tax](#wine-equalisation-tax). `luxury-car-tax` takes `lines` as a list of line objects and refuses a field it or a line does not take; see [Luxury car tax](#luxury-car-tax). `fuel-tax-credits` takes `lines` the same way, `rate_at_period_end` as an object and `return_date` as a `YYYY-MM-DD` string, and refuses a field it or a line does not take; see [Fuel tax credits](#fuel-tax-credits). Input over 8 MiB,
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
of the decision table that classified it. Nor is
`fbt-meal-entertainment-register`: its checks carry their authorities, and its
totals are sums of the register's entries.
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
earlier year's maximum deduction, worked on the figures that year used, and
a borrowing expenses schedule's steps write in each loan's term.
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
1997 ss 27-80, 27-95, 40-185, 40-230 and 40-305. The payroll tax steps cite
each jurisdiction's payroll tax Act by section or clause and version, with the
revenue office page that publishes the same figure, and the ACT's rates cite
its 2026 determination (DI2026-151). The CGT parcels steps cite
ITAA 1997 ss 102-5, 102-10, 102-15, 104-10, 110-55, 115-25 and 115-100 and
write in each counted part's gain or loss; each part in `matches` carries its
own dates and figures.

## Supported periods

| Worksheet | Period and boundary |
| --- | --- |
| ATO penalties and interest | `year` is the financial year of the question's end date, 2020-21 to 2027-28; GIC days from 1 July 1999 and SIC days from 1 April 2005 to 31 December 2026, the last published quarter, and GIC days after it at a stated assumed rate; penalty units from 28 December 2012; established behaviour, positions, disclosures, size and entitlement facts; amounts before remission |
| Luxury car tax | Tax periods in 2019-20 to 2026-27, the years from the $10,000 refunds to the last thresholds the ATO publishes, with supplies from 2012-13 for adjustments; established cars, values, ratings, quotes and refund conditions |
| GST | 2025-26 to 2027-28, one ordinary taxable supply at 10%; no classification or BAS |
| Fuel tax credits | 2022-23 to 2026-27, fuel acquired from 1 July 2022 to 31 January 2027 at the ATO's 16 published rate periods; established entitlement, fuel, quantity and use |
| GST margin scheme | 2025-26 to 2027-28, one taxable supply of real property under a contract from 9 December 2008; the s 75-5 conditions, the consideration or a valuation under s 75-10(3) items 1 to 4, and a lot's share under s 75-15; acquisitions s 75-11 governs are refused |
| Resident tax | 2024-25 to 2027-28; full-year resident ordinary income, before offsets and levies |
| Foreign resident tax | 2024-25 to 2027-28; full-year foreign resident ordinary income, no tax-free threshold; no Medicare levy or low income tax offset applies |
| CGT | 2025-26 and 2026-27, resident individual's established gains and losses; losses before the 50% discount |
| Main residence | CGT events in 2024-25 to 2026-27; one dwelling disposed of by a resident individual, before the discount |
| Borrowing expenses | 2017-18 to 2027-28, flagged for s 26-155 from 2027-28; one loan's established expenditure, use and dates |
| Rental property | 2020-21 to 2026-27, the effect on tax 2024-25 to 2026-27; one residential property let whole at a market rent by an individual not in a business of letting; established income, expense lines, loans, construction expenditure and decline in value; not holiday homes s 26-50 denies, the 4% manner or s 26-155 |
| Borrowing expenses schedule | 2017-18 to 2027-28, loans first drawn from 1 July 2017, every later year of each loan shown and those after 2027-28 marked projected; each loan's established expenditure, use by year, dates and refinancing |
| Balancing adjustment | Balancing adjustment events in 2024-25 to 2026-27, before Act No. 49 of 2026's CGT changes from 1 July 2027; one depreciating asset's established cost, termination value and each year's decline and deduction, with s 40-370 for a car from 2015-16; no pooled or written-off assets |
| Franking account | 2025-26 and 2026-27, income years from 1 July to 30 June; one corporate tax entity that meets the residency requirement and is a franking entity all year; established events, amounts and corporate tax rate for imputation purposes; no consolidated groups, ceasing entities or late balancers |
| CGT parcels | Contracts in 2025-26 and 2026-27, before Act No. 49 of 2026's changes; parcels of shares or units acquired after 21 September 1999 with established cost base elements; first in, first out or selected |
| Trade-in | 2025-26 and 2026-27, at the car limits of $69,674 and $69,883; one depreciating asset traded in at arm's length for a replacement first held and used that year; established GST status, credits, allowance and price |
| DASP tax | Payments in 2025-26 to 2027-28; one departing Australia superannuation payment's established components, at the ordinary or working holiday maker rates; no excess untaxed roll-over amount |
| Super benefit tax | Payments in 2020-21 to 2026-27; one superannuation lump sum from a complying plan to a member, a death benefits dependant or a non-dependant; the low rate cap only to 2023-24, as nobody is between preservation age and 60 from 1 July 2024; no income streams, roll-overs or Medicare levy |
| Payroll tax | 2026-27; one employer outside a group that pays wages all year in one state or territory; established taxable and Australian wages and, in Victoria, regional status; before exemptions, rebates, discounts and caps |
| Business capital expenditure | 2020-21 to 2027-28, items incurred from 1 July 2020 under s 40-880 as amended by Act No 92 of 2020; each item's established capital nature, business, taxable purpose extent, start-up facts and exclusions |
| Wine equalisation tax | Dealings in 2023-24 to 2027-28, after the 1 July 2023 end of the transitional rebate rules; the rebate cap of $350,000 to 2025-26 and $400,000 from 2026-27; established dealings, quotes, rebate conditions and earlier claims |
| FBT | Years ended 31 March 2026 and 2027; established type 1 and type 2 taxable values, ordinary employer |
| FBT car benefits | Years ended 31 March 2026 and 2027; one car's established base value or operating costs, the 20% statutory percentage and a deemed depreciation rate of 200% over the established effective life |
| FBT electric cars exemption | Years ended 31 March 2026 and 2027, before the change announced for 1 April 2027; first retail sales in 2012-13 to 2026-27, at the ATO's fuel-efficient luxury car tax thresholds; established vehicle type, dates, value and commitments |
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
| FBT meal entertainment register | Years ended 31 March 2026 and 2027; one 12 week register's period, entries and validity years, with the 20% rule from the yearly expenses given; not whether the period is representative or an entry false |
| FBT minor benefits | Years ended 31 March 2026 and 2027; one benefit's established category, notional taxable value and s 58P(1)(f) criteria, weighed as TR 2007/12's examples weigh them |
| First Home Super Saver scheme | Determinations made from 1 July 2018 to 31 December 2026; established eligibility and eligible employer voluntary and personal contributions from 1 July 2017, with no excess contributions; associated earnings an estimate |
| PPSR timing | Registrations, possession or attachment and security agreements in 2025-26 and 2026-27, ACT legal time; s 588FL critical times from 1 July 2025 to 30 June 2027; the public holidays the count spans as listed by the caller |
| FBT entertainment classifier | Years ended 31 March 2026 and 2027; an ordinary taxable employer's entertainment expense lines with established facts, split per head between attendee classes; no tax-exempt bodies, Subdivision 32-B exceptions, salary packaging or reportable fringe benefits amounts |
| Depreciation | 2025-26 and 2026-27, an ordinary tangible Division 40 asset's start year, or a later year from its established opening adjustable value; established life and use |
| Depreciation pools | 2020-21 to 2026-27; one holder's low-value pool, or a small business entity's general small business pool with each year's low pool value threshold; established taxable use amounts |
| Capital works | 2020-21 to 2026-27, works begun from 16 September 1987 at 2.5% or 4%; established expenditure, days, use and undeducted construction expenditure |
| SG | Complete quarters of 2025-26; established OTE, eligibility and qualifying contributions |
| Contribution caps | 2024-25 to 2026-27; one individual's classified contributions, no bring-forward period started in the 2 previous years |
| Pension minimum | 2024-25 to 2027-28; one account-based pension paying under SISR Schedule 7 |
| Study loan repayment | 2025-26 and 2026-27; one individual's established repayment income at the marginal rates; the formula amount before the loan-balance limit |
| Study loan withholding | Regular weekly, fortnightly, monthly or quarterly pays from 24 September 2025 on scale 1, 2, 3, 5 or 6; the Schedule 1 amount plus the Schedule 8 component |
| Medicare levy | 2024-25 to 2027-28; full-year resident; the 2% rate, the low-income reduction at the single or SAPTO thresholds, the family reduction from established spouse, offset and dependent child facts, and established full and half exemption days |
| Medicare levy surcharge | 2024-25, 2025-26 and 2026-27, and 2027-28 with no day without hospital cover (nil); full-year resident in no exemption category, single without dependants or married for the whole year; established income for surcharge purposes and days without hospital cover |
| Private health insurance rebate | 2020-21 to 2026-27, premiums paid to 31 March 2027; one adult insured under complying policies whose insured are all eligible persons, from the statement's lines; established income for surcharge purposes; no dependant-only policy, spouse's claim or trustee's offset |
| Low income tax offset | 2024-25 to 2027-28; resident individual's established taxable income; the offset amount, not applied against tax |
| Seniors and pensioners tax offset | 2024-25 to 2027-28; an established eligible individual's rebate income and whole-year status, and a spouse's rebate income; the offset amount with any transfer from a resident spouse, not applied against tax |
| Beneficiary tax offset | 2020-21 to 2026-27; an established rebatable benefit in assessable income; the offset amount, not compared with SAPTO or applied against tax |
| Invalid and invalid carer tax offset | 2020-21 to 2025-26; one dependant at a time, with established adjusted taxable incomes and the days that count; the offset amount, not applied against tax |
| Zone and overseas forces tax offsets | 2020-21 to 2026-27; one zone, special area or overseas forces a run, with residence days and days carried from an earlier year, and the dependant, sole parent and invalid carer base amounts; the offset amount, not applied against tax |
| Net medical expenses tax offset | 2016-17 to 2018-19; established net expenses for disability aids, attendant care or aged care and adjusted taxable income; the offset amount, not applied against tax |
| Lump sum payment in arrears tax offset | Years of receipt 2013-14 to 2026-27; a full-year resident individual's established eligible lump sums by accrual year, recent accrual years from 2012-13 at resident rates and earlier distant years at their average; the offset amount, not applied against tax, without the Medicare levy and surcharge relief |
| Government super amounts | 2020-21 to 2026-27; a full-year resident's `income-tests` components, eligible income, contributions received in the year and stated tests; the co-contribution, LISTO (s 12C(1) only) and the spouse contribution tax offset; no temporary visa holder |
| Division 293 | 2024-25 to 2027-28; established Division 293 income and low tax contributions; no defined benefit interests |
| Small business CGT concessions | CGT event A1 in 2025-26 and 2026-27, before Act No. 49 of 2026's changes to Division 152; an asset other than a share, a trust interest or a partnership right, with established turnover, net value, active days and retirement facts; conditions only, not amounts |
| Small business income offset | 2020-21 to 2026-27; one individual, not a trustee or a prescribed person under s 102AC of the ITAA 1936; established net small business income, partnership and trust amounts, basic income tax liability and offsets applied first |
| Non-commercial losses | 2020-21 to 2026-27; one individual's business activity, alone or in partnership, with established figures; no bankruptcy or section 40-880 expenditure; the Commissioner's discretion not decided |
| PSI test | 2020-21 to 2026-27; one individual's PSI with the operator's answers to each test; no s 87-40 for an agent on 75% or more commission; a determination not decided |
| Work expenses | 2020-21 to 2026-27; one employee's working from home, car (cents per kilometre or log book) or self-education deduction, one `method` a call; no reimbursed expenses or allowance for them; occupancy expenses not covered |
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
liability. SMSF fund tax, Division 296 fund earnings and their attribution, the transfer balance cap and trusts remain
outside these worksheets.

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

## Private health insurance rebate

`private_health_insurance_rebate` works out one person's private health
insurance tax offset (ITAA 1997 Subdivision 61-G) and any excess premium
reduction to repay (Private Health Insurance Act 2007 s 282-18) for an income
year from 2020-21 to 2026-27. It takes the lines of the person's private
health insurance statements as `lines`: each line's `premiums_eligible`
(label J, the person's share of the premiums paid without lifetime health
cover loading), `rebate_received` (label K, the person's share of the premium
reduction) and `benefit_code` (label L: 30, 35 or 40 for premiums paid
1 July to 31 March, 31, 36 or 41 for 1 April to 30 June, with the oldest
person insured under 65, 65 to 69 or 70 or over).

The tier comes from `income_for_surcharge_purposes` as established
(`income-tests` works it out for 2024-25 to 2026-27), against the singles
thresholds, or the family thresholds (PHI Act s 22-30) for a person married
on 30 June, whose `spouse_income_for_surcharge_purposes` is added, or who
maintained a dependent person who is their child or dependent sibling in the
year (`maintained_dependent_person`). The family thresholds are double, and
$1,500 more for each `dependent_children` after the first (s 22-40(4)); the
count alone does not put a person on them. Each line's percentage is the
ATO's for its period, band and tier, and its share of the PHII benefit is
rounded to the cent. The offset is the share less the rebate received; the
excess is the rebate received less the share. Each line stands alone, so an
offset on one and an excess on another are both shown; give premiums whose
reductions within a period differed as separate lines.
The offset is refundable (s 67-23).

It refuses premiums paid from 1 April 2027, whose percentages the ATO had not
published (`private_health_insurance_rebate.rate_unpublished`), a line with a
lifetime health cover loading (`private_health_insurance_rebate.lhc_loading`)
and malformed lines (`private_health_insurance_rebate.lines_invalid`).
`personal-tax-estimate` composes it for a full-year resident, taking the
income figures from `medicare-levy-surcharge`: the offset comes off the total
and the excess goes on it. The tests hold it to the ATO's Narelle, Peta, Mike
and Elle, Zoe and Charlie, Toby and Donna examples and every published
percentage and threshold; see
[calculation evidence](docs/calculation-evidence.md#private-health-insurance-rebate).

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

## FBT electric cars exemption

`fbt-ev-exemption-check` decides whether a car's benefits in the FBT year
ended 31 March 2026 or 2027 are exempt under FBTAA s 8A. It lists each
condition with its authority and whether it is met, names every one that fails
in `failing_conditions`, and gives `exempt`:

- `car`: the vehicle is a car, designed to carry a load of less than 1 tonne
  and fewer than 9 passengers, not a motor cycle (s 136(1));
- `current_employee`: the benefit is for a current employee (s 8A(1)(a));
- `zero_or_low_emissions_vehicle`: `vehicle` is `battery_electric` or
  `hydrogen_fuel_cell_electric` (s 8A(2) to (4)). A `plug_in_hybrid_electric`
  vehicle has stopped qualifying from 1 April 2025, except under the Electric
  Car Discount Act 2022 Sch 2 item 5(2), which takes 3 more inputs and shows
  3 rows in its place: `commitment_before_1_april_2025` (a binding commitment
  to provide the car to the employee, made before 1 April 2025 for a period
  that began before then), `new_commitment_from_1_april_2025` (such as an
  exercised option to extend, a break in a novation, changed lease payments or
  a new employer, which ends the exemption) and
  `exempt_benefit_before_1_april_2025`;
- `first_held_and_used_from_1_july_2022`: the earliest day anyone both held
  and used the car, `first_held_and_used`, is on or after 1 July 2022 (Sch 1
  item 7(2));
- `no_luxury_car_tax_at_first_retail_sale`: no luxury car tax became payable
  on the first retail sale before the benefit (s 8A(1)(c)). For a sale on
  `first_retail_sale` within or before the FBT year, `luxury_car_tax_value`,
  the price with GST and without luxury car tax or on-road charges (LCT Act
  s 5-20), does not exceed the fuel-efficient threshold for the sale's
  financial year (s 25-1(1)): the ATO's table, from $75,375 for 2012-13 to
  $91,387 for 2024-25 and 2025-26 and $91,661 for 2026-27. An earlier sale is
  refused. A sale after the FBT year comes after every benefit in it, so the
  condition is met and no threshold applies;
- `no_luxury_car_tax_on_other_supply`: `luxury_car_tax_on_other_supply` is
  `false` (s 8A(1)(c)).

A car's result carries `car_worksheet_input`, `{"electric_car_exempt": ...}`,
for `fbt-car-statutory` and `fbt-car-operating-cost`, and `consequences`: for
an exempt car, a nil taxable value whose notional value still counts towards
the employee's reportable fringe benefits amount (s 135P(3)), and car expense
benefits exempt under s 53(1), which do not. It refuses a car first held and
used after the FBT year ends, a first retail sale within the FBT year, after
the car was first held and used, on which luxury car tax was payable (the
benefits before it are exempt and those after it are not), plug-in hybrid
facts for another vehicle, and the year ended 31 March 2028: the government announced on 5 May
2026 that from 1 April 2027 only a car costing $75,000 or less keeps the full
exemption, and the ATO's page said on 19 November 2026 that this was not yet
law. A car imported by its user rather than sold at retail, and a plug-in
hybrid that was not a fuel-efficient car when first sold, are outside its
scope. See [calculation evidence](docs/calculation-evidence.md#fbt-ev-exemption-check).

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
the recipients' payments come off each benefit up to its value. For these
methods and market value, give one benefit's payment as a decimal string.
With several benefits and a combined payment above one benefit's value, give
`employee_contribution` as an object with `capped_total`, a decimal string after
capping each payment at its own benefit's value and adding them. Keep any
half-cent from average cost, up to 3 decimal places. The worksheet works
from the exact gross value and rounds its reported amounts to cents. When using
that object, the worksheet reports the applied contribution without the excess
you already took out. `market_value` values each benefit at the
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

## FBT meal entertainment register

`fbt-meal-entertainment-register` checks whether a 12 week meal entertainment
register is valid for the FBT year ended 31 March 2026 or 2027 (FBTAA ss 37CA
to 37CF), and works out the register's values for `fbt-meal-entertainment`.
`entries` lists what the register records for its period, one object for each
provision of meal entertainment: `date`, `recipients` (counts of `employees`,
`associates` and `others`), `cost`, `kind`, `location`, `on_premises`,
`in_house_dining_facility` (on the employer's premises only) and
`fringe_benefits_value`, the part of the cost that is meal entertainment
fringe benefits, which the employer works out. A detail the register does not
record is left out or `null`.

The result lists 5 checks, each with its authority, whether it passed and why:
the period is continuous and at least 12 weeks, 84 days with both ends
counted (s 37CC(1) and (3)); the register records meal entertainment; every
entry records the 6 details s 37CE(1) requires, naming each entry and detail
missing; the register is valid for the year (s 37CD(1) and (2): the FBT year
its period ends in, or the second year where it spans 2, and the 4 years
after); and no later year's meal entertainment expenses were more than 20%
higher than its first valid year's (s 37CD(3)), from
`register_years_expenditure`. `valid` is true only when all 5 pass, and then
`fbt_meal_entertainment_input` holds the register's values, first valid year
and yearly expenses for `fbt-meal-entertainment`, which needs only the year's
`meal_entertainment_expenditure` added. The register percentage is kept exact
and shown to the cent; an entry with no cost is left out of the totals and
counted in `entries_without_cost`.

It refuses an entry dated outside the period, a fringe benefits value above
the entry's cost or above nil with no employee or associate among its
recipients, an in-house dining facility answer for meal entertainment off the
employer's premises, a period that ends before it starts or spans more than
2 FBT years (`period_invalid`) and a detail it cannot read (`entry_invalid`).
Whether the register was kept, and meal entertainment provided, throughout
the period (s 37CC(1)), whether the period is representative of the first
valid year (s 37CC(2)), whether an entry is false or misleading (s 37CF) or
was made late (s 37CE(2)), and whether a later register replaces this one, are
the employer's to establish. The car
parking register is outside it. See
[calculation evidence](docs/calculation-evidence.md#fbt-meal-entertainment-register).

## FBT minor benefits

`fbt-minor-benefit-check` screens one benefit, in the FBT year ended 31 March
2026 or 2027, against the minor benefits exemption of FBTAA s 58P. It decides
the conditions and lists each with its authority, naming every one that fails
in `failing_conditions`:

- `not_in_house`: an `expense_payment`, `property` or `residual` benefit that,
  as a fringe benefit of that kind, would not be in-house (`in_house` false:
  s 58P(1)(c));
- `tax_exempt_body_exception`: for `tax_exempt_body_entertainment`, the
  entertainment is incidental to entertaining outsiders and neither a meal nor
  provided in connection with one, other than light refreshments
  (`incidental_to_outsiders_no_meal`), or on eligible premises of the employer
  (s 136(1)) solely to recognise the employee's special achievements
  (`special_achievement_on_premises`), not `none` (s 58P(1)(d));
- `not_fifty_fifty_split`: `meal_entertainment` or
  `entertainment_facility_leasing` is not valued by the 50/50 split method
  (Division 9A or s 152B: TR 2007/12 para 14). It is never met:
  `fifty_fifty_split` false is refused, and the benefit is then screened under
  its own category;
- `not_salary_sacrifice`: not under a salary sacrifice arrangement
  (TR 2007/12 para 16);
- `less_than_300`: `notional_taxable_value`, for this one benefit to this one
  person in the current year, is less than $300 (s 58P(1)(e)); $300 itself
  fails.

The criteria of s 58P(1)(f) are stated facts: `frequency`
(`infrequent_and_irregular`, `infrequent_and_regular` such as each Christmas,
or `frequent`), whether the total of similar benefits and of other associated
benefits is substantial, whether the value is hard to work out, whether it
helped with an unexpected event and whether it is a reward for services.
`criteria` shows which way each weighs, and `screen` weighs them as TR 2007/12,
the Commissioner's public ruling, weighs its examples: `exempt` where similar
benefits are not frequent, neither total is substantial and the benefit is
not both regular and a reward for services (examples 1 to 3, 5 to 7 and 9 to
11: a yearly gift or party, flowers, a car lent after a storm,
ad hoc tolls, a recognition voucher, a one-off gym membership, babysitting
during unexpected overtime), `not_exempt` where they are frequent with a
substantial total, a reward for services, readily valued and not for an
unexpected event (example 8, monthly incentive vouchers) or a condition fails,
and `judgement_required` otherwise, with `exempt` null. `consequences` says
what each outcome means: an exempt benefit is not a fringe benefit
(s 136(1), para (g)), so it adds nothing to the FBT return or the reportable
amount. Whether benefits are associated (s 58P(2)), the notional taxable
value, benefits exempt under another provision and the income tax and GST
treatment of exempt entertainment are out of scope. TR 2007/12 para 11 still
lists airline transport benefits; Act No. 88 of 2013 repealed that exclusion.
See [calculation evidence](docs/calculation-evidence.md#fbt-minor-benefit-check).

## First Home Super Saver scheme

`fhss` works out a first home super saver determination's components and
what its release pays and costs, for a determination made from 1 July 2018 to
31 December 2026 (`year` is its financial year). Give one of:

- `contributions`, to estimate the determination: each with `date`, `amount`
  and `kind`, `employer_voluntary` (salary sacrifice or another employer
  contribution that is not mandated, so concessional) or `personal` (made by
  you), with `determination_requested` and, for a year in which some personal
  contributions were deducted, `personal_deductions`. Only these 2 kinds are
  eligible (TAA 1953 Sch 1 s 138-35(2)); any other is refused. Contributions
  count earliest first, a non-concessional one before a concessional one made
  at the same time, and in a year where you personally made both kinds the
  year's non-concessional ones first (s 138-30(2) and (3)); the deducted ones
  are taken to be the year's latest, as in the section's example. At most
  $15,000 of a financial year's contributions and $50,000 in all count
  ($30,000 for a determination requested before 1 July 2022), on contributions
  in full (s 138-35(1)). `years` and `contributions` show what each year and
  each contribution counts.
- `determination`, the components a determination states: `concessional`
  (the 85% amount), `non_concessional` and `associated_earnings`.

The maximum release amount is the non-concessional contributions counted,
85% of the concessional and the associated earnings (ss 138-25 and 138-30).
Associated earnings (s 138-40) compound daily at the shortfall interest
charge rate (s 280-105(2): the ATO's published annual rate for the quarter
over the days in the calendar year) on each contribution's counted amount, 85%
of it for a concessional one, from the first day of its month (1 July 2017 for
2017-18) to the determination day, rounded down to the dollar per
contribution. Worked from contributions they are an estimate:
`associated_earnings_basis` says `estimate`, because the Commissioner's
determination is what counts and GN 2024/1 example 6 (Jill) states $5,190
where this method gives $7,859 (see
[calculation evidence](docs/calculation-evidence.md#fhss)).

On release of `amount_requested` (the maximum unless given, never more:
s 131-10(1) item 4) it gives the assessable FHSS released amount (ITAA 1997
s 313-20: the concessional and earnings components, less any shortfall below
the maximum, not below nil), the tax withheld (TAA Regulations 2017 reg 53A:
`estimated_marginal_rate` plus 2% Medicare levy less 30%, not below nil, as
GN 2024/1 section 7 says, or 17% with `commissioner_cannot_estimate` true;
one of the 2 is required; rounded down to the dollar as its examples are;
`withholding_basis` is `estimate` for the formula and `reg_53A_1_b` for the
17%), what is paid to you, the 30% non-refundable offset
(s 313-25), the FHSS tax of 20% if neither a contract nor a recontribution is
notified in time (s 313-60), and the recontribution that avoids it (s 313-50).
Deadlines, KiwiSaver and foreign transfers, excess contributions, amended
determinations, a release under the transitional rules of item 30 of
Schedule 4 to Act No. 75 of 2023 (which has no FHSS tax or recontribution;
`transitional_note` flags every determination made before 15 September 2024)
and foreign residents are out of scope.

## PPSR timing

`ppsr-timing` answers what the Personal Property Securities Register's law
decides without a search, for a registration, possession or attachment, or
security agreement in 2025-26 or 2026-27. Times are `YYYY-MM-DDTHH:MM` in ACT
legal time (PPSA s 294); an end of day is shown as `YYYY-MM-DD end of day`.
An input time or a calculated six-month reference in the skipped or repeated
02:00 to 02:59 hour is refused with `ppsr_timing.time_unresolved`. Check the
underlying timing records if either transition affects the result.
[ACT Standard Time and Summer Time Act 1972](https://www.legislation.act.gov.au/a/1972-34/)

Every answer carries the fee schedule of the Personal Property Securities
(Fees) Determination 2018 in `rates`: registration $6.00, $25.00 or $115.00,
a search $2.00 online or $7.00 by the contact centre, maintenance $4.00 and a
Registrar's report $300.00 for each hour or part of an hour. The fees are not
subject to GST.

- `registration`: `registration_time`, `collateral` (`consumer` or
  `commercial`), `serial_numbered` and an optional `end_date`. It gives the
  latest end time of PPSA s 153 item 5 (`latest_end_time`: the end of the day
  7 years on for consumer property or property described by serial number, 25
  years on otherwise), the end time that applies (`end_time`, with
  `end_time_basis` `requested`, `default_time` or `no_end_time`: under
  s 153(2) and (3) an end time later than the default time, earlier than the
  registration time, or missing where item 5(b) requires one is taken to be
  the default time) and `registration_fee` (Determination s 4 items 1 to 3).
- `pmsi`: `inventory`, `goods`, `trigger_time` (possession for goods,
  attachment otherwise) and, for collateral that is not inventory,
  `excluded_days`. `register_by` is the trigger time itself for inventory
  (s 62(2)(b)) and otherwise the end of the 15th business day after the
  trigger's day (s 62(3)(b)). With `registration_time` and `pmsi_stated`
  (the s 153 item 7 statement) it gives `registered_in_time` and
  `super_priority`.
- `company_grantor`: `agreement_in_force_time`, `place_holidays` and
  `foreign_law_agreement` (true is refused). `register_by` is the end of the
  20th business day after the agreement came into force (Corporations Act
  s 588FL(2)(b)(ii)). With `critical_time` (1 July 2025 to 30 June 2027) and an
  optional `court_ordered_time` (s 588FM) it gives `latest_time`, the latest
  of the s 588FL(2)(b) times, and with `registration_time`, `vests`: true when
  the registration time is after it, false when not, and null with
  `registered_after_critical_time` for a registration after the critical time,
  as s 588FL(2)(a) then turns on when the interest arose, which is not given.

PPSA s 10 business days exclude weekends, a public holiday for the whole of
any State, the ACT or the NT, the days between Christmas Day and New Year's
Day and days the Registrar suspended the register; the PPS Regulations 2010
prescribe no other day. Corporations Act s 9 business days exclude weekends
and public or bank holidays in the place concerned. The caller lists the
holidays (`excluded_days`, `place_holidays`): a count that spans 25 December
or 1 January on a weekday without it listed is refused. Whether an
arrangement is a security interest or a purchase money security interest,
searches, priority between other interests, perfection by possession or
control, amendments, ending a registration, transitional registrations,
s 588FN's exceptions and court extensions under PPSA s 293 are out of scope.
See [calculation evidence](docs/calculation-evidence.md#ppsr-timing).

## ATO penalties and interest

`ato-interest-penalties` answers one `question` at a time, each amount before
any remission (the Commissioner's discretion: `question: remission` is
refused with `ato_interest_penalties.remission_discretionary`, as is
`scheme_penalty` with `ato_interest_penalties.scheme_penalty`). `year` is the
financial year of the question's end date, named below; another year is
refused with `ato_interest_penalties.year_mismatch`. Dates are `YYYY-MM-DD`.

- `gic`: `amount` unpaid from `due_date` (the first day of the charge, ITAA
  1997 s 5-15), `to` (the last day, such as the day before payment in full)
  and optional `payments`, a list of `{"date", "amount"}` from `due_date` to
  `to`. Each day's charge is the day's rate (TAA 1953 s 8AAD: the published
  annual GIC rate over the days in the calendar year) on the amount and GIC
  still unpaid at the end of the day (s 8AAC(3)), a payment that day applied
  first. It is worked exactly and rounded half up to the cent once, as the
  site's GIC payment plan oracle does: `gic`, `paid` and `balance`, with each
  stretch of days at one rate. Days after 31 December 2026 need
  `assumed_annual_rate` (a percentage), and the result counts them in
  `assumed_days`. End date: `to`.
- `sic`: `amount` of additional income tax, the first assessment's
  `due_date` and the amended assessment's `notice_date`. SIC (TAA Sch 1
  ss 280-100 and 280-105, the published SIC rates) compounds daily from the
  due day to the day before the notice, whether or not the shortfall was paid
  earlier. End date: `notice_date`.
- `failure_to_lodge`: `due_date`, `given_on` (or an as-at day for a document
  not yet given) and `size` (`small`, `medium`, `large` or
  `significant_global`). One penalty unit for each 28 days or part, at most 5
  (s 286-80(2)), each at the unit applying at the start of its 28 days
  (s 286-80(5)), times 1, 2, 5 or 500. End date: `given_on`.
- `statement_penalty`: `statement_date`, `significant_global_entity` and
  `parts`, the matters in one statement for one accounting period (a
  statement for another period is its own question), one matter each: `shortfall_amount` (or `null` for a false or
  misleading statement with no shortfall amount), `behaviour`
  (`intentional_disregard`, `recklessness`, `failure_to_take_reasonable_care`
  or `reasonable_care`) and optionally `not_reasonably_arguable`,
  `accepted_way_amount` (s 284-224), `increase` (s 284-220) and `disclosure`
  (`none`, `before_examination` or `during_examination`, s 284-225). A part
  with a shortfall amount disclosed before an examination needs
  `other_disclosed_before_examination`: the period's shortfall amounts
  disclosed so in other statements (`0.00` when none), added to these parts'
  for the $1,000 test of s 284-225(3) (MT 2012/3 para 35); a later disclosure
  for the period means working the earlier statements again. A
  `not_reasonably_arguable` part needs `income_tax_payable` on the return for
  the threshold of s 284-90(3)(a); `trust_or_partnership: true` with one is
  refused. The base penalty amount is 75%, 50% or 25% of the shortfall less
  the accepted part, 25% for a position not reasonably arguable above the
  threshold, the greater where 2 apply, or 60, 40 or 20 penalty units without
  a shortfall amount; doubled for a significant global entity; then
  s 284-85's formula with the increase and reduction. An increase beside a
  disclosure that reduces the amount to nil is refused
  (`ato_interest_penalties.increase_with_nil`). End date: `statement_date`.
- `early_payment_interest`: `amount`, `liability` (`income_tax`,
  `shortfall_interest_charge`, `compulsory_study_loan_repayment` or
  `fs_assessment_debt`), `paid_on`, `due_date`, `full_self_assessment` and,
  when that is false, `notice_issued`; `paid_on` after `due_date` is refused.
  Interest is due only on a payment more
  than 14 days early (Overpayments Act s 8A), from the later of the payment
  and notice days (the payment day for a full self-assessment taxpayer) to
  the due day, both counted (s 8B). End date: `due_date`.
- `overpayment_interest`: `payments` (`{"paid_on", "amount"}`), `refunded`,
  `refunded_on`, `notice_issued`, and `relevant_tax` and
  `decision_to_which_act_applies`, both true (false is refused). `refunded` is
  the one refund or application of these payments; a later refund of the same
  payments is not worked. A partial refund goes to the payments in reverse order (s 9(3)); each part earns
  interest from the later of the notice and its payment to the refund day
  (s 10). End date: `refunded_on`.

Credit interest is simple interest at the base interest rate (the GIC rate
less 7 points, s 8AAD(1)), day by day at each day's rate and year length,
rounded to the nearest cent with half a cent up (s 12D); under 50 cents it is
not payable (s 12E), so `interest_payable` is then nil. Whether a position is
reasonably arguable, the behaviour, whether a disclosure was voluntary or
saved resources, the entity's size and significant global entity status, and
Part III entitlement are the caller's established facts. Division 296 GIC,
trust, partnership, petroleum resource rent tax and minimum tax positions,
the s 284-75(3) penalty, refunds before the due day and other credit interest
are out of scope. See
[calculation evidence](docs/calculation-evidence.md#ato-penalties-and-interest).

## Wine equalisation tax

`wine-equalisation-tax` works out the wine tax on a tax period's assessable
dealings in wine and the producer rebate on them, for dealings in 2023-24 to
2027-28 (A New Tax System (Wine Equalisation Tax) Act 1999). Each dealing in
`dealings` has a `kind`:

- `wholesale_sale`: taxed on its price excluding WET and GST (AD1a, AD1b and
  AD11b), with GST of 10% on the WET inclusive price;
- `retail_sale`: taxed on the notional wholesale selling price, by default
  half the retail price including WET and GST (ss 9-30 and 9-35), or for grape
  wine, with `average_wholesale`, the weighted average price of the tax
  period's wholesale sales of the same vintage and grape varieties (s 9-40),
  which needs at least 10% of those sales by value to be wholesale
  (s 9-25(3)); the GST is 1/11 of the retail price;
- `own_use`: an application to own use connected with retail sales, such as
  tastings, valued the same way at the price the wine would normally be sold
  for by retail;
- `quoted_sale`: a sale under quote, not taxable (s 7-10), with the
  purchaser's stated intention;
- `own_use_purchased_under_quote`: taxed on the purchase price excluding GST
  (AD3c(a) and AD13c(a)).

The tax is 29% of the taxable value (s 5-5(3)), rounded half up to the cent.
A dealing claims the producer rebate with `rebate_conditions`: whether the
seller is the producer, owned at least 85% of the source product and meets
the container and trade mark conditions in force since 1 July 2018 (s 19-5).
A quoted sale earns it only where the purchaser will make a taxable dealing,
as QC22758's table shows. The rebate is 29% of the price or notional wholesale
selling price (s 19-15(1)), capped for the financial year at $350,000 to
2025-26 and $400,000 for dealings from 1 July 2026 (Act No. 12 of 2026
Schedule 6), less `rebate_claimed_earlier_in_year`. `associated_producer`
states that the cap is shared by a group (s 19-20); the command then warns
that the earlier claims must be the group's. `label_1c` is the tax and
`label_1d` the rebate, each rounded down to whole dollars and never netted
(QC22762, QC17458).

It refuses a kind, intention or field it does not take, the average wholesale
price for other than grape wine or without the 10% share, and a dealing in a
year outside its range. Whether a sale is wholesale or retail, a quote's
validity, the price for which wine would normally be sold by retail,
associated producers, the 85% ownership and the container and trade mark
facts, the s 19-25 recovery of an excess rebate, and the wine tax credits of
Division 17 are the producer's to establish. See
[calculation evidence](docs/calculation-evidence.md#wine-equalisation-tax).

## Luxury car tax

`luxury-car-tax` works out the luxury car tax on a tax period's lines for a
tax period in 2019-20 to 2026-27 (A New Tax System (Luxury Car Tax) Act
1999). Each line in `lines` has a `kind`:

- `sale` and `importation`: a taxable supply or importation of a luxury car,
  with its `luxury_car_tax_value` (GST and customs duty included, luxury car
  tax not: ss 5-20 and 7-15(2)) and `fuel_consumption_litres_per_100km`, the
  combined rating, or `null` for none. A sale may take
  `lct_previously_payable`, the tax on an earlier supply or importation of
  the car (s 5-15(2)). Importation tax is shown apart, as it is paid to the
  Department of Home Affairs;
- `price_adjustment`: an adjustment event on a sale made in `supplied_in`,
  with the `corrected_luxury_car_tax_value` and the
  `previously_attributed_lct` (ss 15-20 and 15-25);
- `change_of_use_increasing`: a car bought under quote, or one whose buyer
  had a decreasing adjustment under s 15-30(1), then used other than for a
  quotable purpose (s 15-30(3)), with the supply's `supplied_in` and
  `luxury_car_tax_value`, so that the amount is the tax payable or that would
  have been payable on the supply (s 15-30(4));
- `change_of_use_decreasing`: a registered buyer who did not quote and has
  used the car only for a quotable purpose, with the `lct_payable_on_supply`
  (s 15-30(1));
- `refund`: a primary producer's or tourism operator's refund on a
  refund-eligible car, the lesser of `lct_borne` and $10,000 (ss 18-5 and
  18-10), for cars supplied or imported from 1 July 2019.

The tax is 33% x 10/11 x (value - threshold), nil at or below the threshold,
rounded half up to the cent (s 5-15(1)). The threshold is the financial
year's luxury car tax threshold, or its fuel-efficient car limit for a car
rated at most 3.5 litres per 100 km for supplies and importations from
1 July 2025 and 7 litres before (s 25-1(4); Act No. 29 of 2025 Schedule 1).
A car rated above 3.5 and at most 7 litres from 1 July 2025 needs
`used_before_1_july_2025`: true keeps the 7 litre rating where, before that
day, the car was supplied or imported and used in Australia other than for a
quotable purpose (item 4(2)). The thresholds for 2012-13 to 2026-27 are the
ATO's published table (QC38161). `label_1e` is the tax on sales with the
increasing adjustments and `label_1f` the decreasing adjustments, each
rounded down to whole dollars (QC33694, QC17458). Refunds are claimed on the
ATO's approved form, not the BAS (QC39467), so they are a total of their own.

It refuses a year outside its range, a `supplied_in` year outside the table
or after the tax period's, an adjustment without `supplied_in`, a rating it
cannot read, a second primary producer refund in the lines given (s 18-5(4),
and it warns that a refund in an earlier period of the year counts too) and a
refund for a car supplied before 1 July 2019. Whether a vehicle is a car or a luxury car (s 25-1(2) excludes
emergency vehicles, cars fitted for wheelchairs, commercial vehicles and
motor homes), quoting, the value of a car sold with other goods, whether a
car is refund-eligible, bad debts, the s 15-30(1A) and (3A) adjustments and
the tax period a line belongs to are the operator's to establish. See
[calculation evidence](docs/calculation-evidence.md#luxury-car-tax).

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

## Fuel tax credits

`fuel_tax_credits` works out the fuel tax credits for business fuel acquired
in one income year from 2022-23 to 2026-27 (Fuel Tax Act 2006 ss 41-5 and
43-5). Each line states the fuel (`diesel`, `petrol`, `b5`, `b20`, `e10`,
`e85`, `lpg`, `lng_cng` or `b100`), the day it was `acquired`, the `quantity`
(litres; kilograms for LNG and CNG) and its `use`:

- `heavy_vehicle_public_road`: a vehicle above 4.5 tonnes GVM travelling on a
  public road, at the rate net of the road user charge (s 43-10(3)), including
  fuel to power a bus's or coach's passenger air-conditioning (QC107638
  note 1);
- `other_business`: every other business use, including a heavy vehicle's
  other auxiliary equipment, travel off public roads and public road travel
  incidental to the vehicle's main use (s 43-10(4));
- `light_vehicle_public_road`: 4.5 tonnes GVM or less on a public road, which
  gets nil (s 41-20).

Each line takes the rate the ATO publishes for the day the fuel was acquired
(s 43-5(2A) item 1). The rates are
[`data/fuel-tax-credit-rates.json`](data/fuel-tax-credit-rates.json): 16
periods from 1 July 2022, each with its ATO page, table, the page's last
update and the day it was read (19 November 2026), all 192 rates equal to
data.gov.au's 'Historical FTC rates' workbook. The ATO publishes the last
period as from 3 August 2026 with no end; it is held open to 31 January 2027,
before the February 2027 indexation, and fuel acquired after the day the
rates were read carries a warning to check for a change since. A line's credit
is quantity x rate in cents / 100, rounded half up to the cent (a Method 1
line: cost x rate / average price, so no rounding comes before the cent);
`label_7d` is the total with the cents dropped, as the ATO's examples claim
it. Each line is rounded, so give the fuel of one type and use acquired on
one day as one line, rather than one line per receipt; the ATO's worksheet
works one amount for each quantity and rate (QC44511), and a line keeps its
day for the rate and the s 47-5(2) time limit.

A small claimant (`small_claimant`: credits of $10,000 or less in the previous
12 months and expected in the current 12 months) can also use:

- the basic method for heavy vehicles (PCG 2021/2): a line with `method`
  `basic_heavy_vehicle`, `fuel` `diesel`, `litres`, `public_road_km` (actual)
  or `total_km` (98.4% taken as on public roads), and
  `rigid_truck_gross_weight_tonnes` (the ATO's rigid truck table) or
  `litres_per_km` (the engine's diagnostic report). The public road litres
  take the heavy vehicle rate and the rest of the diesel the other rate;
- PCG 2016/2 Method 1: a line with `method` `litres_from_cost`, its `cost`
  (GST inclusive) and `average_price_cents_per_litre`, the average GST
  inclusive retail price of that fuel in the area of operation over the tax
  period. Read it from the Australian Institute of Petroleum's weekly pump
  prices (https://aip.com.au/pricing/pump-prices-retail/), the source the
  guideline names (para 9). The site reserves all rights, so the command holds
  no prices;
- PCG 2016/2 Method 2: `rate_at_period_end` (`start` and `end` of the tax
  period) takes every line at the rate on the period's last day when
  indexation changes an applicable rate. When all applicable rates stay
  flat, the worksheet uses each acquisition date. It refuses period-end
  requests across changes for reasons other than indexation alone (as on
  29 September 2022, 1 April 2026 and 3 August 2026).

Methods 1 and 2 are for liquid fuels only (PCG 2016/2 para 4(b)). With
`return_date`, the day the return is given, `time_limit_lines` lists the
lines acquired more than 4 years before it, which s 47-5(2) may have ended
unless the credit was already taken into account in an assessment.

The command refuses a year outside 2022-23 to 2026-27, an acquisition outside
the income year or after 31 January 2027 (or a Method 2 tax period ending
after it), a simplified method without the
small claimant statement or outside its guideline, and a basic method line
whose public road litres exceed its diesel. It does not decide entitlement,
hire chains, public roads, the environmental criteria (s 41-25), apportionment
between uses, the auxiliary equipment percentages (PCG 2016/11), articulated
combinations' consumption rates, packaging, supply, domestic and
non-business claims, Division 44 adjustments or which tax period a credit is
attributable to; see
[calculation evidence](docs/calculation-evidence.md#fuel-tax-credits).

## CGT parcels

`cgt_parcels` works out the capital gains and losses from selling shares or
units bought in parcels at different times, and the year's net capital gain or
loss, for an individual, a trust, a complying superannuation entity or a
company, with contracts in 2025-26 or 2026-27. Each parcel is a separate CGT
asset (the ATO's 'Keeping records of shares and units'), so the worksheet
matches each disposal to parcels and works out each part on its own:

- `parcels`: each parcel's `parcel` (a unique name), `security` (the same text
  for identical shares or units), `acquired` (the contract date, ITAA 1997
  s 109-5), `units` and cost base elements (s 110-25): `acquisition_cost` and
  `incidental_costs` (brokerage and stamp duty, s 110-35), and where there are
  any `ownership_costs`, `capital_expenditure` and `title_costs`;
- `disposals`: each sale's `security`, `date` (the contract date, s 104-10(3)),
  `units`, `capital_proceeds` and `incidental_costs`; with `identification`
  `selected`, also `parcels`, the parcels and units the taxpayer chose, which
  must add up to the sale;
- `identification`: `first_in_first_out`, which takes the earliest acquired
  units of the security still held, or `selected`. TD 33 accepts both.
- the year's other capital gains and losses, and earlier years' unapplied net
  capital losses, as established totals.

A parcel's cost base elements 1, 2, 4 and 5 together, its costs of owning
(element 3) and a sale's proceeds and selling costs are each split by units,
to the cent, the last units taking what is left, so the parts always add up
(ss 112-30 and 116-40). Each part is a capital gain where
the proceeds exceed its cost base and a capital loss where its reduced cost
base, which leaves out costs of owning, exceeds them (ss 104-10(4) and
110-55(2)). A gain is a discount capital gain when the parcel was acquired at
least 12 months before the sale, counting neither day (s 115-25(1); the ATO's
'CGT discount'), except for a company (s 115-10; a life insurance company is
out of scope). The year's capital losses and then the earlier net capital losses reduce the other gains first and the
discount capital gains after (s 102-5(1) steps 1 and 2, the order the ATO says
usually gives the smallest net capital gain), and the discount percentage, 50%
or 33 1/3% for a complying superannuation entity (s 115-100), reduces each
discount capital gain left (step 3). For an individual, `total_capital_gains`,
`net_capital_gain` and `net_capital_losses_carried_forward` are question 18
labels H, A and V when every gain and loss of the year is stated.

`matches` lists each part: the disposal and parcel, the dates and income year,
whether it counts in the year, the units, proceeds, cost base, reduced cost
base, gain or loss and why it is or is not a discount capital gain. Sales
before the year use up units and are listed but not counted. `holdings` lists
the units left in each parcel after the year's sales with their cost base and
reduced cost base, to carry into the next year.

It refuses a parcel sold in the year that was acquired at or before
21 September 1999, where indexation (ss 114-1 and 114-10) or the pre-CGT rule
(s 104-10(5)) applies, a sale of more units than the parcels acquired by then
hold, selected units that do not add up, and a sale or parcel after the year.
Cost base adjustments (bonus shares, rights, demergers, returns of capital,
AMIT adjustments) are established before the call. Shares held as trading
stock, employee share scheme interests, interests to which s 115-45 applies,
other CGT events, foreign and temporary residents, collectables, the small
business concessions and CGT payable are out of scope. From 2027-28 Act
No. 49 of 2026 changes how capital gains are worked out, so 2027-28 is
refused. The tests hold it to the ATO's Kathleen and Boris examples and the
Library's David Jones example; see
[calculation evidence](docs/calculation-evidence.md#cgt-parcels).

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
in the year. A loan first drawn on 29 February has its 5 years end on
28 February (Acts Interpretation Act 1901 s 2G(2)). From 2027-28 the year is
worked out and flagged: s 26-155 may then quarantine the deduction, which the
worksheet does not apply. Section 26-102 can limit deductions for vacant
land holding costs incurred from 1 July 2019, including ongoing costs of
borrowing to acquire the land. The `deduction` amount is the section 25-25
calculation before any applicable limit. The tests hold it to the ATO's and the Library's
worked examples; see
[calculation evidence](docs/calculation-evidence.md#borrowing-expenses).
`borrowing_expenses_schedule` takes several loans, refinancing and use by
year.

## Borrowing expenses schedule

`borrowing_expenses_schedule` works out borrowing expenses under ITAA 1997
s 25-25 for several loans at once and every income year of each, for loans
first drawn from 1 July 2017. It takes `loans`, a list with one object for
each loan, each stating `borrowing_expenses`, `loan_start`, `loan_term_end`,
`repaid_on` and `section_26_50_applies` as `borrowing_expenses` takes them,
and `income_use`: one fraction for every year, or an object giving the
extent the money is used to produce assessable income in each income year
from the year the loan began to the year its period ends. A loan that
repays an earlier one on the day it is first drawn names it in
`refinances` (its number in the list, from 1); the earlier loan's
`repaid_on` must be that day. `other_borrowing_expenses` optionally gives,
by income year, the income-producing share of borrowing expenses on loans
not listed.

The $100 test in s 25-25(6) adds the income-producing shares of everything
incurred for borrowing in an income year, across the loans first drawn that
year and the other amounts; at $100 or less each is deducted that year.
Otherwise each loan is spread over its own period of the loan exactly as
`borrowing_expenses` spreads one, and each year's deduction is that year's
maximum times that year's use (s 25-25(2) and (3)). A refinanced loan's
period ends on repayment (s 25-25(5)(b)), so the year of refinancing deducts
what remains, and the new loan's expenditure spreads from its own first
borrowing.

The amounts are the year asked about's, summed over the loans, with a step
that writes in each loan's term. `loans` gives each loan's period, how it is
deducted, `schedule` (every income year's remaining expenditure, days,
maximum, use, deduction and what remains), its total deduction and notes on
refinancing; `schedule` at the top gives each year's totals. A row from
2027-28 has `section_26_155` true and the result warns that s 26-155 may
quarantine it: s 26-155(1) applies after s 25-25 has worked the amount out,
and the worksheet does not apply it or decide its exceptions. The schedule's
`deduction` amounts are section 25-25 calculations before any applicable
section 26-102 or 26-155 limit. A row after
2027-28 has `projected` true: it applies s 25-25 as read on 19 November
2026. It refuses a loan first drawn before 1 July 2017 or after the year
asked about, a leisure facility unless s 26-50(3) applies at all times, a
use missing for a year, and a refinancing whose dates do not meet. The
tests hold it to the ATO's and the Library's worked examples and to
`borrowing_expenses` row by row; see
[calculation evidence](docs/calculation-evidence.md#borrowing-expenses-schedule).

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
not quote an ABN for luxury car tax, needs `gst.separate_tax_invoice`.
Set `tax_invoice_held` to true only if a held tax invoice separately identifies
the taxable goods and any finance supply. Supply `goods_consideration`,
`finance_consideration`, `goods_gst` and `finance_gst` from it. The amounts
must reconcile to the agreement and luxury car tax and agree with separate
cent rounding for each supply under GST Act s 9-90. Enter zero for both finance
amounts where there is no finance charge. Without that evidence, `gst.review`
explains the missing figure. If the invoice rounds GST differently, review the
figures with the preparer.

GST Act s 69-10(1) limits the goods credit to 1/11 of the car limit. The tax
on the separate taxable finance supply is added, and the creditable-purpose
fraction applies to both. See the ATO's GST issues register 'Financial services
- questions and answers', issues 1.1, 1.2 and 1.17. The worksheet leaves the
attribution date and GST journals for these separate supplies to the preparer.

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
deduction) and `notes`, worked at full precision. Each total in `amounts` is the sum of the exact
item values, rounded once to the cent, half up.

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

## Franking account

`franking_account` keeps one corporate tax entity's franking account for an
income year from 1 July to 30 June, 2025-26 or 2026-27, under ITAA 1997
Division 205, and gives the franking return's amounts. Give the balance at
the end of the previous year (a deficit as a negative amount, which that
year's franking deficit tax credits on 1 July), the corporate tax rate for
imputation purposes (25% or 30%), the R&D deferral left from earlier years,
each dated event by type (PAYG instalments, income tax paid and refunded,
franked distributions received and the other credits and debits of the
s 205-15 and s 205-30 tables it takes) and each frankable distribution in
date order with the franking credit allocated.

Each distribution's franking credit is the allocated credit up to the
distribution over the gross-up rate (ss 202-60 and 202-65), and its franking
percentage the allocated credit over that maximum, at most 100%. The first
distribution in each franking period sets the benchmark: the income year for a
private company, otherwise its first 6 months and the rest. A later one
franked above it gives over-franking tax and one below it an item 3 debit, each
the distribution times the differential over the gross-up rate (s 203-50),
unless s 203-20 exempts the entity or the Commissioner permitted the
percentage. The R&D deferral reduces later PAYG instalment and income tax
credits (s 205-15(4)). A refund paid in the 3 months after the year's end that
would put the account into deficit, or further into it, is taken into the year
(s 205-50(2)).

The deficit at the year's end is the franking deficit tax. The year's offset
is that tax less 30% of the part attributable to item 1, 2, 3, 5 and 6 debits
when that part exceeds 10% of the year's credits, by the ATO's method; it is
whole without an item 1, 3, 5 or 6 debit (code F), for a private company that
meets s 205-70(5) (code P) and under a Commissioner's determination. With
`income_tax` the offset and the offset brought forward are applied to the
income tax left after the offsets applied first, and the rest is carried
forward. Each franking period's benchmark is tested for a significant variation
(s 204-70). The result gives the ledger with a running balance, each
distribution's working, each franking period, the refunds after the year's
end, the return's labels A to F and K, whether the return is required and its
due date, the last day of the month after the year's end (s 214-150). Label B
includes label F, the tax from refunds after the year's end; the ATO's section B
fills label F only on a further return for those refunds. An entity applying
for the Commissioner's discretion prints code C and shows the reduced offset at
label C, so give `commissioner_determination` only once the Commissioner has
determined.

Amounts are worked exactly and shown to the cent; the return's whole-dollar
figures are not worked out. Consolidated groups, entities that cease to be
franking entities, New Zealand franking companies, pooled development funds,
friendly societies, life insurance companies, exempting entities, late
balancers, diverted profits tax and earlier non-resident years are outside it.
The tests hold it to the ATO's franking return examples 2, 4, 5 and 7 to 9 and
the s 205-70(2) note; see
[calculation evidence](docs/calculation-evidence.md#franking-account).

`the-exchequer-tally` keeps its own `franking-account` for 7 entry types, with
no franking periods, R&D deferral, refunds after the year's end or due dates.
For those types and an opening balance of nil or more, this worksheet gives
the same closing balance, franking deficit tax and offset.

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

## Payroll tax

`payroll_tax` works out one employer's payroll tax for the whole of 2026-27 in
one state or territory. It takes:

- `jurisdiction`: `nsw`, `vic`, `qld`, `wa`, `sa`, `tas`, `act` or `nt`;
- `taxable_wages`, the wages taxable in that jurisdiction after its exemptions,
  and `australian_wages`, those wages and the employer's interstate wages
  together, which cannot be less;
- `group_member` and `employed_all_year`, flags that must be `false` and
  `true`;
- in Victoria only, `regional_employer`, whether at least 85% of the taxable
  wages are paid to regional employees (Payroll Tax Act 2007 (Vic) s 3A).

Each jurisdiction works out a deduction or threshold on the Australian wages
and shares it by the taxable wages over the Australian wages; the rest is the
taxable amount. The deduction is a fixed $1,200,000 in New South Wales,
$600,000 in South Australia, $1,250,000 in Tasmania and $1,750,000 in the ACT.
It phases out in Victoria ($1,000,000 less half the wages above $3,000,000),
Queensland ($1,300,000 less $1 for every $7 above it), Western Australia
($1,000,000, nil at $7,500,000) and the Northern Territory ($2,500,000 less $1
for every $2 above it). The rate is then:

| Jurisdiction | Rate on the taxable amount |
| --- | --- |
| New South Wales | 5.45% |
| Victoria | 4.85%, or 1.2125% for a regional employer |
| Queensland | 4.75%, or 4.95% when the Australian wages are more than $6,500,000 |
| Western Australia | 5.5% |
| South Australia | nil to $1,500,000 of Australian wages, phased in to 4.95% at $1,700,000, then 4.95% |
| Tasmania | 4% on the share of the wages from $1,250,000 to $2,000,000 and 6.1% on the share above $2,000,000 |
| ACT | 6.75% to $20 million of Australian wages, 6.85% to $50 million, 7.35% to $100 million, 7.85% to $150 million and 8.75% above |
| Northern Territory | 5.5%, or 6.5% when the Australian wages are $100,000,000 or more |

Victoria adds the mental health and wellbeing surcharge and the COVID-19 debt
temporary payroll tax surcharge, each 0.5% of the Victorian share of the
Australian wages above $10 million and another 0.5% above $100 million.
Queensland adds the mental health levy, 0.25% of its share above $10 million
and another 0.5% above $100 million. Each surcharge or levy is its own step,
and `total` adds every amount to the cent. With no wages every step is nil.

It refuses a member of a group, whose threshold, rate and any surcharge or
levy are tested on the group's wages and only one member claims the deduction,
and an employer that paid wages for only part of the year, whose threshold
each jurisdiction reduces by the days. It leaves out which payments are
taxable wages and where, exemptions, rebates and discounts (such as
Queensland's regional discount and apprentice and trainee rebate), the ACT's
cap for an eligible university, monthly and other periodic returns and the
annual reconciliation of amounts paid. The tests hold it to the examples
Revenue NSW, the Victorian and Tasmanian State Revenue Offices, the Queensland
Revenue Office, RevenueWA and RevenueSA publish, and to examples worked from
the ACT and Northern Territory Acts; see
[calculation evidence](docs/calculation-evidence.md#payroll-tax).

## GST margin scheme

`gst-margin-scheme` decides whether the margin scheme of GST Act Division 75
can apply to one taxable supply of real property in 2025-26 to 2027-28, and
works out the GST with and without it. It lists each condition of s 75-5 with
its authority and whether it is met, names each one that fails in
`failing_conditions`, and gives `eligible`:

- `real_property_interest`: `interest` is `freehold_interest`, `stratum_unit`
  or `long_term_lease` (at least 50 years: s 195-1), not `other`
  (s 75-5(1));
- `written_agreement`: `agreement` is `written_on_or_before_supply` or
  `written_in_further_period_allowed` (a further period the Commissioner
  allows), not `none` (s 75-5(1) and (1A)); a contract applying the scheme if
  the supply turns out to be taxable is enough (GSTR 2006/8 para 27);
- `acquisition_eligible`: `acquisition`, how you acquired the entire interest,
  is not `taxable_supply_without_margin_scheme` (s 75-5(2) and (3)(a)). It is
  otherwise `taxable_supply_under_margin_scheme`, `other_supply_from_1_july_2000`
  (a supply that was not taxable, such as from a seller not registered for GST)
  or `before_1_july_2000` (acquired, or contracted to acquire, before then:
  GSTR 2006/8 para 39A; a pre-1 July 2000 contract settled later as a fully
  taxable supply is `taxable_supply_without_margin_scheme`).

The margin is the amount by which `consideration`, the GST-inclusive price,
exceeds the base, or nil (s 75-10(2) and (3)). `method` sets the base:

- `consideration`: `acquisition_consideration`, what you gave for the
  interest, less `unpaid_acquisition_consideration`, any of it still unpaid at
  the supply (s 75-12), disregarding improvements, development, legal fees and
  stamp duty (s 75-14); for an interest acquired before 1 July 2000, this
  states that no approved valuation as at the day s 75-10(3) fixes has been
  made, since one that has sets the margin;
- `approved_valuation`, for an interest acquired before 1 July 2000 only: an
  `approved_valuation` made under the Commissioner's requirements (s 75-35) as
  at `valuation_day`. Supply `registered_on_1_july_2000` (true if registered
  or required to be registered on that day) and
  `government_unimproved_on_1_july_2000`. Items 1 and 3 use 1 July 2000.
  For item 2, supply `improved_on_1_july_2000`, `registration_effective_day`
  and `applied_for_registration`. If you applied, also give
  `registration_application_day`. The earlier of the application and effective
  dates sets the valuation day, which may be on or before 1 July 2000.
  A government supplier within item 4 supplies `supply_day` in the selected
  financial year and values the land as if unimproved on that day (s 75-10(3A)).
  If item 2 overlaps with item 3 or 4, the command refuses the valuation.

For a supply of part of the land, `subdivision_share` and `subdivision_whole`
(area, expected price or another reasonable measure) take that proportion of
the base, `apportioned_base` (s 75-15). `gst_under_margin_scheme` is 1/11 of
the margin (s 75-10(1)), `gst_without_margin_scheme` 1/11 of the consideration
(ss 9-70 and 9-75(1)), each rounded to the cent, 0.5 up, as one supply on one
invoice (s 9-90(1)); `gst` is the first when every condition is met and the
second otherwise. Both are shown either way, so the 2 can be compared before
the agreement is signed. `consequences` says what follows: under the scheme the
recipient cannot claim an input tax credit (s 75-20), and can use the scheme
on its own later sale.

It refuses an acquisition whose eligibility or margin other provisions govern,
naming them: from a fellow GST group member, a GST joint venture operator or an
associate, by inheritance, as a GST-free going concern or farm land, through a
supply GST-free under s 38-445(1A), or in part through an ineligible supply
(ss 75-5(3)(b) to (g), 75-10(3) item 2A, 75-11 and 75-22), and a supply to an
associate (`recipient_associate` true), whose margin s 75-13 works from the
GST inclusive market value. It also refuses a valuation for an interest acquired from 1 July 2000, a valuation day s
75-10(3) does not fix, unpaid consideration above the consideration, and a
share that is nil or above the whole. Deciding whether the supply is taxable,
the valuation, the apportionment's basis, GST at settlement, adjustments under
ss 75-22 and 75-27, contracts before 9 December 2008 and several supplies on
one invoice are out of scope. See
[calculation evidence](docs/calculation-evidence.md#gst-margin-scheme).

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
apply to the whole taxable component. The Act states no rounding: the command
rounds each element's tax to the cent, 0.5 cents up, so the tax and
`net_payment` add back to the payment. The
payment is not assessable income and not exempt income (s 301-175), so the
tax is final and nothing goes in a tax return; a payment under $200 is taxed
like any other. It refuses a nil payment. The caller establishes that the
payment is a DASP and its components. `unclaimed_money_rollover` is true for a
roll-over paid under s 20H(2), (2AA) or (2A) of the Superannuation (Unclaimed
Money and Lost Members) Act 1999; with an element untaxed in the fund it is
refused, as s 5(2) taxes any excess untaxed roll-over amount in that element
at nil. The 2014-15 to 2016-17 budget repair levy rates (s 6) and the fund's
withholding and reporting are outside it. See
[calculation evidence](docs/calculation-evidence.md#dasp-tax).

## Super benefit tax

`super-benefit-tax` takes one superannuation lump sum paid in 2020-21 to
2026-27 from a complying plan, not rolled over, and gives its components by
the proportioning rule (ITAA 1997 s 307-125), what is assessable and the most
income tax, before Medicare levy, each part can bear:

```text
tax_free_component = cents(benefit x interest_tax_free_component / interest_value)
taxable_component  = benefit - tax_free_component;  taxed_element = taxable_component - untaxed_element

member 60 or over          taxed nil (s 301-10); untaxed 15% to the untaxed plan cap, 45% above (s 301-95)
member preservation age-59 taxed 0% to the low rate cap, 15% above (s 301-20);
                           untaxed 15% to the low rate cap left, 30% to the plan cap, 45% above (s 301-105)
member under preservation  taxed 20% (s 301-35); untaxed 30% to the plan cap, 45% above (s 301-115)
death benefits dependant   nil (s 302-60)
non_dependant              taxed 15%, untaxed 30% (s 302-145); tax free component nil (s 302-140)
maximum_tax = the sum of each part x its rate, each rounded to the cent
```

`recipient` is `member`, `death_benefits_dependant` or `non_dependant`. A
member gives `date_of_birth`, `terminal_medical_condition` and
`disability_superannuation_benefit`; a death benefit gives none of them. Age
is taken on `payment_date`, and the preservation age from the date of birth
(55 before 1 July 1960, rising a year for each later financial year of birth
to 60 from 1 July 1964). From 1 July 2024 nobody is at or above preservation
age and under 60, so `low_rate_cap_used` and `other_lump_sums_this_year` are
asked for only to 2023-24. `low_rate_cap_used` is the amounts counted towards
the s 301-20(2) and 301-105(4) offsets in earlier income years (s 307-345(2));
another lump sum in the same year in that band is refused while any of the
low rate cap is left, as ss 301-20(3) and
301-105(5) and (6) share the year's cap across the lump sums, the element taxed
in the fund first. `untaxed_plan_cap_used`, asked for with an element untaxed
in the fund, is the total by which earlier lump sums from the same plan reduced
the untaxed plan cap: each one's `untaxed_plan_cap_counted`, as an excess
reduces the cap to nil rather than counting (s 307-350(2)). Each cap left is
the year's cap less the amount counted, and the output shows what this benefit
counts against each. A terminal medical
condition (s 303-10) or a lump sum under $200 that leaves the interest's value
nil (s 301-225(1)) makes a member's benefit tax free; its element untaxed in
the fund still counts against the untaxed plan cap.

The rates are ceilings: each offset ensures the rate on its part 'does not
exceed' the rate, so the person's marginal rates can give less, and the ATO
adds the 2% Medicare levy to each. `assessable_amount` goes in the tax return.
It refuses a disability superannuation benefit (s 307-145), an element untaxed
in the fund with service before 1 July 1983 (s 307-150), facts that conflict
and cap amounts counted above the year's cap. Income streams and their
commutations, benefits accessed illegally (s 304-10), roll-overs,
departing Australia superannuation payments (`dasp-tax`), death benefit
termination payments, foreign and non-complying funds, a deceased estate's
benefit and withholding are outside it. See
[calculation evidence](docs/calculation-evidence.md#super-benefit-tax).

## Depreciation pools and capital works

`depreciation-low-value-pool` works out one holder's low-value pool for an
income year from 2020-21 to 2026-27 (ITAA 1997 ss 40-440 and 40-445). Each
input is already the taxable use percentage of the amount (s 40-435):

```text
new_assets_decline       = 18.75% x (low_cost_assets + second_element_costs)
pool_decline             = 37.5% x (previous_closing_pool_balance + low_value_assets)
decline_in_value         = new_assets_decline + pool_decline
closing_before_disposals = previous + low cost + low value + second element - decline
closing_pool_balance     = closing_before_disposals - termination_values, not below 0
assessable_excess        = termination_values - closing_before_disposals, if above 0
```

`depreciation-small-business-pool` works out a small business entity's
general small business pool for 2020-21 to 2026-27 (ss 328-190 to 328-215).
Its s 328-210(2) amount, the opening pool balance plus the year's `additions`
and `cost_additions` less `termination_values`, each the taxable purpose
proportion, is deducted in full when above zero and below the year's low pool
value threshold, and the pool closes at zero. The threshold is none for
2020-21 to 2022-23 (IT(TP)A 1997 s 328-181(5), temporary full expensing's pool
write-off), $20,000 for 2023-24 to 2025-26 (IT(TP)A s 328-180(6)(e)) and
$20,000 for 2026-27 (s 328-210(1) as amended by Act No. 71 of 2026).
Otherwise it deducts 30% of the opening pool balance and 15% of the additions
and cost additions, each rounded to the cent, and the rest is the closing pool
balance. An amount or closing balance below zero is assessable and the pool
closes at zero (s 328-215). In 2020-21 an amount of exactly zero with
additions is refused, as the 57.5% backing business investment rate could
apply. Which assets the instant asset write-off or temporary full expensing
took instead of the pool is the caller's to establish.

`capital-works` works out one area's Division 43 deduction for 2020-21 to
2026-27, for capital works begun from 16 September 1987:

```text
rate_amount    = construction_expenditure x days_used x rate / 365   s 43-210 steps 1 and 3, s 43-215 step 1
reduced_amount = rate_amount x taxable_use                            s 43-210 steps 2 and 4, s 43-215 step 3
deduction      = min(reduced_amount, undeducted_construction_expenditure)
```

The rate is 4% when `four_percent_manner` is true (s 43-145, works begun after
26 February 1992) and 2.5% otherwise. `days_used` can be 366 in 2023-24; the
denominator stays 365. It refuses works begun before 16 September 1987, the 4%
manner for works begun before 27 February 1992, undeducted construction
expenditure above the construction expenditure and more days than the year has
from the later of 1 July and `construction_start`. R&D use counts in
`taxable_use` only at 2.5% for works other than a hotel or apartment building
(s 43-210 step 4(b), s 43-215 step 3(b)). The caller establishes that the
works qualify for their start date and type, the days, the use and the
undeducted construction expenditure (s 43-235), which this command does not
carry forward.

The single-asset `depreciation` command is unchanged. See
[calculation evidence](docs/calculation-evidence.md#depreciation-low-value-pool).

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

## Individual tax offsets

4 commands work out an individual's remaining non-refundable tax offsets,
each the offset amount only: applying it against tax, and choosing between
offsets the law allows only one of, are the operator's.

`beneficiary_tax_offset` (ITAA 1936 s 160AAA(3)) takes the
`rebatable_benefit` in assessable income (s 160AAA(1)), as established, for
2020-21 to 2026-27. The offset is 15%
of the amount, rounded down to the dollar, above $6,000, plus 15% of it above
$45,000, rounded up to the dollar (Income Tax Assessment (1936 Act)
Regulations 2025 s 13 and the 2015 Regulation before 2026-27). A person also
entitled to the seniors and pensioners tax offset gets only the greater
(s 160AAA(4)).

`invalid_carer_tax_offset` (ITAA 1997 Subdivision 61-A) works out the offset
for one `dependant`, `spouse` or `other`, for 2020-21 to 2025-26: the indexed
amount the ATO publishes, or for part of the year `days` x its daily rate,
less $1 for every $4 of the dependant's adjusted taxable income above $282,
and nil where the person's adjusted taxable income, with a spouse's who is not
the dependant, is more than the FTB Part B limit. Run it once for each
dependant. It refuses a dependant also maintained by someone else, whose
apportionment the law and the ATO's worksheets order differently, FTB Part B
at a shared-care rate, a different spouse during different parts of the year
(s 61-20(3)), and 2026-27, whose amounts are not yet published.

`zone_tax_offset` works out the zone tax offset (ITAA 1936 s 79A) for one
`area`, `special_area`, `zone_a` or `zone_b`, or the overseas forces tax
offset (s 79B) for `overseas_forces`, for 2020-21 to 2026-27. The area is the
operator's, from the ATO's [Australian zone
list](https://www.ato.gov.au/calculators-and-tools/tax-offsets-australian-zones),
which is not exhaustive, and ITAA 1936 Schedule 2. A person is a resident of a
zone with more than one-half of the year's `days` there, or fewer with
`prior_days` in `prior_year` adding to more than 182 (s 79A(3B)(d), or (e)
for a year 2 to 4 years back with `continuous_residence` and
`includes_first_day`). 183 days of 2023-24 (366 days), and days carried from
the fifth year back, and carrying 183 days of 2023-24, are refused: the law and
the ATO's guidance read them differently. The amount is the fixed amount plus 50%, or 20% for Zone B, of
the dependants' notional offsets (`dependants`), the sole parent notional
offset (`sole_parent_days`) and the `invalid_carer_tax_offset`, less any
`remote_area_allowance`; overseas service of not more than one-half of the
year gives the amount x days / 183, the ATO's method. Cents are dropped, as on
the return.

`net_medical_expenses_tax_offset` (ITAA 1936 ss 159P and 159Q) works out the
offset for 2016-17 to 2018-19, the last years it ran: 20% of the net expenses
for disability aids, attendant care or aged care over the lower limit, or 10%
over the higher limit where adjusted taxable income, with a spouse's on
30 June, exceeds $90,000, or $180,000 plus $1,500 for each dependent child
after the first for a family. For a person who died in the year, the spouse is
the one at death (s 159Q(2)). The former dependent spouse tax offset last ran
in 2013-14 and is not worked out.

`tests/individual_offsets.rs` holds each command to the ATO's examples and
every published amount and limit; see [calculation
evidence](docs/calculation-evidence.md#zone-tax-offset).

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

## Rental property

`rental_property` works out one residential rental property's income and
deductions for one owner and income year from 2020-21 to 2026-27, the net rent
and, given `other_taxable_income` for 2024-25 to 2026-27, the tax at the
resident scale without and with it. The owner is an individual not carrying on
a business of letting; the whole dwelling is let as residential accommodation
at a market rent.

- `income`, `expenses` and `rental_only_expenses` are the whole property's
  amounts by the lines of the ATO's rental property worksheet;
  `ownership_share` (the legal interest, TR 93/32) divides them. `expenses`
  relate to the property for the whole period held and are apportioned by
  `days_available` (let or available for rent on commercial terms) over the
  days held from `acquired` to `disposed` in the year; `rental_only_expenses`
  relate solely to renting and are not. Travel is shown and denied
  (ITAA 1997 s 26-31).
- `loans` are the owner's own: each loan's interest times the principal used
  for the rental over the total (TR 2000/2), then the time-based share. A
  redraw or further drawing in the year is refused.
- `borrowing_loans` go to `borrowing-expenses-schedule` unchanged, and its
  deduction for the year is the borrowing expenses line.
- `capital_works` items take the construction expenditure as given, never
  derived from a price: 4% for a building begun from 18 July 1985 to 15
  September 1987 (or later but taken to have begun earlier under s 43-220),
  2.5% otherwise, nil for a building begun before 22 August 1979 or a
  structural improvement begun before 27 February 1992; the days used over 365,
  no more than the undeducted construction expenditure (Division 43). A
  building begun from 22 August 1979 to 17 July 1985 is refused, as it
  qualifies only by its intended use on completion.
- `depreciating_assets` take each asset's decline in value as given, from its
  start time (`first_used`), apportioned like the other lines unless it was
  first used after the property was first held in a year available every day,
  and set it to nil where s 40-27 reduces it: a second-hand asset not supplied with new
  residential premises, or one used in a residence or privately, acquired from
  7.30 pm on 9 May 2017, or earlier with nothing deductible for 2016-17 and
  first used by 30 June 2017 (Act No. 126 of 2017 Schedule 2 item 13).
- A building begun after 15 September 1987 needs
  `taken_to_begin_before_16_september_1987` (s 43-220).

It refuses a holiday home whose costs s 26-50 denies, a redraw, an asset
acquired on 9 May 2017, works or an asset first used after the property was first held in a year with private use, a building begun from 22 August 1979 to 17 July 1985, and the effect on tax before
2024-25. From 2027-28 s 26-155 can quarantine rental deductions; that year is
not supported. `tests/rental_property.rs` holds it to the ATO's Rental
properties guide 2026 Examples 12, 20, 21, 23, 29, 33 and 34.

## Foreign resident withholding

`foreign_resident_withholding` works out the foreign resident capital gains
withholding amount a purchaser pays the Commissioner under TAA 1953 Schedule 1
s 14-200 on acquiring taxable Australian real property or a company title
interest, for an acquisition date (the contract date, ITAA 1997 s 109-5) from
1 July 2017 to 30 June 2027; `year` is the income year it falls in.

- The rate is 12.5% before 1 January 2025 (Act No. 57 of 2017) and 15% from
  then (Act No. 135 of 2024). Before 1 January 2025 a `market_value` under
  $750,000 excludes the transaction (s 14-215(1)(a)); from then there is no
  threshold.
- The base is `cost_base_first_element` less `option_payments`
  (s 14-200(3)(a)).
- Every vendor counts unless it gave a clearance certificate (s 14-210(1)(e)
  and (2)). Each vendor's `interest` share of the base is withheld at the full
  rate, at its notice of variation's rate up to the notice's maximum price, or
  not at all with a certificate, as the multiple vendor class variations
  (F2016L01123; LI 2026/38 s 6) and the ATO's examples do; each amount is
  rounded to the cent.
- `exclusion` (a s 14-215 paragraph) and `class_variation_to_nil`
  (s 14-235(5), such as LI 2026/38 ss 7 to 10) give nil.

It refuses other indirect Australian real property interests, options or
rights as the asset, look-through earnouts (s 14-205), and interest,
dividend, royalty, managed investment trust and s 12-315 withholding, which
are not built. `tests/foreign_resident_withholding.rs` holds it to the ATO's
foreign resident capital gains withholding examples; see
[calculation evidence](docs/calculation-evidence.md#foreign-resident-withholding).

## Small business CGT concessions

`sb_cgt_concessions` shows which of the 4 small business CGT concessions in
ITAA 1997 Division 152 are open for a capital gain from CGT event A1 (a
disposal) under a contract made in 2025-26 or 2026-27, and each condition met
or not, with its provision. It takes the facts the tests turn on; what they
are, and the amounts, stay with the operator:

- the entity (an individual, a company or a trust), when it acquired the
  asset and the CGT event, both contract dates (ss 109-5 and 104-10(3));
- whether the asset is an interest in a partnership asset
  (s 152-10(1)(c)(iii)) or passively held, used in the business of an
  affiliate or connected entity (s 152-10(1A)) or of a partnership you are a
  partner in (s 152-10(1B)), and then whether you carry on a business of your
  own outside partnership;
- the business entity's business in the year and its aggregated turnovers:
  the previous year's, the year before's, the likely turnover and the
  turnover at the year's end. The business entity is you, the partnership, or
  the entity whose business uses a passively held asset: s 152-10(1A)(a) and
  (1B)(b) make that entity's turnover the test;
- the net value of the CGT assets s 152-15 counts, the days in the test period
  the asset was active (s 152-40) and when the business ceased, if it did;
- the date of birth, retirement and permanent incapacity of the individual,
  or for a company or trust of its significant individual, and for a company
  or trust whether it had a significant individual just before the event and
  for 15 years of its ownership;
- for an individual, the CGT exempt amounts of earlier retirement exemption
  choices and when this choice is made.

The CGT small business entity test is s 328-110 with $2 million in place of
each $10 million (s 152-10(1AA)): the previous year's turnover, the likely
turnover unless the 2 years before were each $2 million or more
(s 328-110(3)), or the turnover at the year's end (s 328-110(4)); a business
being wound up counts only if the entity was a small business entity in the
year it stopped, on the same $2 million test (s 328-110(5)). The maximum net asset value test is
$6,000,000 (s 152-15). The active asset test needs half the test period, both
days counted, or 7.5 years of it for an asset owned more than 15 years
(s 152-35); the period ends at the business's cessation if it ceased in the
12 months before the event or a longer period the Commissioner allows.

The 15-year exemption (Subdivision 152-B) takes the whole gain and has
priority (ss 152-215, 152-330 and 152-430): when it is open, the others are
not. Otherwise the basic conditions open the 50% active asset reduction
(s 152-205) and the roll-over (s 152-410), and the retirement exemption
(s 152-305) for an individual with some of the $500,000 limit left
(s 152-320) or a company or trust with a significant individual just before
the event. `super_contribution_required` says whether an individual under 55
just before the choice must contribute the exempt amount to super
(s 152-305(1)(b)); `replacement_asset_period` gives one year before to 2 years
after the last CGT event in the year for which the roll-over is obtained,
`last_rollover_event` or, if that is `null`, this one, or to 6 months after
`earnout_latest_due` if that is later (s 104-190(1A)).

It refuses a share or trust interest (s 152-10(2)), an asset acquired before
20 September 1985 (s 104-10(5)), an asset acquired on the same date 15 years
before the event, where the time of day decides, and active days of 2,737 to
2,740 for an asset owned more than 15 years, which may or may not be 7.5
years. It refuses 2027-28, as Act No. 49 of 2026 changes Division 152 from
then. The tests hold it to the Library's Neil Swan and Wendy Simpson
examples; see
[calculation evidence](docs/calculation-evidence.md#small-business-cgt-concessions).

## Small business income offset

`small_business_income_offset` works out an individual's small business
income tax offset (ITAA 1997 Subdivision 328-F) for 2020-21 to 2026-27 from
established figures:

- taxable income, in whole dollars, and the basic income tax liability on it
  (s 4-10(3) step 2: tax at the rates, before offsets, without the Medicare
  levy); `resident-tax` gives the liability on the resident scale for 2024-25
  onwards;
- the offsets the operator applies against that liability before this one,
  such as the low income tax offset (s 63-10);
- net small business income as a sole trader (s 328-365) and, when it is
  above nil, the business's state and the 4 aggregated turnovers for the
  small business entity test;
- each share of a small business entity's net small business income, or
  other amount s 328-355(c) covers, with the deductions attributable to it
  that s 328-370 covers.

The small business entity test is s 328-110 with $5 million in place of each
$10 million (s 328-357): the previous year's turnover, the likely turnover
unless the 2 years before were each $5 million or more (s 328-110(3)), or the
turnover at the year's end (s 328-110(4)); a business being wound up counts
only if you were a small business entity in the year you stopped
(s 328-110(5)). Own net small business income counts only for a small
business entity. Each share less its deductions is not below nil, and the
total is no more than taxable income (s 328-360(1)). The offset is the rate
times the basic income tax liability times total net small business income
over taxable income: 13% for 2020-21 and 16% from 2021-22, at most $1,000
(s 328-360(2)). `offset_applied` is what the liability left after
`offsets_applied_first` absorbs; the rest is lost (s 63-10(1) item 20).

It refuses a prescribed person under s 102AC of the ITAA 1936 (s 328-375),
and years before 2020-21 and after 2026-27. The tests hold it to the site's
6 sole trader vectors and the ATO's rate table; see
[calculation evidence](docs/calculation-evidence.md#small-business-income-offset).

## Non-commercial losses

`non_commercial_losses` says whether ITAA 1997 Division 35 defers an
individual's loss from a business activity, carried on alone or in
partnership, for an income year from 2020-21 to 2026-27, and how much. It
takes the operator's figures for the activity, or a group of similar ones
(s 35-10(3)):

- the activity's kind (primary production and professional arts can use the
  s 35-10(4) exception), its assessable income and deductions for the year
  and the amounts brought forward to it under s 35-10(2)(b) and (2C);
- whether it started or stopped in the year, and then a full-year estimate of
  its assessable income (s 35-30(b));
- the 4 earlier years' income and deductions, or `null` for a year it was not
  carried on, for the profits test (s 35-35);
- the real property and other assets values (ss 35-40, 35-45 and 35-50);
- taxable income (or a loss), the business losses it deducts, the assessable
  FHSS released amount, reportable fringe benefits, reportable super
  contributions and total net investment losses for the income requirement
  (s 35-10(2E));
- net exempt income left after ss 36-10 and 36-15 (s 35-15).

The income requirement adds back the business losses and takes out the FHSS
amount (nil if that is negative), then adds the other 3 amounts: under
$250,000 is met. The tests are an assessable income of at least $20,000, a
profit in at least 3 of the past 5 years, this one included, real property of
at least $500,000 and other assets of at least $100,000. Section 35-10(2)
defers the excess of the year's deductions, with the amounts brought forward,
over the activity's assessable income unless the income requirement is met
and a test is passed, or the activity is excepted with other income under
$40,000; the amount deferred is reduced by net exempt income. The
Commissioner's discretion (s 35-55) is reported as not decided.

It refuses an activity kind it does not know and earlier years other than 4.
Bankruptcy (s 35-20) and section 40-880 expenditure (s 35-10(2A), (2B)) are
outside its scope. The tests hold it to the ATO's Slaide, Nguyen, Joe, Fred,
Alex and Michael examples and the s 35-10(2) Jennifer example; see
[calculation evidence](docs/calculation-evidence.md#non-commercial-losses).

## PSI test

`psi_test` says whether the personal services income rules (ITAA 1997
Part 2-42) apply to one individual's personal services income for an income
year from 2020-21 to 2026-27, in the order and with the results of the site's
PSI page:

1. income not mainly a reward for personal efforts or skills is not PSI
   (s 84-5(1)), and PSI received as an employee or office holder, or a
   religious practitioner's payment, is outside Division 85 (s 85-35(1)
   and (2));
2. a personal services business determination in force settles it
   (s 87-15(1)(a) or (b));
3. the results test: at least 75% of the PSI is for a result, with the tools
   supplied and liability for defects (s 87-18(1), or (3) for a company,
   partnership or trust);
4. where the 80% rule is reached, an agent on 75% or more commission is
   refused, as s 87-40 may treat the principal's customers as the sources;
   under 75%, the principal stays the payer; an agent's payments are checked
   whenever an agent is entered;
5. 80% or more from one payer and its associates (the rows with one
   `source` label, or one row) without the results test:
   not a personal services business unless a determination is in force
   (s 87-15(3));
6. otherwise the unrelated clients test (2 or more unrelated payers won by
   offers to the public, s 87-20), the employment test (others' principal
   work at least 20% of the PSI by market value, or an apprentice for half the
   year, s 87-25) and the business premises test (all 4 conditions at all
   times, s 87-30): any one met makes a personal services business.

Each share is in per cent to 2 places, rounded down, and each threshold is
compared exactly. Whether the Commissioner would make a determination
(ss 87-60, 87-65) is not decided. The tests hold it to the site's 19 PSI
cases, which include TR 2022/3 examples 20, 29, 30 and 40; see
[calculation evidence](docs/calculation-evidence.md#psi-test).

## Government super amounts

`government_super_amounts` works out, for one full-year resident in 2020-21
to 2026-27, the super co-contribution and the low income super tax offset
(LISTO) under the Superannuation (Government Co-contribution for Low Income
Earners) Act 2003, and with the spouse inputs the spouse contribution tax
offset under ITAA 1997 Subdivision 290-D. Each comes with its conditions, met
or not, and their provisions. It takes `income-tests`' components and works
total income (s 8) and adjusted taxable income (Family Assistance Act Sch 3,
disregarding cls 3 and 3A) as that command does.

- Co-contribution: 50% of the eligible personal contributions (s 9(1)(d)), at
  most $500, less 3.333 cents a dollar of total income above the lower income
  threshold (s 10(1C)), at least $20 (s 11), where every s 6(1) test is met:
  a personal contribution, 10% or more of total income before business
  deductions from employment or a business (`eligible_income`, s 6(1)(b) and
  s 8(3)), total income under the higher threshold, a return lodged, the
  non-concessional cap not exceeded, a total super balance under the general
  transfer balance cap, under 71 at the year's end and no temporary visa. The
  thresholds are the ATO's: $39,837 and $54,837 in 2020-21 up to $49,293 and
  $64,293 in 2026-27.
- LISTO: 15% of the concessional contributions to the cent, at most $500 and
  at least $10 (s 12E), where adjusted taxable income is $37,000 or less and
  the 10% test is met (s 12C(1)).
- Spouse offset: 18% of the lesser of the contributions and $3,000 less the
  spouse's income above $37,000, where that income (assessable income without
  the FHSS amount, reportable fringe benefits and reportable employer
  contributions less excess concessional contributions) is under $40,000 and
  the spouse is within the non-concessional cap and under the general transfer
  balance cap (ss 290-230 and 290-235).

`co_contribution_paid` and `listo_paid` are increased to a multiple of
5 cents, as the Regulations require of an amount paid (2022 reg 15(2), 2004
reg 24(2)). The balance and cap facts take `null` when not established; the
command refuses one only where it would decide an amount. It refuses a
temporary visa held in the year, as the New Zealand citizen and prescribed
visa exceptions of s 6(1)(f) are not worked out, nil total income where the
10% test would decide an amount, and 2027-28, when Act No. 8 of 2026
Schedule 4 changes LISTO. Concessional contributions that s 12E(3) disregards
(constitutionally protected funds and other ITAA 1997 s 291-165(1)(c)
amounts) are out of scope. The tests hold it to the ATO's Angelo, Julie and
3 spouse examples and each year's thresholds; see
[calculation evidence](docs/calculation-evidence.md#government-super-amounts).

## Work expenses

`work_expenses` works out an employee's deduction for one kind of work
expense in an income year from 2020-21 to 2026-27, chosen by `method`:

- `home_office`: from the `hours` worked from home and the work-related amount
  of each expense (`energy`, `phone`, `internet`, `stationery`, `cleaning`,
  `furniture_decline`, `asset_decline` and `repairs`), the fixed rate method
  (PCG 2023/1: 67c an hour for 2022-23 and 2023-24 and 70c from 1 July 2024,
  plus decline in value, repairs and cleaning, which the rate does not cover)
  and the actual cost; for 2020-21 and 2021-22 the 52c fixed rate (only with
  `dedicated_home_office` true, and with phone, internet, stationery, other
  assets and repairs added) and the 80c shortcut (PCG 2020/3) instead. The
  hourly rates need `running_expenses_incurred` true (PCG 2023/1 paras 18(b)
  and 49 to 55); without it only the actual cost remains. Each method's total
  comes with its return amount, cents disregarded, and the higher is named;
- `car_cents_per_kilometre`: the first 5,000 `business_kilometres` (ITAA 1997
  s 28-25(2)) at the year's rate: 72c for 2020-21 and 2021-22, 78c for
  2022-23, 85c for 2023-24, 88c for 2024-25 and 2025-26 and 91c for 2026-27;
- `car_log_book`: `car_expenses` x `business_kilometres` / `total_kilometres`
  (s 28-90), with a log book of `log_book_weeks`, at least 12 (s 28-120(1)),
  kept for `log_book_year`, the year claimed or one of the 4 before it
  (s 28-115(2)), and still representative (`log_book_representative`);
- `self_education`: `category_a` to `category_d`, nil unless the taxpayer was
  employed when the expenses were incurred and they maintain or improve the
  skills of the current employment or are likely to increase its income
  (s 8-1, TR 2024/3); before 2022-23, for a `prescribed_course`, ITAA 1936
  s 82A takes $250 off category A, less categories C, D and E (`category_e`,
  expenses incurred but not deductible). A category D above 0 needs
  `log_book_car_in_category_a` false, as a car's expenses take one method.

Both car methods need `owned_or_leased` true (s 28-12(1)). It refuses
reimbursed expenses, an allowance for them, an expired, short or
unrepresentative log book, and
2027-28, for which no cents per kilometre rate has been determined. Occupancy
expenses have no input. `derive` shows each step with its provision. The
tests hold it to the ATO's Keisha, Yang, Wanda, Johan, Lana and Maureen
examples, PCG 2023/1's Gerry and Piruntha and TR 98/9's Con; see
[calculation evidence](docs/calculation-evidence.md#work-expenses).

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
