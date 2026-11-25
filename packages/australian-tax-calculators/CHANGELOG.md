# Changes

## Unreleased

### Added

- `thin-capitalisation` works out the debt deductions a company that is a
  general class investor has disallowed under ITAA 1997 Subdivision 820-AA
  for 2023-24 to 2026-27, by the fixed ratio, group ratio or third party
  debt test, with each deduction's share, the special deduction for earlier
  fixed ratio test disallowed amounts and the ss 820-35, 820-37 and 820-39
  exemptions. `derive` takes it. New codes: `thin_capitalisation.carry_forward_invalid`,
  `carry_forward_unsettled`, `debt_deductions_invalid`,
  `entity_class_not_built`, `fact_invalid`, `gr_group_ebitda_not_positive`,
  `group_ratio_negative`, `input_missing`, `input_not_used` and
  `plantation_forestry`.
- `debt-forgiveness` applies ITAA 1997 Division 245 to one debtor's
  commercial debts forgiven in 2020-21 to 2026-27: each debt's gross and net
  forgiven amounts and the total's reduction of tax losses, net capital
  losses, expenditure and cost bases in the statutory order. `derive` takes
  it. New codes: `debt_forgiveness.attributable_income`, `debt_invalid`,
  `forgone_exceeds_provisional`, `item_invalid` and `partnership`.

## 1.4.0

### Added

- `gst-food-retailer-sam` works out, for one tax period from 28 September
  2017 to 30 June 2027, the GST-free share of trading sales and trading stock
  purchases under the food retailers' simplified accounting methods: the
  business norms, stock purchases and snapshot methods of F2017L01274, the
  supermarket and convenience store method of F2025L01055 and the restaurant,
  cafe and caterer method of F2026L01275 (including s 7(d) and the 6-month
  first-tax-period proportion rule), with trading stock input tax
  credits for the last 2. New codes: `gst_food_retailer_sam.amount_exceeds_total`,
  `components_unbalanced`, `first_period_sample_required`, `method_invalid`,
  `method_not_in_force`, `new_business_unsettled`,
  `not_eligible`, `retailer_type_invalid`, `sample_invalid`,
  `stock_role_invalid`, `tax_period_invalid`, `tax_period_spans_halves` and
  `total_nil`. `derive` takes it.
- `gst-error-correction` tests whether an earlier GST error may be corrected
  in a later return under F2023L01284 ss 4, 6 and 7, with the debit error time
  and value limits by current GST turnover and excluding TAA 1953 Sch 1
  s 105-65 amounts; `gst-credit-time-limit` tests whether an input tax credit
  can be claimed under GST Act s 29-10(3) and Division 93 (including s 29-10(5)
  attribution elections, s 93-15's supplier assessment qualification and
  s 93-10(4) and (5) refusals). New codes:
  `gst_error_correction.dates_invalid`, `fact_invalid`, `lodged_outside_year`,
  `gst_credit_time_limit.dates_invalid`, `fact_invalid`,
  `partial_supplier_gst`, `return_given_outside_year`,
  `section_93_10_exception` and `special_attribution`.
- `aml-cdd-review`: deterministic Australian AML/CTF Tranche 2 customer due
  diligence helper for 2026-27 with zero personal data (`country_risk` using
  AUSTRAC's Basel AML Index 0-5 / 5.01-6 / 6.01-10.00 bands and FATF/sanctions
  overrides plus statutory s 32(d) FATF Call-for-Action ECDD;
  `beneficial_ownership` for exact 25% indirect ownership chains and s 11
  control limbs with Rules ss 6-7 and 6-8; and `cdd_review` for statutory CDD
  tiers, ECDD/SoW-SoF/immediate triggers and periodic review dates cleanly
  distinguishing AUSTRAC Accountant Program Starter Kit 12/24/36-month defaults,
  custom firm policies and statutory-only mode). New codes:
  `aml_cdd_review.basel_score_invalid`,
  `aml_cdd_review.cadence_basis_invalid`, `aml_cdd_review.chains_invalid`,
  `aml_cdd_review.date_outside_year`, `aml_cdd_review.duplicate_field`,
  `aml_cdd_review.entity_kind_invalid`,
  `aml_cdd_review.exemption_inconsistent`, `aml_cdd_review.exemption_invalid`,
  `aml_cdd_review.months_invalid`, `aml_cdd_review.ownership_above_100`,
  `aml_cdd_review.pep_change_inconsistent`, `aml_cdd_review.pep_invalid`,
  `aml_cdd_review.question_invalid`, `aml_cdd_review.risk_invalid` and
  `aml_cdd_review.starter_kit_risk_inconsistent`.
- `residential-vacancy-fee` works out the Australian federal annual vacancy fee
  for a foreign owner of a residential dwelling for vacancy years starting in
  2020-21 to 2026-27 under FATA 1975 Part 6A (ss 115A to 115T), FATR 2015
  Division 4A (ss 43C and 43D), Fees Imposition Act 2015 ss 5 to 9 and Fees
  Imposition Regulations 2020 Part 3 (ss 42 to 46), with the 183-day occupancy
  test, late-return deemed liability (s 115D(3) and FATR s 43D(4)), 9 April
  2024 fee doubling (s 67(2)), established dwelling 3x tripling (ss 10(2) and
  53(1)(c)), 9 verified indexed schedules from 1 January 2021 to 30 June 2027,
  joint tenancy and tenants in common apportionment (s 57), full-waiver
  substitution (ss 43(b) and 46(b)), statutory fee cap (Act s 6(3), Regs
  s 61A) and `derive` steps. New codes:
  `residential_vacancy_fee.acquisition_basis_invalid`,
  `residential_vacancy_fee.application_date_unsupported`,
  `residential_vacancy_fee.application_fee_invalid`,
  `residential_vacancy_fee.build_to_rent_unsupported`,
  `residential_vacancy_fee.cap_unsettled`,
  `residential_vacancy_fee.consideration_invalid`,
  `residential_vacancy_fee.date_outside_year`,
  `residential_vacancy_fee.days_invalid`,
  `residential_vacancy_fee.de_minimis_developer_ec_unsettled`,
  `residential_vacancy_fee.discretionary_waiver_unsupported`,
  `residential_vacancy_fee.dwelling_kind_invalid`,
  `residential_vacancy_fee.exemption_invalid`,
  `residential_vacancy_fee.other_ec_waiver_unsupported`,
  `residential_vacancy_fee.ownership_invalid`,
  `residential_vacancy_fee.partial_waiver_unsupported`,
  `residential_vacancy_fee.pre_2024_waiver_boundary_unsettled` and
  `residential_vacancy_fee.waiver_invalid`. `tests/residential_vacancy_fee.rs`
  holds it to the ATO's and Treasury's examples. `derive` takes it.
- `portable-lsl-levy` works out portable long service leave levies for the
  national black coal mining industry scheme (Coal LSL, mirroring
  `tools/coal-lsl-levy` under s 3B of the Payroll Levy Collection Act 1992 and
  s 6 of the Payroll Levy Regulations 2018 at 2.7% from 1 July 2023) and the
  verified Australian state and territory portable LSL schemes across
  construction, community services, contract cleaning and security (NSW LSC,
  Vic LeavePlus and PLSA, Qld QLeave, SA Portable Long Service Leave, WA
  MyLeave, ACT Leave, Tas TasBuild and NT Build), including NSW whole-dollar
  rounding downwards (verified on the LSC Building Levy Calculator) and
  statutory nearest-dollar rounding with 50 cents downwards under Qld s 72(2)
  and NT s 33(2). New codes: `portable_lsl_levy.bonus_invalid`,
  `portable_lsl_levy.branch_field_mismatch`, `portable_lsl_levy.branch_invalid`,
  `portable_lsl_levy.employee_ineligible`,
  `portable_lsl_levy.exemption_unsupported`,
  `portable_lsl_levy.jurisdiction_invalid`,
  `portable_lsl_levy.project_cost_above_tier`,
  `portable_lsl_levy.reimbursements_unconfirmed`,
  `portable_lsl_levy.reporting_month_invalid`,
  `portable_lsl_levy.scheme_invalid`, `portable_lsl_levy.scheme_unavailable`
  and `portable_lsl_levy.scheme_year_unsupported`. `tests/portable_lsl_levy.rs`
  holds Coal LSL to the site parity vectors and checks all 19 state and
  territory schemes. `derive` takes it.
- `non-resident-withholding` works out PAYG withholding on monetary payments
  to foreign residents for 2024-25 to 2026-27 under TAA 1953 Schedule 1
  Subdivision 12-F (dividends, interest and royalties, with franked amounts,
  conduit foreign income, stated treaty rates, TAR 2017 s 26 rounding and the
  s 12-300 ceiling), Subdivision 12-H (withholding MIT fund payments including
  clean building MIT and non-concessional MIT income components for EOI and
  non-EOI recipients) and Subdivision 12-FB / s 12-315 (casino gaming junkets,
  entertainment or sports activities for companies and individuals, and
  construction and related activities, with s 12-319 exemptions, s 15-15
  variations and s 48 prior-withholding reductions). New codes:
  `non_resident_withholding.already_withheld_exceeds_payment`,
  `non_resident_withholding.base_rate_entity_invalid`,
  `non_resident_withholding.construction_contract_before_cutoff`,
  `non_resident_withholding.exempt_exceeds_payment`,
  `non_resident_withholding.mit_parts_exceed_payment`,
  `non_resident_withholding.payee_entity_unsupported`,
  `non_resident_withholding.payment_type_invalid`,
  `non_resident_withholding.treaty_rate_invalid` and
  `non_resident_withholding.variation_rate_invalid`.
  `tests/non_resident_withholding.rs` verifies every statutory rate, rounding
  boundary and refusal code.
