# Changes

## Unreleased

### Added

- `wine-equalisation-tax` works out the wine tax on a tax period's wholesale
  sales, retail sales, applications to own use and sales under quote in
  2023-24 to 2027-28 (WET Act ss 5-5, 7-10 and Division 9), at 29% of the
  price, half the retail price or the average wholesale price, with GST, and
  the producer rebate under s 19-5 at 29%, capped at $350,000 a year to
  2025-26 and $400,000 from 2026-27, less earlier claims in the year. It
  gives labels 1C and 1D rounded down, never netted. New codes:
  `wine_equalisation_tax.dealing_invalid`,
  `wine_equalisation_tax.dealings_invalid`,
  `wine_equalisation_tax.intention_invalid`,
  `wine_equalisation_tax.kind_invalid`,
  `wine_equalisation_tax.method_unavailable` and
  `wine_equalisation_tax.quantity_invalid`. `tests/wine_equalisation_tax.rs`
  holds it to the ATO's Alison, Happy Valley and Good Wines examples, QC22758's
  rebate table, WETR 2009/1 examples 15 and 16 and the Library's ¶22-280.
- `fbt-meal-entertainment-register` checks a 12 week meal entertainment
  register for the FBT year ended 31 March 2026 or 2027: the continuous
  12 week period (FBTAA s 37CC), each entry's 6 details (s 37CE(1)), the
  years it is valid for (s 37CD(1) and (2)) and the 20% rule (s 37CD(3)),
  each reported with its authority and the entries it fails on. A valid
  register gives `fbt-meal-entertainment` its values and first valid year.
  New codes: `fbt_meal_entertainment_register.entries_invalid`,
  `fbt_meal_entertainment_register.entry_invalid` and
  `fbt_meal_entertainment_register.period_invalid`.
  `tests/fbt_meal_entertainment_register.rs` holds it to the sections and the
  Library's ¶3-220 register example.
- `etp-tax-free-component` works out the tax free component of a life benefit
  employment termination payment in 2026-27 or 2027-28: the invalidity
  segment from the days to retirement (ITAA 1997 s 82-150), the pre-July 83
  segment (s 82-155) and the taxable component left (ss 82-140 and 82-145),
  with the day counts and whether the payment is an excluded payment
  (s 82-10(6)(c)), whose taxable component `termination-payment` then
  withholds from. Where s 82-150(1) is not met the invalidity segment is nil
  and its step names the condition. New code:
  `etp_tax_free_component.dates_invalid`. `derive` covers it.
  `tests/etp_tax_free_component.rs` holds it to the ATO's Jackson example and
  the Library's Jim and Jon examples.

- `chattel-mortgage-schedule`: a loan to buy goods under a chattel mortgage,
  at the rate its instalments imply or a stated `annual_rate` (monthly or
  weekly in arrears) from which the instalment is worked out, with the
  schedule, a printable `statement`, interest by income year and the
  purchaser's input tax credit on the price, capped for a car at 1/11 of the
  car limit (GST Act s 69-10(1)). New codes:
  `chattel_mortgage.balloon_too_large`,
  `chattel_mortgage.final_instalment_unsupported`,
  `chattel_mortgage.stated_rate_unsupported` and
  `chattel_mortgage.terms_conflict`; the shared instalment, rate, start and
  GST facts codes keep their `hire_purchase.` names.
- `loan-transactions`: a loan's statement from dated drawings and repayments,
  with daily interest at a rate that changes on dated events, debited at each
  month's end, interest by income year and a printable `statement`. New codes:
  `loan_transactions.balance_below_nil`,
  `loan_transactions.date_outside_statement`,
  `loan_transactions.end_date_invalid`, `loan_transactions.entry_invalid`,
  `loan_transactions.not_in_date_order` and
  `loan_transactions.start_outside_year`.
- `hire-purchase-schedule` takes weekly, fortnightly, quarterly and annual
  instalments (`frequency`), or dated `segments` of level instalments with
  payment holidays, in place of level monthly ones only. Each instalment's
  period is an arrangement payment period (ITAA 1997 s 240-70), an annual one
  2 periods of 6 months, and the implicit rate is a monthly (or weekly)
  compound rate. A level monthly agreement gives the same result as before,
  with new keys beside it.
- `deposit` and `trade_in`, paid or credited at or before the start
  (s 240-25(3)), give the `consideration`.
- `journals`: balanced lines for the start, each period's interest, each
  payment on its day and a payout, totalled by income year: the interest by
  its period, with the periods spanning 30 June set apart, and each payment
  by its day.
- `payout`: an early payout at the end of a period, with the notional interest
  and payments to its date, the balance outstanding, the excess or shortfall
  and its Subdivision 240-G consequence, flagged for review.
- `gst`: a registered hirer's input tax credit for an agreement entered into
  from 1 July 2012, on every component of the supply (the ATO's 'GST – Hire
  purchase and leasing'), less luxury car tax, with the GST rounded to the
  cent (s 9-90), times the creditable purpose, and attributable up front on a
  cash basis too (s 158-5). A missing fact gives no figure, and so does a
  passenger car worth more than the car limit and not quoted for, flagged in
  `gst.review`: no source read settles whether the s 69-10 cap takes in the
  GST on the credit component. The 2024-25 car limit, $69,674, joins the
  2025-26 and 2026-27 ones.
- `payout` takes `known_at_start`, which must be `false`.
- `contract-register` reads a contracts file, CSV with one hire purchase or
  chattel mortgage contract a line, sets out each contract through
  `hire-purchase-schedule` or `chattel-mortgage-schedule` unchanged, and reports cashflow by month, payments, interest and principal
  by income year, the amount outstanding at a balance date split into
  current and non-current (AASB 101 paragraph 69(c)), contracts expiring by
  a date, and totals by financier and by client, as JSON, text or CSV.
  A malformed line is refused with its number. New codes:
  `contract_register.argument_invalid`, `contract_register.column_invalid`,
  `contract_register.contract_duplicate`, `contract_register.csv_invalid`,
  `contract_register.kind_unsupported` and `contract_register.text_invalid`.
  `examples/contract-register.csv` is a fabricated register of 8 contracts,
  and `tests/contract_register.rs` holds it to each contract's own schedule.
  `docs/contract-register.md` documents the columns and reports.
- New codes: `hire_purchase.car_limit_unknown`,
  `hire_purchase.frequencies_mixed`, `hire_purchase.frequency_invalid`,
  `hire_purchase.gst_fact_conflict`, `hire_purchase.gst_fact_invalid`,
  `hire_purchase.payout_date_unsupported`,
  `hire_purchase.payout_known_at_start`,
  `hire_purchase.payout_gst_unsupported`, `hire_purchase.payout_invalid`,
  `hire_purchase.segments_invalid` and
  `hire_purchase.segments_not_contiguous`. `tests/hire_purchase_worksheet.rs`
  works the cases by hand and from the ATO's Albert and Darren examples.
- `fhss` works out the First Home Super Saver scheme for a determination
  made from 1 July 2018 to 31 December 2026: from contributions, the eligible
  contributions counted against the $15,000 yearly and $50,000 (or $30,000)
  limits in the order of TAA 1953 Sch 1 s 138-30, the releasable contributions
  amount and the associated earnings at the SIC rate (s 138-40), labelled an
  estimate; or from a determination's stated components. Then the assessable
  FHSS released amount (ITAA 1997 s 313-20), the withholding (TAA Regulations
  2017 reg 53A: the Commissioner's estimate, worked from the estimated
  marginal rate, or 17% where he is stated to be unable to estimate), the offset, the FHSS tax and the recontribution. New codes:
  `fhss.basis_invalid`, `fhss.contribution_kind_invalid`,
  `fhss.contribution_out_of_period`, `fhss.contributions_invalid`,
  `fhss.deductions_invalid`, `fhss.determination_invalid`,
  `fhss.determination_requested_invalid`, `fhss.rate_unpublished`,
  `fhss.request_exceeds_maximum` and `fhss.year_mismatch`. `tests/fhss.rs` runs
  GN 2024/1 examples 6, 7 and 11 to 13, the ATO's Mary and George examples
  and the s 138-30 example.
- `fbt-minor-benefit-check` screens one benefit in the FBT year ended
  31 March 2026 or 2027 against the minor benefits exemption (FBTAA s 58P).
  It decides the exclusions (an in-house expense payment, property or
  residual benefit, s 58P(1)(c); tax-exempt body entertainment outside
  s 58P(1)(d); the 50/50 split and salary sacrifice, TR 2007/12 paras 14 and
  16) and the less than $300 test (s 58P(1)(e)), and weighs the s 58P(1)(f)
  criteria, given as facts, as TR 2007/12's examples weigh them: `exempt`,
  `not_exempt` or `judgement_required`. New codes:
  `fbt_minor_benefit.category_unsettled` and `fbt_minor_benefit.fact_invalid`.
  `tests/fbt_minor_benefit.rs` runs TR 2007/12 examples 1 to 3 and 5 to 13 and
  the Library's flowers, gym and golf day examples.
- `gst-margin-scheme` decides whether the GST margin scheme can apply to a
  taxable supply of real property in 2025-26 to 2027-28, condition by
  condition (GST Act s 75-5: a freehold interest, stratum unit or long-term
  lease, a written agreement on or before the supply or in a further period
  the Commissioner allows, and no acquisition through a taxable supply worked
  without it), and gives the GST with and without it: 1/11 of the margin over
  the acquisition's consideration less any unpaid (ss 75-10(2), 75-12 and
  75-14) or over an approved valuation as at the day s 75-10(3) items 1 to 4
  fix, for part of the land on its share (s 75-15), rounded to the cent
  (s 9-90(1)). Acquisitions that s 75-5(3)(b) to (g), s 75-10(3) item 2A,
  s 75-11 or s 75-22 govern, and a supply to an associate (s 75-13), are
  refused, naming the provision. New codes:
  `gst_margin_scheme.acquisition_unsupported`,
  `gst_margin_scheme.fact_conflict`, `gst_margin_scheme.fact_invalid`,
  `gst_margin_scheme.supply_to_associate`,
  `gst_margin_scheme.valuation_day_invalid`,
  `gst_margin_scheme.valuation_item_unsettled` and
  `gst_margin_scheme.valuation_unavailable`. `derive` explains it.
  `tests/gst_margin_scheme.rs` runs the Library's Joanne Black, Hans and Luke,
  improvements, subdivision and 1 July 2000 examples and GSTR 2006/8's
  Tureia example.

