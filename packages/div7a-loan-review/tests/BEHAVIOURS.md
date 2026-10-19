# Python behaviour coverage

Base 83288b0, Python 0.3.2 (unchanged since the `div7a-loan-review/v0.3.2` tag):
286 test functions. The archive pytest suite passed before the differential
harness captured the API calls and command runs they made, with the files each
named. `tests/data/python-cases.json` freezes the first 60 distinct cases of each
test plus generated API calls and command runs: 3,557 cases in all.
`tests/parity.rs` replays them and compares complete results, key order and
output bytes included. API results over 4 KiB and command streams over 4 KiB are
stored as an FNV-1a fingerprint and length.

Tests whose checks made no engine call, or more than 60, run natively in the
unit tests and in `tests/properties.rs`, `tests/evaluation.rs` and
`tests/docs.rs`. The property tests keep their seeds, run counts and input
domains; a local SplitMix64 replaces fast-check's generator, so the random cases
differ.

| Original test | Rust coverage |
| --- | --- |
| `test_cli.py::test_json_output_carries_no_floats` | Frozen replay in `parity.rs` |
| `test_cli.py::test_no_amount_is_written_as_a_json_number` | Frozen replay in `parity.rs` |
| `test_cli.py::test_amounts_are_strings_with_two_decimal_places` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_emitter_refuses_a_float_it_is_handed` | Not applicable: the Rust JSON writer has no float value to refuse |
| `test_cli.py::test_a_clean_register_exits_zero` | Frozen replay in `parity.rs` |
| `test_cli.py::test_text_output_exposes_the_annual_interest_limitation` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_register_needing_attention_exits_two` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_known_rate_exits_zero` | Frozen replay in `parity.rs` |
| `test_cli.py::test_an_unreviewed_rate_year_exits_two` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_missing_file_exits_one_without_a_traceback` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_register_that_is_not_utf8_exits_one_with_one_error_line` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_register_the_csv_reader_refuses_exits_one_with_one_error_line` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_malformed_year_exits_one` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_year_whose_halves_disagree_exits_one` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_text_output_leads_with_the_disclaimer` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_json_envelope_names_the_compilation_it_was_written_against` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_rate_command_reports_the_may_figure_and_its_provenance` | Frozen replay in `parity.rs` |
| `test_cli.py::test_an_unknown_rate_gives_a_reason_not_a_number` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_gate_command_reports_no_repayment_figures` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_myr_command_reports_no_gate_block` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_review_command_reports_both` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_trace_flag_prints_the_statutory_trace` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_short_row_labels_its_exposure_as_a_review_aid` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_rates_override_flag_reaches_the_rate_command` | Frozen replay in `parity.rs` |
| `test_cli.py::test_an_override_without_a_citation_exits_one` | Frozen replay in `parity.rs` |
| `test_cli.py::test_every_json_command_names_the_rate_table_it_used` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_manifest_digest_matches_the_shipped_table` | Frozen replay in `parity.rs` |
| `test_cli.py::test_each_myr_line_carries_its_own_loan_id` | Frozen replay in `parity.rs` |
| `test_cli.py::test_stdout_is_switched_to_utf8_where_it_can_be` | Not applicable: output is always UTF-8 with backslash escapes (documented divergence) |
| `test_cli.py::test_a_row_without_a_borrower_reference_is_labelled_by_its_loan_id` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_myr_command_prints_no_gate_block_in_text` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_gate_run_labels_the_report_with_the_first_known_loan_year` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_gate_run_with_no_known_loan_year_uses_the_table_coverage` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_trace_prints_whichever_result_a_report_carries` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_review_applies_the_rates_override_to_every_row` | Frozen replay in `parity.rs` |
| `test_cli.py::test_an_operating_system_error_exits_one_with_one_error_line` | Not replayable: it monkeypatched the review to raise OSError; read errors are corpus cases |
| `test_docs.py::test_the_first_screen_says_it_is_not_a_determination` | Ported to `docs.rs` |
| `test_docs.py::test_the_first_screen_states_the_v1_scope_and_its_gaps` | Ported to `docs.rs` |
| `test_docs.py::test_the_first_screen_warns_that_a_rising_benchmark_raises_the_repayment` | Ported to `docs.rs` |
| `test_docs.py::test_the_first_screen_links_the_no_install_explainer` | Ported to `docs.rs` |
| `test_docs.py::test_the_banner_is_a_t_account_with_dr_and_cr_sides` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_rate_table_matches_the_frozen_table` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_explains_the_may_versus_june_trap` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_names_the_rba_series` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_review_date_matches_the_table_header` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_documents_the_rounding_mode_the_engine_uses` | Ported to `docs.rs` |
| `test_docs.py::test_every_register_column_is_documented` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_documents_no_column_the_code_does_not_read` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_warns_about_the_maximum_term_years_name` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_lists_each_refusal` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_refuses_to_predict_an_assessment` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_avoids_the_names_this_engine_must_not_go_by` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_records_the_mcp_adapter_boundary` | Ported to `docs.rs` |
| `test_docs.py::test_the_disclaimer_covers_the_required_ground` | Ported to `docs.rs` |
| `test_docs.py::test_the_disclaimer_preserves_the_professional_boundary` | Ported to `docs.rs` |
| `test_docs.py::test_the_disclaimer_disclaims_the_ca_anz_endorsement` | Ported to `docs.rs` |
| `test_docs.py::test_the_disclaimer_states_the_factual_limits` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_carries_the_authorship_statement` | Ported to `docs.rs` |
| `test_docs.py::test_security_records_the_no_network_position` | Ported to `docs.rs` |
| `test_docs.py::test_the_citation_and_pyproject_agree_on_the_version` | Ported to `docs.rs` |
| `test_docs.py::test_current_release_metadata_points_to_the_canonical_monorepo` | Ported to `docs.rs` |
| `test_docs.py::test_the_package_records_the_compilation_it_was_written_against` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_links_the_ato_calculator_comparison` | Ported to `docs.rs` |
| `test_docs.py::test_the_ato_calculator_example_is_this_engines_arithmetic` | Ported to `docs.rs` |
| `test_docs.py::test_the_rates_review_is_linked_and_reads_the_rba_table` | Ported to `docs.rs` |
| `test_docs.py::test_the_readme_links_the_source_review` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_names_the_compilation_it_read` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_records_which_sections_were_read` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_records_how_the_formula_was_recovered` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_records_the_reviewed_position_on_109N_1_b` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_states_the_rounding_position` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_rate_table_matches_the_frozen_table` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_records_the_tables_shelf_life` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_says_what_it_does_not_establish` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_records_the_blocked_sources_honestly` | Ported to `docs.rs` |
| `test_docs.py::test_the_source_review_disclaims_ca_anz_endorsement` | Ported to `docs.rs` |
| `test_evaluation_pack.py::test_every_fixture_exists` | Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_gate_verdict_matches` | Frozen replay in `parity.rs`; Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_myr_verdict_matches` | Frozen replay in `parity.rs`; Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_amounts_match_to_the_cent` | Frozen replay in `parity.rs`; Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_exit_code_matches` | Frozen replay in `parity.rs`; Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_the_inputs_the_readme_hand_works_are_the_inputs_used` | Frozen replay in `parity.rs`; Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_the_hand_working_reproduces_the_engines_answer` | Frozen replay in `parity.rs`; Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_the_readme_prints_every_hand_worked_figure` | Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_the_pack_names_its_sources_and_review_date` | Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_the_pack_covers_a_met_a_short_a_refusal_and_an_unknown` | Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_at_least_three_fixtures_are_hand_worked` | Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_evaluation_pack.py::test_no_fixture_carries_anything_that_looks_like_a_client_record` | Ported to `evaluation.rs`: `every_scenario_reproduces_its_pinned_result` and the pack checks |
| `test_fastcheck_bridge.py::test_the_same_seed_repeats_the_same_cases` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_discarded_inputs_do_not_reduce_the_accepted_budget` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_failure_is_shrunk_and_its_replay_repeats_the_counterexample` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_python_error_is_reported_as_a_failure` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_an_unknown_property_fails_closed` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_an_invalid_budget_is_refused` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_missing_installation_is_bootstrapped_before_the_property` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_failed_bootstrap_fails_closed` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_stale_installation_fails_closed` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_missing_module_fails_closed` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_fastcheck_bridge.py::test_a_hung_worker_is_killed_and_reaped` | Removed with the fast-check bridge; the properties are seeded Rust tests in `properties.rs` |
| `test_gate.py::test_seven_year_unsecured_at_exactly_the_benchmark_is_complying` | Frozen replay in `parity.rs` |
| `test_gate.py::test_seven_year_unsecured_one_basis_point_under_is_not_complying` | Frozen replay in `parity.rs` |
| `test_gate.py::test_twenty_five_year_loan_without_a_registered_mortgage_is_not_complying` | Frozen replay in `parity.rs` |
| `test_gate.py::test_twenty_five_year_loan_with_mortgage_but_109_per_cent_cover_is_not_complying` | Frozen replay in `parity.rs` |
| `test_gate.py::test_twenty_five_year_loan_with_mortgage_and_110_per_cent_cover_is_complying` | Frozen replay in `parity.rs` |
| `test_gate.py::test_the_110_per_cent_boundary_is_inclusive` | Frozen replay in `parity.rs` |
| `test_gate.py::test_a_twenty_five_year_loan_carries_the_refinancing_caveat` | Frozen replay in `parity.rs` |
| `test_gate.py::test_an_unestablished_fact_gives_unknown_never_complying` | Frozen replay in `parity.rs` |
| `test_gate.py::test_written_agreement_unknown_does_not_become_false` | Frozen replay in `parity.rs` |
| `test_gate.py::test_written_agreement_false_is_not_complying` | Frozen replay in `parity.rs` |
| `test_gate.py::test_terms_not_in_place_before_lodgment_day_is_not_complying` | Frozen replay in `parity.rs` |
| `test_gate.py::test_a_failed_limb_beats_an_unknown_one` | Frozen replay in `parity.rs` |
| `test_gate.py::test_unknown_security_does_not_make_a_short_term_unknown` | Frozen replay in `parity.rs` |
| `test_gate.py::test_unknown_security_does_make_a_long_term_unknown` | Frozen replay in `parity.rs` |
| `test_gate.py::test_unknown_coverage_with_a_long_term_is_unknown` | Frozen replay in `parity.rs` |
| `test_gate.py::test_an_unknown_benchmark_year_makes_the_rate_limb_unknown` | Frozen replay in `parity.rs` |
| `test_gate.py::test_no_year_at_all_is_unknown` | Frozen replay in `parity.rs` |
| `test_gate.py::test_the_floor_year_defaults_to_the_year_the_loan_was_made` | Frozen replay in `parity.rs` |
| `test_gate.py::test_each_selected_year_result_exposes_the_unresolved_annual_requirement` | Frozen replay in `parity.rs` |
| `test_gate.py::test_testing_a_later_year_carries_a_caveat_naming_the_divergence` | Frozen replay in `parity.rs` |
| `test_gate.py::test_complying_always_carries_the_out_of_scope_caveat` | Frozen replay in `parity.rs` |
| `test_gate.py::test_from_mapping_reads_a_csv_row` | Frozen replay in `parity.rs` |
| `test_gate.py::test_a_blank_cell_is_unknown_not_false` | `src/gate.rs`: `mapped_facts_keep_unknown_apart_from_false` |
| `test_gate.py::test_an_unrecognised_boolean_is_an_error_not_a_false` | `src/gate.rs`: `mapped_facts_keep_unknown_apart_from_false` |
| `test_gate.py::test_a_negative_interest_rate_is_refused` | `src/gate.rs`: `mapped_facts_keep_unknown_apart_from_false` |
| `test_gate.py::test_a_nan_rate_is_refused` | `src/gate.rs`: `mapped_facts_keep_unknown_apart_from_false` |
| `test_money.py::test_parse_money_rejects_values_outside_domain` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_money_refuses_every_grouped_or_bracketed_form` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_money_accepts_maximum_to_cents` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_money_accepts_an_exact_cent_amount_written_with_trailing_zeros` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_money_still_refuses_a_real_sub_cent_amount` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_money_refuses_an_empty_field` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_money_refuses_a_negative_amount` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_ratio_refuses_a_negative_ratio` | Frozen replay in `parity.rs` |
| `test_money.py::test_an_unusable_term_is_a_fact_error` | Frozen replay in `parity.rs` |
| `test_myr.py::test_bare_formula_refuses_fractional_term` | Frozen replay in `parity.rs` |
| `test_myr.py::test_formula_regression` | Frozen replay in `parity.rs` |
| `test_myr.py::test_formula_agrees_with_exact_rational_arithmetic` | Frozen replay in `parity.rs` |
| `test_myr.py::test_formula_agrees_with_an_independent_implementation` | Frozen replay in `parity.rs` |
| `test_myr.py::test_formula_matches_exact_arithmetic_everywhere` | Frozen replay in `parity.rs` (first 60 calls); `properties.rs`: `formula_matches_exact_arithmetic_everywhere` |
| `test_myr.py::test_paying_the_figure_every_year_clears_the_loan_to_rounding` | Frozen replay in `parity.rs` (first 60 calls); `properties.rs`: `paying_the_figure_every_year_clears_the_loan_to_rounding` |
| `test_myr.py::test_a_longer_term_never_raises_the_figure` | Frozen replay in `parity.rs` (first 60 calls); `properties.rs`: `a_longer_term_never_raises_the_figure` |
| `test_myr.py::test_the_figure_covers_interest_and_a_straight_line_share` | Frozen replay in `parity.rs` (first 60 calls); `properties.rs`: `the_figure_covers_interest_and_a_straight_line_share` |
| `test_myr.py::test_a_single_remaining_year_is_principal_plus_one_year_of_interest` | Frozen replay in `parity.rs`; `properties.rs`: `regression_points_hold` |
| `test_myr.py::test_an_astronomical_term_reaches_the_interest_only_limit_at_once` | Frozen replay in `parity.rs`; `properties.rs`: `regression_points_hold` |
| `test_myr.py::test_the_result_is_a_decimal_quantised_to_cents` | Frozen replay in `parity.rs` |
| `test_myr.py::test_the_formula_is_refused_at_a_nil_rate` | Frozen replay in `parity.rs` |
| `test_myr.py::test_the_formula_is_refused_at_a_nil_term` | Frozen replay in `parity.rs` |
| `test_myr.py::test_remaining_term_rounds_up_per_the_closing_words_of_109E_6` | Frozen replay in `parity.rs` |
| `test_myr.py::test_repayment_met_exactly_to_the_cent` | Frozen replay in `parity.rs` |
| `test_myr.py::test_repayment_short_by_one_dollar` | Frozen replay in `parity.rs` |
| `test_myr.py::test_repayment_short_by_one_cent_is_still_short` | Frozen replay in `parity.rs` |
| `test_myr.py::test_an_overpayment_does_not_produce_a_negative_shortfall` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_nil_remaining_term_is_refused_without_dividing_by_zero` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_negative_remaining_term_is_refused` | Frozen replay in `parity.rs` |
| `test_myr.py::test_the_year_of_the_loan_is_refused` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_year_before_the_loan_was_made_is_refused` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_non_complying_gate_refuses_the_repayment_figure` | Frozen replay in `parity.rs` |
| `test_myr.py::test_an_unknown_gate_refuses_the_repayment_figure` | Frozen replay in `parity.rs` |
| `test_myr.py::test_no_gate_result_at_all_is_refused` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_missing_unpaid_balance_is_unknown` | Frozen replay in `parity.rs` |
| `test_myr.py::test_missing_payments_are_unknown_not_nil` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_missing_remaining_term_is_unknown` | Frozen replay in `parity.rs` |
| `test_myr.py::test_an_unreviewed_rate_year_is_unknown` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_refusal_outranks_an_unknown` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_fractional_remaining_term_is_rounded_up_and_noted` | Frozen replay in `parity.rs` |
| `test_myr.py::test_every_result_carries_the_genuine_repayment_caveat` | Frozen replay in `parity.rs` |
| `test_myr.py::test_every_result_says_it_is_not_a_determination` | Frozen replay in `parity.rs` |
| `test_myr.py::test_the_statutory_trace_shows_the_values_plugged_in` | Frozen replay in `parity.rs` |
| `test_myr.py::test_the_benchmark_provenance_reaches_the_result` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_missing_year_loan_made_is_noted_rather_than_assumed` | Frozen replay in `parity.rs` |
| `test_myr.py::test_every_money_field_is_a_decimal` | Frozen replay in `parity.rs` |
| `test_myr.py::test_gate_caveats_survive_every_result` | Frozen replay in `parity.rs` |
| `test_myr.py::test_gate_from_another_benchmark_year_cannot_authorise_myr` | Frozen replay in `parity.rs` |
| `test_myr.py::test_missing_loan_year_cannot_produce_myr` | Frozen replay in `parity.rs` |
| `test_myr.py::test_every_way_of_not_getting_a_figure_carries_its_code` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_figure_carries_no_reason_codes` | Frozen replay in `parity.rs` |
| `test_myr.py::test_the_codes_stay_in_step_with_the_prose` | Frozen replay in `parity.rs` |
| `test_myr.py::test_every_emitted_code_is_in_the_published_vocabulary` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_reason_without_a_code_is_refused_at_construction` | Structural: reasons and codes are built as pairs |
| `test_myr.py::test_a_gate_run_against_another_year_is_refused_with_its_code` | Frozen replay in `parity.rs` |
| `test_myr.py::test_a_nil_benchmark_rate_is_refused_with_its_code` | Frozen replay in `parity.rs` |
| `test_myr.py::test_the_final_repayment_year_is_principal_plus_one_year_of_interest` | Frozen replay in `parity.rs` |
| `test_myr.py::test_an_exact_half_cent_rounds_up` | Frozen replay in `parity.rs` |
| `test_rates.py::test_every_frozen_year_matches_the_published_table` | Frozen replay in `parity.rs` |
| `test_rates.py::test_every_year_reads_the_may_figure` | Frozen replay in `parity.rs` |
| `test_rates.py::test_2025_26_uses_the_may_figure_not_the_june_one` | Frozen replay in `parity.rs` |
| `test_rates.py::test_2023_24_is_the_year_that_carries_0_0827` | Frozen replay in `parity.rs` |
| `test_rates.py::test_year_before_coverage_is_unknown_with_a_reason` | Frozen replay in `parity.rs` |
| `test_rates.py::test_year_after_coverage_is_unknown_with_a_reason` | Frozen replay in `parity.rs` |
| `test_rates.py::test_table_carries_its_own_review_metadata` | `src/rates.rs`: `the_frozen_table_carries_its_review_and_primary_sources` |
| `test_rates.py::test_provenance_reaches_the_result` | Frozen replay in `parity.rs` |
| `test_rates.py::test_override_extends_coverage` | Frozen replay in `parity.rs` |
| `test_rates.py::test_override_without_citation_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_override_without_verified_until_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_override_cannot_reach_past_its_own_verified_until` | Frozen replay in `parity.rs` |
| `test_rates.py::test_override_with_no_rates_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_table_refuses_a_row_past_its_reviewed_until` | Frozen replay in `parity.rs` |
| `test_rates.py::test_table_refuses_a_duplicated_year` | Frozen replay in `parity.rs` |
| `test_rates.py::test_table_refuses_a_percentage_written_as_a_whole_number` | Frozen replay in `parity.rs` |
| `test_rates.py::test_rates_are_decimals_not_floats` | Frozen replay in `parity.rs` |
| `test_rates.py::test_override_refuses_duplicate_years` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_known_rate_names_the_table_it_came_from` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_unknown_rate_still_names_the_table_it_was_looked_for_in` | Frozen replay in `parity.rs` |
| `test_rates.py::test_the_digest_is_the_digest_of_the_table_that_was_read` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_edited_table_changes_the_digest` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_crlf_checkout_and_an_lf_checkout_agree` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_override_is_named_in_the_manifest_without_the_operator_path` | Frozen replay in `parity.rs` |
| `test_rates.py::test_every_row_cites_a_primary_source_on_an_official_host` | `src/rates.rs`: `the_frozen_table_carries_its_review_and_primary_sources` |
| `test_rates.py::test_a_missing_table_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_table_without_its_reviewed_on_header_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_table_whose_reviewed_until_is_not_a_year_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_table_with_no_rows_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_row_without_its_rba_table_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_missing_override_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_override_that_is_not_json_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_override_that_is_not_an_object_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_override_whose_verified_until_is_not_a_year_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_override_rate_that_is_not_an_object_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_override_rate_written_as_a_json_number_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_override_rate_that_is_not_a_number_is_refused` | Frozen replay in `parity.rs` |
| `test_reason_codes.py::test_every_gate_limb_that_does_not_pass_carries_its_code` | Frozen replay in `parity.rs`; `the_reason_code_vocabulary_is_exactly_what_the_engine_emits` checks the vocabulary |
| `test_reason_codes.py::test_a_gate_with_no_year_to_read_a_benchmark_for_carries_its_code` | Frozen replay in `parity.rs`; `the_reason_code_vocabulary_is_exactly_what_the_engine_emits` checks the vocabulary |
| `test_reason_codes.py::test_a_complying_gate_carries_no_reason_codes` | Frozen replay in `parity.rs`; `the_reason_code_vocabulary_is_exactly_what_the_engine_emits` checks the vocabulary |
| `test_reason_codes.py::test_the_gate_codes_stay_in_step_with_its_prose` | Frozen replay in `parity.rs`; `the_reason_code_vocabulary_is_exactly_what_the_engine_emits` checks the vocabulary |
| `test_reason_codes.py::test_a_gate_reason_without_a_code_is_refused_at_construction` | Structural: reasons and codes are built as pairs |
| `test_reason_codes.py::test_no_code_in_the_vocabulary_is_unreachable` | Frozen replay in `parity.rs`; `the_reason_code_vocabulary_is_exactly_what_the_engine_emits` checks the vocabulary |
| `test_reason_codes.py::test_no_scenario_here_emits_a_code_outside_the_vocabulary` | Frozen replay in `parity.rs`; `the_reason_code_vocabulary_is_exactly_what_the_engine_emits` checks the vocabulary |
| `test_register.py::test_review_mode_rejects_nonboolean_options` | Frozen replay in `parity.rs` |
| `test_register.py::test_review_mode_cannot_suppress_both_results` | Frozen replay in `parity.rs` |
| `test_register.py::test_review_mode_preserves_the_three_literal_choices` | Frozen replay in `parity.rs` |
| `test_register.py::test_review_mode_is_checked_before_consuming_rows` | `src/register.rs`: `the_review_mode_is_checked_before_any_row_or_table` |
| `test_register.py::test_review_mode_is_checked_before_loading_a_file` | Frozen replay in `parity.rs` |
| `test_register.py::test_review_mode_rejects_conflicting_options_for_an_empty_register` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_missing_required_column_is_refused` | Frozen replay in `parity.rs` |
| `test_register.py::test_the_error_names_every_missing_column` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_register_without_the_repayment_columns_is_refused_for_myr` | Frozen replay in `parity.rs` |
| `test_register.py::test_the_same_register_is_accepted_for_the_gate` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_header_with_no_rows_is_refused` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_missing_file_is_refused` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_byte_order_mark_does_not_break_the_first_column` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_register_that_is_not_utf8_is_refused_with_a_reason` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_quoted_multiline_value_preserves_its_newline` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_multiline_numeric_cell_is_not_joined_into_a_valid_amount` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_clean_register_needs_no_attention` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_short_row_reports_exposure` | Frozen replay in `parity.rs` |
| `test_register.py::test_counts_cover_every_summary_key` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_row_marked_out_of_scope_is_skipped_not_reviewed` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_loan_made_before_4_december_1997_is_skipped` | Frozen replay in `parity.rs` |
| `test_register.py::test_the_year_straddling_4_december_1997_is_also_skipped` | Frozen replay in `parity.rs` |
| `test_register.py::test_the_first_reviewable_year_is_not_skipped` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_non_complying_row_refuses_the_repayment` | Frozen replay in `parity.rs` |
| `test_register.py::test_an_unknown_gate_counts_once_as_unknown_and_once_as_refused` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_bad_cell_is_reported_against_its_row` | Frozen replay in `parity.rs` |
| `test_register.py::test_exposure_comes_first_largest_shortfall_first` | Frozen replay in `parity.rs` |
| `test_register.py::test_clean_rows_sit_above_skipped_ones` | Frozen replay in `parity.rs` |
| `test_register.py::test_ties_keep_the_registers_own_order` | Frozen replay in `parity.rs` |
| `test_register.py::test_the_clean_sample_runs_to_a_verdict_with_no_flags` | Frozen replay in `parity.rs` |
| `test_register.py::test_the_mixed_sample_carries_unknowns_and_a_non_complying_loan` | Frozen replay in `parity.rs` |
| `test_register.py::test_the_mixed_sample_leads_with_its_exposure` | Frozen replay in `parity.rs` |
| `test_register.py::test_the_gate_can_be_anchored_to_a_later_year` | Frozen replay in `parity.rs` |
| `test_register.py::test_total_exposure_is_a_decimal` | Frozen replay in `parity.rs` |
| `test_register.py::test_padded_required_header_is_refused` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_register_of_skipped_rows_reads_no_rate_table` | Frozen replay in `parity.rs` |
| `test_register.py::test_a_reviewed_register_names_the_table_once` | Frozen replay in `parity.rs` |
| `test_workbook.py::test_workbook_is_macro_free_and_carries_no_build_path` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_cached_values_were_calculated_by_desktop_excel` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_mixed_sample_matches_the_engine_line_by_line` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_rates_sheet_matches_the_frozen_table` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_the_summary_year_selector_grows_with_the_rates_table` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_input_columns_are_text_or_validated_so_pasted_values_stay_inert` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_any_nonblank_exclusion_reason_skips_the_row` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_only_reviewed_rows_are_validated` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_the_loan_year_label_is_read_before_the_historical_skip` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_the_year_guard_matches_the_engine_grammar_and_bounds` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_the_money_guard_carries_the_engine_limits` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_the_money_guard_survives_a_text_cell` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_scope_and_lookup_read_the_trimmed_year` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_year_whitespace_matches_python_without_erasing_internal_gaps` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_every_semantic_year_read_uses_the_shared_helper` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_missing_floor_year_precedes_gate_failures_and_maximum_term` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_year_helpers_extend_the_table_without_moving_public_columns` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_workbook.py::test_changed_formulas_fit_excels_formula_limit` | Moved to the repository's `tests/test_div7a_workbook.py`, held to `tests/data/workbook-sample.json` (`evaluation.rs`: `the_workbook_sample_review_is_the_engines`) |
| `test_years.py::test_a_year_must_start_on_a_whole_year` | Frozen replay in `parity.rs` |
| `test_years.py::test_a_year_outside_the_plausible_range_is_refused` | Frozen replay in `parity.rs` |
| `test_years.py::test_a_year_prints_as_its_label` | Frozen replay in `parity.rs` |
| `test_years.py::test_a_year_does_not_order_against_a_bare_number` | Frozen replay in `parity.rs` |