- `deferred-revenue` schedules, for 2020-21 to 2026-27, the revenue
  recognised and the contract asset, contract liability and receivable
  balances at each stated period end for one performance obligation over time
  under AASB 15 (daily straight-line, whole-month straight-line or stated
  milestones under paras 39, 40, 105 to 108, B14, B15 and B18), refusing
  variable consideration, significant financing components and changed
  estimates (para 43; AASB 108 paras 36 to 38). New codes:
  `deferred_revenue.amount_invalid`, `consideration_invalid`, `dates_invalid`,
  `estimate_changed`, `financing_component`, `milestones_invalid`,
  `month_boundary`, `pattern_invalid`, `period_ends_invalid` and
  `variable_consideration`. `tests/deferred_revenue.rs` checks D1 to D7 and
  every boundary and refusal.
- `source_quotes`: `motor-vehicle-duty`, `insurance-duty` and
  `vic-property-taxes` now return verbatim primary-source rate, scale and
  grant or tax passages in each result and in `worksheet-catalogue`
  (`data/source-quotes.json`, `tests/source_quotes.rs`).

### Fixed

- `vic-property-taxes` (`commercial_industrial`): ends the 10-year CIPT
  transition period on the entry date's corresponding tenth anniversary under
  Interpretation of Legislation Act 1984 (Vic) ss 44(1) and 44(6)(a), so an
  entry on 28 February 2026 ends on 28 February 2036 (`29 February` clamps to
  `28 February` in a non-leap year).
- `long-service-leave`: narrows the Tasmanian transition refusal under Long
  Service Leave Act 1976 (Tas) Schedule 3 cl 2(4) and old s 8(2)(a) to leave
  taken where 15 years of service were complete before 1 July 2012, and updates
  the `long-service-leave`, `workers-compensation-premium` and `transfer-duty`
  site parity fixtures and evidence notes against the site's `main` at
  `e0cdd7f51`.

## 1.3.0

### Added

- `wip-lockup` ages unbilled WIP and returns WIP, receivable and lock-up
  days from stated figures; lock-up is a management measure, not an
  accounting result.
- `impairment-test` works out value in use, recoverable amount and loss
  allocation under AASB 136 from stated cash flows and a stated pre-tax
  discount rate.
- `payroll-tax-group` works out, from stated facts, one employer's payroll tax
  group for 2026-27 in Victoria, Queensland, Western Australia, South
  Australia or the ACT under each Act's grouping Part (related bodies
  corporate, common employees, common control, tracing and subsumed groups,
  less excluded persons), its designated group employer and the group's single
  deduction, with Queensland's excess deduction shared in order. New South
  Wales, Tasmania and the Northern Territory are refused. New codes:
  `payroll_tax_group.act_unread`, `business_unstated`, `dge_conflict`, `dge_not_member`,
  `dge_wages_below_deduction`, `excess_order_invalid`,
  `excess_unnominated`, `exclusion_barred`, `fact_invalid`, `formula_unread`,
  `interest_cycle`, `jurisdiction_invalid`, `no_dge_unsettled`,
  `not_grouped`, `part_year`, `percent_invalid`, `person_duplicate`,
  `person_unknown` and `test_invalid`. `derive` takes it.
- `charity-check`: whether each ABN is on the 22 November 2026 copy of the ACNC
  Register of Australian charities, with its legal name, registration date and
  ACNC Act s 25-5(5) subtypes, and the limits of one copy. The copy answers for
  a check date up to 31 days after its own and refuses any other, with
  `charity_check.as_at_before_snapshot` and `charity_check.snapshot_stale`.
  `examples/acnc_register.rs` rebuilds it from the dataset's CSV.
- `tourist-refund` tests one traveller's purchases from one registered
  supplier against the tourist refund scheme (GST Act s 168-5(1), GST
  Regulations 2019 Division 168) for departures from 1 November 2025 to
  30 June 2027, and works the refund of GST and of the wine tax borne
  (WET Act s 25-5(1), 29% of half the price). New codes:
  `tourist_refund.date_invalid`, `departure_not_covered`,
  `departure_outside_year`, `fact_invalid`, `gst_exceeds_price_share`,
  `item_invalid`, `threshold_unsettled` and `wine_tax_reduced`.
- `foreign-purchaser-surcharge`: the foreign purchaser duty surcharge on one
  buyer's share in 2026-27 and the foreign or absentee owner land tax
  surcharge for a full land tax year, in each of the 8 states and
  territories, with foreign status taken as established. It mirrors the
  site's foreign surcharge calculator and agrees with it on every case except
  where the Act differs: Queensland's surcharge from $349,999 and Tasmania's
  rounding to 10 cents. New codes:
  `foreign_purchaser_surcharge.absentee_trust_unsupported`,
  `foreign_purchaser_surcharge.dutiable_value_nil`,
  `foreign_purchaser_surcharge.jurisdiction_invalid`,
  `foreign_purchaser_surcharge.owner_invalid` and
  `foreign_purchaser_surcharge.use_invalid`. `derive` takes it.
- `esic` works out an investor's early stage investor tax offset under ITAA
  1997 Subdivision 360-A for 2016-17 to 2026-27: the early stage innovation
  company tests at the time of issue (s 360-40), the 100-point test from the
  stated items (s 360-45), the investor tests and the $50,000 limit (ss 360-15
  and 360-20), 20% of the amount (s 360-25(1)) and the $200,000 cap with
  affiliates and carried forward offsets (s 360-25(2)). New codes:
  `esic.amount_above_income`, `esic.clawback_year_unsettled`,
  `esic.incorporation_after_year`, `esic.innovation_items_conflict`,
  `esic.innovation_items_invalid`, `esic.investor_invalid`,
  `esic.non_cash_before_2018_19`,
  `esic.retail_limit_unsettled`, `esic.six_year_route_before_2018_19`,
  `esic.trust_or_partnership_unsupported` and `esic.year_invalid`.

- `contribution-caps` takes 10 optional downsizer inputs, all or none, and
  tests one downsizer contribution under ITAA 1997 s 292-102: age 55, a
  contract from 1 July 2018, 90 days after the change of ownership and the
  $300,000 and capital proceeds limits, with the other criteria stated.
  `pension-minimum` takes an optional `transition_to_retirement` and adds the
  10% maximum of SISR reg 6.01(2). Without the new inputs both commands answer
  as before. New codes: `downsizer.contribution_before_settlement`,
  `downsizer.date_outside_year` and `downsizer.settlement_before_contract`.
- `super-withholding` takes one superannuation income stream payment with
  `income_stream_period`: Schedule 13 Part A withholding in 2024-25 to
  2026-27 for an account-based stream, or a capped defined benefit stream to
  a payee under 60, paid to a member or a death benefits dependant, with
  `tax_free_threshold_claimed`, `capped_defined_benefit` and
  `deceased_aged_60_or_over`. Lump sums are unchanged. New codes:
  `super_withholding.capped_defined_benefit`,
  `super_withholding.deceased_60_recipient_under_60`,
  `super_withholding.income_stream_foreign_resident`,
  `super_withholding.income_stream_period_invalid` and
  `super_withholding.income_stream_year`. `tests/super_income_stream.rs` runs
  each schedule's Veronica example and each rule.
- `land-tax`: one owner's land tax for a full land tax year in New South
  Wales and Victoria (2026) and Queensland, Western Australia, South
  Australia, Tasmania and the ACT (2026-27), at the general or trust rates,
  with each jurisdiction's foreign or absentee owner surcharge and Western
  Australia's metropolitan region improvement tax. It mirrors the site's land
  tax calculator and agrees with it on every case except where the Act
  differs: Queensland's surcharge over $349,999, South Australia's $20
  minimum and Tasmania's rounding to 10 cents. New codes:
  `land_tax.absentee_trust_unsupported`, `land_tax.jurisdiction_invalid`,
  `land_tax.metropolitan_region_invalid`, `land_tax.metropolitan_region_part`,
  `land_tax.rounding_unsettled`, `land_tax.surcharge_unavailable`,
  `land_tax.surcharge_value_above_taxable_value` and
  `land_tax.trust_rates_unavailable`. `derive` takes it.
- `vic-property-taxes`: Victoria's vacant residential land tax (1%, 2% or
  3% of capital improved value by the land's liable years, 1% for
  undeveloped land and the s 88EB concession), the commercial and
  industrial property tax (nil in 2025 and 2026, with the land's earliest
  possible tax year) and windfall gains tax on a rezoning's aggregated uplift, for the
  2025 and 2026 tax years. New codes:
  `vic_property_taxes.concession_unavailable`,
  `vic_property_taxes.entry_date_before_reform`,
  `vic_property_taxes.parcels_invalid`,
  `vic_property_taxes.residential_land_invalid`,
  `vic_property_taxes.tax_invalid`,
  `vic_property_taxes.transition_end_unsettled` and
  `vic_property_taxes.undeveloped_before_2026`. `derive` takes it.
- `pcg-2021-4` gives the ATO's risk zone under PCG 2021/4 for one IPP's
  allocation of professional firm profits, from stated totals or from up to 6
  recipients with the tax paragraph 85 counts on the year's resident scale or
  company rate, for 2020-21 and 2022-23 to 2026-27. It mirrors the site's PCG
  2021/4 risk scorer and allocation calculator and agrees with them on all 432
  rows of their cases, tests and a seeded sweep. New codes:
  `pcg_2021_4.benchmark_nil`, `pcg_2021_4.entitlement_nil`,
  `pcg_2021_4.gateway_failed`, `pcg_2021_4.recipient_kind_unsupported`,
  `pcg_2021_4.recipients_invalid`, `pcg_2021_4.returned_above_entitlement`
  and `pcg_2021_4.tax_above_entitlement`.