### Changed

- `hire-purchase-schedule`'s `instalment` and `instalments` are required only
  without `segments`, and `hire_purchase.instalments_unsupported` and
  `hire_purchase.rate_unsupported` name the frequency's limits.

## 1.0.0

Standalone Rust binary, replacing Python package 0.2.0 and its `austaxcalc`
import API. The fourteen worksheet functions, `worksheet_catalogue()` and
`periods()` are JSON stdin subcommands, documented in README.md. Worksheet
methods, rounding, periods, scope statements, sources, warnings and refusal
messages change only as 'Added' and 'Documented divergences' below record.

### Added

- `--version` prints `australian-tax-calculators 1.0.0`. The release workflow
  checks it in each archive, and callers can require an exact version.
- `fbt-return` composes one employer's FBT return for the FBT year ended
  31 March 2026 or 2027 from a list of benefits, each valued by its own FBT
  benefit worksheet and classified by the operator as type 1 or type 2 and as
  excluded or not. It runs `fbt` once on the aggregate amounts and gives items
  14A, 14B, 15, 16, 19 and 23, each employee's individual and reportable fringe
  benefits amounts (FBTAA ss 5E and 135P, compilation 97), the employee
  contributions applied, a reconciliation to the benefits, and the labels it
  leaves to the preparer. New codes: `fbt_return.benefit_command_unsupported`,
  `fbt_return.car_statutory_lower`, `fbt_return.exclusion_conflict`,
  `fbt_return.gst_type_missing`, `fbt_return.mixed_years`,
  `fbt_return.multiple_employers` and `fbt_return.payroll_tax_unsupported`.
  `tests/fbt_return.rs` holds it to the ATO's worked examples.
- `business-capital-expenditure` works out the deduction for business
  capital expenditure (black hole and start-up costs) under ITAA 1997
  s 40-880 for 2020-21 to 2027-28, item by item: one fifth a year over 5
  income years from the year incurred, or in full that year for a start-up
  cost meeting s 40-880(2A) and (2B), to the extent s 40-880(3) or (4)
  allows, with each item's schedule and the paragraphs of s 40-880(4)(b),
  (5), (7), (8) and (9) that exclude it. It refuses an item that forms part
  of a depreciating asset's cost (s 40-880(5)(a)) or could be taken into
  account in a capital gain or loss (s 40-880(5)(f)), a revenue outlay, an
  item incurred before 1 July 2020 and a missing fact, by name. `derive`
  covers it. `tests/business_capital_expenditure.rs` holds it to TR 2011/6
  examples 14, 16 to 18, 21 to 25, 40A and 41 and the ATO's start-up
  examples.
- `balancing-adjustment` works out a depreciating asset's balancing
  adjustment under ITAA 1997 s 40-285 for events in 2024-25 to 2026-27,
  reduced for use other than for a taxable purpose (s 40-290), with a
  car's car limit (s 40-325), a car claimed by both car expense methods
  (s 40-370) and the capital gain or loss of CGT event K7 (s 104-240),
  disregarded for a car or motor cycle (s 118-5). It refuses a car year
  before 2015-16, years that skip one or stop before the event's, a car
  whose cost is not the car limit plus the second element, and 2027-28, and
  leaves pools, s 40-291A and roll-overs
  out of scope. New codes: `balancing.asset_invalid`,
  `balancing.car_limit_cost_mismatch`, `balancing.car_limit_invalid`,
  `balancing.car_limit_not_reduced`, `balancing.decline_above_cost`,
  `balancing.deduction_above_decline`, `balancing.fact_invalid`,
  `balancing.fact_missing`, `balancing.no_decline`,
  `balancing.year_after_event`, `balancing.year_before_scope`,
  `balancing.year_repeated`, `balancing.years_incomplete` and
  `balancing.years_invalid`. `derive`
  covers it. `tests/balancing_adjustment.rs` holds it to the Guide to
  depreciating assets 2026's Bridget, Andrew and Louise examples and the
  example in s 118-10(1).
- `trade-in` works out trading in a depreciating asset for a replacement
  in 2025-26 or 2026-27: the GST on the trade-in and the GST Act s 132-5
  decreasing adjustment, the old asset's termination value (ITAA 1997
  s 40-305 and Subdivision 27-B) and its balancing adjustment as
  `balancing-adjustment` works it, and the replacement's input tax
  credit (GST Act ss 11-30 and 69-10) and cost (ss 40-185, 27-80 and
  40-230). It refuses a trade-in not at arm's length and an involuntary
  disposal, saying what s 40-365 would change. New codes:
  `trade_in.asset_invalid`, `trade_in.fact_conflict`,
  `trade_in.involuntary_disposal` and `trade_in.not_arms_length`.
  `derive` covers it. `tests/trade_in.rs` holds it to the ATO's examples
  for Darren's car and Ian's vehicle.
- `contribution-caps` can work out the unused concessional cap from each
  earlier year's contributions instead of taking it as a total: given
  `earlier_concessional_contributions` from at least the 5th year before,
  with `defined_benefit_interest` and an optional
  `proposed_concessional_contribution`, it gives each year's cap and unused
  cap from 2018-19, the unused cap earlier years applied, earliest first, the
  total carried into the year, and, after the proposal, the amount applied from
  each year, what is left of each, what lapses at the end of the year and
  `unused_cap_expiry`, each with its step in `derive` (ITAA 1997 s 291-20(1)
  to (7), compilation 268). It refuses a defined benefit interest, years
  before 2018-19, a total across years, a gap, and a year over its cap whose
  increase cannot be worked out from what was given; it refuses a field it
  does not take, as `division-296` does. New refusal codes:
  `contribution_caps.defined_benefit_interest`,
  `contribution_caps.earlier_balance_missing`,
  `contribution_caps.earlier_years_invalid`,
  `contribution_caps.earlier_years_short` and
  `contribution_caps.year_before_carry_forward`. `tests/contribution_caps.rs`
  holds it to the ATO's Sam example and its table 2. A supplied
  `unused_concessional_cap` gives the same result as before.
- `unused-leave-payment` works out, for unused annual leave, leave loading and
  unused long service leave paid on a termination in 2026-27, each part's
  assessable share and tax offset (ITAA 1997 ss 83-10 to 83-115), the
  Schedule 7 withholding (32% on the parts the offset caps at 30%, the marginal
  rate calculation once on the rest, with the $300 limit, or without a TFN 47%
  of the payment, 45% for a foreign resident) and the income
  statement amount at lump sum A (type R or T), lump sum B and paid leave
  type U, with `qualifying_earnings` nil under the Superannuation Guarantee
  (Administration) Act and the ATO's qualifying earnings table. `derive`
  covers it. Leave paid after death, working holiday makers, a missing or
  impossible accrual part, a payment under $300 beside pre-16 August 1978
  leave and other years are refused, under the new codes
  `input.termination_reason_invalid`, `leave.accrual_split_invalid`,
  `leave.accrual_split_missing`, `leave.death`,
  `leave.under_300_with_pre_1978` and `leave.working_holiday_maker`. No Python
  release has it. `tests/parity.rs` drops its catalogue entry before comparing,
  and `tests/unused_leave_payment.rs` holds it to Schedule 7's example and the
  parts of the ATO's 6 unused leave examples that do not depend on the 2014-15
  tax tables.

