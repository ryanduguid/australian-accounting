# Python behaviour coverage

Base 83288b0, Python 0.2.0 (unchanged since the `australian-tax-calculators/v0.2.0`
tag): 109 test functions. The archive pytest suite passed before the
differential harness captured the API calls they made. `tests/data/python-api.json`
freezes the first 60 distinct calls of each test, all their parametrised cases
together, plus 2,008 generated cases: 3,221 calls in all.
`tests/parity.rs` replays them through the JSON boundary and compares complete
results, including key order, decimal scale and refusal messages.

Tests whose cases exceeded 60 calls, and the table checks that made no API call,
run in full as native tests in `tests/properties.rs` and `tests/tables.rs`. The
property tests keep their seeds, run counts, explicit examples and input domains;
a local SplitMix64 replaces fast-check's generator, so the random cases differ.

| Original test | Rust coverage |
| --- | --- |
| `test_calculations.py::test_gst_inclusive_and_exclusive_reconcile` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_resident_brackets` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_every_supported_resident_period_has_its_own_contiguous_scale` | `tables.rs`: `every_supported_resident_period_has_its_own_contiguous_scale` |
| `test_calculations.py::test_resident_result_reports_the_scale_it_applied` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_losses_precede_discount_and_excess_carries_forward` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_fbt_type_one_and_two` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_fbt_return_items_round_to_the_nearest_dollar_before_adding` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_fbt_results_cite_the_return_instructions` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_fbt_retains_gross_up_precision_until_final_presentation` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_first_year_depreciation` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_depreciation_cannot_exceed_cost` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_quarterly_sg_maximum_and_amount_still_needed` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_bad_amounts_rejected` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_scope_must_be_established` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_unknown_periods_and_non_boolean_flags_fail` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_depreciation_rejects_invalid_scope_inputs` | Frozen replay in `parity.rs` |
| `test_calculations.py::test_worksheets_do_not_inherit_the_callers_decimal_precision` | Frozen replay in `parity.rs` |
| `test_catalogue.py::test_every_advertised_period_and_method_runs` | Frozen replay in `parity.rs` |
| `test_catalogue.py::test_unsupported_period_and_unconfirmed_scope_remain_refused` | Frozen replay in `parity.rs` |
| `test_catalogue.py::test_fbt_has_its_own_year_boundary` | Frozen replay in `parity.rs` |
| `test_catalogue.py::test_caller_cannot_mutate_the_engine_through_discovery` | Frozen replay in `parity.rs` |
| `test_catalogue.py::test_result_date_belongs_to_the_selected_rule` | Frozen replay in `parity.rs` |
| `test_catalogue.py::test_discovery_fields_a_client_reads` | Frozen replay in `parity.rs` |
| `test_catalogue.py::test_income_year_and_quarter_periods` | Frozen replay in `parity.rs` |
| `test_division_293.py::test_ato_example_jan` | Frozen replay in `parity.rs` |
| `test_division_293.py::test_lesser_of_contributions_and_excess` | Frozen replay in `parity.rs` |
| `test_division_293.py::test_each_supported_year_uses_the_250000_threshold` | Frozen replay in `parity.rs` |
| `test_division_293.py::test_tax_never_exceeds_15_percent_of_contributions` | Frozen replay in `parity.rs` (first 60 calls); `properties.rs`: `division_293_tax_never_exceeds_15_percent_of_contributions` |
| `test_division_293.py::test_negative_and_sub_cent_amounts_are_refused` | Frozen replay in `parity.rs` |
| `test_fastcheck_bridge.py::test_the_same_seed_repeats_the_same_cases` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_discarded_inputs_do_not_reduce_the_accepted_budget` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_failure_is_shrunk_and_its_replay_repeats_the_counterexample` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_python_error_is_reported_as_a_failure` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_an_unknown_property_fails_closed` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_an_invalid_budget_is_refused` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_missing_installation_is_bootstrapped_before_the_property` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_failed_bootstrap_fails_closed` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_stale_installation_fails_closed` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_missing_module_fails_closed` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_hung_worker_is_killed_and_reaped` | Removed with the fast-check bridge; the properties it ran are seeded Rust tests in `properties.rs` |
| `test_library_evidence.py::test_independently_derived_library_results` | Frozen replay in `parity.rs` |
| `test_low_income_tax_offset.py::test_ato_example_jeff` | Frozen replay in `parity.rs` |
| `test_low_income_tax_offset.py::test_table_items_and_boundaries` | Frozen replay in `parity.rs` |
| `test_low_income_tax_offset.py::test_second_step_amount_and_cut_out_follow_from_the_table` | `tables.rs`: `lito_second_step_amount_and_cut_out_follow_from_the_table` |
| `test_low_income_tax_offset.py::test_each_supported_year_applies_the_same_table` | Frozen replay in `parity.rs` |
| `test_low_income_tax_offset.py::test_offset_never_rises_with_income` | Frozen replay in `parity.rs` (first 60 calls); `properties.rs`: `low_income_tax_offset_never_rises_with_income` |
| `test_low_income_tax_offset.py::test_cents_are_refused` | Frozen replay in `parity.rs` |
| `test_medicare_levy.py::test_ato_example_angie` | Frozen replay in `parity.rs` |
| `test_medicare_levy.py::test_threshold_and_phase_in_boundaries` | Frozen replay in `parity.rs` |
| `test_medicare_levy.py::test_reduction_is_the_difference_from_the_full_rate` | Frozen replay in `parity.rs` |
| `test_medicare_levy.py::test_thresholds_match_the_published_table` | `tables.rs`: `medicare_thresholds_match_the_published_table_and_phase_in` |
| `test_medicare_levy.py::test_phase_in_limit_is_where_ten_percent_of_the_excess_meets_two_percent` | `tables.rs`: `medicare_thresholds_match_the_published_table_and_phase_in` |
| `test_medicare_levy.py::test_levy_never_falls_as_income_rises` | Frozen replay in `parity.rs` (first 60 calls); `properties.rs`: `medicare_levy_never_falls_as_income_rises` |
| `test_medicare_levy.py::test_unpublished_year_and_cents_are_refused` | Frozen replay in `parity.rs` |
| `test_medicare_levy.py::test_result_names_the_rates_applied` | Frozen replay in `parity.rs` |
| `test_medicare_levy.py::test_section_9a_lump_sum_in_arrears_is_refused` | Frozen replay in `parity.rs` |
| `test_medicare_levy.py::test_without_section_9a_the_standard_levy_applies` | Frozen replay in `parity.rs` |
| `test_medicare_levy.py::test_section_9a_flag_is_required_and_boolean` | Frozen replay in `parity.rs` |
| `test_payg_withholding.py::test_the_sample_is_complete` | `tables.rs`: `every_scale_matches_the_ato_sample` |
| `test_payg_withholding.py::test_every_scale_matches_the_ato_sample` | Frozen replay in `parity.rs` (first 60 calls); `tables.rs`: `every_scale_matches_the_ato_sample` |
| `test_payg_withholding.py::test_weekly_earnings_ignore_cents_then_add_99_cents` | Frozen replay in `parity.rs` |
| `test_payg_withholding.py::test_a_monthly_amount_ending_in_33_cents_gains_a_cent` | Frozen replay in `parity.rs` |
| `test_payg_withholding.py::test_the_result_names_the_coefficients_used` | Frozen replay in `parity.rs` |
| `test_payg_withholding.py::test_unsupported_scales_are_refused` | Frozen replay in `parity.rs` |
| `test_payg_withholding.py::test_unsupported_pay_periods_are_refused` | Frozen replay in `parity.rs` |
| `test_payg_withholding.py::test_invalid_earnings_are_refused` | Frozen replay in `parity.rs` |
| `test_payg_withholding.py::test_a_year_without_its_own_coefficients_is_refused` | Frozen replay in `parity.rs` |
| `test_properties.py::test_gst_and_the_exclusive_amount_add_to_the_inclusive_amount_to_the_cent` | Frozen replay of its first 60 calls in `parity.rs`; full run in `properties.rs`: `gst_and_the_exclusive_amount_add_to_the_inclusive_amount_to_the_cent` |
| `test_properties.py::test_resident_tax_never_falls_and_its_marginal_rate_never_falls` | Frozen replay of its first 60 calls in `parity.rs`; full run in `properties.rs`: `resident_tax_never_falls_and_its_marginal_rate_never_falls` |
| `test_properties.py::test_payg_withholding_never_falls_as_earnings_rise` | Frozen replay of its first 60 calls in `parity.rs`; full run in `properties.rs`: `payg_withholding_never_falls_as_earnings_rise` |
| `test_properties.py::test_payg_withholding_never_falls_across_any_bracket_boundary` | Frozen replay of its first 60 calls in `parity.rs`; full run in `properties.rs`: `payg_withholding_never_falls_across_any_bracket_boundary` |
| `test_properties.py::test_study_loan_repayment_never_falls_and_never_exceeds_a_tenth` | Frozen replay of its first 60 calls in `parity.rs`; full run in `properties.rs`: `study_loan_repayment_never_falls_and_never_exceeds_a_tenth` |
| `test_properties.py::test_pension_minimum_never_falls_with_balance_or_age` | Frozen replay of its first 60 calls in `parity.rs`; full run in `properties.rs`: `pension_minimum_never_falls_with_balance_or_age` |
| `test_properties.py::test_capital_losses_are_all_accounted_for_and_more_losses_never_raise_the_gain` | Frozen replay of its first 60 calls in `parity.rs`; full run in `properties.rs`: `capital_losses_are_all_accounted_for_and_more_losses_never_raise_the_gain` |
| `test_properties.py::test_cap_room_and_excess_reconcile_to_the_cap_available` | Frozen replay of its first 60 calls in `parity.rs`; full run in `properties.rs`: `cap_room_and_excess_reconcile_to_the_cap_available` |
| `test_result_contract.py::test_every_worksheet_has_a_contract_case` | `parity.rs`: `python_api_corpus_matches` requires a case for every command |
| `test_result_contract.py::test_result_envelope` | Frozen replay in `parity.rs` |
| `test_result_contract.py::test_result_amount_keys` | Frozen replay in `parity.rs` |
| `test_result_contract.py::test_refusal_messages` | Frozen replay in `parity.rs` |
| `test_result_contract.py::test_input_bounds_are_inclusive` | Frozen replay in `parity.rs` |
| `test_result_contract.py::test_values_just_outside_the_bounds_are_refused` | Frozen replay in `parity.rs` |
| `test_result_contract.py::test_fbt_return_items_round_half_up_to_the_dollar` | Frozen replay in `parity.rs` |
| `test_result_contract.py::test_pension_starting_in_june_needs_no_payment_before_rounding` | Frozen replay in `parity.rs` |
| `test_result_contract.py::test_study_loan_marginal_amount_is_nil_below_the_minimum` | Frozen replay in `parity.rs` |
| `test_study_loan.py::test_ato_worked_examples_for_2026_27` | Frozen replay in `parity.rs` |
| `test_study_loan.py::test_ato_example_2_differs_by_the_rounded_base_it_prints` | Frozen replay in `parity.rs` |
| `test_study_loan.py::test_every_band_boundary_matches_the_published_table` | Frozen replay in `parity.rs` |
| `test_study_loan.py::test_the_published_crossover_is_derived_from_the_exact_base` | `tables.rs`: `study_loan_crossover_is_derived_from_the_exact_base` |
| `test_study_loan.py::test_amounts_and_facts_travel_with_the_result` | Frozen replay in `parity.rs` |
| `test_study_loan.py::test_refusals` | Frozen replay in `parity.rs` |
| `test_study_loan_withholding.py::test_ato_worked_examples` | Frozen replay in `parity.rs` |
| `test_study_loan_withholding.py::test_example_1_shows_the_coefficients_and_the_total` | Frozen replay in `parity.rs` |
| `test_study_loan_withholding.py::test_the_schedule_1_amount_is_the_payg_worksheet_amount` | Frozen replay in `parity.rs` |
| `test_study_loan_withholding.py::test_published_starting_earnings_select_the_first_component_row` | Frozen replay in `parity.rs` |
| `test_study_loan_withholding.py::test_components_agree_with_the_combined_coefficients_to_a_dollar` | Frozen replay in `parity.rs` (first 60 calls); `properties.rs`: `study_loan_components_agree_with_the_combined_coefficients_to_a_dollar` |
| `test_study_loan_withholding.py::test_combined_rows_are_schedule_1_plus_the_component_rows` | `tables.rs`: `combined_rows_are_schedule_1_plus_the_component_rows` |
| `test_study_loan_withholding.py::test_every_component_year_has_schedule_1_coefficients` | `tables.rs`: `every_component_year_has_schedule_1_coefficients` |
| `test_study_loan_withholding.py::test_unsupported_scales_are_refused` | Frozen replay in `parity.rs` |
| `test_study_loan_withholding.py::test_unsupported_pay_periods_are_refused` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_first_year_cap_matches_the_published_table_at_every_boundary` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_75_or_older_all_year_keeps_only_the_annual_cap` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_bring_forward_triggers_only_above_the_annual_cap` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_nil_cap_at_the_transfer_balance_cap_makes_every_contribution_excess` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_carry_forward_needs_a_balance_strictly_below_500000` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_caps_refuse_a_non_boolean_age_condition` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_pension_factor_by_age` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_pension_rounds_to_the_nearest_10_dollars` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_pension_is_pro_rated_from_commencement_and_nil_from_1_june` | Frozen replay in `parity.rs` |
| `test_super_worksheets.py::test_pension_refuses_impossible_age_or_days` | Frozen replay in `parity.rs` |
