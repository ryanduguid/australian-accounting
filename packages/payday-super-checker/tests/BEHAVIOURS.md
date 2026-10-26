# Python behaviour coverage

Base 55a037c, Python 0.2.2: 672 test functions. The archive pytest suite
passed before the differential harness captured the top-level engine calls and
command runs they made, with the files each named. `tests/data/python-cases.json`
freezes the first 60 distinct cases of each test that the harness could encode
as JSON or command-line input, 50 counterexamples from the final reviews
and 3,400 generated API calls and command runs: 4,883 cases in all.
`tests/parity.rs` replays them and compares complete results, key order, output
bytes and written files included.
Results and streams over 4 KiB are stored as an FNV-1a fingerprint and length.
`tests/data/amendments.json` replaces the answers of cases that a deliberate
change since 0.2.2 alters, by case index, with the reason for each change.

The generated cases include malformed dates, amounts and flags in the CSV
columns, mapping and calendar override files, Xero, MYOB and Employment Hero
shaped importer exports and argparse prefix, help and missing-value
combinations, under varied colour and width controls.

Some tests cannot be replayed from frozen inputs, and run natively instead: those
that call an internal phase with chosen facts, build a result by hand, read the
machine's clock for a defaulted `--as-at`, name a directory or a symlink, or edit
a bundled table. A table edit cannot reach the binary, which compiles its data in,
so `src/rates.rs`, `src/calendar.rs` and `src/profiles.rs` hold every Python
message for malformed bundled data to their own parsers. Where the JSON API
refuses a field its typed contract excludes (a `null` where Python's argument was
not optional, another JSON type, an integer outside 64 bits, or a required field
left out), the harness records the refusal in place of Python's answer and counts
it; it checks that the input really breaks the contract for the field named. Each frozen case records the complete file state after its run: what it
wrote, the inputs it left unchanged, and that nothing else exists. The property
tests keep the fast-check properties' input domains; a local generator replaces
fast-check's, so the random cases differ.