- `derive` also covers `gst`, `depreciation` and `hire-purchase-schedule`.
  Their steps cite the GST Act ss 9-70 and 9-75 (compilation 96) and ITAA 1997
  ss 40-25, 40-70, 40-72, 40-75, 40-85, 240-25 and 240-60 (compilation 268).
  `payg-gross-up` stays outside `derive`: its gross pay is found by search, so
  no step can work it out before the withholding worked on it.
  `tests/depreciation.rs` and `tests/hire_purchase.rs` derive each case they
  run and check the result is the plain output. No command's plain output
  changes.
- `derive` also covers `resident-tax`, `foreign-resident-tax`,
  `low-income-tax-offset`, `seniors-pensioners-tax-offset`, `medicare-levy`
  and `medicare-levy-surcharge`. Their steps cite the Income Tax Rates Act
  1986 Schedule 7, ITAA 1997 s 4-10 and Subdivision 61-D, ITAA 1936 ss
  160AAAA and 251U, the Income Tax Assessment (1936 Act) Regulation 2015 and
  Regulations 2025 ss 9 to 12, the Medicare Levy Act 1986 ss 3 to 9 and the
  A New Tax System (Medicare Levy Surcharge—Fringe Benefits) Act 1999; a
  formula may now use `dollars_up`. The kinds' own test files derive each case
  they run and check the result is the plain output. No command's plain output
  changes.
- `derive` also covers `division-296`, `contribution-caps`,
  `study-loan-repayment`, `capital-gains` and `main-residence`. Their steps cite
  ITAA 1997 Divisions 102, 115, 118, 291, 292 and 296 as compiled on
  1 October 2026, the Income Tax (Transitional Provisions) Act 1997 s 296-1,
  the Superannuation (Building a Stronger and Fairer Super System) Imposition
  Act 2026 s 5 and HESA ss 154-1 to 154-25. `tests/division_296.rs`,
  `tests/main_residence.rs`, the main residence property in
  `tests/properties.rs` and `tests/tables.rs` derive each case of these
  commands they run and check the result is the plain output; a negative
  Division 296 very large component is null in its step, as in the result. No
  command's plain output changes.
- `fbt-loan`, `fbt-meal-entertainment` and `fbt-car-parking` give the taxable
  value of loan (FBTAA ss 18 and 19), meal entertainment (Division 9A: the
  50/50 split and 12 week register methods) and car parking fringe benefits
  (Division 10A: the commercial parking station, average cost and 12 week
  register methods, behind the car parking threshold) for the years ended 31
  March 2026 and 2027, for `fbt` to gross up. They take their inputs by name,
  refuse an input their method does not use, and derive every amount. New
  refusal codes: `fbt.register_not_valid`,
  `fbt_loan.deduction_exceeds_interest`,
  `fbt_meal_entertainment.register_values_invalid`,
  `input.benefit_count_invalid` and `input.fbt_method_invalid`. The ATO's and
  the Library's worked examples are in `docs/calculation-evidence.md`.
- `fbt-debt-waiver`, `fbt-expense-payment` and `fbt-property` give the
  taxable value of debt waiver (FBTAA s 15), expense payment (Division 5:
  external s 23 and in-house property s 22A(1), with the otherwise
  deductible rule, s 24, for external payments) and property fringe
  benefits (Division 11: the 6 in-house rules of s 42(1), the 3 external
  rules of s 43 and the otherwise deductible rule, s 44) for the years
  ended 31 March 2026 and 2027, for `fbt` to gross up. In-house benefits
  take the employee's $1,000 reduction (s 62) left for the year. New
  refusal codes: `fbt_expense_payment.deduction_exceeds_portion`,
  `fbt_expense_payment.in_house_deductible_not_covered` (the otherwise
  deductible rule for in-house expense payments, s 24(1)(b)(i), is not
  covered) and `fbt_expense_payment.payment_exceeds_expenditure`.
- `fbt-entertainment-classifier` classifies an ordinary taxable employer's entertainment expense lines: for each
  line and attendee class, whether it is entertainment, the fringe benefit and its type or the s 41 exemption, the
  ITAA 1997 Division 32 deduction and the GST credit that follows it, and whether each fringe benefit is excluded from
  reporting (FBTAA s 5E(3)(a)), each cited, with TR 97/17 and TR 2007/12 as the
  Commissioner's view. A missing fact leaves a line undetermined; minor benefits are shown only as possible. Under the
  50/50 split or 12 week register method it totals the meal entertainment expenditure for `fbt-meal-entertainment`.
  It refuses tax-exempt bodies, unsplit lines, a cost without its GST status and a missing valuation method, with
  codes `fbt_entertainment_classifier.fact_invalid`, `.gst_status_invalid`, `.line_not_split`, `.lines_invalid`,
  `.method_required`, `.register_percentage_invalid` and `.tax_exempt_body`.