- `asic-fees` gives ASIC's 2026-27 fee for a company registration, name
  reservation, annual review or business name, and the late fee tier for a
  late review fee payment or lodgement from its due and received dates. It
  mirrors the site's ASIC fees page and agrees with it on its 29 cases, the 32
  vectors of its tests and all 181,476 due and received date pairs from
  1 June 2026 to 31 July 2027. It cites company registration to Schedule 2
  item 1(a) of the Corporations (Fees) Regulations 2001. New codes:
  `asic_fees.dates_outside_year`, `asic_fees.fee_unknown` and
  `asic_fees.weekend_undecided`.
- `instant-asset-write-off` works out one asset's instant asset write-off for
  2020-21 to 2026-27: a small business entity's under ITAA 1997 s 328-180 with
  the limit for its dates (IT(TP)A 1997 ss 328-180(4) and (4A) and 328-181(2),
  Act No. 71 of 2026 Sch 2), or a medium sized business's under s 40-82(2A).
  It refuses assets temporary full expensing may take for other entities. New
  codes: `instant_asset_write_off.acquired_after_first_use`,
  `instant_asset_write_off.budget_time_day`,
  `instant_asset_write_off.entity_invalid`,
  `instant_asset_write_off.first_use_outside_year`,
  `instant_asset_write_off.precision_exceeded` and
  `instant_asset_write_off.temporary_full_expensing`.
- `depreciation` also accepts 2020-21 to 2024-25, with up to 366 days held in
  2023-24, and flags temporary full expensing for 2020-21 to 2022-23, backing
  business investment for a 2020-21 start year and the adjusted prime cost
  formula a later year may need. 3 frozen 2023-24 inputs refused as an
  unsupported period now meet their day count or effective life refusals, and
  the catalogue's `days` text, source date and passage change with the years
  (`tests/parity.rs`). 2027-28 stays refused, with a reworded reason.
- `resident-tax` takes a part-year resident: with `part_year_residency_months`
  (1 to 12) and `eligible_pensioner` it pro-rates the tax-free threshold to
  $13,464 plus $4,736 x the months / 12 under Income Tax Rates Act 1986
  s 20(1), or keeps $18,200 for an eligible pensioner (s 18(2)(a)), for
  2024-25 to 2027-28. A near miss of either input name is refused as
  `input.field_unknown`; other unknown fields are still ignored.
- `medicare-levy` takes `non_resident_days`: a part-year resident's days not
  resident on which every dependant was also a prescribed person (ITAA 1936
  s 251U(1)(d) and (2)), each a whole day off the levy under Medicare Levy Act
  1986 s 9(1). `personal-tax-estimate` composes a part-year resident before
  2027-28 with `part_year_residency_months`, `eligible_pensioner` and those
  days, and refuses for one the surcharge, the seniors and pensioners tax
  offset, the private health insurance rebate, a study loan repayment, worked
  tax withheld and every year from 2027-28. Full-year and foreign resident
  results do not change.
- `workers-compensation-premium`: one employer's workers compensation premium
  for a 2026-27 policy in each state and territory, with the adjustment on
  actual wages: icare's small employer formula, the Victorian Premiums Order
  (No. 34) for a new or small employer, WorkCover Queensland's notice for a
  policy commencing in 2026-27 with insurance duty, ReturnToWorkSA's formula,
  and the insurer's rate elsewhere. It mirrors the site's workers
  compensation calculator and agrees with it on every page case, refusing
  where a source does not settle a case: a South Australian base premium
  just under a band's bound, a New South Wales apprentice incentive, a
  Queensland premium that rounding order would move and Queensland actual
  wages. New
  codes: `workers_compensation_premium.adjustment_unsettled`,
  `apprentice_above_wages`, `apprentice_unavailable`,
  `apprentice_unsettled`, `band_edge_unsettled`, `below_minimum`,
  `days_invalid`, `employer_invalid`, `experience_rated`, `group_rated`,
  `jurisdiction_invalid`, `policy_after_year`, `policy_before_year`,
  `rate_invalid`, `remuneration_nil` and `rounding_unsettled`.
- `defined-benefit-income-cap` works out the defined benefit income cap for
  2017-18 to 2026-27 (ITAA 1997 s 303-4), with a part year from `first_day`,
  half the excess as assessable income (s 303-2) and the reduced untaxed
  offset (s 303-3), held to LCR 2017/1 Examples 2 to 6. New code:
  `defined_benefit_income_cap.first_day_outside_year`.
- `foreign-pension-upp` works out the deductible amount of a foreign
  pension's undeducted purchase price under ITAA 1936 s 27H(2), or the ATO's
  2026 D11 shortcuts for UK and Dutch pensions, with the excluded and
  assessable amounts. New codes: `foreign_pension_upp.commuted`,
  `method_invalid`, `method_year_unsupported`, `part_year`,
  `relevant_number_invalid`, `residual_exceeds_price`, `share_invalid` and
  `uk_category_invalid`.
- `scholarship-exemption` tests a scholarship against ITAA 1997 s 51-10 item
  2.1A and s 51-35, held to the ATO's 6 examples. New codes:
  `scholarship_exemption.commonwealth_payment`,
  `commonwealth_payment_invalid`, `friendly_society_plan` and
  `payment_type_invalid`.
- `super-crystallisation` works out a taxed super interest's crystallised
  segment at 30 June 2007 (ITTPA 1997 s 307-225), with the pre-July 83
  component under former ITAA 1936 s 27AA(1)(d). New codes:
  `super_crystallisation.components_exceed_value`, `dates_invalid`,
  `income_stream_before_july_2007` and `untaxed_source`.
- `endorsement-review` works through the ATO's 3 endorsement review
  worksheets for charities and DGRs, each answer tied to its provision,
  complementing `dgr-check`. New codes: `endorsement_review.answers_invalid`
  and `worksheet_invalid`.
- `derive` explains `defined-benefit-income-cap`, `foreign-pension-upp` and
  `super-crystallisation`.
- `motor-vehicle-duty` works out the duty on one passenger vehicle in each
  state and territory on a date in 2026-27 from each Act's scale, with
  Queensland's engine rows, Victoria's green car and primary producer rate,
  the ACT's emissions categories under the determination in force on the day,
  Tasmania's fleet discount and the Northern Territory's electric vehicle
  exemption. New codes: `motor_vehicle_duty.date_outside_year`,
  `motor_vehicle_duty.emissions_category_invalid`,
  `motor_vehicle_duty.engine_invalid`,
  `motor_vehicle_duty.jurisdiction_invalid` and
  `motor_vehicle_duty.vehicle_class_unsupported`.
- `insurance-duty` works out the duty on one general insurance premium in each
  state and territory for a contract effected or renewed in 2026-27, at each
  Act's rate for the class stated: New South Wales Types A, B and C,
  Victoria's general and business insurance, Queensland's class 1, class 2
  and CTP insurance, and general insurance elsewhere, with the ACT's duty and
  New South Wales Type C (for a policy taken out from 1 January 2018) nil.
  `rates.exact_duty` gives the unrounded duty. New codes:
  `insurance_duty.class_invalid`, `insurance_duty.date_outside_year`,
  `insurance_duty.jurisdiction_invalid` and `insurance_duty.term_unsettled`.
- `first-home-owner-grant` gives the first home owner grant in each state
  and territory for a new home bought or built under a contract made from
  1 July 2025 to 22 November 2026, as the site's grant page works it, with
  each First Home Owner Grant Act read on 23 November 2026: the amount on
  the contract date, the value cap (with an optional `cap_value` where
  Queensland and Western Australia test a value other than the
  consideration) and the lesser-of-consideration rule, from eligibility
  facts the operator states. An optional `contract_price` for a home built
  is its consideration: below the amount, the grant is the contract price
  in every state and territory but New South Wales, whose Act could not be
  read, and the Victorian cap tests it. New codes:
  `first_home_owner_grant.below_grant`, `first_home_owner_grant.date_invalid`,
  `first_home_owner_grant.date_uncovered`, `first_home_owner_grant.fact_needed`,
  `first_home_owner_grant.facts_invalid`, `first_home_owner_grant.facts_not_stated`,
  `first_home_owner_grant.home_invalid`, `first_home_owner_grant.jurisdiction_invalid`,
  `first_home_owner_grant.location_needed`, `first_home_owner_grant.value_basis`
  and `first_home_owner_grant.value_invalid`. `tests/first_home_owner_grant.rs`
  holds it to the page's 26 cases.
- `long-service-leave`: one employee's long service leave under the general
  Act of each state and territory, as weeks to take or pay and their value,
  with the hours a `leave-provisions` line takes. It mirrors the site's long
  service leave calculator, except that Queensland uses the 8.6667 weeks the
  Industrial Relations Act 2016 s 95 states and the last day must fall from
  1 July 2020 to 30 June 2027 and within each Act's text as read, and it
  refuses endings the Acts leave to facts the page does not ask (a Queensland
  fixed term before 10 years, a South Australian departure without notice,
  Tasmanian leave taken across the 2012 transition). New codes:
  `long_service_leave.before_current_text`, `end_before_start`,
  `excluded_invalid`, `hours_invalid`, `jurisdiction_invalid`,
  `pay_invalid`, `reason_invalid`, `start_too_early`, `status_invalid`,
  `taken_above_weeks`, `taken_invalid`, `unsettled` and `year_mismatch`.
