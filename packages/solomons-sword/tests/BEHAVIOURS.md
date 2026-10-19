# Python behaviour coverage

Base 1c2a17d, Python 0.2.0 (its engine and command code are unchanged in 0.3.2): 90 test functions. The archive pytest suite
passed before its observed API and CLI inputs were frozen (package metadata tests
deselected). Rust replays their Python results plus generated cases from
`tests/data/`. Decimal values and scale, complete result objects, exit codes and
output bytes are compared. API results over 64 KiB and CLI streams over 4 KiB
are stored as an FNV-1a fingerprint and length.

| Original test | Rust coverage |
| --- | --- |
| `test_boolean_facts.py::test_resolution_refuses_non_boolean_facts` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_boolean_facts.py::test_risk_assessment_refuses_non_boolean_facts` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_boolean_facts.py::test_allocation_refuses_non_boolean_facts` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_boolean_facts.py::test_foreign_receipt_refuses_non_boolean_residency` | Frozen replay in `parity.rs`: `section_99b_exemptions_caveats_and_range_checks` |
| `test_cli.py::test_s100a_check_reports_the_risk_zone_and_the_factors` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_100a_zoning_and_unestablished_facts`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_a_narrow_console_codepage_does_not_end_the_run` | Native `a_narrow_console_codepage_does_not_end_the_run`; encoding overrides are a documented divergence (output is always UTF-8) |
| `test_cli.py::test_s100a_check_takes_the_stated_receipt_from_the_other_flag` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_100a_zoning_and_unestablished_facts`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_s100a_check_reports_the_facts_it_was_not_given` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_100a_zoning_and_unestablished_facts`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_s99b_check_deducts_the_corpus_exemption` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_99b_exemptions_caveats_and_range_checks`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_the_two_receipt_flags_cannot_both_be_given` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_decimal_type_refuses_what_is_not_a_decimal` | Frozen replay in `parity.rs`: `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_decimal_type_refuses_the_non_finite_amounts` | Frozen replay in `parity.rs`: `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_an_s100a_amount_too_large_to_print_is_one_error_line` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_100a_zoning_and_unestablished_facts`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_an_s99b_add_back_that_cancels_the_corpus_is_still_range_checked` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_99b_exemptions_caveats_and_range_checks`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_a_cancelling_add_back_at_the_supported_limit_still_evaluates` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_99b_exemptions_caveats_and_range_checks`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_amounts_whose_arithmetic_overflows_are_one_error_line` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_99b_exemptions_caveats_and_range_checks`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_existing_validation_still_answers_first_for_astronomical_amounts` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_100a_zoning_and_unestablished_facts`, `section_99b_exemptions_caveats_and_range_checks`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_amounts_above_one_trillion_still_evaluate` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_100a_zoning_and_unestablished_facts`, `section_99b_exemptions_caveats_and_range_checks`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_decimal_type_keeps_the_exact_decimal` | Frozen replay in `parity.rs`: `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_a_money_argument_that_is_not_a_decimal_stops_the_run` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_a_refused_amount_is_reported_as_an_error_line_not_a_traceback` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_100a_zoning_and_unestablished_facts`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_no_subcommand_prints_the_help_and_fails` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes` |
| `test_cli.py::test_s99b_check_refuses_a_run_that_does_not_state_residency` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_99b_exemptions_caveats_and_range_checks`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_s99b_check_names_the_nil_exemptions_it_was_given` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_99b_exemptions_caveats_and_range_checks`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_cli.py::test_streams_that_cannot_be_reconfigured_still_receive_the_report` | Frozen replay in `parity.rs`: `original_cli_exit_stdout_stderr_bytes`, `section_100a_zoning_and_unestablished_facts`, `decimal_type_refuses_what_is_not_a_finite_decimal` |
| `test_fastcheck_bridge.py::test_the_same_seed_repeats_the_same_cases` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_discarded_inputs_do_not_reduce_the_accepted_budget` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_a_failure_is_shrunk_and_its_replay_repeats_the_counterexample` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_a_python_error_is_reported_as_a_failure` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_an_unknown_property_fails_closed` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_an_invalid_budget_is_refused` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_a_missing_installation_fails_closed` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_a_stale_installation_fails_closed` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_a_missing_module_fails_closed` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_fastcheck_bridge.py::test_a_hung_worker_is_killed_and_reaped` | Retired: tests the removed Python fast-check bridge, not engine behaviour |
| `test_louisgoldberg.py::test_division6_proportionate_approach` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_franking_credits_reconcile_without_changing_taxable_income` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_negative_franking_credit_pool_is_refused` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_section100a_risk_zones` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_section99b_corpus_exemption` | Frozen replay in `parity.rs`: `section_99b_exemptions_caveats_and_range_checks` |
| `test_louisgoldberg.py::test_trust_resolution_validation` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_louisgoldberg.py::test_section100a_does_not_default_to_green` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_an_unstated_fact_cannot_reach_the_green_zone` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_a_corporate_upe_without_a_loan_is_outside_green_not_red` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_parental_retention_alone_is_outside_green_not_red` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_a_green_scenario_needs_every_condition_its_paragraph_names` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_an_established_red_zone_trigger_stands_while_other_facts_are_unstated` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_section100a_rejects_non_positive_or_non_finite_distributions` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_division6_rejects_percentages_that_do_not_total_100` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_trust_resolution_rejects_zero_percent_and_missing_deed_facts` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_louisgoldberg.py::test_streamed_amounts_are_refused_not_misallocated` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_shares_reconcile_to_the_net_income` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_franking_credits_are_not_added_on_top_of_the_net_income_share` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_unmodelled_cases_fail_closed` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_minor_beneficiary_is_assessed_to_the_trustee` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_s99b_corpus_proviso_and_residency` | Frozen replay in `parity.rs`: `section_99b_exemptions_caveats_and_range_checks` |
| `test_louisgoldberg.py::test_green_zone_does_not_claim_the_ordinary_family_dealing_exception` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_entitlements_that_do_not_reconcile_exactly_are_refused` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_equal_fixed_entitlements_that_exhaust_the_income_are_allocated` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_rounding_residual_moves_only_sub_cent_dust` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_section95_loss_is_not_allocated_to_beneficiaries` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_section95_rounding_never_creates_negative_taxable_shares` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_resolution_before_the_income_year_started_is_refused` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_louisgoldberg.py::test_funds_not_received_is_recorded_as_a_risk_factor` | Frozen replay in `parity.rs`: `section_100a_zoning_and_unestablished_facts` |
| `test_louisgoldberg.py::test_cent_exact_mixed_entitlements_are_allocated` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_residency_is_a_required_input_not_a_default` | Native `residency_is_a_required_input_not_a_default` (an omitted JSON fact is unstated and refused) |
| `test_louisgoldberg.py::test_an_unestablished_legal_disability_is_refused_not_read_as_an_adult` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_an_unestablished_resolution_fact_is_neither_valid_nor_a_breach` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_louisgoldberg.py::test_s99b_names_every_nil_exemption_in_one_caveat` | Frozen replay in `parity.rs`: `section_99b_exemptions_caveats_and_range_checks` |
| `test_louisgoldberg.py::test_s99b_residency_is_a_required_input_not_a_default` | Frozen replay in `parity.rs`: `section_99b_exemptions_caveats_and_range_checks` |
| `test_louisgoldberg.py::test_s99b_names_an_omitted_corpus_add_back_in_the_caveat` | Frozen replay in `parity.rs`: `section_99b_exemptions_caveats_and_range_checks` |
| `test_louisgoldberg.py::test_an_ordinary_resolution_needs_no_streaming_powers` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_louisgoldberg.py::test_missing_streaming_powers_is_a_breach_only_when_the_resolution_streams` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_louisgoldberg.py::test_a_resolution_after_the_deeds_earlier_deadline_is_late` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_louisgoldberg.py::test_an_unread_deed_deadline_is_not_established` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_louisgoldberg.py::test_unstreamed_division_6e_amounts_are_refused` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_non_finite_division_6e_amounts_are_refused_as_value_errors` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_a_beneficiary_with_streamed_amounts_is_refused` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_a_fixed_entitlement_must_be_positive` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_a_beneficiary_needs_a_percentage_or_a_fixed_entitlement` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_the_last_share_in_order_absorbs_the_rounding_residual` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_louisgoldberg.py::test_an_unsigned_resolution_is_reported` | Frozen replay in `parity.rs`: `trust_resolution_deadlines_and_unestablished_facts` |
| `test_package_identity.py::test_published_distribution_uses_the_project_identity` | Retired: Python distribution metadata; `tests/test_rust_engine.py` at the root checks the Cargo identity |
| `test_package_identity.py::test_source_tree_without_installation_reports_a_development_version` | Retired: Python distribution metadata; `tests/test_rust_engine.py` at the root checks the Cargo identity |
| `test_properties.py::test_three_near_equal_percentage_shares_foot_to_the_trust_income` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_properties.py::test_a_fixed_entitlement_is_reported_as_supplied_and_percentages_absorb_the_cent` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_properties.py::test_all_fixed_entitlements_are_reported_as_supplied` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_properties.py::test_a_sub_cent_fixed_entitlement_is_refused_rather_than_rounded` | Frozen replay in `parity.rs`: `division_6_allocation_refusals_rounding_and_footing` |
| `test_properties.py::test_every_allocated_column_foots_to_its_pool` | Native seeded property `every_allocated_column_foots_to_its_pool` (500 accepted cases, same domain) plus frozen replay of the fast-check cases |
| `test_properties.py::test_percentages_helper_splits_exactly_one_hundred` | Native `percentages_helper_splits_exactly_one_hundred` |