- `fbt-residual`, `fbt-housing`, `fbt-lafha`, `fbt-board` and `fbt-tax-exempt-body-entertainment` give the taxable
  value of residual (Division 12, with the s 62 in-house reduction), housing
  (Division 6: market value, or the statutory annual value from a base year
  indexed by the ATO's published factors), living-away-from-home allowance
  (Division 7), board (Division 9) and tax-exempt body entertainment fringe
  benefits (Division 10) for the years ended 31 March 2026 and 2027, for
  `fbt` to gross up, and derive every amount. New refusal codes, the
  first 2 shared with `fbt-property` and `fbt-expense-payment`:
  `fbt.deduction_exceeds_contribution`, `fbt.in_house_reduction_invalid`,
  `fbt_housing.base_year_invalid`, `fbt_lafha.components_exceed_allowance`,
  `fbt_tax_exempt_body_entertainment.heads_invalid` and `input.state_invalid`.
- `fbt-car-parking` takes 2 more methods: `market_value` (FBTAA s 39D), each
  benefit at the valuer's arm's length amount, and `spaces` (ss 39FA to
  39FE), the statutory formula over groups of spaces in `space_groups`, with
  the s 39FB reduction where the election's employees average fewer than its
  eligible spaces. `fbt-expense-payment` takes in-house residual expense
  payments (s 22A(2) and (3)) by 3 new methods,
  `in_house_residual_salary_packaging`, `in_house_residual_public_price` and
  `in_house_residual_other`. `fbt-meal-entertainment`'s `12_week_register`
  now requires `register_years_expenditure`, each year's expenses since the
  register's first valid year, and refuses a register that ceased to be valid
  under s 37CD(3)'s 20% rule. FBT inputs given as a JSON list now reach the
  worksheet as a list. New codes: `fbt_car_parking.head_counts_invalid`,
  `fbt_car_parking.space_groups_invalid`,
  `fbt_car_parking.spaces_without_benefit`,
  `fbt_meal_entertainment.register_ceased` and
  `fbt_meal_entertainment.register_years_invalid`. Each reproduces the
  employer's guide's example 18 (spaces), example 17's market value and
  recipient contribution, and the 18.4 rental paid as an expense; see
  `docs/calculation-evidence.md`.
- `personal-tax-estimate` composes one full-year resident's income tax and
  levies for a year from `resident-tax`, `low-income-tax-offset`,
  `medicare-levy`, `medicare-levy-surcharge` and, when given,
  `seniors-pensioners-tax-offset`, `study-loan-repayment` and `division-293`,
  each called as it is alone; for a full-year foreign resident, from
  `foreign-resident-tax` and `division-293`, with no low income tax offset,
  Medicare levy or surcharge. It applies SAPTO and then LITO against basic tax
  only (ITAA 1997 s 63-10(1)), names the worksheet behind each line, keeps
  every part's result, and shows Division 293 tax outside the total. A part's
  refusal keeps its code and names the part. New codes:
  `estimate.residency_unsupported` (part-year residents, working holiday
  makers, and SAPTO or a study loan for a foreign resident),
  `estimate.surcharge_with_exemption`, `estimate.family_mismatch` (parts that
  describe different spouses or dependants) and `input.residency_invalid`.
  `tests/personal_tax_estimate.rs` holds it to the ATO's figures, and
  `tests/properties.rs` to the sum of its parts.
- `personal-tax-estimate` can work out taxable income from whole-dollar
  `assessable_labour_income`, `other_assessable_income`,
  `work_related_deductions` and `other_deductions` in place of
  `taxable_income` (ITAA 1997 s 4-15(1)), with the standard deduction for
  work-related expenses for a resident from 2026-27 (s 25-130, Act No. 49 of
  2026 Schedule 4), and compare the total with `tax_withheld` (TAA 1953 Sch 1
  s 18-15(1)) to give `refund` and `amount_payable`. Its sources add those 3
  sections and the Act. An estimate given neither `taxable_income` nor
  `assessable_labour_income` is refused as `input.missing_field` with its own
  message. The result for an established `taxable_income` without
  `tax_withheld` is unchanged apart from its sources, scope and
  `source_checked`.
- `personal-tax-estimate` can work out `tax_withheld` from one employer's
  regular pays: one pay's `payg-withholding`, `study-loan-withholding` or
  `seniors-pensioners-withholding` inputs and `pays_in_year` (at most 53
  weekly, 27 fortnightly, 12 monthly or 4 quarterly pays), with
  `schedule-5-method-a` for one bonus paid with one of them, but not with a
  Schedule 9 or quarterly pay or beside declared tax offsets, Medicare levy
  adjustment or extra pay amount, which Method A here does not take. The tax withheld
  is the pay's withholding times the pays, plus Method A's on the bonus, and
  the amounts add `withholding_per_pay` and `additional_payment_withholding`.
  A study loan pay needs `study-loan-repayment`; Schedule 9 or a study loan
  for a foreign resident is refused as `estimate.residency_unsupported`, and a
  scale that does not match the residency as the new code
  `estimate.scale_mismatch`. A study loan pay in 2025-26, whose Schedule 8
  coefficients changed on 24 September 2025, is refused as
  `period.unsupported`, and `extra_pay_withholding` needs 53 weekly or 27
  fortnightly pays. Its sources add TAA 1953 Sch 1 s 12-35. Results
  without the new inputs are unchanged apart from the scope.
- `derive` also covers `working-holiday-maker-withholding`,
  `study-loan-withholding`, `seniors-pensioners-withholding`, `quarterly-sg`
  and `borrowing-expenses`. Their steps cite the headings of Schedules 1, 8, 9
  and 15 to the Taxation Administration (Withholding Schedules) Instruments
  2024 to 2026, the Superannuation Guarantee (Administration) Act 1992
  compilation 77 and ITAA 1997 s 25-25. Their test files derive each case they
  run and check the result is the plain output. No command's plain output
  changes.
- `derive` also covers `payg-withholding`, the 3 `schedule-5` methods and
  `termination-payment`. Their steps cite the headings and method steps of
  Schedules 1, 5, 8 and 11 to the Taxation Administration (Withholding
  Schedules) Instruments 2024 to 2026, and ITAA 1997 s 82-10 and 83-170; a
  formula may now use `floor`. `tests/properties.rs`, `tests/schedule_5.rs`
  and `tests/termination_payment.rs` derive each case they run and check the
  result is the plain output. No command's plain output changes.
- `derive` also covers `division-293` and `pension-minimum`. Their steps cite
  ITAA 1997 s 293-20 and the Superannuation (Sustaining the Superannuation
  Contribution Concession) Imposition Act 2013 s 5, and SISR Schedule 7
  clauses 1, 3, 4 and 5; a formula may now use `max`.
  `tests/derivation.rs` also derives every case the Python corpus holds for a
  derived command. Neither command's plain output changes.
- `derive` returns an FBT worksheet's result with one step per amount: the
  FBTAA section or ATO page that sets it, its formula on the values it used,
  and the amount. It covers `fbt`, `fbt-car-statutory` and
  `fbt-car-operating-cost`; the worksheets render their amounts from these
  steps, and `tests/derivation.rs` recomputes each amount from its working.
  Another command is refused with the new code `derivation.unsupported`, and
  a key other than `command` and `input` with `input.field_unknown`. The
  plain commands' output is unchanged, and `tests/parity.rs` skips `derive`,
  which has no Python original.
- `--refusal-json`, after a command, adds a stable refusal code on stdout as
  `{"ok":false,"code":...,"message":...}`. The `error:` line, the message and
  exit 2 stay as they were, and stdout stays empty on a refusal without the flag.
  `REFUSAL_CODES` in `src/texts.rs` is the catalogue.
- `main-residence` works out the taxable share of an individual's capital gain
  or loss on one dwelling under the main residence exemption (ITAA 1997
  Subdiv 118-B), for CGT events in 2024-25 to 2026-27, from caller-established
  day periods. It covers partial income use (s 118-190), the absence choice
  with its 6-year limit (s 118-145), changing homes (s 118-140) and the home
  first used to produce income rule (s 118-192). No Python release has it. The
  worksheet catalogue lists it, so `tests/parity.rs` also drops that entry
  before comparing, and `tests/main_residence.rs` covers it. Catalogue examples
  can now hold JSON values.
- `borrowing-expenses` works out one income year's deduction for one loan's
  borrowing expenses under ITAA 1997 s 25-25, for 2024-25 to 2026-27: the
  s 25-25(6) $100 rule, the spread by days over the period of the loan (the
  shortest of the contract's period, the period to repayment and 5 years), the
  write-off in the year of repayment and the income-producing share. It refuses
  a leisure facility unless s 26-50(3) applies at all times in the year, and
  stops before 2027-28, when s 26-155 (Treasury Laws Amendment (Tax Reform No.
  1) Act 2026, Schedule 2) quarantines residential rental deductions. No Python
  release has it. `tests/parity.rs` drops its catalogue entry before comparing,
  and `tests/borrowing_expenses.rs` holds it to the ATO's Peter and Fiona and
  Max examples and Library ¶4-160 (Ben).
- `division-296` estimates one individual's Division 296 tax on large super
  balances for 2026-27 (ITAA 1997 Subdivision 296-B; Superannuation (Building a
  Stronger and Fairer Super System) Imposition Act 2026 s 5): 15% of the
  earnings share above the $3 million threshold and a further 10% of the share
  above $10 million, each share a percentage rounded to 2 decimal places, from
  the established balance at 30 June 2027 and total superannuation earnings,
  which may be negative. The 3 exceptions give nil tax with the reason in
  `liability`, and every result carries a warning that it is an estimate.
  Defined benefit and other s 296-70 interests, departing temporary residents
  and years from 2027-28 are outside it. No Python release has it.
  `tests/parity.rs` drops its catalogue entry before comparing, and
  `tests/division_296.rs` holds it to the Explanatory Memorandum's examples 1.1
  to 1.3 and the threshold and rounding boundaries. The new refusal code
  `input.signed_amount_invalid` covers its earnings input, and
  `input.field_unknown` refuses a field it does not take, such as an opening
  balance, which the other worksheets ignore.
- `termination-payment` works out the tax-free part of a genuine redundancy or
  early retirement scheme payment ($13,598 plus $6,801 for each completed year
  of service, ITAA 1997 s 83-170) and the Schedule 11 withholding from one life
  benefit employment termination payment made in 2026-27: the excluded part
  within the $270,000 ETP cap, then the non-excluded part within the smaller of
  the ETP cap left and the $180,000 whole-of-income cap less other taxable
  payments (s 82-10(4) and (7)), at 17% from preservation age at 30 June 2027,
  32% below it and 47% above the cap, each part rounded to the dollar. Death
  benefit ETPs, ETPs with a tax-free component, earlier ETPs and the tax
  assessed on the ETP are outside it. No Python release has it.
  `tests/parity.rs` drops its catalogue entry before comparing,
  `tests/termination_payment.rs` holds it to the ATO's Schedule 11 examples 1
  to 4 and each cap, limit, preservation age and rounding boundary, and
  the repository's root `tests/test_dated_rates.py` keeps its 4 dollar figures
  equal to the rate records. The new refusal code `input.years_of_service_invalid` covers its
  years of service, and `input.field_unknown` refuses a field it does not take.
- `schedule-5-method-a`, `schedule-5-method-b-i` and `schedule-5-method-b-ii`
  work out withholding from a weekly, fortnightly or monthly pay that includes
  back payments, commissions, bonuses or similar payments by Schedule 5 (NAT
  3348) Methods A, B(i) and B(ii), for payments made in 2024-25 to 2026-27,
  using the year's Schedule 1 amounts from `payg-withholding` and, for a payee
  with a study and training support loan, the Schedule 8 component from
  `study-loan-withholding` (from 1 July 2026 only). Methods A and B(ii) apply
  the 47% limit. Like `division-296`, each refuses a field it does not take.
  No Python release has them. `tests/parity.rs` drops their
  catalogue entries before comparing, and `tests/schedule_5.rs` holds them to
  the ATO's examples 1 to 4 in the 2024, 2025 and 2026 schedules and to each
  rule's boundary. New refusal codes: `input.pay_periods_invalid` and
  `input.back_payment_periods_invalid`.
- `foreign-resident-tax` gives a full-year foreign resident's basic income tax
  for 2024-25 to 2026-27 under Income Tax Rates Act 1986 Schedule 7 Part II:
  30% to $135,000, 37% to $190,000 and 45% above, with no tax-free threshold.
  The ATO's foreign resident page has no 2026-27 table, so a 2026-27 result
  warns that its rates come from the Act. Its scope says that a foreign
  resident pays no Medicare levy and has no low income tax offset. Like
  `division-296`, it refuses a field it does not take. No Python release has
  it. `tests/parity.rs` drops its catalogue entry before comparing, and
  `tests/foreign_resident_tax.rs` holds it to the ATO's 2024-25 and 2025-26
  table at every bracket boundary. `resident-tax`, `foreign-resident-tax` and
  `low-income-tax-offset` share one argument list.
- `working-holiday-maker-withholding` applies Schedule 15 to one weekly,
  fortnightly or monthly payment to a working holiday maker, for 2024-25 to
  2026-27: a registered employer's Table A rate (15%, 30%, 37% or 45%, by the
  column the payments made before it in the income year select), an
  unregistered employer's foreign resident rates through Schedule 1 scale 3,
  and 45% from any employer whose payee gave no TFN. It refuses payments before of
  $135,000.01 to $135,000.99, which no column covers
  (`working_holiday_maker.band_unsettled`), and, like `division-296`, a field
  it does not take. No Python release has it. `tests/parity.rs` drops its
  catalogue entry before comparing, and `tests/working_holiday_maker.rs` holds
  it to the ATO's lookup tool example and examples 1 and 2, each column
  boundary and the rounding rules.
- `seniors-pensioners-withholding` applies Schedule 9 (NAT 4466), the tax
  table for seniors and pensioners, to one weekly, fortnightly or monthly
  payment for 2024-25 to 2026-27, by the payee's status (single, a member of an
  illness-separated couple or a member of a couple) and any full or half
  Medicare levy exemption: the schedule's 9 scales, with Schedule 1's weekly
  earnings, formula, rounding and pay period conversions. Other tax offsets and
  the Medicare levy adjustment are not inputs, and quarterly pays, for which the
  schedule has no rule, are refused, as, like `division-296`, is a field it does
  not take. New refusal codes: `input.status_invalid` and
  `input.medicare_levy_exemption_invalid`. No Python release has it.
  `tests/parity.rs` drops its catalogue entry before comparing, and
  `tests/seniors_pensioners.rs` holds it to each schedule's worked example
  ($662.95 a week), every amount in each schedule's weekly tax table (2,120
  earnings in 3 columns) and hand-worked amounts on the exemption scales and
  for fortnightly and monthly pays. Its optional `study_loan` adds the study
  and training support loan component that Schedule 9 sends the payer to the
  weekly table (NAT 2173) for: Schedule 8's tax-free threshold rates on the
  same weekly earnings, for 2026-27 and, with `pay_date`, 2025-26 pays from
  24 September 2025, as `study-loan-withholding` takes them, shown beside the
  Schedule 9 amount and their total. Without it the amounts and rates are
  unchanged. `tests/seniors_pensioners_study_loan.rs` holds it to column 2 of
  all 2,616 rows of the table from 1 July 2026, both tables' examples and
  Schedule 8's examples, and `derive` shows the component and the total as
  steps.