- `transfer-duty`: transfer duty on one dutiable transaction in 2026-27 in each
  of the 8 states and territories, at the general rate or with an eligible
  first home buyer's concession for an established or new home. It mirrors
  the site's transfer duty calculator and agrees with it on every case except
  Victoria's, which Duties Act 2000 (Vic) s 28(1) charges to the nearest
  dollar. New codes: `transfer_duty.buyer_invalid`,
  `transfer_duty.dutiable_value_nil`,
  `transfer_duty.first_home_not_residential`,
  `transfer_duty.jurisdiction_invalid` and
  `transfer_duty.non_residential_unsupported`. `derive` takes it.
- `work-expenses` reads the records themselves. For `home_office` from
  2022-23, `hours_diary` (dated rows of hours) stands in for `hours`; for
  `car_log_book`, `log_book` (the period, its odometer readings and each
  journey with its purpose) stands in for `log_book_weeks` and
  `business_kilometres`, checks the 12 week period, that it includes part of
  its year and that the journeys fit inside it, and applies its percentage to
  the year's kilometres. Both methods now list the records PCG 2023/1 and
  Division 28 require in `records_to_confirm`, as facts for the operator to
  confirm. Other results are unchanged. New codes:
  `work_expenses.diary_year_unsupported`,
  `work_expenses.log_book_entry_incomplete` and
  `work_expenses.log_book_outside_year`.
- `work-expenses` works out the standard deduction for work-related expenses
  (ITAA 1997 s 25-130, Act No. 49 of 2026) in 2026-27 when given
  `assessable_labour_income`, `other_work_related_deductions` and
  `australian_resident`: the $1,000 limit, what is left beside the covered
  deductions claimed, and what claiming this method adds over relying on
  the standard deduction. Results without these inputs are unchanged. New
  code: `work_expenses.standard_deduction_unavailable`.
- `notice-period` works out the minimum notice of termination under the
  National Employment Standards (Fair Work Act 2009 s 117) for notice given
  from 27 March 2021 to 30 June 2027, with the over-45 week, the s 123
  exclusions as stated, and the payment in lieu, rounded up to the cent,
  from a stated full rate of pay and hours a week. New codes: `notice_period.age_unsettled`,
  `before_current_text`, `birth_after_start`, `excluded_invalid`,
  `exclusion_invalid`, `hours_invalid`, `start_after_notice` and
  `year_mismatch`.
- `trading-stock` works out the change in value of trading stock under ITAA
  1997 s 70-35 for 2020-21 to 2026-27, each closing item valued by its own
  election (cost, market selling value, replacement value, the lowest, or an
  obsolescence value under s 70-50), and the s 328-285 choice for a small
  business entity, or from 2021-22 an entity under $50 million, with the
  $5,000 test and the s 328-295 values. New codes:
  `trading_stock.entity_invalid`, `trading_stock.items_invalid`,
  `trading_stock.method_invalid`, `trading_stock.obsolescence_not_below`,
  `trading_stock.simplified_unavailable` and `trading_stock.value_missing`.
- `prepayments` works out when a prepaid amount incurred in 2020-21 to
  2026-27 is deductible under ITAA 1936 Subdivision H: excluded expenditure,
  the 12-month rule of s 82KZM, or apportionment by days under s 82KZM,
  s 82KZMD or s 82KZMF, with each year's deduction. New codes:
  `prepayments.agreement_before_cutoff`, `prepayments.choice_unavailable`,
  `prepayments.dates_invalid`, `prepayments.exclusion_invalid`,
  `prepayments.tax_shelter_conflict` and `prepayments.taxpayer_invalid`. `tests/trading_stock_prepayments.rs` holds
  both to the ATO's examples.
- `minors-income` works out the tax on a minor's taxable income under
  ITAA 1936 Division 6AA and the Income Tax Rates Act 1986 ss 13 and 15 and
  Schedule 11 for 2024-25 to 2026-27: the prescribed person and excepted
  person tests (s 102AC, the full-time occupation test on stated facts),
  excepted income by its s 102AE(2) ground and trust share parts (s 102AG),
  eligible taxable income with apportionable deductions (s 102AD), and the
  resident and foreign resident special rates with their phase-in caps.
  New codes: `minors_income.excepted_income_invalid`,
  `minors_income.excepted_person_invalid`, `minors_income.facts_inconsistent`,
  `minors_income.residency_invalid`, `minors_income.special_income_component`
  and `minors_income.trust_shares_invalid`. `tests/minors_income.rs` holds it
  to the ATO's under 18 rate tables.
- `foreign-income-conversion` translates foreign currency income,
  deductions, asset costs, CGT amounts and foreign tax paid into AUD under
  ITAA 1997 s 960-50 for 2020-21 to 2026-27, each at the stated rate for the
  day its item names or an average over at most 12 months (item 12 and
  Schedule 2 of the 1997 or 2021 Regulations). New codes:
  `foreign_income_conversion.average_period`, `.basis_invalid`,
  `.currency_invalid`, `.items_invalid`, `.kind_invalid`, `.rate_date` and
  `.text_invalid`. `tests/foreign_income_conversion.rs` holds it to the
  ATO's rental expense and income examples.
- `foreign-income-tax-offset` works out the foreign income tax offset
  (ITAA 1997 Division 770) for 2020-21 to 2026-27 with the $1,000 default
  limit, a limit from stated income tax figures, or, for 2024-25 to 2026-27,
  one worked at the resident rates with a part-year resident's tax-free
  threshold (Income Tax Rates Act 1986 ss 18 and 20). New codes:
  `foreign_income_tax_offset.limit_method_invalid`,
  `.residency_periods_invalid` and `.worked_year`.
  `tests/foreign_income_tax_offset.rs` holds it to Examples 16 and 17 of the
  ATO's foreign income tax offset guides for 2024 to 2026 and its part-year
  threshold examples.
- `smsf-tax` works out a complying SMSF's income tax calculation statement
  (item 13 of the annual return) for 2020-21 to 2026-27: exempt current
  pension income by the segregated method, the actuary's proportion or both
  (ITAA 1997 ss 295-385 and 295-390), the non-arm's length component of
  s 295-545(2A) at 45% and the low tax component at 15%, the 32% extra tax on
  no-TFN-quoted contributions, the offsets in the return's order, the $259
  supervisory levy with the new and wound up fund adjustments, and the amount
  due or refundable. New codes: `smsf_tax.disregarded_small_fund_assets`,
  `smsf_tax.foreign_fund`, `smsf_tax.non_complying`,
  `smsf_tax.part_exceeds_total` and `smsf_tax.proportion_invalid`.
  `tests/smsf_tax.rs` holds it to the ATO's SMSF annual return 2026
  instructions' examples.

- `alcohol-excise` works out the excise duty on beer, spirits and other
  excisable beverages entered for home consumption from 1 July 2022 to 31
  January 2027, line by line at the Commissioner's substituted rate for the
  day (Excise Tariff Act 1921 Schedule items 1 to 3, ss 6A and 6L), with the
  remission for alcohol manufacturers under its $350,000 or $400,000 yearly
  cap and the refund of duty paid where it was available (Excise Regulation
  2015 Sch 1 cl 2(1) item 10 and cl 2(2)). New codes:
  `alcohol_excise.classification_unsettled`,
  `alcohol_excise.date_outside_year`, `alcohol_excise.fact_conflict`,
  `alcohol_excise.line_invalid`, `alcohol_excise.lines_invalid`,
  `alcohol_excise.not_excisable`, `alcohol_excise.product_invalid`,
  `alcohol_excise.product_out_of_scope`, `alcohol_excise.quantity_invalid`
  and `alcohol_excise.rate_unpublished`. `tests/alcohol_excise.rs` holds it
  to the ATO's and the Excise guidelines' examples and every notice's rates.

- `transfer-balance` works one person's transfer balance account from
  1 July 2017 to a day in 2017-18 to 2026-27 from its credits and debits
  (ITAA 1997 ss 294-25 and 294-80): the personal transfer balance cap with
  proportional indexation (s 294-40), each day's excess, excess transfer
  balance earnings at the GIC rate credited daily (s 294-235), and excess
  transfer balance tax at 15% or 30% for each excess transfer balance period
  (Imposition Act 2016 s 5), with IT(TP)A 1997 s 294-30. Reversionary, child,
  capped defined benefit, structured settlement and regulation events and the
  Commissioner's determinations are refused. `excess-contributions` works
  the excess concessional contributions charge for 2013-14 to 2020-21 and the
  release of excess non-concessional contributions with their associated
  earnings for contributions years 2013-14 to 2025-26 (TAA 1953 Sch 1
  ss 97-25 and 97-30). `contribution-caps` takes an optional
  `determination_date` that adds the release figures to its own excess. New
  codes: `excess_contributions.charge_abolished`,
  `excess_contributions.date_order`, `excess_contributions.fact_invalid`,
  `excess_contributions.question_invalid`,
  `excess_contributions.rate_unpublished`, `transfer_balance.date_order`,
  `transfer_balance.event_refused`, `transfer_balance.fact_invalid`,
  `transfer_balance.rate_unpublished`,
  `transfer_balance.transition_undecided` and
  `transfer_balance.year_mismatch`. `tests/transfer_balance.rs` holds them to
  the ATO's examples and to an exact-fraction oracle over the ATO's rates.