| Original test | Rust coverage |
| --- | --- |
| `test_assess_complexity.py::test_assess_line_only_orchestrates_decision_phases` | Not applicable: a limit on the Python function's length and branch count; `assess_line` in `src/assess.rs` calls the same decision phases, each tested below |
| `test_assess_complexity.py::test_received_phase_keeps_partial_prepayment_boundaries` | Unit test `the_received_phase_keeps_partial_prepayment_boundaries` in `src/assess.rs` |
| `test_assess_complexity.py::test_received_phase_keeps_stale_and_partial_receipt_outcomes` | Unit test `the_received_phase_keeps_stale_and_partial_receipt_outcomes` in `src/assess.rs` |
| `test_assess_complexity.py::test_no_receipt_phase_keeps_item4_uncertainty` | Unit test `the_no_receipt_phase_keeps_item4_uncertainty` in `src/assess.rs` |
| `test_assess_complexity.py::test_exposure_phase_keeps_horizon_and_new_starter_boundaries` | Unit test `the_exposure_phase_keeps_horizon_and_new_starter_boundaries` in `src/assess.rs`; its loader calls are also frozen in `parity.rs` |
| `test_assess_complexity.py::test_a_partial_prepayment_past_its_deadline_is_unpaid_when_nothing_is_uncertain` | Unit test `a_partial_prepayment_past_its_deadline_is_unpaid_when_nothing_is_uncertain` in `src/assess.rs` |
| `test_assess_complexity.py::test_a_partial_prepayment_names_only_the_uncertainty_that_applies` | Unit test `a_partial_prepayment_names_only_the_uncertainty_that_applies` in `src/assess.rs` |
| `test_assess_complexity.py::test_no_receipt_past_a_deadline_item4_may_extend_is_left_unknown` | Unit test `no_receipt_past_a_deadline_item4_may_extend_is_left_unknown` in `src/assess.rs` |
| `test_assessment_module.py::test_report_route_preserves_the_representative_result_object` | Frozen replay in `parity.rs` |
| `test_assessment_module.py::test_report_route_preserves_aggregate_validation_order` | Frozen replay in `parity.rs` |
| `test_assessment_module.py::test_assessment_module_owns_the_historical_report_route` | Not applicable: Python module layout |
| `test_atomic_io.py::test_atomic_text_output_keeps_the_previous_file_when_writing_fails` | Unit test `a_failed_replacement_keeps_the_destination_and_no_staged_file` in `src/atomic.rs`; the Rust writer stages the whole text at once, so no caller code runs mid-write |
| `test_atomic_io.py::test_generated_output_requires_an_explicit_csv_filename` | Unit test `the_csv_name_is_checked_before_anything_is_staged` in `src/atomic.rs` |
| `test_atomic_io.py::test_a_stream_that_cannot_be_opened_closes_its_descriptor_and_names_the_output` | Not reproduced: it injects a failing `os.fdopen`. The Rust writer opens the staging file once, closes it by scope, and reports every staging error under the output's name (`write` in `src/atomic.rs`) |
| `test_atomic_io.py::test_a_failed_cleanup_does_not_replace_the_write_failure` | Not reproduced: it injects failing `os.replace` and `os.unlink`. `write` in `src/atomic.rs` ignores the cleanup's result and returns the replacement error |
| `test_boolean_controls.py::test_assessment_rejects_malformed_controls_even_when_the_row_does_not_need_them` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_daily_rate_requires_a_boolean_for_known_and_stale_dates` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_earnings_reject_a_malformed_control_when_no_day_accrues` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_attention_requires_boolean_confirmation` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_console_requires_boolean_confirmation` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_csv_identifier_option_requires_a_boolean` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_evidence_pack_does_not_record_a_malformed_confirmation` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_import_requires_boolean_confirmation_before_writing` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_literal_report_controls_keep_their_meaning` | Frozen replay in `parity.rs` |
| `test_boolean_controls.py::test_literal_controls_work_when_assessment_and_accrual_are_empty` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_weekends_are_not_business_days` | Unit test `only_whole_of_state_holidays_and_weekends_stop_the_clock` in `src/calendar.rs` |
| `test_calendar.py::test_ekka_is_a_business_day` | Unit test `only_whole_of_state_holidays_and_weekends_stop_the_clock` in `src/calendar.rs` |
| `test_calendar.py::test_state_wide_holiday_anywhere_stops_the_clock` | Unit test `only_whole_of_state_holidays_and_weekends_stop_the_clock` in `src/calendar.rs` |
| `test_calendar.py::test_locally_substitutable_dates_do_not_stop_the_national_clock` | Unit test `only_whole_of_state_holidays_and_weekends_stop_the_clock` in `src/calendar.rs` |
| `test_calendar.py::test_no_bank_holiday_reaches_the_shipped_table` | Unit test `no_bank_holiday_reaches_the_shipped_table` in `src/calendar.rs` |
| `test_calendar.py::test_add_business_days_skips_weekends` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_ato_new_employee_example` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_window_containing_picnic_day_extends_by_one` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_zero_business_days_returns_same_day` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_negative_business_days_rejected` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_horizon_warning` | Unit test `the_horizon_warns_past_the_calendar_coverage` in `src/calendar.rs` |
| `test_calendar.py::test_a_declared_override_raises_the_coverage_end_past_verified_until` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_an_undeclared_override_does_not_raise_the_coverage_end` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_one_far_future_holiday_does_not_cover_the_gap_before_it` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_an_override_declaring_an_earlier_date_cannot_shrink_coverage` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_check_horizon_reads_the_declared_coverage_not_the_verified_date` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_a_bad_declared_coverage_date_is_named` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_an_override_entry_names_the_key_it_is_missing` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_a_non_list_jurisdictions_value_is_named` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_override_entries_reject_ambiguous_metadata` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_override_rejects_duplicate_additions` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_an_unpatched_calendar_computes_the_earlier_deadline` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_a_good_bundled_table_still_loads` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_a_missing_top_level_key_is_named` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_a_list_at_the_top_level_is_refused` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_non_business_days_must_be_a_list` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_an_unreadable_verified_date_is_named` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_the_cli_prints_a_calendar_error_without_a_traceback` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_provisional_hits_flags_grand_final_day` | Unit test `provisional_hits_flag_grand_final_day` in `src/calendar.rs` |
| `test_calendar.py::test_override_add_and_remove` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_override_removing_unknown_date_errors` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_override_unknown_key_errors` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_christmas_new_year_cluster` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_easter_cluster_2027` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_data_files_ship_inside_the_package` | Not applicable: the data files are compiled into the binary (`include_str!`); `calendar.rs` and `rates.rs` unit tests load them |
| `test_calendar.py::test_a_malformed_override_is_refused_with_its_reason` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_an_override_that_is_not_json_is_refused` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_a_bundled_calendar_that_is_not_json_is_refused` | Frozen replay in `parity.rs` |
| `test_calendar.py::test_a_bundled_calendar_with_a_repeated_date_is_refused` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_streams_without_reconfigure_still_receive_the_error` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_an_as_at_with_a_utc_offset_is_refused_with_the_flag_named` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_an_as_at_that_is_not_a_date_is_refused` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_an_input_that_cannot_be_read_names_it` | `an_input_that_cannot_be_read_is_named` in `tests/cli.rs` |
| `test_cli_paths.py::test_a_report_that_cannot_be_written_names_it` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_a_date_overflow_is_one_error_line` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_more_than_ten_unassessable_lines_are_counted_after_the_first_ten` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_an_import_without_warnings_prints_no_warning_block` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_review_pack_names_a_report_that_is_not_there` | Frozen replay in `parity.rs` |
| `test_cli_paths.py::test_review_pack_names_a_report_it_cannot_read` | `an_input_that_cannot_be_read_is_named` in `tests/cli.rs` |
| `test_cli_paths.py::test_review_pack_names_a_pack_it_cannot_write` | Frozen replay in `parity.rs` |
| `test_csv_guard_whitespace.py::test_guard_quotes_formula_after_leading_whitespace` | Frozen replay in `parity.rs` |
| `test_csv_guard_whitespace.py::test_guard_preserves_ordinary_text` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_parses_the_sample_fixture` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_blank_optional_dates_become_none` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_day_first_dates_are_accepted` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_currency_formatting_is_accepted` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_comma_in_the_decimal_position_is_refused_not_read_as_thousands` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_thousands_separator_is_still_read` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_space_after_the_dollar_sign_is_read` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_every_amount_shape_that_parsed_before_the_separator_rule_still_parses` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_separator_out_of_the_thousands_position_stays_refused` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_impossible_date_is_rejected_with_row_number` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_date_with_trailing_junk_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_zone_less_iso_datetime_is_accepted_as_its_calendar_day` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_datetime_carrying_a_zone_offset_is_refused_not_read_a_day_early` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_the_offset_refusal_reaches_the_row_reader_with_a_row_number` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_long_fractional_seconds_parse_the_same_on_every_supported_python` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_dotnet_timestamp_is_accepted_as_its_calendar_day` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_iso_shapes_beyond_the_documented_surface_are_refused` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_short_fractional_seconds_parse_the_same_on_every_supported_python` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_unreadable_amount_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_negative_amount_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_empty_employee_id_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_unreadable_flag_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_missing_required_column_names_it` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_header_only_file_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_utf8_bom_header_is_handled` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_mapping_overrides_apply` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_mapping_file_is_read` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_unknown_mapping_field_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_malformed_map_argument_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_duplicate_column_names_are_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_trailing_empty_headers_are_tolerated` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_truncated_row_is_rejected_not_assumed_blank` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_mapped_column_that_does_not_exist_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_unmapped_optional_column_absence_is_fine` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_shipped_example_mapping_file_loads` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_nan_and_infinity_amounts_are_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_row_with_more_values_than_columns_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_mapping_file_that_is_not_an_object_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_mapping_file_that_is_not_json_is_a_csv_error_naming_the_file` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_absurdly_large_amount_is_rejected` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_both_amount_parsers_agree_on_every_input_both_can_receive` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_the_check_path_reads_amounts_to_the_cent` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_sub_half_cent_amount_is_refused_not_rounded_to_nothing` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_the_importer_reads_an_accounting_format_negative_as_negative` | Unit test `an_amount_under_half_a_cent_or_negative_is_refused` in `src/importers.rs` |
| `test_csv_io.py::test_a_nine_column_file_still_treats_a_remitted_date_as_full_credit` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_remitted_amount_is_read_from_an_appended_column` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_matched_amount_is_read_without_a_vendor_remittance_date` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_remitted_amount_without_a_date_is_refused_as_ambiguous` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_remitted_amount_greater_than_sg_amount_is_refused` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_matched_amount_greater_than_sg_amount_is_refused` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_remitted_amount_greater_than_matched_amount_is_refused` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_partial_matched_amount_with_a_date_requires_remitted_amount` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_parse_date_text_reads_blank_and_impossible_dates_as_none` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_malformed_mapping_file_is_refused` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_file_with_no_header_row_is_refused` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_a_file_that_is_not_utf8_names_the_byte` | Frozen replay in `parity.rs` |
| `test_csv_io.py::test_an_importer_amount_cell_that_is_empty_is_named` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_contrib_line_keeps_the_pre_matched_amount_positional_signature` | Not applicable: Python positional construction; Rust fields are named |
| `test_deadlines.py::test_usual_period_is_seven_business_days` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_first_contribution_to_fund_gets_twenty_business_days` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_out_of_cycle_rides_the_next_standard_payday` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_out_of_cycle_without_next_payday_is_rejected` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_next_payday_without_the_flag_names_the_item_2_deadline` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_missing_flag_does_not_invent_an_absent_next_qe_day` | Unit test `the_missing_flag_does_not_invent_an_absent_next_qe_day` in `src/deadlines.rs` |
| `test_deadlines.py::test_next_payday_caveat_names_the_deadline_the_row_actually_got` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_next_payday_caveat_still_fires_where_item_2_would_win` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_the_missing_flag_caveat_reads_the_deadline_item_4_left` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_next_payday_without_the_flag_is_flagged_when_it_is_not_later` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_out_of_cycle_row_gets_no_missing_flag_caveat` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_out_of_cycle_next_payday_must_be_later` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_defined_benefit_lines_are_skipped` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_pre_regime_qe_day_is_rejected` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_regime_starts_on_first_july_2026` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_item4_aligns_a_later_payday_inside_an_extended_window` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_item4_does_not_shorten_a_later_deadline` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_item4_is_per_employee` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_unconfirmed_dates_are_flagged_but_do_not_extend_the_deadline` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_item4_does_not_align_contributions_sharing_a_qe_day` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_item4_result_is_independent_of_row_order` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_item4_aligns_paydays_given_out_of_date_order` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_later_qe_day_inherits_the_longest_window_from_a_group` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_both_item_1_and_item_2_apply_taking_the_later_deadline` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_out_of_cycle_wins_when_its_window_ends_later` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_deadline_past_the_calendar_horizon_warns` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_a_nil_payday_does_not_seed_an_item_4_alignment` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_a_real_payday_still_seeds_an_item_4_alignment` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_item_1_does_not_hide_an_invalid_out_of_cycle_claim` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_receipt_amount_cap_is_an_upper_bound_not_evidence` | Unit test `the_receipt_amount_cap_is_an_upper_bound_not_evidence` in `src/deadlines.rs` |
| `test_deadlines.py::test_a_row_with_no_receipt_at_all_gets_no_receipt_amount_caveat` | Frozen replay in `parity.rs` |
| `test_deadlines.py::test_the_prepayment_window_before_a_leap_day_falls_back_to_28_february` | Unit test `the_prepayment_window_before_a_leap_day_falls_back_to_28_february` in `src/deadlines.rs` |
| `test_diff_coverage_policy.py::test_changed_line_coverage_is_scoped_and_fail_closed` | Not applicable: the Python changed-line coverage gate is removed |
| `test_docs.py::test_rounding_authority_and_experimental_boundary_stay_visible` | `the_rounding_authority_and_experimental_boundary_stay_visible` in `tests/docs.rs` |
| `test_docs.py::test_final_out_of_cycle_determination_is_pinned_in_user_guidance` | `the_final_out_of_cycle_determination_is_pinned_in_user_guidance` in `tests/docs.rs` |
| `test_docs.py::test_final_ruling_status_and_transition_gate_are_current` | `the_final_ruling_status_and_transition_gate_are_current` in `tests/docs.rs` |
| `test_docs.py::test_item4_and_import_allocation_boundaries_stay_visible` | `the_item_4_and_import_allocation_boundaries_stay_visible` in `tests/docs.rs` |
| `test_docs.py::test_regulations_11_and_12_are_an_operator_input_boundary` | `regulations_11_and_12_are_an_operator_input_boundary` in `tests/docs.rs` |
| `test_docs.py::test_primary_review_pins_regulations_and_direct_mcb_provenance` | `the_primary_review_pins_regulations_and_direct_mcb_provenance` in `tests/docs.rs` |
| `test_docs.py::test_calendar_authority_horizon_and_exclusions_are_pinned` | `the_calendar_authority_horizon_and_exclusions_are_pinned` in `tests/docs.rs` |
| `test_docs.py::test_calendar_regeneration_guidance_uses_the_locked_environment` | `the_calendar_regeneration_guidance_uses_the_locked_environment` in `tests/docs.rs` |
| `test_evaluation_pack.py::test_evidence_boundary_scenarios` | Frozen replay in `parity.rs` |
| `test_evaluation_pack.py::test_fabricated_fixtures_have_the_canonical_header_and_synthetic_row` | `every_fixture_is_declared_once_and_fabricated` in `tests/evaluation.rs` |
| `test_evaluation_pack.py::test_evaluation_contract_retains_semantic_and_privacy_boundaries` | `the_evidence_pack_guide_keeps_its_boundaries_and_is_discoverable` in `tests/docs.rs` and `every_fixture_is_declared_once_and_fabricated` in `tests/evaluation.rs` (its sources) |
| `test_evaluation_pack.py::test_product_discovery_files_link_the_pack` | `the_evidence_pack_guide_keeps_its_boundaries_and_is_discoverable` in `tests/docs.rs` |
| `test_evaluation_pack.py::test_only_declared_evaluation_csvs_are_allowlisted` | `only_the_declared_evaluation_csvs_are_allowlisted` in `tests/evaluation.rs`; the `git check-ignore` probe is not repeated |
| `test_evaluation_pack.py::test_confirmation_changes_the_at_risk_exit_code_but_not_the_verdict` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_pinned_fabricated_pack` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_existing_directory_is_preserved` | `an_existing_evidence_pack_directory_is_preserved` in `tests/cli.rs` |
| `test_evidence_pack.py::test_bad_input_leaves_no_pack` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_reference_table_failure_does_not_block_evidence_pack` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_gic_table_failure_still_blocks_evidence_pack` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_remittance_confirmation_does_not_clear_review_queue` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_render_failure_leaves_no_directory` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_every_non_on_time_verdict_requires_review` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_writer_refuses_traversal_and_preserves_failed_pack` | Unit test `the_writer_refuses_other_names_and_an_existing_directory` in `src/evidence_pack.rs` covers the refusal; a failure part-way through is not injected, and the writer has no cleanup step that could remove a partial pack |
| `test_evidence_pack.py::test_projected_report_validation_refuses_format_drift` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_concurrent_destination_creation_is_not_replaced` | Unit test `the_writer_refuses_other_names_and_an_existing_directory` in `src/evidence_pack.rs`: the directory is created exclusively, so one that appears first is refused untouched |
| `test_evidence_pack.py::test_qualified_reserved_filename_still_runs_the_existing_checker` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_ordinary_checker_writes_csv_in_bounded_chunks` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_the_pack_names_the_gic_table_it_used` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_the_manifest_digest_matches_the_shipped_gic_table` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_an_edited_gic_table_changes_the_digest` | Frozen replay in `parity.rs` |
| `test_evidence_pack.py::test_a_crlf_checkout_and_an_lf_checkout_agree` | Unit test `a_crlf_checkout_and_an_lf_checkout_agree` in `src/rates.rs` |
| `test_evidence_pack.py::test_the_exception_queue_refuses_a_value_it_cannot_write` | Not applicable: the Rust JSON value type has no float, so the value cannot arise |
| `test_fastcheck_bridge.py::test_the_same_seed_repeats_the_same_cases` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_discarded_inputs_do_not_reduce_the_accepted_budget` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_a_failure_is_shrunk_and_its_replay_repeats_the_counterexample` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_a_python_error_is_reported_as_a_failure` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_an_unknown_property_fails_closed` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_an_invalid_budget_is_refused` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_a_missing_installation_is_bootstrapped_before_the_property` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_a_failed_bootstrap_fails_closed` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_a_stale_installation_fails_closed` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_a_missing_module_fails_closed` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_fastcheck_bridge.py::test_a_hung_worker_is_killed_and_reaped` | Not applicable: the fast-check bridge is removed; `properties.rs` generates its own inputs |
| `test_generate_calendar.py::test_raw_generator_is_provisional_until_the_reviewed_table_confirms_dates` | Moved to the repository's `tests/test_payday_generate_calendar.py` (the generator is a Python tool) |
| `test_generate_calendar.py::test_generator_excludes_locally_substitutable_dates` | Moved to the repository's `tests/test_payday_generate_calendar.py` (the generator is a Python tool) |
| `test_generate_calendar.py::test_generator_requests_public_holidays_only_and_emits_no_bank_holiday` | Moved to the repository's `tests/test_payday_generate_calendar.py` (the generator is a Python tool) |
| `test_generate_calendar.py::test_generator_provenance_reflects_the_environment_that_ran` | Moved to the repository's `tests/test_payday_generate_calendar.py` (the generator is a Python tool) |
| `test_importers.py::test_read_super_keeps_only_super_guarantee` | Frozen replay in `parity.rs` |
| `test_importers.py::test_read_super_reads_australian_day_first_dates` | Frozen replay in `parity.rs` |
| `test_importers.py::test_read_super_surfaces_the_resolved_columns_for_this_file` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_date_outside_the_profiles_own_formats_still_reads` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_negative_amount_is_refused_by_both_readers` | Frozen replay in `parity.rs` |
| `test_importers.py::test_read_payroll_reads_payday_and_amount` | Frozen replay in `parity.rs` |
| `test_importers.py::test_read_payroll_surfaces_the_resolved_columns_for_this_file` | Frozen replay in `parity.rs` |
| `test_importers.py::test_xero_pair_is_detected_and_imports` | Frozen replay in `parity.rs` |
| `test_importers.py::test_employment_hero_pair_is_detected_and_imports` | Frozen replay in `parity.rs` |
| `test_importers.py::test_myob_business_pair_is_detected_and_imports` | Frozen replay in `parity.rs` |
| `test_importers.py::test_employment_hero_uses_super_guarantee_not_qualifying_earnings` | Frozen replay in `parity.rs` |
| `test_importers.py::test_myob_ar_membership_number_is_not_compared_with_payroll_card_id` | Frozen replay in `parity.rs` |
| `test_importers.py::test_read_payroll_resolved_columns_omit_an_absent_period_end` | Frozen replay in `parity.rs` |
| `test_importers.py::test_super_file_without_a_contribution_type_column_is_refused` | Frozen replay in `parity.rs` |
| `test_importers.py::test_mis_grouped_amount_is_refused` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_placeholder_year_is_refused_here_not_by_the_checker_afterwards` | Frozen replay in `parity.rs` |
| `test_importers.py::test_the_import_date_ceiling_is_exactly_the_checkers_own` | Frozen replay in `parity.rs` |
| `test_importers.py::test_misaligned_row_is_refused_not_silently_shifted` | Frozen replay in `parity.rs` |
| `test_importers.py::test_truncated_row_is_refused_not_read_as_blank` | Frozen replay in `parity.rs` |
| `test_importers.py::test_duplicate_header_refused_even_when_profile_never_maps_it` | Frozen replay in `parity.rs` |
| `test_importers.py::test_duplicate_normalised_headers_are_refused` | Frozen replay in `parity.rs` |
| `test_importers.py::test_two_identical_punctuation_only_headings_are_refused_like_csv_io_refuses_them` | Frozen replay in `parity.rs` |
| `test_importers.py::test_exact_match_sets_the_remittance_date` | Frozen replay in `parity.rs` |
| `test_importers.py::test_split_payment_takes_the_later_date` | Frozen replay in `parity.rs` |
| `test_importers.py::test_short_payment_is_flagged_partial` | Frozen replay in `parity.rs` |
| `test_importers.py::test_overpayment_is_flagged` | Frozen replay in `parity.rs` |
| `test_importers.py::test_payday_with_no_super_payment_is_flagged_and_left_blank` | Frozen replay in `parity.rs` |
| `test_importers.py::test_super_payment_matching_nothing_becomes_an_orphan` | Frozen replay in `parity.rs` |
| `test_importers.py::test_two_identical_paydays_refuse_rather_than_guess` | Frozen replay in `parity.rs` |
| `test_importers.py::test_name_matching_warns` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_super_row_bracketing_two_paydays_apportions_oldest_first` | Frozen replay in `parity.rs` |
| `test_importers.py::test_ambiguous_coverage_apportions_deterministically_regardless_of_amount` | Frozen replay in `parity.rs` |
| `test_importers.py::test_one_payment_covering_three_fortnightly_paydays_is_apportioned_not_aborted` | Frozen replay in `parity.rs` |
| `test_importers.py::test_touching_period_boundaries_pair_cleanly_one_to_one` | Frozen replay in `parity.rs` |
| `test_importers.py::test_two_identical_paydays_without_a_covering_super_payment_are_not_refused` | Frozen replay in `parity.rs` |
| `test_importers.py::test_reversed_super_period_is_refused` | Frozen replay in `parity.rs` |
| `test_importers.py::test_split_payment_with_one_undated_row_limits_the_date_to_the_dated_subtotal` | Frozen replay in `parity.rs` |
| `test_importers.py::test_all_matched_rows_undated_notes_it_and_leaves_remitted_blank` | Frozen replay in `parity.rs` |
| `test_importers.py::test_id_matching_is_used_when_every_row_has_an_id` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_single_blank_id_falls_back_to_name_matching` | Frozen replay in `parity.rs` |
| `test_importers.py::test_no_super_rows_does_not_vacuously_claim_id_matching` | Frozen replay in `parity.rs` |
| `test_importers.py::test_two_employee_ids_differing_only_in_punctuation_stay_distinct` | Frozen replay in `parity.rs` |
| `test_importers.py::test_two_names_differing_only_in_punctuation_stay_distinct` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_name_with_no_ascii_letters_imports_instead_of_stopping_the_run` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_sub_cent_shortfall_is_not_reported_as_a_partial_payment` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_sub_cent_excess_is_not_reported_as_an_overpayment` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_whole_cent_difference_is_still_flagged_both_ways` | Frozen replay in `parity.rs` |
| `test_importers.py::test_decimal_precision_is_exact_not_float` | Frozen replay in `parity.rs` |
| `test_importers.py::test_no_payroll_period_end_column_warns` | Frozen replay in `parity.rs` |
| `test_importers.py::test_super_missing_one_period_column_warns` | Frozen replay in `parity.rs` |
| `test_importers.py::test_super_missing_both_period_columns_warns` | Frozen replay in `parity.rs` |
| `test_importers.py::test_pay_in_arrears_without_a_period_end_column_misses_silently_unless_warned` | Frozen replay in `parity.rs` |
| `test_importers.py::test_claimed_tracks_object_identity_not_row_number` | Frozen replay in `parity.rs` |
| `test_importers.py::test_period_less_super_row_refusal_names_the_super_file_as_the_cause` | Frozen replay in `parity.rs` |
| `test_importers.py::test_last_known_paid_date_is_used_for_the_dated_subtotal` | Frozen replay in `parity.rs` |
| `test_importers.py::test_last_known_paid_date_is_none_when_nothing_is_dated` | Frozen replay in `parity.rs` |
| `test_importers.py::test_covers_defends_itself_against_a_period_less_row` | Unit test `a_period_less_row_covers_nothing` in `src/join.rs` |
| `test_importers.py::test_an_overpayment_cannot_be_carried_onto_a_later_payday` | Frozen replay in `parity.rs` |
| `test_importers.py::test_global_cap_prevents_a_settled_row_from_starving_another` | Frozen replay in `parity.rs` |
| `test_importers.py::test_three_period_less_payments_settle_three_fortnightly_paydays_in_full` | Frozen replay in `parity.rs` |
| `test_importers.py::test_payday_on_the_periods_own_start_date_matches` | Frozen replay in `parity.rs` |
| `test_importers.py::test_monthly_payment_covering_two_paydays_settles_both_in_full` | Frozen replay in `parity.rs` |
| `test_importers.py::test_two_different_shared_payments_are_named_separately_not_merged` | Frozen replay in `parity.rs` |
| `test_importers.py::test_zero_sg_amount_row_in_an_apportionment_group_is_not_a_missing_payment` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_payment_matching_only_a_zero_sg_row_is_an_orphan_not_a_silent_credit` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_zero_amount_payment_covering_two_paydays_says_it_has_nothing_to_give` | Frozen replay in `parity.rs` |
| `test_importers.py::test_pass2_allocation_is_independent_of_input_order_exhaustive` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_payday_on_the_period_end_does_not_jump_an_earlier_shortfall` | Frozen replay in `parity.rs` |
| `test_importers.py::test_lcr_2026_2_applies_a_short_payment_to_the_earliest_shortfall` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_refuses_unreconciled_statutory_allocation_then_accepts_confirmation` | Frozen replay in `parity.rs` |
| `test_importers.py::test_fund_order_fills_the_earlier_shortfall_before_the_later_payday` | Frozen replay in `parity.rs` |
| `test_importers.py::test_monthly_payment_whose_last_payday_is_the_period_end_exact_short_and_over` | Frozen replay in `parity.rs` |
| `test_importers.py::test_monthly_payment_clear_of_the_period_end_still_apportions_oldest_first` | Frozen replay in `parity.rs` |
| `test_importers.py::test_an_overpayment_on_already_settled_paydays_is_named_not_just_orphaned` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_payment_matching_no_payday_is_reported_differently_from_a_settled_one` | Frozen replay in `parity.rs` |
| `test_importers.py::test_the_shared_note_counts_the_paydays_the_payment_covered` | Frozen replay in `parity.rs` |
| `test_importers.py::test_orphans_are_reported_in_row_order_whatever_order_they_arrived_in` | Frozen replay in `parity.rs` |
| `test_importers.py::test_the_whole_result_is_byte_identical_under_shuffled_inputs` | Frozen replay in `parity.rs` |
| `test_importers.py::test_canonical_output_feeds_the_normal_check` | Frozen replay in `parity.rs` |
| `test_importers.py::test_write_canonical_writes_the_exact_header_and_blank_flag_columns` | Frozen replay in `parity.rs` |
| `test_importers.py::test_write_canonical_rounds_sg_amount_half_up_like_the_rest_of_the_tool` | Frozen replay in `parity.rs` |
| `test_importers.py::test_write_canonical_prefers_employee_id_over_employee_name` | Frozen replay in `parity.rs` |
| `test_importers.py::test_one_employee_is_written_under_one_label_when_only_some_rows_have_an_id` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_partial_payment_is_not_written_as_fully_remitted` | Frozen replay in `parity.rs` |
| `test_importers.py::test_an_absurdly_large_sg_amount_is_refused_with_csverror` | Frozen replay in `parity.rs` |
| `test_importers.py::test_importer_refuses_the_same_magnitude_the_checker_refuses` | Frozen replay in `parity.rs` |
| `test_importers.py::test_write_canonical_header_matches_the_checkers_default_mapping` | Unit test `the_canonical_header_is_the_checkers_default_mapping` in `src/importers.rs` |
| `test_importers.py::test_a_formula_in_an_employee_name_is_guarded` | Frozen replay in `parity.rs` |
| `test_importers.py::test_canonical_csv_round_trips_through_parse_rows_and_the_real_cli` | Frozen replay in `parity.rs` |
| `test_importers.py::test_output_refuses_to_overwrite_the_payroll_input` | Frozen replay in `parity.rs` |
| `test_importers.py::test_output_refuses_to_overwrite_the_super_input` | Frozen replay in `parity.rs` |
| `test_importers.py::test_report_distinguishes_orphan_codes_not_just_a_count` | Frozen replay in `parity.rs` |
| `test_importers.py::test_outcome_counts_cover_every_payroll_row_exactly_once` | Frozen replay in `parity.rs` |
| `test_importers.py::test_dated_subtotal_with_an_undated_remainder_is_classified_partial` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_report_clean_is_false_for_a_partial_payment` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_report_clean_is_true_for_a_wholly_ordinary_file` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_files_derives_payroll_has_period_end_from_the_file` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_files_derives_super_has_period_columns_from_the_file` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_files_does_not_warn_when_both_files_have_full_period_columns` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_payroll_file_spanning_30_june_warns_instead_of_dead_ending` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_subcommand_writes_the_file` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_replaces_an_output_symlink_without_touching_its_target` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_refuses_an_output_symlink_to_the_payroll_input` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_returns_two_when_a_payday_has_no_payment` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_clean_file_returns_zero` | Frozen replay in `parity.rs` |
| `test_importers.py::test_plain_check_dispatch_still_works_with_transition_confirmation` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_error_is_printed_without_a_traceback_and_exits_one` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_catches_arithmetic_error_from_import_files` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_prints_the_partial_warning_and_writes_remitted_amount` | Frozen replay in `parity.rs` |
| `test_importers.py::test_mixed_dated_and_undated_match_uses_the_latest_known_date_conservatively` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_never_truncates_a_partial_warning_however_many_there_are` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_never_truncates_an_undated_warning_however_many_there_are` | Frozen replay in `parity.rs` |
| `test_importers.py::test_undated_partial_survives_receipt_date_enrichment_without_full_credit` | Frozen replay in `parity.rs` |
| `test_importers.py::test_structural_warnings_print_before_the_per_row_block` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_caps_only_non_partial_warnings_at_a_pinned_literal` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_distinguishes_orphan_codes_in_the_console_output` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_vendor_flag_forces_a_profile` | Frozen replay in `parity.rs` |
| `test_importers.py::test_the_vendor_advice_is_followable_for_a_two_file_command` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_names_the_failing_file_in_an_os_error` | `an_input_that_cannot_be_read_is_named` in `tests/cli.rs` |
| `test_importers.py::test_reconfigure_stdout_for_unicode_prevents_a_non_ascii_crash` | Not applicable: the binary writes UTF-8 bytes with no stream reconfiguration |
| `test_importers.py::test_both_cli_paths_call_the_shared_stdout_reconfigure` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_sub_cent_payroll_figure_does_not_drag_the_remittance_date` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_sub_cent_payroll_figure_does_not_blank_the_date_when_the_second_row_is_undated` | Frozen replay in `parity.rs` |
| `test_importers.py::test_the_cent_clean_control_still_reads_the_same` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_sub_cent_figure_on_the_super_side_triggers_nothing_either` | Frozen replay in `parity.rs` |
| `test_importers.py::test_the_blocker_reproduction_checks_clean_end_to_end` | Frozen replay in `parity.rs` |
| `test_importers.py::test_amounts_are_read_to_the_cent_half_up` | Unit test `amounts_are_read_to_the_cent_half_up` in `src/importers.rs` |
| `test_importers.py::test_an_amount_under_half_a_cent_is_refused_rather_than_rounded_away` | Unit test `an_amount_under_half_a_cent_or_negative_is_refused` in `src/importers.rs` |
| `test_importers.py::test_a_sub_cent_row_is_refused_through_the_whole_import` | Frozen replay in `parity.rs` |
| `test_importers.py::test_unmet_never_holds_a_sub_cent_residue_and_money_is_conserved` | Frozen replay in `parity.rs` |
| `test_importers.py::test_the_importer_reads_excels_accounting_format_too` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_vendor_that_matches_nothing_is_not_told_to_type_myob` | Frozen replay in `parity.rs` |
| `test_importers.py::test_import_calls_a_failed_write_a_write_even_with_a_relative_output` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_created_batch_is_not_written_as_a_remitted_payday` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_created_batch_exits_nonzero_through_the_real_cli` | Frozen replay in `parity.rs` |
| `test_importers.py::test_every_not_yet_paid_beam_status_blanks_the_paid_date` | Frozen replay in `parity.rs` |
| `test_importers.py::test_every_sent_beam_status_keeps_the_paid_date` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_status_outside_the_beam_ladder_is_refused_not_guessed` | Frozen replay in `parity.rs` |
| `test_importers.py::test_eh_super_without_a_status_column_is_refused` | Frozen replay in `parity.rs` |
| `test_importers.py::test_identical_paydays_can_both_be_fully_paid` | Frozen replay in `parity.rs` |
| `test_importers.py::test_awaiting_clearance_is_not_remittance_evidence` | Frozen replay in `parity.rs` |
| `test_importers.py::test_write_canonical_refuses_ids_the_formula_guard_would_merge` | Frozen replay in `parity.rs` |
| `test_importers.py::test_write_canonical_still_writes_distinct_guarded_ids` | Frozen replay in `parity.rs` |
| `test_importers.py::test_negative_zero_is_written_as_an_unsigned_zero` | Frozen replay in `parity.rs` |
| `test_importers.py::test_a_sub_cent_amount_is_still_refused_in_both_readers` | Unit test `an_amount_under_half_a_cent_or_negative_is_refused` in `src/importers.rs` |
| `test_importers_paths.py::test_an_export_with_no_header_row_is_refused` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_an_export_that_is_not_utf8_is_refused` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_an_export_with_a_header_and_no_rows_is_refused` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_a_payment_date_with_a_utc_offset_is_refused_with_its_cell_named` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_a_payment_date_that_is_not_a_date_is_refused` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_an_export_with_no_super_guarantee_rows_is_refused` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_a_payroll_row_without_a_payday_is_refused` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_a_super_profile_without_an_amount_column_is_refused` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_a_super_profile_without_a_contribution_filter_reads_every_row` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_a_payroll_row_with_no_employee_is_refused` | Frozen replay in `parity.rs` |
| `test_importers_paths.py::test_a_super_row_with_only_a_period_start_covers_that_day_alone` | Unit test `a_row_with_only_a_period_start_covers_that_day_alone` in `src/join.rs` |
| `test_integration.py::test_verdicts_across_the_fixture` | Frozen replay in `parity.rs` |
| `test_integration.py::test_at_risk_line_carries_the_receipt_caveat` | Frozen replay in `parity.rs` |
| `test_integration.py::test_late_but_received_line_has_no_shortfall` | Frozen replay in `parity.rs` |
| `test_integration.py::test_console_refuses_exposed_result_without_exposure_figures` | Unit test `the_console_refuses_an_exposed_result_without_exposure_figures` in `src/report.rs` |
| `test_integration.py::test_partial_late_receipt_keeps_the_statutory_base_and_nec_running` | Frozen replay in `parity.rs` |
| `test_integration.py::test_near_full_and_full_late_receipts_keep_the_same_base_shortfall` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_on_time_receipt_reduces_only_the_base_shortfall` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_stale_prepayment_receives_no_statutory_credit` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_late_receipt_after_assessment_does_not_reduce_shortfall` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_receipt_before_assessment_keeps_nec_running_to_assessment` | Frozen replay in `parity.rs` |
| `test_integration.py::test_explicit_remitted_amount_without_remittance_date_is_refused` | Frozen replay in `parity.rs` |
| `test_integration.py::test_direct_assessment_refuses_invalid_amount_shapes` | Frozen replay in `parity.rs` |
| `test_integration.py::test_direct_assessment_refuses_partial_match_with_unbounded_remittance` | Frozen replay in `parity.rs` |
| `test_integration.py::test_duplicate_warning_normalises_legacy_and_explicit_full_remittance` | Frozen replay in `parity.rs` |
| `test_integration.py::test_assessment_before_receipt_keeps_the_shortfall` | Frozen replay in `parity.rs` |
| `test_integration.py::test_notional_earnings_run_to_the_receipt_date` | Frozen replay in `parity.rs` |
| `test_integration.py::test_unpaid_line_keeps_the_full_shortfall` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_extends_the_second_payday_for_a_new_starter` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_without_an_evidenced_earlier_contribution_fails_closed` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_remittance_or_late_receipt_is_not_eligible_evidence` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_on_time_received_and_applied_contribution_is_evidence` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_indeterminate_result_is_explicit_in_console_and_csv` | Frozen replay in `parity.rs` |
| `test_integration.py::test_same_dates_without_the_extension_are_late` | Frozen replay in `parity.rs` |
| `test_integration.py::test_transition_allocation_fails_closed_until_an_operator_reconciles_it` | Frozen replay in `parity.rs` |
| `test_integration.py::test_transition_confirmation_is_recorded_on_each_affected_result` | Frozen replay in `parity.rs` |
| `test_integration.py::test_post_transition_contribution_needs_no_confirmation` | Frozen replay in `parity.rs` |
| `test_integration.py::test_pre_july_prepayment_also_needs_transition_confirmation` | Frozen replay in `parity.rs` |
| `test_integration.py::test_cli_rejects_a_non_csv_output_without_a_traceback` | Frozen replay in `parity.rs` |
| `test_integration.py::test_cli_rejects_a_non_csv_output_before_it_reads_the_input` | Frozen replay in `parity.rs` |
| `test_integration.py::test_cli_writes_report_and_flags_late` | Frozen replay in `parity.rs` |
| `test_integration.py::test_console_summary_carries_the_legal_caveats` | Frozen replay in `parity.rs` |
| `test_integration.py::test_console_summary_ranks_by_exposure` | Frozen replay in `parity.rs` |
| `test_integration.py::test_console_summary_does_not_disclose_employee_identifiers` | Frozen replay in `parity.rs` |
| `test_integration.py::test_cli_refuses_to_overwrite_the_input` | `runs_without_an_as_at_date_keep_their_refusals` in `tests/cli.rs` |
| `test_integration.py::test_cli_refuses_to_overwrite_an_override_input` | Frozen replay in `parity.rs` |
| `test_integration.py::test_cli_replaces_an_output_symlink_without_touching_its_target` | Frozen replay in `parity.rs` |
| `test_integration.py::test_cli_reports_missing_file` | `runs_without_an_as_at_date_keep_their_refusals` in `tests/cli.rs` |
| `test_integration.py::test_cli_rejects_pre_regime_paydays` | `runs_without_an_as_at_date_keep_their_refusals` in `tests/cli.rs` |
| `test_integration.py::test_late_remittance_without_a_receipt_date_is_late` | Frozen replay in `parity.rs` |
| `test_integration.py::test_receipt_on_the_due_date_is_on_time` | Frozen replay in `parity.rs` |
| `test_integration.py::test_remittance_on_the_due_date_with_no_receipt_is_at_risk` | Frozen replay in `parity.rs` |
| `test_integration.py::test_prepayment_inside_the_twelve_month_window_is_on_time` | Frozen replay in `parity.rs` |
| `test_integration.py::test_payment_older_than_twelve_months_cannot_offset` | Frozen replay in `parity.rs` |
| `test_integration.py::test_receipt_before_remittance_is_rejected` | Frozen replay in `parity.rs` |
| `test_integration.py::test_receipt_after_the_as_at_date_is_not_used_to_settle_the_report` | Frozen replay in `parity.rs` |
| `test_integration.py::test_remittance_after_the_as_at_date_is_not_used_to_settle_the_report` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_post_as_at_receipt_does_not_claim_no_receipt_was_supplied` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_plain_remitted_only_row_still_carries_the_named_constant` | Frozen replay in `parity.rs` |
| `test_integration.py::test_stale_prepayment_keeps_the_full_shortfall` | Frozen replay in `parity.rs` |
| `test_integration.py::test_stale_prepayment_before_the_deadline_is_not_yet_assessable` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_stale_prepayment_is_still_quiet_on_the_deadline_date_itself` | Frozen replay in `parity.rs` |
| `test_integration.py::test_notional_earnings_stop_before_an_assessment` | Frozen replay in `parity.rs` |
| `test_integration.py::test_zero_amount_late_line_does_not_claim_a_receipt` | Frozen replay in `parity.rs` |
| `test_integration.py::test_zero_amount_line_with_a_late_receipt_is_not_late` | Frozen replay in `parity.rs` |
| `test_integration.py::test_overdue_nil_row_is_not_told_the_deadline_has_not_passed` | Frozen replay in `parity.rs` |
| `test_integration.py::test_nil_row_before_its_deadline_still_says_so` | Frozen replay in `parity.rs` |
| `test_integration.py::test_report_records_the_assessment_assumption` | Frozen replay in `parity.rs` |
| `test_integration.py::test_mcb_caveat_follows_the_as_at_financial_year` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_missing_cap_year_names_the_year_rather_than_saying_the_annual_cap` | Frozen replay in `parity.rs` |
| `test_integration.py::test_an_unreadable_cap_figure_names_the_year_and_the_value` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_later_year_missing_from_the_cap_table_is_named_too` | Frozen replay in `parity.rs` |
| `test_integration.py::test_financial_year_rolls_over_on_1_july` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_june_payday_names_the_financial_year_it_falls_in` | Frozen replay in `parity.rs` |
| `test_integration.py::test_all_date_problems_are_reported_at_once` | Frozen replay in `parity.rs` |
| `test_integration.py::test_unpaid_overdue_line_reports_exposure_and_exit_code` | Frozen replay in `parity.rs` |
| `test_integration.py::test_not_yet_due_line_with_no_dates_stays_unknown` | Frozen replay in `parity.rs` |
| `test_integration.py::test_calendar_caveats_describe_the_aligned_deadline` | Frozen replay in `parity.rs` |
| `test_integration.py::test_receipt_past_the_calendar_horizon_is_not_called_late` | Frozen replay in `parity.rs` |
| `test_integration.py::test_receipt_before_a_past_horizon_deadline_is_on_time` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_receipt_before_a_past_horizon_deadline_leaves_due_date_unknown` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_receipt_after_a_past_horizon_deadline_keeps_all_three_states_visible` | Frozen replay in `parity.rs` |
| `test_integration.py::test_remittance_before_a_past_horizon_deadline_is_at_risk` | Frozen replay in `parity.rs` |
| `test_integration.py::test_receipt_on_a_past_horizon_deadline_is_on_time` | Frozen replay in `parity.rs` |
| `test_integration.py::test_remittance_past_a_past_horizon_deadline_is_unassessable` | Frozen replay in `parity.rs` |
| `test_integration.py::test_an_unassessable_line_gets_its_own_block_and_a_non_zero_exit` | Frozen replay in `parity.rs` |
| `test_integration.py::test_an_unassessable_line_is_not_counted_as_a_plain_data_quality_note` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_deadline_on_the_last_covered_day_is_still_assessed` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_deadline_one_day_past_the_last_covered_day_is_not_assessed` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_deadline_inside_the_horizon_is_still_assessed` | Frozen replay in `parity.rs` |
| `test_integration.py::test_prepayment_past_the_horizon_keeps_its_verdict` | Frozen replay in `parity.rs` |
| `test_integration.py::test_unfunded_payday_past_the_horizon_is_attention_driving_unknown` | Frozen replay in `parity.rs` |
| `test_integration.py::test_stale_prepayment_past_the_horizon_has_no_exposure` | Frozen replay in `parity.rs` |
| `test_integration.py::test_horizon_caveat_says_the_table_may_be_missing_holidays` | Frozen replay in `parity.rs` |
| `test_integration.py::test_an_unrelated_added_holiday_does_not_silence_the_horizon` | Frozen replay in `parity.rs` |
| `test_integration.py::test_an_unpaid_row_past_the_horizon_does_not_claim_the_deadline_passed` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_rates_file_whose_years_are_not_a_map_is_an_error_not_a_traceback` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_integration.py::test_case_variant_employee_ids_are_flagged_not_merged` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_inherited_from_an_unrecorded_payday_is_flagged` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_on_time_receipt_respects_an_unresolved_item4_deadline` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_receipt_inside_an_unresolved_item4_window_keeps_outer_bounds` | Frozen replay in `parity.rs` |
| `test_integration.py::test_partial_item4_receipt_after_the_upper_deadline_has_no_not_yet_due_state` | Frozen replay in `parity.rs` |
| `test_integration.py::test_zero_amount_receipt_cannot_seed_item4_but_positive_partial_can` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_inherited_caveat_survives_a_post_as_at_donor_payment` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_nil_payday_does_not_extend_a_later_real_paydays_verdict` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_nil_donor_does_not_suppress_the_unrecorded_item_4_caveat` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_supplied_2029_calendar_produces_a_real_verdict` | Frozen replay in `parity.rs` |
| `test_integration.py::test_exposure_past_the_horizon_leaves_days_late_blank_and_labels_a_maximum` | Frozen replay in `parity.rs` |
| `test_integration.py::test_days_late_inside_the_horizon_is_still_a_definite_number` | Frozen replay in `parity.rs` |
| `test_integration.py::test_days_late_blank_reaches_the_report_csv` | Frozen replay in `parity.rs` |
| `test_integration.py::test_missed_new_starter_flag_is_suggested_on_late_lines` | Frozen replay in `parity.rs` |
| `test_integration.py::test_prepayment_window_is_leap_safe` | Frozen replay in `parity.rs` |
| `test_integration.py::test_identical_rows_are_flagged_as_duplicates` | Frozen replay in `parity.rs` |
| `test_integration.py::test_console_shows_totals_and_caveats` | Frozen replay in `parity.rs` |
| `test_integration.py::test_at_risk_caveats_reach_the_console` | Frozen replay in `parity.rs` |
| `test_integration.py::test_remittance_only_import_check_exits_nonzero_until_confirmed` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_file_with_any_fund_receipt_is_not_remittance_only` | Frozen replay in `parity.rs` |
| `test_integration.py::test_receipts_after_the_as_at_date_are_still_remittance_only` | Frozen replay in `parity.rs` |
| `test_integration.py::test_the_universal_at_risk_caveat_does_not_fill_the_listing` | Frozen replay in `parity.rs` |
| `test_integration.py::test_the_at_risk_block_names_every_row_and_every_caveat_it_prints` | Unit test `the_at_risk_block_names_every_row_and_every_caveat_it_prints` in `src/report.rs` |
| `test_integration.py::test_the_at_risk_truncation_notice_counts_only_flagged_rows` | Unit test `the_at_risk_truncation_notice_counts_only_flagged_rows` in `src/report.rs` |
| `test_integration.py::test_report_columns_add_up` | Frozen replay in `parity.rs` |
| `test_integration.py::test_employee_id_that_looks_like_a_formula_is_neutralised` | Frozen replay in `parity.rs` |
| `test_integration.py::test_every_formula_lead_is_neutralised_after_whitespace` | Frozen replay in `parity.rs` |
| `test_integration.py::test_sentinel_high_date_is_refused_cleanly` | `runs_without_an_as_at_date_keep_their_refusals` in `tests/cli.rs` |
| `test_integration.py::test_days_late_is_not_truncated_by_the_assessment_date` | Frozen replay in `parity.rs` |
| `test_integration.py::test_stale_prepayment_gets_no_new_starter_hint` | Frozen replay in `parity.rs` |
| `test_integration.py::test_new_starter_hint_wording_matches_the_verdict_it_would_produce` | Frozen replay in `parity.rs` |
| `test_integration.py::test_new_starter_hint_promises_no_verdict_on_partial_evidence` | Frozen replay in `parity.rs` |
| `test_integration.py::test_zero_amount_line_with_late_payment_dates_is_not_exposure` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_caveat_only_fires_on_an_aligned_line` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_caveat_names_the_deadline_without_the_alignment` | Frozen replay in `parity.rs` |
| `test_integration.py::test_item4_caveat_keeps_an_out_of_cycle_lines_own_deadline` | Frozen replay in `parity.rs` |
| `test_integration.py::test_report_csv_carries_a_bom_and_round_trips_a_non_ascii_id` | Frozen replay in `parity.rs` |
| `test_integration.py::test_formula_leading_employee_ids_are_quoted_in_the_report` | Frozen replay in `parity.rs` |
| `test_integration.py::test_input_caveats_are_quoted_in_the_report` | Frozen replay in `parity.rs` |
| `test_integration.py::test_cli_rejects_an_absurd_as_at_date` | Frozen replay in `parity.rs` |
| `test_integration.py::test_the_csv_marks_an_unassessable_row_apart_from_a_nil_one` | Frozen replay in `parity.rs` |
| `test_integration.py::test_an_unpaid_row_does_not_claim_no_date_when_one_was_supplied` | Frozen replay in `parity.rs` |
| `test_integration.py::test_a_not_yet_due_row_does_not_claim_no_date_when_one_was_supplied` | Frozen replay in `parity.rs` |
| `test_integration.py::test_check_reports_error_when_figures_outgrow_the_decimal_context` | Frozen replay in `parity.rs` |
| `test_integration.py::test_check_reports_error_when_totals_outgrow_the_decimal_context` | Frozen replay in `parity.rs` |
| `test_integration.py::test_check_catches_arithmetic_error_from_assessment` | Frozen replay in `parity.rs` |
| `test_integration.py::test_defaulted_as_at_is_named_in_a_notice` | `a_defaulted_as_at_is_named_in_a_notice` in `tests/cli.rs` |
| `test_integration.py::test_an_explicit_as_at_prints_no_default_notice` | Frozen replay in `parity.rs` |
| `test_join_caveats.py::test_each_trigger_writes_its_warning_into_the_canonical_file` | Frozen replay in `parity.rs` |
| `test_join_caveats.py::test_clean_join_writes_no_join_caveats` | Frozen replay in `parity.rs` |
| `test_join_caveats.py::test_duplicate_names_merge_and_the_file_says_how` | Frozen replay in `parity.rs` |
| `test_join_caveats.py::test_join_caveats_survive_the_reader_and_reach_the_report` | Frozen replay in `parity.rs` |
| `test_join_caveats.py::test_competing_identical_paydays_stay_refused` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_register_lists_every_documented_entry` | `the_limitations_register_names_its_entries_and_conditions` in `tests/docs.rs` |
| `test_limitations.py::test_psc_1_extrapolates_past_its_last_quarter_only_on_request` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_psc_1_only_the_exposure_estimate_reads_the_gic_rate` | `psc_1_only_the_exposure_estimate_reads_the_gic_rate` in `tests/docs.rs` |
| `test_limitations.py::test_psc_1_names_every_verdict_and_the_two_that_carry_the_estimate` | `the_limitations_register_names_its_entries_and_conditions` in `tests/docs.rs` |
| `test_limitations.py::test_psc_1_uplift_is_a_range_not_a_flat_rate` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_psc_1_both_exposure_totals_move_with_the_notional_earnings` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_psc_2_join_declares_each_degraded_match` | `psc_2_join_declares_each_degraded_match` in `tests/docs.rs` |
| `test_limitations.py::test_psc_2_register_names_the_same_four_conditions` | `the_limitations_register_names_its_entries_and_conditions` in `tests/docs.rs` |
| `test_openaccountants_handoff.py::test_document_matches_the_engine_run` | `the_openaccountants_handoff_matches_the_engine_run` in `tests/docs.rs` |
| `test_practitioner_pack.py::test_contract_is_deliberately_pinned_to_the_report_writer` | Unit test `the_contract_is_pinned_to_the_report_writer` in `src/practitioner_pack.rs`, against the consumer's own literal header |
| `test_practitioner_pack.py::test_pack_is_deterministic_private_by_default_and_source_bound` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_no_exception_pack_still_requires_human_signoff` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_malformed_or_internally_inconsistent_reports_fail_closed` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_employee_identifier_note_is_not_a_terminal_note` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_note_only_report_is_rejected_instead_of_rendering_no_exceptions` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_terminal_note_requires_non_whitespace_provenance` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_malformed_terminal_note_with_data_fields_fails_closed` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_duplicate_or_changed_headers_fail_closed` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_default_practitioner_workpaper_is_ignored` | `the_review_pack_confinement_is_documented_and_its_default_output_ignored` in `tests/docs.rs`; `MANIFEST.in` is removed with the Python package |
| `test_practitioner_pack.py::test_output_is_atomic_and_replaces_the_link_not_its_target` | `the_practitioner_pack_replaces_a_symlink_rather_than_its_target` in `tests/cli.rs`, skipped where symlinks cannot be created; its engine calls are also frozen in `parity.rs` |
| `test_practitioner_pack.py::test_cli_writes_pack_and_uses_attention_exit_code` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_cli_refuses_input_output_collision_and_wrong_suffix` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_cli_refuses_input_output_collision_through_a_symlink` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_relative_report_path_cannot_leave_cwd` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_rejected_report_path_names_the_remedy` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_review_pack_path_confinement_is_documented` | `the_review_pack_confinement_is_documented_and_its_default_output_ignored` in `tests/docs.rs` |
| `test_practitioner_pack.py::test_report_path_must_be_csv` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_every_report_field_is_checked_before_a_pack_is_written` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_an_exposed_row_with_the_charge_estimate_withheld_is_accepted` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_the_report_path_must_be_a_csv_file_name` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_a_report_that_is_not_utf8_is_refused` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_a_report_the_strict_csv_reader_refuses_is_malformed` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_a_projected_report_row_must_keep_every_field` | Frozen replay in `parity.rs` |
| `test_practitioner_pack.py::test_a_report_row_missing_fields_is_refused` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_normalise_header_folds_case_space_and_punctuation` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_normalise_header_keeps_digits` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_normalise_header_folds_nbsp_and_tab_as_whitespace` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_sg_filter_include_as_a_string_raises_csv_error` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_date_formats_as_a_string_raises_csv_error` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_verified_as_a_string_raises_csv_error` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_all_eight_profiles_load` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_super_profiles_can_isolate_super_guarantee` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_payroll_profiles_map_a_payday` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_profile_data_is_declared_as_package_data` | Not applicable: the profiles are compiled into the binary; Unit test `the_bundled_profile_list_is_the_folder` in `src/profiles.rs` keeps the compiled list and the folder in step |
| `test_profiles.py::test_load_profiles_returns_both_roles` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_load_profiles_filters_by_role` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_every_shipped_profile_is_marked_unverified` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_score_is_none_when_a_signature_heading_is_missing` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_detect_picks_the_profile_whose_signature_fits` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_detect_refuses_when_nothing_matches_and_names_what_was_wanted` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_detect_refuses_a_tie_rather_than_guessing` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_vendor_override_skips_detection` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_vendor_override_rejects_an_unknown_key` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_an_unknown_vendor_is_not_advised_to_type_a_vendor_it_never_mentioned` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_a_vendors_own_key_worn_too_long_is_told_the_stem_it_shares` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_vendor_prefix_matching_exactly_one_profile_resolves_it` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_vendor_prefix_matching_more_than_one_profile_raises_naming_both` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_vendor_exact_key_wins_over_a_prefix_collision` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_detect_picks_the_strictly_higher_scorer_among_several_live_profiles` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_resolve_columns_returns_the_actual_heading` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_shipped_beam_status_ladder_is_fully_classified` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_only_the_employment_hero_super_profile_classifies_a_status` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_remitted_status_list_as_a_string_raises_csv_error` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_remitted_status_with_a_status_in_both_lists_raises_csv_error` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_a_malformed_profile_is_refused_with_its_reason` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_a_profile_without_a_role_is_refused` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_an_unknown_role_is_refused` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_a_profile_file_that_is_not_a_json_object_is_refused` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_two_profiles_with_one_key_are_refused` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_blank_and_repeated_headings_resolve_to_the_first_real_one` | Frozen replay in `parity.rs` |
| `test_profiles.py::test_a_long_vendor_name_is_advised_to_use_its_shared_stem` | Frozen replay in `parity.rs` |
| `test_properties.py::test_business_day_addition_counts_only_days_after_the_start` | `business_day_addition_counts_only_days_after_the_start` in `tests/properties.rs` |
| `test_properties.py::test_exposure_scenarios_conserve_the_shortfall_plus_nec_base` | `exposure_scenarios_conserve_the_shortfall_plus_nec_base` in `tests/properties.rs` |
| `test_quick_proof.py::test_renderer_check_accepts_the_committed_assets` | `the_committed_transcript_is_the_current_run` in `tests/quick_proof.rs` |
| `test_quick_proof.py::test_proof_is_tied_to_the_fabricated_cli_run` | `the_committed_transcript_is_the_current_run` in `tests/quick_proof.rs` |
| `test_quick_proof.py::test_readme_places_the_proof_and_check_before_install` | `the_readme_places_the_proof_and_its_check_before_the_build` in `tests/quick_proof.rs` |
| `test_rates.py::test_a_good_table_still_loads` | Frozen replay in `parity.rs` |
| `test_rates.py::test_unsorted_contiguous_quarters_are_sorted_and_accepted` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_reversed_interval_is_refused` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_non_contiguous_intervals_name_both_ranges` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_an_unreadable_rate_raises_rates_error_naming_the_entry` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_non_finite_rate_is_refused` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_negative_rate_is_refused` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_rate_above_the_ceiling_is_refused` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_zero_rate_is_accepted` | Frozen replay in `parity.rs` |
| `test_rates.py::test_the_ceiling_itself_is_accepted` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_missing_quarters_key_is_named` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_list_at_the_top_level_is_refused` | Frozen replay in `parity.rs` |
| `test_rates.py::test_quarters_must_be_a_list` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_table_that_is_not_json_is_a_rates_error_naming_the_file` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_the_cli_prints_a_top_level_rates_error_without_a_traceback` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_missing_rate_key_is_named` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_null_rate_is_refused` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_an_unreadable_quarter_date_is_named` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_the_cli_prints_a_rate_error_without_a_traceback` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_day_past_the_last_quarter_is_refused_by_default` | Frozen replay in `parity.rs` |
| `test_rates.py::test_the_cli_reports_a_row_the_gic_table_cannot_reach` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_row_past_the_table_keeps_its_verdict_and_loses_only_the_estimate` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_table_that_starts_after_the_period_still_fails_the_run` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_period_inside_the_table_is_unaffected` | Frozen replay in `parity.rs` |
| `test_rates.py::test_a_withheld_estimate_does_not_also_claim_the_rate_was_carried_forward` | Frozen replay in `parity.rs` |
| `test_rates.py::test_an_empty_gic_table_is_refused` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_gic_quarter_that_is_not_an_object_is_refused` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_rates.py::test_a_rates_file_that_is_not_an_object_is_refused` | Unit test `malformed_tables_are_refused_as_python_refused_them` in `src/rates.rs`, which holds every message to Python 0.2.2's for the same table |
| `test_receipt_amount_evaluation.py::test_scenario_reproduces_the_declared_result` | Frozen replay in `parity.rs` |
| `test_receipt_amount_evaluation.py::test_refusal_exits_before_assessment` | Frozen replay in `parity.rs` |
| `test_receipt_amount_evaluation.py::test_every_fixture_is_declared_once` | `every_fixture_is_declared_once_and_fabricated` in `tests/evaluation.rs` |
| `test_receipt_amount_evaluation.py::test_contract_keeps_its_recorded_release_and_pending_review` | `the_receipt_amount_guide_matches_its_contract` in `tests/docs.rs` |
| `test_receipt_amount_evaluation.py::test_readme_table_matches_the_contract` | `the_receipt_amount_guide_matches_its_contract` in `tests/docs.rs` |
| `test_receipt_evidence.py::test_a_timely_receipt_with_no_amount_is_unknown_not_on_time` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_a_timely_receipt_with_no_amount_before_the_due_date_passes_is_still_unknown` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_an_explicit_full_amount_is_on_time` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_a_ten_column_part_payment_evidences_its_own_amount` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_an_explicit_partial_amount_is_not_on_time` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_a_late_receipt_with_no_amount_is_late_and_keeps_the_whole_shortfall` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_a_prepayment_with_no_amount_inside_the_window_is_unknown` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_a_receipt_after_the_due_date_with_a_possible_item4_extension_stays_unknown` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_contradictory_amounts_are_still_refused` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_two_receipts_for_one_payday_are_assessed_on_their_own_amounts` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_the_new_starter_narrative_names_the_missing_amount` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_cli_leaves_a_bare_receipt_row_unassessed_and_exits_two` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_cli_accepts_the_same_row_once_the_amount_is_stated` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_evidence_pack_carries_the_missing_amount_caveat` | Frozen replay in `parity.rs` |
| `test_receipt_evidence.py::test_a_bare_receipt_after_a_past_horizon_deadline_keeps_the_horizon_caveat` | Frozen replay in `parity.rs` |
| `test_remittance_only_example.py::test_every_line_is_remitted_with_no_fund_receipt` | `the_remittance_only_example_isolates_its_boundary` in `tests/docs.rs` |
| `test_remittance_only_example.py::test_paydays_sit_after_the_transition_period` | `the_remittance_only_example_isolates_its_boundary` in `tests/docs.rs` |
| `test_remittance_only_example.py::test_unconfirmed_run_exits_two_and_asks_for_confirmation` | Frozen replay in `parity.rs` |
| `test_remittance_only_example.py::test_confirmed_run_exits_zero_and_records_the_confirmation` | Frozen replay in `parity.rs` |
| `test_remittance_only_example.py::test_confirming_does_not_promote_any_line_to_on_time` | Frozen replay in `parity.rs` |
| `test_remittance_only_example.py::test_both_runs_say_remittance_does_not_prove_receipt` | Frozen replay in `parity.rs` |
| `test_remittance_only_example.py::test_readme_documents_both_commands_and_their_boundary_text` | `the_remittance_only_example_isolates_its_boundary` in `tests/docs.rs` |
| `test_report_invariants.py::test_report_invariants` | Unit test `report_invariants_hold_whatever_the_build` in `src/report.rs`; the `python -O` half is not applicable, as Rust has no assertion-stripping mode |
| `test_sgc.py::test_daily_rate_divides_by_365_in_a_non_leap_year` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_rate_table_covers_prior_quarter` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_published_october_december_2026_rate_and_accrual` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_staleness_warns_past_the_shipped_table` | Unit test `staleness_warns_past_the_shipped_table` in `src/rates.rs` |
| `test_sgc.py::test_no_accrual_before_the_deadline_passes` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_accrual_starts_the_day_after_the_deadline` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_accrual_compounds_daily` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_accrual_uses_the_rate_of_each_quarter` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_negative_shortfall_rejected` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_uplift_matrix_percentages` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_uplift_applies_to_shortfall_plus_notional_earnings` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_exposure_range_spans_best_and_worst_uplift` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_lcr_2026_d3_accrual_boundary` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_daily_rate_uses_366_in_a_leap_year` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_daily_rate_before_the_table_raises` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_daily_rate_past_the_table_is_refused_unless_the_caller_asks` | Frozen replay in `parity.rs` |
| `test_sgc.py::test_notional_earnings_keeps_the_active_precision_rather_than_an_exact_figure` | Frozen replay in `parity.rs` |
| `test_workbook.py::test_workbook_is_macro_free_and_carries_no_build_path` | Moved to the repository's `tests/test_payday_workbook.py` |
| `test_workbook.py::test_charge_estimate_is_withheld_past_the_gic_table` | Moved to the repository's `tests/test_payday_workbook.py` |
| `test_workbook.py::test_cached_values_were_calculated_by_desktop_excel` | Moved to the repository's `tests/test_payday_workbook.py` |
| `test_workbook.py::test_summary_does_not_total_withheld_estimates_as_zero` | Moved to the repository's `tests/test_payday_workbook.py` |
| `test_workbook.py::test_sample_matches_the_engine_line_by_line` | Moved to the repository's `tests/test_payday_workbook.py`, reading the engine answer `workbook.rs` holds in `tests/data/workbook-sample.json` |
| `test_workbook.py::test_holiday_and_gic_tables_match_the_shipped_data` | Moved to the repository's `tests/test_payday_workbook.py` |
| `test_workbook.py::test_input_columns_are_formatted_so_pasted_values_stay_inert` | Moved to the repository's `tests/test_payday_workbook.py` |
| `test_workbook.py::test_every_register_date_is_read_as_a_whole_calendar_day` | Moved to the repository's `tests/test_payday_workbook.py`, then to `examples/build_workbook/builder.rs` with the builder |
| `test_workbook.py::test_a_blank_register_date_stays_blank_through_the_whole_day_columns` | Moved to the repository's `tests/test_payday_workbook.py`, then to `examples/build_workbook/builder.rs` with the builder |
| `test_workbook.py::test_the_register_refuses_the_dates_and_amounts_the_checker_refuses` | Moved to the repository's `tests/test_payday_workbook.py`, then to `examples/build_workbook/builder.rs` with the builder |
| `test_workbook.py::test_amount_invariants_compare_cents_not_raw_input` | Moved to the repository's `tests/test_payday_workbook.py`, then to `examples/build_workbook/builder.rs` with the builder |
| `test_workbook.py::test_the_gic_table_holds_only_published_contiguous_quarters` | Moved to the repository's `tests/test_payday_workbook.py`, then to `examples/build_workbook/builder.rs` with the builder |
| `test_workbook.py::test_a_bare_receipt_date_is_left_unknown_in_the_workbook` | Moved to the repository's `tests/test_payday_workbook.py`, then to `examples/build_workbook/builder.rs` with the builder |