- `seniors-pensioners-tax-offset` works out the seniors and pensioners tax
  offset (ITAA 1936 s 160AAAA) for 2024-25 to 2026-27 by the Income Tax
  Assessment (1936 Act) Regulations 2025 and the 2015 Regulation they replaced
  (ss 9 to 12): the base amount for the person's `status`, the rebate threshold
  and cut-out from the year's Schedule 7 rates, entitlement on the person's
  rebate income or, for a member of a couple, half the couple's
  (`spouse_rebate_income`), 12.5 cents a dollar off above the threshold, and
  the ATO's rounding up to the dollar. With `spouse_taxable_income` it adds a
  resident spouse's unused amount (s 12). 2026-27 thresholds, which the ATO
  has not published, are worked at that year's 15% rate, with a warning. Like
  `division-296`, it refuses a field it does not take. No Python release has
  it. `tests/parity.rs` drops its catalogue entry before comparing, and
  `tests/seniors_pensioners_offset.rs` holds it to the ATO's 9 examples,
  including the transfer in example 9, its 2024-25 and 2025-26 thresholds and
  every cut-out; `tests/properties.rs` checks that it never rises with rebate
  income without a transfer. Schedule 9 and this command share the `status` values and the
  `input.status_invalid` refusal.
- `lump-sum-arrears-tax-offset` works out the tax offset for lump sum payments
  in arrears (ITAA 1936 ss 159ZR to 159ZRD) for a year of receipt from
  2013-14 to 2026-27: the 10% threshold test on normal taxable income (s 159ZRA(1)(b)),
  the tax on the arrears (s 159ZRB), the notional tax in each recent accrual
  year (s 159ZRC) and in distant years at the average of the recent years'
  rates, each truncated to 3 decimal places and raised by 0.001 when the
  fourth is more than 4 (s 159ZRD), and the rebate-year adjustments. A new
  `PRIOR_RESIDENT_TAX_SCALES` table holds the ATO's resident rates for
  2012-13 to 2023-24, with the 2% temporary budget repair levy above $180,000
  in 2014-15 to 2016-17 (Income Tax (Transitional Provisions) Act 1997
  s 4-11); `resident-tax` periods do not change. A recent accrual year before
  2012-13 is refused with `lump_sum_arrears.recent_year_unsupported`, and a
  malformed accrual year with `lump_sum_arrears.accrual_years_invalid`. The
  Medicare levy exemption (Medicare Levy Act 1986 s 9A) and surcharge offset
  (ITAA 1997 Subdivision 61-L) are named as separate questions, not worked
  out. `derive` covers it. Like `division-296`, it refuses a field it does
  not take. No Python release has it. `tests/lump_sum_arrears_offset.rs`
  holds it to the ATO's Sudeep and Anthony results, works the Heather and Yan
  facts by hand with assumed taxable incomes and checks each earlier year's
  rates.
- `depreciation` takes an optional `opening_adjustable_value` for an income
  year after the asset's start time (ITAA 1997 ss 40-70(1), 40-72 and 40-75).
  Diminishing value then applies its rate to that value and prime cost to cost,
  and either decline is limited to it; it may not exceed cost. Without it the
  result is the start year's, as before. The worksheet catalogue lists it under
  a new `optional_inputs` key, and the scope, sources and source passage name
  the later year, which `tests/parity.rs` applies to the frozen results.
  `tests/depreciation.rs` holds it to the ATO's diminishing value and prime
  cost examples. The new refusal code `input.opening_value_above_cost` covers
  an opening value above cost.
- `payg-withholding` completes Schedule 1 for 2024-25 to 2026-27: quarterly
  pays, scale 4 (no tax file number, with a required `foreign_resident`), tax
  offsets claimed on a withholding declaration (`tax_offsets`, scales 2, 5 and
  6), the Medicare levy adjustment on scales 2 and 6
  (`medicare_adjustment_children`, 0 for a spouse only) and the additional
  amount for a year of 53 weekly or 27 fortnightly pays
  (`extra_pay_withholding`). The worksheet catalogue lists them under
  `optional_inputs`; a result gains the new amount keys only when one is given.
  An option on a scale or period it does not apply to is refused, as are
  reductions that exceed the amount withheld before them, which Schedule 1 does
  not provide for; that refusal asks for the declarations and inputs to be
  checked and, if they are right, for the ATO to be asked. `tests/payg_withholding.rs` reproduces all 1,584 Medicare
  adjustments in each schedule's scale 2 and scale 6 sample data, transcribed
  into `tests/data` and checked against the legislative instruments, and every
  worked example on the schedules' Medicare and examples pages, with each
  threshold, band and rounding boundary. A Medicare levy variation that
  increases withholding, bi-monthly, daily and casual pays stay outside it. The
  0.2.0 wheel the MCP server pins has none of this.
- `study-loan-withholding` also accepts 2025-26 pays made from 24 September
  2025, with the component coefficients of Schedule 8 to the Taxation
  Administration (Withholding Schedules) Instrument (No. 2) 2025 (F2025L01016)
  and the Schedule 1 coefficients for payments from 1 July 2024 to 30 June 2026.
  The pay date selects the coefficients within the year: a 2025-26 pay needs
  `pay_date`, and one before 24 September 2025, when F2025L00647's Schedule 8
  worked the component as a percentage of weekly earnings, is refused, as is
  a date outside the income year. `periods` starts 2025-26 on 24 September
  2025. `tests/study_loan_withholding.rs` holds it to the instrument's examples
  1 to 3, its starting earnings and pays on 23 and 24 September 2025; the
  combined coefficient checks in `tests/tables.rs` and `tests/properties.rs`
  cover both years. The 0.2.0 wheel the MCP server pins covers 2026-27 only.