- `fbt-return` takes `employer_kind`: a rebatable employer's items 17 to 19
  (FBTAA s 65J, the $30,000 cap and `rebatable_days`), and a public benevolent
  institution's or health promotion charity's ($30,000) or a public hospital's,
  not-for-profit hospital's or public ambulance service's ($17,000) aggregate
  non-exempt amount at items 14C to 19 (s 5B(1D) to (1M), s 57A), with the
  $5,000 salary packaged entertainment cap marked by a benefit's
  `entertainment`; and `instalments_reported` for items 20 to 22. A taxable
  employer's return is unchanged. New codes:
  `fbt_return.employer_kind_unsupported`, `fbt_return.entertainment_invalid`,
  `fbt_return.nfp_exempt_car_unsupported` and
  `fbt_return.rebatable_days_invalid`. `tests/fbt_return_nfp.rs` holds it to
  the ATO's not-for-profit worked examples 1 to 3.

- `fbt-instalment` works out one quarter's FBT instalment for the years ended
  31 March 2026 and 2027: F1 copied to 6A, or a variation on the year's
  estimate (F2) with the relevant fraction (FBTAA ss 109 to 112A) to F3, 6A
  and a 6B credit, with the F4 reason code. New codes:
  `fbt_instalment.ato_instalment_missing`, `fbt_instalment.quarter_invalid`
  and `fbt_instalment.variation_invalid`. `derive` explains its amounts.
  `tests/fbt_instalment.rs` holds it to the ATO's and the employer's guide's
  examples.

- `ess-discount` works out the employee share scheme amounts ITAA 1997
  Division 83A includes for an income year from 2020-21 to 2026-27 for
  interests acquired from 1 July 2015: the discount, the start-up concession,
  deferral to the taxing point (with cessation of employment before 1 July
  2022 and the 30-day rule), the $1,000 reduction and its $180,000 income
  test, the regulations' value of an unlisted right and the s 83A-310
  forfeiture exclusion. New codes: `ess_discount.acquired_before_2015`,
  `ess_discount.dates_inconsistent`, `ess_discount.employment_end_not_stated`,
  `ess_discount.foreign_employment`, `ess_discount.income_test_invalid`,
  `ess_discount.interests_invalid`, `ess_discount.salary_sacrifice_unclear`,
  `ess_discount.taxing_point_value` and `ess_discount.valuation_unavailable`.
  `tests/ess_discount.rs` holds it to the ATO's examples and every table entry.

- `foreign-resident-withholding` works out the foreign resident capital gains
  withholding amount under TAA 1953 Sch 1 s 14-200 for taxable Australian real
  property or a company title interest acquired from 1 July 2017 to 30 June
  2027: 12.5% with the $750,000 market value exclusion before 1 January 2025,
  15% with none from then, the option payment reduction, clearance
  certificates, notices of variation (applied up to their maximum sale price,
  compared with `sale_price`), class variations to nil and s 14-215
  exclusions as stated facts, and each vendor's interest share. New codes:
  `foreign_resident_withholding.asset_unsupported`,
  `foreign_resident_withholding.date_outside_year`,
  `foreign_resident_withholding.exclusion_invalid`,
  `foreign_resident_withholding.option_exceeds_price`,
  `foreign_resident_withholding.payment_unsupported`,
  `foreign_resident_withholding.variation_invalid`,
  `foreign_resident_withholding.variation_maximum_unsettled` and
  `foreign_resident_withholding.vendors_invalid`.
  `tests/foreign_resident_withholding.rs` holds it to the ATO's examples.

- `help-indexation` works out a HELP debt on each 1 June from 2020 to 2026
  under HESA Division 140: the preceding balance, debts, repayments and
  compulsory repayment amounts, indexation at the ATO's published factor (the
  lower of CPI and WPI from 2022-23), the 2025 20% reduction of ss 137-19A
  and 140-26 and the s 140-30 rounding, and a voluntary repayment's effect
  made before 1 June or on or after it. New codes:
  `help_indexation.date_outside_period`, `help_indexation.debt_missing`,
  `help_indexation.entries_invalid`, `help_indexation.location_preferred`,
  `help_indexation.reduction_already_applied` and
  `help_indexation.repayments_exceed_debt`. `tests/help_indexation.rs` holds
  it to the ATO's rates and its 2 reduction examples.

- `payg-instalment` works out one quarterly PAYG instalment for 2025-26 or
  2026-27 under Division 45 of Schedule 1 to the TAA 1953: the T7 amount
  (s 45-112), an amount varied from estimated tax with its credit (ss 45-410
  and 45-420), T1 x T2 (s 45-110) and a varied rate with its credit
  (ss 45-205 and 45-215), mirroring the site's PAYG instalments page. New
  codes: `payg_instalment.credits_before_third`,
  `payg_instalment.credits_exceed_earlier`,
  `payg_instalment.first_has_earlier`,
  `payg_instalment.instalment_number_invalid`,
  `payg_instalment.method_invalid`, `payg_instalment.rate_invalid` and
  `payg_instalment.tax_exceeds_income`. No step rounds before the instalment
  and credit drop their cents, and a varied rate's credit compares it with
  `previous_rate`, the rate used for the previous quarter
  (s 45-215(1)(b)). `tests/payg_instalment.rs` holds it to the ATO's
  examples and the site's cases.

## 1.2.0

- Relicensed Ryan Duguid's code under AGPL-3.0-or-later with section 7(b) attribution terms.

### Added

- `work-expenses` works out an employee's working from home, car or
  self-education deduction for 2020-21 to 2026-27, one `method` a call: the
  fixed rate (52c with a dedicated home office to 2021-22, then PCG 2023/1's
  67c and 70c) and the 80c shortcut against the actual cost; the cents per
  kilometre method at each year's rate with the 5,000 km cap (ITAA 1997
  s 28-25); the log book method with its 12 weeks and 5 years (ss 28-90,
  28-115 and 28-120); and self-education's connection test and, before
  2022-23, the $250 reduction of ITAA 1936 s 82A. Each has its return amount
  with cents disregarded, and `derive` explains it. New codes:
  `work_expenses.allowance_received`, `work_expenses.car_not_held`,
  `work_expenses.fact_conflict`, `work_expenses.fact_invalid`,
  `work_expenses.log_book_expired`, `work_expenses.log_book_too_short`,
  `work_expenses.log_book_unrepresentative`, `work_expenses.method_invalid`
  and `work_expenses.reimbursed`.
  `tests/work_expenses.rs` holds it to the ATO's, PCG 2023/1's and TR 98/9's
  examples.

- `private-health-insurance-rebate` works out the private health insurance
  tax offset or the excess premium reduction to repay for 2020-21 to 2026-27
  from the statement's lines: the tier from income for surcharge purposes, the
  percentage for each period and age band, and each line's share of the PHII
  benefit. `personal-tax-estimate` composes it for a full-year resident. New
  codes: `private_health_insurance_rebate.lhc_loading`,
  `private_health_insurance_rebate.lines_invalid` and
  `private_health_insurance_rebate.rate_unpublished`.
  `tests/private_health_insurance_rebate.rs` holds it to the ATO's examples
  and every published percentage and threshold.
- `super-withholding` gives Schedule 12 withholding from a superannuation lump
  sum or roll-over paid from 13 October 2020 to 30 June 2027: Table A's rates
  on the same parts `super-benefit-tax` works, nil payments, roll-overs above
  the untaxed plan cap and the no-TFN rates. New codes:
  `super_withholding.no_tfn_unstated`, `super_withholding.recipient_invalid`
  and `super_withholding.schedule_not_read`. `tests/super_withholding.rs` runs the
  schedules' Tom, Chloe, Heather and Dean examples.
- `super-benefit-tax` works out one superannuation lump sum's components by
  the proportioning rule (ITAA 1997 s 307-125) and the most income tax, before
  Medicare levy, each part can bear in 2020-21 to 2026-27: a member's by age
  and preservation age with the low rate cap and untaxed plan cap
  (ss 301-10 to 301-115), a death benefit to a dependant (s 302-60) or a
  non-dependant (s 302-145), a terminal medical condition (s 303-10) and a
  lump sum under $200 that closes the interest (s 301-225). `derive` explains
  each amount. New codes: `super_benefit_tax.benefit_nil`,
  `super_benefit_tax.disability_benefit`, `super_benefit_tax.fact_conflict`,
  `super_benefit_tax.pre_july_1983_service` and
  `super_benefit_tax.recipient_invalid`. `tests/super_benefit_tax.rs` runs the
  ATO's Tony, Suzi, Frankie, Peta, Mei, Howard and Anh examples and
  Schedule 12's Heather, Dean and Chloe.

- `entertainer-withholding`, `return-to-work-withholding`,
  `annuity-withholding` and `voluntary-agreement-withholding` work out
  Schedule 3 (actors, variety artists and other entertainers), Schedule 4
  (return to work payments), Schedule 6 (annuities, on `payg-withholding`'s
  Schedule 1 amount) and Schedule 10 (voluntary agreements), each for 2024-25
  to 2026-27. New codes: `annuity_withholding.deductible_above_payment`,
  `annuity_withholding.extra_pay_without_tfn`,
  `annuity_withholding.income_not_whole_cents`,
  `entertainer_withholding.foreign_resident_tfn`,
  `entertainer_withholding.offsets_exceed_withholding`,
  `entertainer_withholding.offsets_several_performances`,
  `entertainer_withholding.tax_offsets_not_allowed`, `input.count_invalid`,
  `input.percentage_invalid`,
  `voluntary_agreement_withholding.abn_not_quoted`,
  `voluntary_agreement_withholding.agreement_not_in_force`,
  `voluntary_agreement_withholding.gst_above_payment` and
  `voluntary_agreement_withholding.instalment_rate_missing`.
  `tests/payment_withholding.rs` holds them to the published examples.