- `study-loan-withholding` accepts quarterly pays under Schedule 8's rule: the
  weekly equivalent is a thirteenth of the earnings, and 13 times the weekly
  amounts is withheld. Python 0.2.0 refused them; `tests/parity.rs` skips and
  counts the 1 frozen quarterly refusal, and the 2026 schedule and F2025L01016
  have no quarterly example, so `tests/study_loan_withholding.rs` checks
  hand-worked amounts and the quarterly starting earnings.
- The 3 Schedule 5 commands take an optional `pay_date` with `study_loan`, so
  the study loan part of Methods A and B(ii) covers 2025-26 pays made from 24
  September 2025 with F2025L01016's coefficients. A 2025-26 study loan pay
  without its date, or dated before 24 September 2025 or outside the year, is
  refused as by `study-loan-withholding`; 2024-25 still refuses `study_loan`.
  Method B(i) still refuses a 2025-26 study loan, now saying why: Schedule 5
  does not say which coefficients recalculate an earlier period from before 24
  September 2025. A pay date without `study_loan` is refused with the new code
  `payg.pay_date_without_study_loan`.
- `tests/verdict_cases.rs` turns the edge cases in a published reviewer verdict,
  under CC BY 4.0, the default in the bounty's terms, into fabricated cases, or
  states why one does not apply.
- `fbt-car-operating-cost` takes a required `effective_life_years` and works out
  deemed depreciation at 200% over it (FBTAA s 11(1AA)). It had applied 25% to
  every owned car, which is right for an 8-year life and understates a taxi's
  4-year life or a rental car's 5. A life under 2 is refused, except 0 for a leased
  car or an owned car with no depreciated value left.
- `fbt-car-statutory` and `fbt-car-operating-cost` value one car fringe benefit
  under FBTAA s 9 or s 10 for the FBT years ended 31 March 2026 and 2027; the
  operating cost result takes the lower statutory formula value under s 10(5).
  No Python release has them. The worksheet catalogue lists both, so
  `tests/parity.rs` compares the 10 frozen `worksheet-catalogue` results without
  those 2 entries, and `tests/fbt_car.rs` covers them.
- Both car worksheets end their amounts with `reportable_taxable_value`, the
  taxable value before any exemption (an exempt electric car's notional value),
  and add a third warning: the employee's reportable fringe benefits amount is
  the total of all their reportable benefits at the type 2 gross-up rate when it
  exceeds $2,000, a pooled or shared car is not reportable, and the worksheet
  determines neither. Every other amount and warning is unchanged.
- `payg-gross-up` finds the gross pay whose Schedule 1 withholding leaves a
  stated net pay, for one regular pay on the scale, pay period and income year
  `payg-withholding` takes, with `foreign_resident` for scale 4. It searches
  `payg-withholding`'s own results rather than inverting the formulas, and
  refuses with the new code `payg.gross_up_not_unique`, listing every gross,
  where more than one leaves the net: Schedule 1 withholds whole dollars, so a
  gross a dollar higher can withhold a dollar more and leave the same net. A
  net above $499,999,999,974.00 is refused with `payg.gross_up_net_too_large`.
  No Python release has it. `tests/payg_gross_up.rs` grosses up the net of
  every row of the ATO's Schedule 1 sample data for both schedules to the
  row's earnings or a tie that names them, and `tests/properties.rs` checks
  that withholding on each answer leaves the net and that no other gross
  within $60 of withholding does. `tests/refusal_codes.rs` now also reads the
  engine's modules under `src/engine/`, and `payg-withholding` and
  `study-loan-withholding` share one argument list.
- `payg-gross-up` takes `study_loan` and, with it, `pay_date`: it then
  searches `study-loan-withholding`'s results, so the gross carries Schedule
  8's component for 2026-27 and for 2025-26 pays from 24 September 2025. Scale
  4 and `foreign_resident` are refused with it, as is a net above
  $333,333,333,300.00. `tests/payg_gross_up.rs` grosses up the net of Schedule
  8's examples 1 to 3 for 2026-27 and example 1 for 2025-26, worked by hand,
  and `tests/properties.rs` checks random nets with a study loan as without.
  The result without `study_loan` is unchanged.
- `hire-purchase-schedule` sets out a hire purchase agreement month by month
  for the hirer, the notional buyer under ITAA 1997 s 240-17(2), for
  agreements starting in 2024-25 to 2026-27: level monthly instalments in
  arrears or in advance and any balloon at the end of the term. It works out
  the implicit interest rate from the payments, as s 240-60(1) step 3 does,
  rather than taking a stated rate, and shows each month's notional interest
  as the interest to date rounded to cents less what earlier months show, so
  the months add up to the total charges and the last balance is nil. New
  codes: `hire_purchase.amount_nil`, `hire_purchase.instalments_unsupported`,
  `hire_purchase.rate_imprecise`, `hire_purchase.rate_unsupported`,
  `hire_purchase.start_outside_year` and `input.timing_invalid`. No Python
  release has it. `tests/hire_purchase.rs` holds it to agreements at exactly
  10% and 5% a month worked by hand, to fabricated agreements worked
  independently and to 2 schedules LodgeiT Labs publishes at a stated rate,
  where it closes at nil as they do not, and `tests/properties.rs` checks that the months add up and
  that each month's interest is within 2 cents of the rate shown times what
  it accrues on.
  `tests/refusal_codes.rs` now also reads the engine's modules under
  `src/engine/`.
- `hire-purchase-schedule` also gives `interest_by_income_year`, the
  interest of the months wholly in each income year, and
  `interest_across_30_june`, each month that spans 30 June with its interest
  and its days in each year, which s 240-60(2) leaves to accounting
  principles and the worksheet does not split. `tests/hire_purchase.rs`
  holds it to agreements worked by hand at 10% a month.
- `dgr-check` says, for a list of donations by ABN or name, whether each
  recipient is shown as a deductible gift recipient in the ABN Bulk Extract of
  11 November 2026 (CC BY 3.0 AU), from what date, how that reads on the
  payment date, and which ITAA 1997 Division 30 conditions stay open; it
  applies the $2 minimum only to gifts made before 1 July 2024 (Act No. 58,
  2026, Sch 1) and never says a payment is deductible. A name alone gives
  unconfirmed candidates. `data/dgr-register.jsonl` keeps only the extract's
  31,876 ABNs with DGR entries, built by `examples/dgr_register.rs`, which
  keeps an entry for a fund the extract leaves unnamed as a fund, as ABN Lookup
  lists it, and has its own tests. New codes:
  `input.abn_invalid`, `input.donations_invalid` and `input.name_invalid`.
  `tests/dgr_check.rs` holds it to ABNs read the same in ABN Lookup's DGR
  listing and to the Library's worked example ¶7-020, and `tests/properties.rs`
  checks that it never reads an ABN in its data as absent.
- `income-tests` works out 6 income measures for a full-year resident for
  2024-25 to 2026-27 from taxable income and the components each definition
  adds back: income for surcharge purposes (ITAA 1997 s 995-1), Division 293
  income (s 293-20(1)(a)), rebate income (ITAA 1936 s 6(1)), repayment income
  (HESA s 154-5), adjusted taxable income for offsets (Family Assistance Act
  Sch 3) and the super co-contribution's total income (Superannuation
  (Government Co-contribution for Low Income Earners) Act 2003 s 8). Each
  measure is a `derive` step citing its definition, and `not_used` lists the
  components each leaves out. Every component is required; a negative one, a
  component larger than the amount that includes it, a part-year or foreign
  resident and another year are refused. Income for surcharge purposes follows
  s 995-1, which leaves out the exempt foreign employment income the ATO's
  surcharge page adds, and warns when they differ. No command's behaviour
  changes. No Python release has it. `tests/parity.rs` drops its catalogue
  entry before comparing, and `tests/income_tests.rs` holds it to the ATO's
  Christina and Tom examples, passes each measure to the command that takes
  it and checks each adjustment and refusal.
- `gst`, `resident-tax`, `foreign-resident-tax`, `low-income-tax-offset`,
  `medicare-levy`, `seniors-pensioners-tax-offset`, `division-293` and
  `pension-minimum` also accept 2027-28, the year whose every rate and
  threshold each takes the law already sets: Income Tax Rates Act 1986
  Schedule 7 (14%, 30%, 37% and 45% for a resident; 30%, 37% and 45% for a
  foreign resident), ITAA 1997 ss 61-115 and 293-20, the Medicare Levy Act
  1986 thresholds Act No. 58, 2026 applies to later years, the SAPTO
  regulations' formulas at the 14% lowest rate, the GST rate and SISR
  Schedule 7, with 366 days for the pension minimum. The ATO had published
  none of them for 2027-28 when they were read on 16 November 2026, so each
  result but GST, LITO, Division 293 and the pension minimum adds a warning
  saying so; the `resident-tax` one also says the s 119-10 extra tax on a
  minimum tax capital gain, from 2027-28, is not worked out. A worksheet that
  still refuses 2027-28 adds why to its `period.unsupported` message: the
  withholding schedules, Medicare levy surcharge thresholds, study loan
  thresholds, contribution caps, ETP caps and indexed Division 296
  thresholds were not yet published; CGT changes from 1 July 2027 (Act No.
  49 of 2026, Schedule 1) and s 26-155 limits rental deductions; or 2027-28's
  366 days exceed the 365 `depreciation` takes. `medicare-levy-surcharge`
  takes 2027-28 only with `days_without_cover` 0, when Medicare Levy Act 1986
  ss 8B, 8C and 8D leave no surcharge whatever the thresholds, and refuses a
  day without cover. `personal-tax-estimate` composes 2027-28 for a foreign
  resident, and for a resident applies the working Australians tax offset
  (ITAA 1997 ss 61-155 and 61-160) first (s 63-10(1) item 3): the lesser of
  $250 and the basic income tax on net labour income, which it takes as the
  new `net_labour_income` (required beside `taxable_income`) or works out from
  the labour inputs. The amounts add `net_labour_income`,
  `working_australians_tax_offset` and `working_australians_tax_offset_applied`;
  its sources add ss 61-155 and 61-160. It does not work out the s 119-10
  extra tax on a minimum tax capital gain, which its scope excludes. Results
  before 2027-28 are unchanged apart from the scope and sources.
  `hire-purchase-schedule`'s 2027-28 refusal gives no reason: s 26-155 does
  not touch it. `tests/parity.rs` adds 2027-28
  to the 6 frozen commands' supported periods, catalogue and refusals, adds
  the reason to the 13 frozen 2027-28 refusals of the others, and skips the
  17 frozen calls that expected 2027-28 to be refused; `tests/year_2027_28.rs`
  checks that year with hand-worked figures.

### Documented divergences

- `contribution-caps` results and its catalogue entry give the new scope,
  which describes the carry-forward by year, cite the ATO's concessional
  contributions cap page and ITAA 1997 s 291-20 on the ATO's legal database,
  and are checked on 16 November 2026; the catalogue lists the 3 new optional
  inputs. No amount or rate changes. `tests/parity.rs` applies this to the frozen
  `contribution-caps` and `worksheet-catalogue` results.
- `depreciation`'s closing adjustable value is the opening value less the
  decline in value as shown in cents. Python 0.2.0 rounded the 2 separately, so
  a decline ending in exactly half a cent showed a cent too much in both: $10,001
  over 8 years by prime cost gave $1,250.13 and $8,750.88, now $8,750.87.
  Carried forward year by year, the old figures wrote off more than cost. No
  frozen result changes.
- `depreciation` divides once, last, so an exact half cent is not lost: $1,000.50
  over 14 years by diminishing value at 21% taxable use is exactly $30.015,
  now $30.02; Python 0.2.0 gave $30.01. No frozen result changes. The result
  is exact whenever the effective life's significant digits and the taxable
  use's decimal places total 22 or fewer, within the 40-digit working
  precision, and `depreciation` refuses anything longer with
  `depreciation.precision_exceeded`. Python 0.2.0 accepted it, and an
  effective life of 37 significant digits could land a cent from the exact
  value. No frozen call has such an input.
- `depreciation` refuses a field it does not take with `input.field_unknown`,
  as `division-296` does; Python 0.2.0 ignored it. A misspelt
  `opening_adjustable_value` would otherwise give the start year's result for a
  later year. No frozen call passes such a field.
- `payg-withholding` and `study-loan-withholding` refuse a field they do not
  take with `input.field_unknown`, as `division-296` does; Python 0.2.0
  ignored it. A
  misspelt `tax_offsets` would otherwise withhold too much, and a misspelt
  `pay_date` would be ignored. No frozen call passes such a field.
- Python imports, keyword signatures, the `metadata` dictionaries and package
  version lookup are removed. `engine_version` in every result is the Cargo
  version, 1.0.0; Python 0.2.0 reported `0.2.0`. That field is the only
  difference in the 3,247 frozen results that carry it.
- Amounts and ratios arrive as decimal strings. A JSON number or `null` reaches
  the worksheet as the int, float or `None` it would have been in Python and is
  refused with the original message; text that is not a decimal is refused as
  `decimal.InvalidOperation`, as `Decimal(text)` raised it. Flags, counts, ages,
  days, quarters and scales are JSON values checked by the original type rules,
  so `2.0` is not a scale and `true` is not an integer. An integer of up to 4,300
  digits, the limit Python's `json.loads` applied, is an `int`, as in Python, and a list or object period is quoted in the
  refusal as Python's `repr()` wrote it.
- The JSON boundary refuses input over 8 MiB, a decimal string over 8,192 bytes
  and a missing argument (`<name> is required`; Python raised `TypeError`). JSON
  uses serde_json's nesting limit and refuses lone Unicode surrogate escapes.
- A JSON object that repeats a key is refused as `input.invalid_json`, with a
  message naming the key, instead of keeping the last value and dropping the
  others. Python 0.2.0 took keyword arguments, which cannot repeat.
- `periods` with a worksheet name it does not know is refused as
  `unknown worksheet '<name>'`, a missing `kind` as `kind is required` and a
  non-string `kind` as `kind must be a string`; Python raised `KeyError` or
  `TypeError`.
- A field a worksheet does not take is ignored; Python raised `TypeError` for an
  unexpected keyword argument.
- Results are compact JSON on stdout with native text-mode newlines. A refusal
  is one `error:` line on stderr with exit 2.
- The Python/PyPI release workflow, installed-distribution smoke script and the
  fast-check property bridge are removed. The properties run as seeded Rust
  tests over the same input domains. Until the MCP server links this crate,
  the uv workspace installs the 0.2.0 Python release from PyPI for the MCP
  server and `lodgeit-calculator-adapter`. A signed native publisher remains a separate
  release task.
- The ATO moved and retitled its study loan page to "Study and training loan
  minimum repayment income and rates", which now says "minimum repayment
  income" where it said "repayment threshold". Read again on 20 October 2026,
  it prints the same figures, so no calculation changes. Every
  `study-loan-repayment` result cites the new address, with `source_checked`
  `2026-10-20`; the `worksheet-catalogue` entry also carries the new title and
  the page's 13 October 2026 update. `tests/parity.rs` applies these changes to
  the 227 `study-loan-repayment` and 10 `worksheet-catalogue` frozen results.
- The ATO moved its "Lump sum payment in arrears tax offsets" page to "Lump
  sum payment in arrears" under income you must declare; the old address
  redirected there on 26 October 2026. The fourth `medicare-levy` source and
  the s 9A refusal message cite the new address, with no calculation change.
  `tests/parity.rs` applies this to the 162 `medicare-levy` results, 56
  `medicare-levy` refusals and 10 `worksheet-catalogue` frozen results.
- `contribution-caps` reports as `carry_forward_applied` only the unused
  concessional cap that covers contributions over the year's cap, as ITAA 1997
  s 291-20(4) applies it; Python 0.2.0 reported all the eligible unused cap,
  even when contributions were under the cap. With a $30,000 cap, $15,000
  unused and $40,000 contributed, it is now $10,000, not $15,000. The cap
  available, room and excess are unchanged. `tests/parity.rs` applies this to
  the 233 frozen `contribution-caps` results; 18 change.