- `horticulture-shearing-withholding`, `upward-variation-withholding`,
  `daily-casual-withholding` and `no-abn-withholding` work out Schedule 2
  (horticulture and shearing), Schedule 14 (an agreed additional amount on
  the Schedule 1 amount), the tax table for daily and casual workers and
  withholding where no ABN is quoted (TAA 1953 Sch 1 s 12-190, Taxation
  Administration Regulations 2017 s 38), each for 2024-25 to 2026-27. New
  codes: `daily_casual_withholding.foreign_resident_tfn`,
  `daily_casual_withholding.offsets_exceed_withholding`,
  `daily_casual_withholding.tax_offsets_not_allowed`,
  `daily_casual_withholding.threshold_not_claimed`,
  `horticulture_shearing_withholding.labour_mobility_scheme`,
  `horticulture_shearing_withholding.threshold_not_claimed`,
  `no_abn_withholding.gst_above_payment`,
  `no_abn_withholding.total_below_payment` and
  `withholding.working_holiday_maker`. `tests/withholding_schedules.rs` holds
  them to the published examples.
- `rd-tax-offset` works out an R&D entity's refundable or non-refundable
  R&D tax offset (ITAA 1997 s 355-100: the corporate tax rate plus 18.5
  percentage points, or plus 8.5% and 16.5% by intensity, with the $20,000
  threshold and the $150 million cap) and the amount s 355-450 includes in
  assessable income for a recoupment or feedstock clawback amount, for
  2021-22 to 2026-27. New codes: `rd_tax_offset.clawback_above_total`,
  `rd_tax_offset.clawback_below_threshold`,
  `rd_tax_offset.feedstock_expenditure_above_total` and
  `rd_tax_offset.threshold_expenditure_above_total`. `tests/rd_tax_offset.rs`
  holds it to the ATO's clawback examples and the schedule's Part E method.
- `gst-going-concern`, `gst-second-hand-goods` and `gst-bad-debt` test
  whether a supply of a going concern is GST-free (s 38-325), work the
  Subdivision 66-A credit for second-hand goods and a Division 21 bad debt
  adjustment reduced under Subdivision 136-A, each for 2020-21 to 2026-27.
  New codes: `gst_bad_debt.accounting_basis_changed`,
  `gst_bad_debt.annual_apportionment`, `gst_bad_debt.dates_invalid`,
  `gst_bad_debt.fact_conflict`, `gst_bad_debt.fact_invalid`,
  `gst_bad_debt.reduced_credit_acquisition`, `gst_bad_debt.special_rule`,
  `gst_going_concern.fact_invalid`, `gst_going_concern.farm_land`,
  `gst_going_concern.part_outside_arrangement`,
  `gst_second_hand_goods.divided_for_resupply`,
  `gst_second_hand_goods.fact_invalid`,
  `gst_second_hand_goods.leased_before_sale` and
  `gst_second_hand_goods.valuable_metal`. `tests/gst_special_rules.rs` holds
  them to GSTR 2002/5, GSTD 2012/6 and GSTR 2000/2.

- `gst-creditable-purpose-change`, `gst-annual-apportionment` and
  `gst-financial-acquisitions-threshold` work a Division 129 adjustment for a
  change in creditable purpose with its adjustment periods, Division 131
  annual apportionment eligibility with the credit and s 131-55 adjustment,
  and the Division 189 financial acquisitions threshold tests, each for
  2020-21 to 2026-27. New codes: `gst_annual_apportionment.car`,
  `gst_annual_apportionment.election_status_unsettled`,
  `gst_annual_apportionment.fact_conflict`,
  `gst_annual_apportionment.not_partly_creditable`,
  `gst_annual_apportionment.reduced_credit_acquisition`,
  `gst_creditable_purpose_change.basis_missing`,
  `gst_creditable_purpose_change.last_period_unsettled`,
  `gst_creditable_purpose_change.period_end_invalid`,
  `gst_creditable_purpose_change.tax_period_unsupported` and
  `gst_financial_acquisitions_threshold.fact_conflict`.
  `tests/gst_adjustments.rs` holds them to the examples of GSTR 2000/24, the
  GST Act and GSTR 2003/9.

- `deferred-tax` works out a company's tax effect worksheet under AASB 112
  for 2020-21 to 2026-27: temporary differences from carrying amounts and
  tax bases, the deferred tax liability and asset at 25% or 30% (paras 15,
  24, 34 and 47; Income Tax Rates Act 1986 s 23(2)), the movement split
  into rate change, recognition change and origination (para 80), current
  tax at the year's rate (26% for a base rate entity in 2020-21) and the
  para 81(c)(i) reconciliation with the average effective tax rate. New
  codes: `deferred_tax.business_combination`,
  `deferred_tax.expected_rate_invalid`, `deferred_tax.item_invalid`,
  `deferred_tax.offsets_above_tax`, `deferred_tax.opening_rate_unscheduled`,
  `deferred_tax.outside_profit_or_loss`,
  `deferred_tax.rate_change_unscheduled`,
  `deferred_tax.tax_consolidated_group`,
  `deferred_tax.uncertain_tax_treatment`,
  `deferred_tax.unrecognised_above_deductible` and
  `deferred_tax.unrecognised_used_above_opening`.
- `cgt-cost-base-adjustments` applies returns of capital (CGT event G1, ITAA
  1997 s 104-135), a non-AMIT trust's non-assessable parts (CGT event E4,
  ss 104-70 and 104-71), AMIT cost base net amounts (ss 104-107A to 104-107C,
  CGT event E10) and the Coles (2018) and Endeavour (2021) demergers
  (Subdivision 125-B, CR 2018/59 and CR 2021/50) to `cgt-parcels`' parcels,
  for 2018-19 to 2026-27, and returns the gains and the adjusted parcels. New
  codes: `cgt_cost_base.below_reduced_cost_base`,
  `cgt_cost_base.company_indexation`, `cgt_cost_base.demerger_unknown`,
  `cgt_cost_base.event_year_unsupported` and `cgt_cost_base.pre_cgt`.
  `derive` covers it. `tests/cgt_cost_base_adjustments.rs` holds it to the
  ATO's examples. `cgt-parcels` is unchanged.
- `depreciation-low-value-pool`, `depreciation-small-business-pool` and
  `capital-works` work out a low-value pool (ITAA 1997 ss 40-440 and
  40-445), a general small business pool with each year's low pool value
  threshold (ss 328-190 to 328-215, IT(TP)A 1997 ss 328-180(6) and
  328-181(5), Act No. 71 of 2026) and a Division 43 capital works deduction
  (ss 43-210 and 43-215), each for 2020-21 to 2026-27. New codes:
  `capital_works.begun_before_16_september_1987`,
  `capital_works.four_percent_before_27_february_1992`,
  `capital_works.precision_exceeded`,
  `capital_works.undeducted_above_expenditure` and
  `depreciation_small_business_pool.backing_business_investment`.
  `tests/depreciation_pools.rs` holds them to the ATO's examples. The
  single-asset `depreciation` command is unchanged.
- `rental-property`: one residential rental property's income and deductions
  for one owner and income year from 2020-21 to 2026-27, by the lines of the
  ATO's rental property worksheet at the owner's legal interest, apportioned
  by the days let or available for rent; travel denied (s 26-31); interest on
  a mixed-use loan (TR 2000/2); borrowing expenses through
  `borrowing-expenses-schedule`; capital works at 2.5% or 4% within the
  undeducted construction expenditure (Division 43); decline in value nil
  where s 40-27 reduces it; the net rent and, for 2024-25 to 2026-27, its
  effect on tax at the resident scale. New codes:
  `rental_property.asset_acquired_9_may_2017`,
  `rental_property.days_unclear`,
  `rental_property.fact_conflict`, `rental_property.fact_invalid`,
  `rental_property.interest_redraw`, `rental_property.s26_50_denied`,
  `rental_property.tax_effect_year` and
  `rental_property.works_intended_use`. `tests/rental_property.rs` holds it to
  the ATO's Rental properties guide 2026 examples.

- `beneficiary-tax-offset`, `invalid-carer-tax-offset`, `zone-tax-offset` and
  `net-medical-expenses-tax-offset` work out the beneficiary tax offset
  (ITAA 1936 s 160AAA) and the zone and overseas forces tax offsets
  (ss 79A and 79B) for 2020-21 to 2026-27, the invalid and invalid carer tax
  offset (ITAA 1997 Subdivision 61-A) for 2020-21 to 2025-26 and the net
  medical expenses tax offset for 2016-17 to 2018-19, its last years. New
  codes: `invalid_carer_tax_offset.dependant_invalid`, `.ftb_shared_care`,
  `.rate_unpublished`, `.shared_maintenance` and `.successive_spouses`;
  `zone_tax_offset.area_invalid`,
  `.dependants_invalid`, `.half_year_unclear`, `.prior_year_invalid`,
  `.prior_year_unclear`, `.shared_maintenance` and
  `.sole_parent_without_dependant`. `tests/individual_offsets.rs` holds them
  to the ATO's examples and every published amount and limit.

- `government-super-amounts` works out the super co-contribution, the low
  income super tax offset and the spouse contribution tax offset for a
  full-year resident in 2020-21 to 2026-27, from `income-tests`' components,
  the contributions and the stated tests, with each condition and its
  provision. New codes: `government_super_amounts.fact_conflict`,
  `government_super_amounts.fact_not_supplied`,
  `government_super_amounts.no_total_income` and
  `government_super_amounts.temporary_visa`.
  `tests/government_super_amounts.rs` holds it to the ATO's Angelo, Julie
  and spouse examples.