- `gst`, `capital-gains` and `depreciation` also accept 2026-27; Python 0.2.0
  took 2025-26 only. The law they apply is unchanged for that year: the GST rate
  is 10%, the decline in value formulas are those the ATO's prime cost and
  diminishing value page gives. CGT reforms take effect from 1 July 2027
  ([ATO](https://softwaredevelopers.ato.gov.au/reforming-negative-gearing-dsp-special-purpose-working-group),
  read on 3 November 2026); `tests/tables.rs` keeps 2027-28 outside
  `capital-gains`. `quarterly-sg`
  stays at 2025-26. `tests/parity.rs` adds the period to the 19 `periods` and
  10 `worksheet-catalogue` frozen results and to the 34 refusals that list the
  supported periods.
- `payg-withholding` also accepts 2024-25 and 2025-26, using the ATO's Schedule
  1 coefficients for payments from 1 July 2024 to 30 June 2026 (published 17
  June 2024); Python 0.2.0 took 2026-27 only. The earnings conversion and
  rounding rules are the same in both schedules. `tests/tables.rs` reproduces
  all 720 amounts in that schedule's sample data for each of the 2 years.
  The scope statement now covers payments from 1 July 2024 and says the income
  year selects the coefficients; the sources add the 2024-26 schedule, and the
  source check date is 4 November 2026, when both schedules' coefficient pages
  were read. `tests/parity.rs` applies this to the 789 frozen
  `payg-withholding` results, 7 `periods` results, 10 `worksheet-catalogue`
  results and the 8 refusals that list the supported periods. It skips the 1
  frozen call that expected 2025-26 to be refused; `tests/tables.rs` now
  checks that 2023-24 and 2027-28 are refused. The catalogue example stays at
  2026-27.
- `payg-withholding` accepts quarterly pays and scale 4, which Python 0.2.0
  refused. The scope statement, the source passage, the library evidence and
  the source check date (14 November 2026) name the new coverage, and the
  catalogue entry lists the optional inputs; `tests/parity.rs` applies this to
  the 789 frozen `payg-withholding` results and the 10 `worksheet-catalogue`
  results. The 36 frozen pay period refusals and 20 scale refusals now list
  quarterly and scale 4, and the 17 frozen scale 4 calls are refused for want
  of `foreign_resident`. It skips the 1 frozen call that expected a 2026-27
  quarterly pay to be refused; `tests/payg_withholding.rs` covers quarterly
  pays.
- `fbt` also accepts the FBT year ended 31 March 2027; Python 0.2.0 took 2026
  only. The ATO's FBT rates and thresholds page (updated 20 May 2026, read 4
  November 2026) gives the 47% rate and the 2.0802 and 1.8868 gross-up rates
  for years ending 31 March 2023 to 2027, so no figure changes. The Federal
  Register's 2026 Acts, read on 4 November 2026 to the latest made on 26
  October, change no FBT rule for that year: the Tax Reform No. 1 Act's FBT
  amendments apply from 1 April 2027, and no Act amends the electric car
  exemption in s 8A. The result's period now names its own year. The scope
  statement names both years and their rates and says the 2027 return items
  follow the FBT return 2026 instructions until the ATO publishes the 2027
  return; the sources add the rates page, and the source check date is 4
  November 2026. `tests/parity.rs` applies this to the 103 frozen `fbt`
  results, 6 `periods` results, 10 `worksheet-catalogue` results and the 56
  refusals that list the supported periods. It skips the 1 frozen call that
  expected 2027 to be refused; `tests/tables.rs` checks 2027 against the ATO's
  MantCo example and that 2028 is refused. The catalogue example stays at 2026.
- `medicare-levy` also accepts 2026-27; Python 0.2.0 refused it because the
  ATO had published no thresholds for it. Treasury Laws Amendment (Delivering
  an Efficient and Trusted Tax System) Act 2026 (No. 58, 2026, assented 30 June
  2026) Sch 5 item 14 applies the s 3(1) threshold amount and phase-in limit it
  set, $28,011 and $35,013, to the 2025-26 and later years of income, and the
  Medicare Levy Act 1986 as compiled at 1 July 2026 (compilation 53, the latest
  on 14 November 2026) still holds them. Both were read on 14 November 2026,
  when the ATO's low-income page still gave 2025-26 figures only. A 2026-27
  result adds a warning saying so and that a later Act can raise them for
  2026-27, as Act No. 58 did for 2025-26. The sources add Act No. 58, 2026
  and the source check date is 14 November 2026. `tests/parity.rs` applies
  this to the 162 frozen `medicare-levy` results, 6 `periods` results, 10
  `worksheet-catalogue` results and the 7 refusals that list the supported
  periods. It skips the 1 frozen call that expected 2026-27 to be refused;
  `tests/tables.rs` checks 2026-27 at each threshold and that 2027-28 is
  refused. The catalogue example stays at 2024-25.
- `medicare-levy` takes 5 optional inputs: `sapto_entitled`, which applies the
  seniors and pensioners thresholds in s 3(1); and `spouse_taxable_income`
  with `spouse_sapto_entitled`, or `dependant_offset_entitled` for a person
  without a spouse, with `dependent_children`, which apply the s 8 family
  reduction for 2024-25 to 2026-27. A spouse whose taxable income is above the
  spouse's threshold amount shares the reduction by taxable income, and the
  part of either share above that spouse's levy reduces the other's (s 8(3)
  and (4)); the result then adds a warning that its amounts are rounded only
  for display. Without the optional inputs every result is as before, except
  that its scope now describes them and its sources add the ATO's family
  income page, compilation 52 and Act No. 28, 2025. Like `depreciation`, it
  now refuses a field it does not take, and it refuses a spouse input without
  a spouse, `dependant_offset_entitled` with one and dependent children
  without either (`input.field_unknown`), and a child count that is not a
  non-negative integer (`input.dependants_invalid`). The worksheet catalogue
  lists the inputs under `optional_inputs` and names the family page and its
  Ashton example. `tests/parity.rs` applies the new scope, sources, passage
  and evidence to the 162 frozen `medicare-levy` results and 10
  `worksheet-catalogue` results, and `tests/medicare_family.rs` holds the
  worksheet to the Ashton example and every threshold.
- `medicare-levy` takes 2 more optional inputs, the days a full-year resident
  qualified for a full or a half Medicare levy exemption
  (`medicare_levy_exemption_full_days` and `medicare_levy_exemption_half_days`,
  return labels M1 V and W), as the operator establishes them. Medicare Levy
  Act 1986 s 9(1) then reduces the levy after the low-income and family
  reductions by the share of the year's days for which the person was a
  prescribed person, a half exemption day counting as half (ITAA 1936
  s 251U(3)); the result adds `exemption_reduction` and the day counts. Days
  outside 0 to the year's days, or more than the year together, are refused
  (`input.day_count_invalid`). Without the inputs every result is as before,
  except that the scope now describes exemptions and excludes foreign
  residents, and the sources, passage and evidence add ITAA 1936 s 251U and
  the ATO's exemption pages, which `tests/parity.rs` applies to the frozen
  results. `tests/medicare_exemptions.rs` holds the worksheet to the ATO's
  Priya example and hand-worked days after each reduction, and
  `tests/properties.rs` checks that more exemption days never raise the levy.
- `medicare-levy-surcharge` works out the Medicare levy surcharge for
  2024-25 to 2026-27 (Medicare Levy Act 1986 ss 8B and 8D and the A New Tax
  System (Medicare Levy Surcharge - Fringe Benefits) Act 1999 ss 12, 15 and
  16) for a full-year resident in no exemption category who was single without
  dependants or married for the whole year. Income for surcharge purposes,
  combined with the spouse's for a married person, sets the tier against the
  ATO's singles or family thresholds, which rise by $1,500 for each dependent
  child after the first; the 1%, 1.25% or 1.5% rate applies to taxable income
  plus reportable fringe benefits only, for the days without appropriate
  private patient hospital cover as a share of the year. A married person
  whose own income for surcharge purposes is at or below the s 3(1) threshold
  amount pays none. Children without a spouse (s 8C) are refused
  (`input.field_unknown`), as, like `division-296`, is a field it does not
  take. No Python release has it. `tests/parity.rs` drops its catalogue entry
  before comparing, and `tests/medicare_surcharge.rs` holds it to the ATO's
  Tom, Josh, Jacinta and Jill and Kevin examples and every tier boundary.

### Verification

A differential harness exported base 83288b0, ran its pytest suite and captured
the calls each test made, capped at 60 per test, then compared Python 3.15.0
with the Rust binary on those calls, 2,008 generated cases, 1,215 cases at every
statutory threshold in the base tables and its neighbours (uncapped), and 23 of
the final review's counterexamples. All 4,459 results
match byte for byte, including key order, after the `engine_version` adaptation.
They are frozen in `tests/data/python-api.json` and replayed by
`tests/parity.rs`. `tests/tables.rs` and `tests/properties.rs` run the table
checks, every value in the ATO's Schedule 1 sample data, the seeded properties
and the whole-range sweeps in full. `tests/BEHAVIOURS.md` maps each of the 109
original tests to its Rust coverage.

### Dependencies and reuse

Python 0.2.0 had no third-party runtime dependencies. Its Decimal dependency is
replaced by the BigUint-based Python Decimal compatibility code from The WIP
Tally's port (adapted from The Exchequer Tally and accounting-review-pipeline's
elizabeth-anne-alexander port, Apache-2.0), extended here with a per-thread
precision for the worksheets' 40-digit `localcontext`, ROUND_DOWN, remainder and
integral rounding. Rust num-bigint, num-integer and num-traits implement exact
coefficient arithmetic; serde_json, with insertion order preserved, provides the
JSON boundary. Scope statements and sources in `src/texts.rs` are generated from
the base revision. This component owns its copies and depends on no other
engine.