- `small-business-income-offset` works out an individual's small business
  income tax offset (ITAA 1997 Subdivision 328-F) for 2020-21 to 2026-27: the
  small business entity test of s 328-110 at $5 million (s 328-357), total net
  small business income from your own and each partnership or trust amount
  net of its s 328-370 deductions, capped at taxable income, the 13% (2020-21)
  or 16% share of the basic income tax liability, the $1,000 cap and the part
  the liability left after other offsets absorbs. New codes:
  `small_business_income_offset.business_invalid`,
  `small_business_income_offset.prescribed_person` and
  `small_business_income_offset.shares_invalid`.
  `tests/small_business_income_offset.rs` holds it to the site's sole trader
  vectors.

- `non-commercial-losses` says whether ITAA 1997 Division 35 defers an
  individual's business activity loss for 2020-21 to 2026-27: the income
  requirement, the assessable income, profits, real property and other assets
  tests, the primary production and professional arts exception, and the
  excess deferred after net exempt income, with the Commissioner's discretion
  reported as not decided. New codes:
  `non_commercial_losses.activity_invalid` and
  `non_commercial_losses.earlier_years_invalid`.
  `tests/non_commercial_losses.rs` holds it to the ATO's examples.

- `psi-test` says whether the personal services income rules (ITAA 1997
  Part 2-42) apply to an individual's PSI for 2020-21 to 2026-27: s 84-5,
  s 85-35, a determination in force, the results test, the 80% rule and the
  unrelated clients, employment and business premises tests, step for step as
  the site's PSI page. New codes: `psi_test.agent_payments_invalid`,
  `psi_test.agent_rules`, `psi_test.earner_invalid`, `psi_test.no_psi`,
  `psi_test.others_exceed_psi`, `psi_test.payers_invalid` and
  `psi_test.premises_invalid`. `tests/psi_test.rs` runs the page's 19 cases.

- `ato-interest-penalties` works the general interest charge (TAA 1953
  ss 8AAC and 8AAD), the shortfall interest charge (Sch 1 s 280-105), the
  failure to lodge on time penalty (s 286-80), the false or misleading
  statement and not reasonably arguable penalties (ss 284-85 to 284-225) and
  the interest the Commissioner pays on early payments and overpayments, each
  before remission, on the ATO's published rates and penalty units read on
  22 November 2026. Its GIC agrees with the site's GIC payment plan oracle.
  New codes: `ato_interest_penalties.date_order`,
  `ato_interest_penalties.fact_invalid`,
  `ato_interest_penalties.increase_with_nil`,
  `ato_interest_penalties.item_not_worked`,
  `ato_interest_penalties.liability_not_listed`,
  `ato_interest_penalties.not_entitled`, `ato_interest_penalties.overpaid`,
  `ato_interest_penalties.penalty_unit_unpublished`,
  `ato_interest_penalties.question_invalid`,
  `ato_interest_penalties.rate_unpublished`,
  `ato_interest_penalties.remission_discretionary`,
  `ato_interest_penalties.scheme_penalty`,
  `ato_interest_penalties.sge_before_2017` and
  `ato_interest_penalties.year_mismatch`.
- `residency-tests` says whether an individual was a resident of Australia
  for an income year from 2020-21 to 2026-27 under each test in ITAA 1936
  s 6(1), resident (a): resides, domicile, 183 days (counted from the dates
  in Australia, 184 in 2023-24) and the Commonwealth superannuation fund
  test. Residing, domicile, places of abode and intention are the
  operator's answers. New codes: `residency_tests.presence_invalid`,
  `residency_tests.super_invalid` and `residency_tests.treaty_tie_breaker`.
  `tests/residency_tests.rs` holds it to TR 2023/1's examples.
- `lodgment-required` says whether an individual must lodge their own
  income tax return for 2020-21 to 2025-26 under that year's lodgment
  instrument (the Notices for 2020-21 to 2022-23, the Taxation Laws
  instruments from 2023-24): s 6(2), 6(3) and 6(10), the exceptions in
  s 7(1)(f), 7(2) and 7(3), and the 31 October due date. New codes:
  `lodgment_required.child_support_parent`,
  `lodgment_required.group_invalid`,
  `lodgment_required.instrument_not_made`,
  `lodgment_required.residency_invalid` and
  `lodgment_required.triggers_invalid`. `tests/lodgment_required.rs` holds it
  to each year's thresholds.

### Fixed

- `fuel-tax-credits` with `rate_at_period_end` (PCG 2016/2 Method 2) takes each
  line at the rate for its acquisition date when every applicable rate stays
  flat across the period. It still takes the period's last day when
  indexation changes an applicable rate, and still refuses a period across
  any other rate change.

- Both borrowing expenses worksheets identify their deduction amounts as
  section 25-25 calculations before the vacant land and residential limits.
  The worksheets flag section 26-102 from 2019-20 and section 26-155 from
  2027-28. The amounts and JSON fields are unchanged.
- JSON worksheet amounts and ratios now use the same plain decimal notation as
  the MCP calculator. Use `"1000.00"` instead of `"1e3"` or `"1_000"`, and
  `"0.50"` instead of `".50"`. A leading minus sign and surrounding whitespace
  still work where the worksheet allows the value.
- The frozen Python 0.2.0 replay now refuses these 57 zero-based entries with
  `decimal.InvalidOperation`. The replay also checks each entry with its
  amount written in plain decimal notation.
  - `capital-gains`: 2497, 2553, 2763, 2903, 3467, 3593, 3635, 3719, 3817,
    3873, 3929, 4001, 4169, 4309.
  - `contribution-caps`: 2829, 3055, 3153, 3279, 3293, 3307, 3335, 3349,
    3645, 3701, 3869, 3925, 4347.
  - `fbt`: 2456, 2498, 2708, 2946, 3424, 3902, 4408.
  - `pension-minimum`: 2746, 2928, 3014, 3168, 3280, 3308, 4446.
  - `quarterly-sg`: 2696, 3104, 3216, 3328, 3638, 4130.
  - `division-293`: 2969, 3923, 4037.
  - `gst`: 27, 3253, 3577.
  - `payg-withholding`: 517, 4201.
  - `depreciation`: 3173. `study-loan-withholding`: 3288.

- `gst-margin-scheme` checks the valuation date against the registration facts
  for item 2 or the taxable supply date for item 4 of GST Act s 75-10(3).
  Supply those facts with the affected valuations. The command refers a
  possible overlap of items 2 and 3 for separate review.

- `ppsr-timing` refuses the skipped spring hour and repeated autumn hour in ACT
  local inputs and the derived six-month time. The refusal code is
  `ppsr_timing.time_unresolved`.

## 1.1.0

### Added

- `franking-account` keeps a corporate tax entity's franking account for
  2025-26 or 2026-27 under ITAA 1997 Division 205: credits and debits by
  table item, the R&D deferral (s 205-15(4)), refunds after the year's end
  (s 205-50(2)), franking deficit tax and its offset with the 30% reduction
  of the attributable part and the s 205-70(5) and (6) exceptions, the
  benchmark rule's over-franking tax and debits per franking period, the
  significant variation test (s 204-70), the franking return's labels and
  its due date. New codes: `franking_account.distribution_invalid`,
  `franking_account.event_invalid`, `franking_account.facts_conflict`,
  `franking_account.income_tax_invalid`,
  `franking_account.income_tax_needed`,
  `franking_account.last_benchmark_invalid` and
  `franking_account.rate_invalid`. `tests/franking_account.rs` holds it to
  the ATO's Franking account tax return 2026 examples.
- `cgt-parcels` works out the capital gains and losses from selling shares
  or units held in parcels bought at different times, for contracts in
  2025-26 or 2026-27: each sale matched first in, first out or to the parcels
  the taxpayer selects (TD 33), each part's cost base and reduced cost base
  (ITAA 1997 ss 110-25 and 110-55), its gain or loss (s 104-10) and whether it
  is a discount capital gain (s 115-25), then the year's net capital gain or
  loss with other gains and losses in the order of s 102-5(1) and the
  discount percentage by entity (s 115-100). It lists each part and the units
  left in each parcel. It refuses parcels acquired at or before 21 September
  1999 that are sold in the year. New codes: `cgt_parcels.entity_invalid`,
  `cgt_parcels.fact_conflict`, `cgt_parcels.fact_invalid`,
  `cgt_parcels.fact_missing`, `cgt_parcels.identification_invalid`,
  `cgt_parcels.indexation_or_pre_cgt`, `cgt_parcels.list_invalid` and
  `cgt_parcels.units_not_held`. `derive` covers it. `tests/cgt_parcels.rs`
  holds it to the ATO's Kathleen and Boris examples and the Library's David
  Jones example.
- `ppsr-timing` answers what the Personal Property Securities Register's law
  decides without a search, for 2025-26 and 2026-27: a registration's latest
  end time, the end time s 153(2) and (3) apply and its fee (PPSA s 153
  item 5; Personal Property Securities (Fees) Determination 2018 s 4); a
  purchase money security interest's registration deadline (PPSA s 62(2)(b)
  and (3)(b)) and whether a registration meets it; and a company grantor's
  20 business day time and, with a critical time, whether the interest vests
  (Corporations Act s 588FL). Business days follow PPSA s 10 and Corporations
  Act s 9 over the holidays the caller lists. New codes:
  `ppsr_timing.date_outside_year`, `ppsr_timing.date_unsettled`,
  `ppsr_timing.days_invalid`, `ppsr_timing.fact_invalid`,
  `ppsr_timing.foreign_law_unsupported`, `ppsr_timing.holidays_incomplete`,
  `ppsr_timing.law_unread` and `ppsr_timing.time_invalid`.
  `tests/ppsr_timing.rs` holds it to each fee tier and constructed calendar
  cases.
- `payroll-tax`: one employer's payroll tax for 2026-27 in each state and
  territory, for an employer outside a group that pays wages all year. It
  shares each deduction or threshold by the taxable wages over the Australian
  wages, phases out the Victorian, Queensland, Western Australian and Northern
  Territory deductions, applies South Australia's phased rate and the
  Tasmanian, ACT and Northern Territory rate steps, and adds Victoria's 2
  surcharges and Queensland's mental health levy as their own steps. Each
  figure cites its Act and revenue office page, read on 19 November 2026. A
  group member, a part-year employer, Australian wages below the taxable wages
  and any other year are refused. `derive` takes it.
- `borrowing-expenses-schedule` works out borrowing expenses under ITAA 1997
  s 25-25 for several loans and every income year of each, for loans first
  drawn from 1 July 2017: the $100 test across the loans incurred in a year
  (s 25-25(6)), use that changes by year (s 25-25(2) and (3)), refinancing as
  the repayment of one listed loan by another (s 25-25(5)(b)) and the totals
  by year. Rows from 2027-28 are flagged for s 26-155 and rows after 2027-28
  marked projected. New codes: `borrowing.income_use_invalid`,
  `borrowing.loan_before_2017_18`, `borrowing.loan_fact_missing`,
  `borrowing.loans_invalid`, `borrowing.other_expenses_invalid` and
  `borrowing.refinance_invalid`. `derive` covers it.
  `tests/borrowing_expenses_schedule.rs` holds it to the ATO's Peter and
  Fiona and Max, the Library's Ben and `borrowing-expenses` row by row.

- `luxury-car-tax` works out the luxury car tax on a tax period's sales and
  importations in 2019-20 to 2026-27 at 33% of the GST exclusive excess over
  the year's threshold (LCT Act s 5-15), from the ATO's thresholds for
  2012-13 to 2026-27, with the 3.5 litre fuel-efficient rating from 1 July
  2025 and its 7 litre carry-over (Act No. 29 of 2025 Sch 1), price and
  change of use adjustments, BAS labels 1E and 1F rounded down, and the
  primary producer and tourism operator refunds of up to $10,000 apart from
  the BAS. New codes: `luxury_car_tax.consumption_invalid`,
  `luxury_car_tax.kind_invalid`, `luxury_car_tax.line_invalid`,
  `luxury_car_tax.lines_invalid`, `luxury_car_tax.refund_invalid`,
  `luxury_car_tax.refund_limit` and `luxury_car_tax.year_invalid`.
  `tests/luxury_car_tax.rs` holds it to the ATO's QC22107 and QC22108
  examples, every published threshold year and hand-worked cases.
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
- `fuel-tax-credits` works out the fuel tax credits for business fuel
  acquired from 1 July 2022 to 31 January 2027, line by line at the rate the
  ATO publishes for the day of acquisition (Fuel Tax Act 2006 ss 41-5, 41-20,
  43-5 and 43-10), from `data/fuel-tax-credit-rates.json`: 16 periods, each
  with its ATO page and the day read. It totals the lines for label 7D in
  whole dollars and, for a small claimant, offers the basic method for heavy
  vehicles (PCG 2021/2) and PCG 2016/2 Methods 1 (litres from the cost and the
  average price) and 2 (the rate on the period's last day across an
  indexation change only). `return_date` flags lines s 47-5(2) may have
  ended. New codes: `fuel_tax_credits.date_outside_year`,
  `fuel_tax_credits.fact_conflict`, `fuel_tax_credits.fuel_invalid`,
  `fuel_tax_credits.lines_invalid`, `fuel_tax_credits.method_invalid`,
  `fuel_tax_credits.period_invalid`, `fuel_tax_credits.quantity_invalid`,
  `fuel_tax_credits.rate_unpublished`,
  `fuel_tax_credits.simplified_method_unavailable` and
  `fuel_tax_credits.use_invalid`. `tests/fuel_tax_credits.rs` holds it to
  every period's liquid fuel rates, the ATO's Jim and PCG 2021/2's We Deliver
  examples (in the module's unit tests) and hand-worked cases.
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
- `fbt-ev-exemption-check` decides whether a car's benefits in the FBT year
  ended 31 March 2026 or 2027 are exempt under FBTAA s 8A, the electric cars
  exemption, and names each failing condition: a car, a current employee, a
  battery electric or hydrogen fuel cell electric vehicle (or a plug-in
  hybrid under the Electric Car Discount Act 2022 Sch 2 item 5(2): committed
  to before 1 April 2025, no new commitment since, and exempt before then),
  first both held and used on or after 1 July 2022 (Sch 1 item 7(2)), and no
  luxury car tax before the benefit (s 8A(1)(c)): a value at a first retail
  sale within or before the FBT year not above the ATO's fuel-efficient
  threshold for its financial year, 2012-13 to 2026-27 (LCT Act s 25-1), and
  none on any other supply. A sale after the FBT year meets the condition; a
  taxed sale within it, after the car was first held and used, is refused. It gives
  `electric_car_exempt` for the 2 car worksheets and the reportable fringe
  benefit consequence (s 135P(3)). The year ended 31 March 2028 is refused:
  the change announced for 1 April 2027 is not law. New codes:
  `fbt_ev_exemption.fact_conflict`, `fbt_ev_exemption.sale_within_year`,
  `fbt_ev_exemption.threshold_year_unsupported` and
  `fbt_ev_exemption.vehicle_invalid`. `tests/fbt_ev_exemption.rs` runs the
  ATO's Shelly, John, GHI Co, Sonia, Simon, Tony, Courtney and pool car
  examples, the Library's Cleancut example and each failing condition.
- `dasp-tax` works out the tax on a departing Australia superannuation
  payment in 2025-26 to 2027-28 by component: nil on the tax free component,
  35% on the element taxed in the fund and 45% on the element untaxed in the
  fund, or 65% on both where the payment includes contributions made whilst a
  working holiday maker (Superannuation (Departing Australia Superannuation
  Payments Tax) Act 2007 s 5(1) and (3)). `derive` explains each amount. New
  codes: `dasp_tax.payment_nil` and `dasp_tax.unclaimed_money_rollover`, for
  an s 5(2) roll-over with an element untaxed in the fund. `tests/dasp_tax.rs`
  runs the Library's $42,000 example and the working holiday maker rate.
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

- `sb-cgt-concessions` shows which small business CGT concessions in ITAA
  1997 Division 152 are open for a capital gain from CGT event A1 in 2025-26
  or 2026-27: the basic conditions of s 152-10 (the CGT small business entity
  test of s 328-110 at $2 million, the $6 million net asset value test, the
  passively held asset and partnership rules and the active asset test), then
  the 15-year exemption, the 50% active asset reduction, the retirement
  exemption and the roll-over, each with the conditions not met and their
  provisions. New codes: `sb_cgt_concessions.active_period_unclear`,
  `sb_cgt_concessions.entity_invalid`, `sb_cgt_concessions.fact_conflict`,
  `sb_cgt_concessions.fact_invalid`,
  `sb_cgt_concessions.fifteen_years_same_day`,
  `sb_cgt_concessions.pre_cgt_asset` and
  `sb_cgt_concessions.share_or_trust_interest`. `tests/sb_cgt_concessions.rs`
  holds it to the Library's Neil Swan and Wendy Simpson examples.

### Changed

- `borrowing-expenses` takes 2017-18 to 2027-28, not 2024-25 to 2026-27:
  s 25-25 sets no figure by year and has its present text from 1 April 2017.
  From 2027-28 it works the year out and warns that s 26-155 may quarantine
  it, instead of refusing the year. A loan first drawn on 29 February is
  worked out, its 5 years ending on 28 February (Acts Interpretation Act 1901
  s 2G(2)); `borrowing.loan_start_29_february` is withdrawn.

- `hire-purchase-schedule`'s `instalment` and `instalments` are required only
  without `segments`, and `hire_purchase.instalments_unsupported` and
  `hire_purchase.rate_unsupported` name the frequency's limits.

### Fixed

- The FBT estimate and return use the full statutory gross-up formulas in
  FBTAA s 5B(1B) and (1C). The return rounds items 14A and 14B to whole
  dollars before calculating item 16. Displayed rates keep the published
  four-decimal figures.
- The capital gains worksheet cites the CGT method statement that applies in
  2025-26 and 2026-27. Borrowing expenses refusals describe the leisure
  facility exceptions in ITAA 1997 s 26-50. The CLI usage lists the CSV
  contract register command.
- The car parking worksheet requires contributions capped per benefit when an
  unallocated payment could reduce other benefits. The meal entertainment
  register needs positive costs to yield a percentage. A reimbursed associate's
  benefit type stays undetermined until their relationship is established.
- The business capital deduction states that it adds exact item amounts before
  rounding the total to cents.
- For a capped hire purchase car, a held invoice with separate, reconciled
  goods and finance amounts can establish a GST credit. The car limit applies
  to the goods credit. The worksheet leaves attribution and the split GST
  journals for review.
- The study loan repayment worksheet cites the ATO rate page's current title
  and update date. It uses the 2026-27 threshold in table 1. Example 3 still
  quotes the 2025-26 threshold. Repayment amounts are unchanged.
- `worksheet_catalogue()`'s evidence note for the 2 car worksheets, the 3
  Schedule 5 methods, working holiday maker and seniors and pensioners
  withholding, and the seniors and pensioners tax offset now opens their
  heading in `docs/calculation-evidence.md`; before, it named an anchor the
  document does not have.
- `contribution-caps` cites a first year of bring forward for the
  non-concessional cap only when the contributions are over the general cap
  (ITAA 1997 s 292-85(3)(a)). At or under it, the step says that twice or
  3 times the general cap applies only if bring forward is triggered. No
  amount changes.

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
