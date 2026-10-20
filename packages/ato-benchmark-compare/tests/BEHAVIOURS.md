# Python behaviour coverage

Base 83288b0, Python 0.2.2 (identical to the PyPI 0.2.2 release): 303 test
functions. The archive pytest suite passed before the differential harness
captured the top-level engine calls and command runs they made, with the files
each named. `tests/data/python-cases.json` freezes the first 60 distinct cases
of each test plus 2,800 generated API calls and command runs: 3,818 cases in
all. `tests/parity.rs` replays them and compares complete results, key order,
output bytes and written files included. Results and streams over 4 KiB are
stored as an FNV-1a fingerprint and length.

The generated cases include malformed decimal strings on the API's decimal
fields, the CSV field limit at and past 131,072 characters, argparse prefix,
help and missing-value combinations, and colour controls side by side
(`FORCE_COLOR` with `PYTHON_COLORS=0` or `NO_COLOR`).

Tests whose checks made no engine call, or more than 60, run natively, as do tests
that injected a failure with `monkeypatch` (a forced digest collision, a failing
`os.replace` or `os.fsync`): replaying their inputs cannot recreate the failure.
A `review` group adds every counterexample from the final review. Each frozen case
records the complete file state after its run: what it wrote, the inputs it left
unchanged, and that nothing else exists. The
property tests keep their seeds, examples and input domains; a local SplitMix64
replaces fast-check's generator, so the random cases differ.

| Original test | Rust coverage |
| --- | --- |
| `test_atomic_io.py::test_failed_json_replacement_keeps_the_reviewed_output` | Unit test in `src/atomic.rs` |
| `test_atomic_io.py::test_failed_mapping_generation_never_truncates_the_prior_mapping` | Unit test in `src/atomic.rs` covers a failed replacement; a failed flush takes the same cleanup path |
| `test_atomic_io.py::test_exception_inside_staged_writer_leaves_no_partial_output` | Unit test in `src/atomic.rs`; the Rust writer stages the whole text at once, so no caller code runs mid-write |
| `test_atomic_io.py::test_a_staged_file_already_gone_does_not_mask_the_failure` | Unit test in `src/atomic.rs` covers the failed replacement; the cleanup ignores a staged file already gone, so the rename error is the one reported |
| `test_benchmark_history.py::test_reclassification_is_separate_from_unchanged_business` | Frozen replay in `parity.rs`; the example's own tests are in `examples/benchmark_history.rs` |
| `test_benchmark_history.py::test_unsupported_industry_gets_no_invented_range` | Frozen replay in `parity.rs`; the example's own tests are in `examples/benchmark_history.rs` |
| `test_benchmark_history.py::test_unavailable_counterfactual_preserves_standalone_results` | Frozen replay in `parity.rs`; the example's own tests are in `examples/benchmark_history.rs` |
| `test_benchmark_history.py::test_expense_signs_are_explicit_for_each_snapshot` | Frozen replay in `parity.rs`; the example's own tests are in `examples/benchmark_history.rs` |
| `test_cli.py::test_industries_lists_every_business_type` | Frozen replay in `parity.rs` |
| `test_cli.py::test_industries_search` | Frozen replay in `parity.rs` |
| `test_cli.py::test_industries_search_with_no_match` | Frozen replay in `parity.rs` |
| `test_cli.py::test_show` | Frozen replay in `parity.rs` |
| `test_cli.py::test_show_reports_a_ratio_the_ato_does_not_publish` | Frozen replay in `parity.rs` |
| `test_cli.py::test_buckets` | Frozen replay in `parity.rs` |
| `test_cli.py::test_worked_example_runs_clean` | Frozen replay in `parity.rs` |
| `test_cli.py::test_json_output_to_stdout` | Frozen replay in `parity.rs` |
| `test_cli.py::test_json_output_to_a_file` | Frozen replay in `parity.rs` |
| `test_cli.py::test_map_then_compare_round_trip` | Frozen replay in `parity.rs` |
| `test_cli.py::test_compare_reports_the_rows_it_left_out_of_the_totals` | Frozen replay in `parity.rs` |
| `test_cli.py::test_accept_unreviewed_clears_the_review_exit` | Frozen replay in `parity.rs` |
| `test_cli.py::test_map_does_not_silently_bucket_cost_of_sales_as_turnover` | Frozen replay in `parity.rs` |
| `test_cli.py::test_map_will_not_overwrite_a_reviewed_mapping` | Frozen replay in `parity.rs` |
| `test_cli.py::test_unmapped_account_blocks_the_comparison` | Frozen replay in `parity.rs` |
| `test_cli.py::test_outside_the_key_range_exits_two` | Frozen replay in `parity.rs` |
| `test_cli.py::test_outside_exit_needs_the_turnover_basis_evidenced` | Frozen replay in `parity.rs` |
| `test_cli.py::test_suggested_only_bucket_is_not_evidence` | Frozen replay in `parity.rs` |
| `test_cli.py::test_no_income_text_withholds_ranges_and_dependent_notes` | Frozen replay in `parity.rs` |
| `test_cli.py::test_routing_notes_reach_the_json_payload` | Frozen replay in `parity.rs` |
| `test_cli.py::test_flip_expense_signs` | Frozen replay in `parity.rs` |
| `test_cli.py::test_w1_is_applied_to_the_labour_ratio` | Frozen replay in `parity.rs` |
| `test_cli.py::test_bad_w1_is_reported` | Frozen replay in `parity.rs` |
| `test_cli.py::test_unknown_industry_is_reported` | Frozen replay in `parity.rs` |
| `test_cli.py::test_previous_benchmark_year_can_be_selected` | Frozen replay in `parity.rs` |
| `test_cli.py::test_map_then_compare_keeps_guarded_and_apostrophe_accounts_distinct` | Frozen replay in `parity.rs` |
| `test_cli.py::test_map_fails_when_distinct_accounts_share_a_forced_digest` | Unit test in `src/mapping.rs` with a forced key, through the command line |
| `test_cli.py::test_map_still_collapses_normalisation_equivalent_accounts` | Frozen replay in `parity.rs` |
| `test_cli.py::test_map_says_repeated_names_must_be_fixed_in_the_export` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_profit_and_loss_that_is_not_utf8_is_reported_as_an_error` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_mapping_that_is_not_utf8_is_reported_as_an_error` | Frozen replay in `parity.rs` |
| `test_cli.py::test_an_amount_too_large_to_format_is_reported_as_an_error` | Frozen replay in `parity.rs` |
| `test_cli.py::test_an_amount_too_large_to_ratio_is_reported_as_an_error` | Frozen replay in `parity.rs` |
| `test_cli.py::test_a_quoted_crlf_in_an_account_name_reaches_the_reader_intact` | Frozen replay in `parity.rs` |
| `test_cli.py::test_an_export_whose_rows_end_in_a_bare_cr_is_read` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_reported_byte_position_counts_from_the_start_of_the_file` | Frozen replay in `parity.rs` |
| `test_cli.py::test_comparison_rejects_a_forced_digest_collision_before_routing_amounts` | Unit test in `src/mapping.rs` with a forced key (`route_rejects_a_hash_collision_before_routing_amounts`) |
| `test_cli.py::test_map_lists_ten_unreadable_rows_and_counts_the_rest` | Frozen replay in `parity.rs` |
| `test_cli.py::test_compare_lists_ten_unreadable_rows_and_counts_the_rest` | Frozen replay in `parity.rs` |
| `test_cli.py::test_show_names_no_industry_page_for_a_year_without_page_ranges` | Frozen replay in `parity.rs` |
| `test_cli.py::test_the_overwrite_guard_skips_an_input_that_was_not_given` | Unit test in `src/cli.rs` |
| `test_cli.py::test_map_refuses_when_an_input_path_cannot_be_resolved` | Not reproduced: it monkeypatches `Path.resolve` to fail; Python's non-strict resolve did not fail on a symlink loop (checked on Python 3.14), and the Rust guard falls back the same way there; other resolution failures, such as a removed working directory, were not compared |
| `test_cli.py::test_map_into_a_folder_that_is_a_file_exits_one_with_one_error_line` | Frozen replay in `parity.rs` |
| `test_cli.py::test_streams_that_cannot_be_reconfigured_still_receive_the_output` | Not applicable: the binary writes bytes to its streams with no reconfigure step |
| `test_cli.py::test_python_dash_m_runs_the_command_line` | Not applicable: there is no Python module to run |
| `test_csv_guard_whitespace.py::test_guard_quotes_formula_after_leading_whitespace` | Frozen replay in `parity.rs` |
| `test_csv_guard_whitespace.py::test_guard_preserves_ordinary_text` | Frozen replay in `parity.rs` |
| `test_csv_guard_whitespace.py::test_guard_quotes_a_leading_control_character` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_both_years_ship_and_load` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_default_year_is_the_most_recent` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_bakery_matches_the_published_ato_page` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_service_industry_has_no_cost_of_sales_benchmark` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_two_band_industry_is_handled` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_band_selection_covers_the_printed_boundaries` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_turnover_below_the_lowest_band_has_no_band` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_bands_do_not_overlap_anywhere_in_either_year` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_every_published_range_is_plausible` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_key_ratio_follows_whether_cost_of_sales_is_published` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_search_is_punctuation_and_case_insensitive` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_ambiguous_name_lists_the_candidates` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_unknown_name_points_at_the_industries_command` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_unknown_year_lists_what_is_available` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_future_schema_version_is_refused` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_non_object_business_type_is_refused` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_non_object_turnover_band_is_refused` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_reversed_range_is_refused` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_provenance_is_recorded` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_ratios_are_exact_decimals_not_floats` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_band_requires_identity_and_explicit_boolean` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_a_malformed_dataset_is_refused_with_its_reason` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_a_dataset_that_is_not_a_json_object_is_refused` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_canonical_digest_refuses_text_that_is_not_json` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_a_folder_without_benchmark_files_is_refused` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_a_missing_data_folder_offers_no_years` | Frozen replay in `parity.rs` |
| `test_dataset.py::test_names_lists_every_business_type_in_dataset_order` | Frozen replay in `parity.rs` |
| `test_diff_coverage_policy.py::test_changed_line_coverage_is_scoped_and_fail_closed` | Not applicable: the Python coverage gate is removed |
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
| `test_guards.py::test_section_heading_carrying_its_own_total_is_not_an_account` | Frozen replay in `parity.rs` |
| `test_guards.py::test_quoted_newline_in_an_account_name_does_not_split_the_row` | Frozen replay in `parity.rs` |
| `test_guards.py::test_map_refuses_to_write_over_its_own_input` | Frozen replay in `parity.rs` |
| `test_guards.py::test_compare_refuses_to_write_json_over_the_mapping` | Frozen replay in `parity.rs` |
| `test_guards.py::test_repeated_account_names_block_the_run` | Frozen replay in `parity.rs` |
| `test_guards.py::test_mapping_rows_that_match_nothing_are_reported` | Frozen replay in `parity.rs` |
| `test_guards.py::test_subtotal_mapped_to_a_real_bucket_is_called_out` | Frozen replay in `parity.rs` |
| `test_guards.py::test_money_never_prints_negative_zero` | Frozen replay in `parity.rs` |
| `test_guards.py::test_no_runtime_module_imports_anything_outside_the_standard_library` | Replaced by the repository's `tests/test_rust_engine.py`, which limits the crate's dependencies |
| `test_guards.py::test_amount_column_is_refused_where_it_cannot_apply` | Frozen replay in `parity.rs` |
| `test_guards.py::test_trailing_credit_marker` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_register_lists_every_documented_entry` | `docs.rs` |
| `test_limitations.py::test_abc_1_serialisers_diverge_on_the_key_ratio` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_library_payload_stays_internally_consistent` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_unsupplied_ratios_are_a_figure_in_one_payload_and_withheld_in_the_other` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_parity_also_needs_both_income_fields` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_labour_parity_also_needs_associated_persons_once_w1_is_given` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_trigger_is_an_absent_bucket_not_an_unmapped_account` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_mapping_error_lists_at_most_twenty_rows` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_benchmark_range_survives_a_withheld_value` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_cli_serialiser_cannot_see_which_buckets_were_supplied` | `docs.rs`: the `to_dict` output ignores `output_supplied` and the evidenced output reads it |
| `test_limitations.py::test_abc_1_fallback_note_is_still_emitted` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_1_compare_withholds_the_verdict_for_a_direct_library_caller` | Frozen replay in `parity.rs` |
| `test_limitations.py::test_abc_2_a_single_figure_range_reads_outside_but_never_sets_the_exit_code` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_suggestions` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_unmatched_account_without_a_section_needs_review` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_section_defaults_catch_unmatched_accounts` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_income_wording_in_an_expense_section_is_sent_for_review` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_every_suggested_bucket_is_a_real_bucket` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_round_trip` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_written_account_names_cannot_become_formulas` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_review_marker_blocks_the_run` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_unknown_bucket_is_refused` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_empty_bucket_is_refused` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_duplicate_account_is_refused` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_missing_column_names_what_is_missing` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_byte_order_mark_and_odd_case_headers_are_tolerated` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_missing_file_is_reported` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_account_key_ignores_case_and_spacing` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_keyed_round_trip_preserves_formula_guarding_and_logical_identity` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_formula_and_genuine_apostrophe_accounts_keep_distinct_keys` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_read_then_write_does_not_change_guarded_account_identity` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_keyed_mapping_rejects_display_that_does_not_match_the_key` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_keyed_mapping_rejects_blank_or_noncanonical_key` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_keyed_file_does_not_fall_back_to_legacy_for_a_blank_row_key` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_populated_mapping_row_with_empty_logical_account_is_rejected` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_write_rejects_empty_logical_account_before_replacing_existing_file` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_case_and_spacing_only_keyed_display_edit_is_accepted` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_safe_legacy_mapping_remains_compatible` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_legacy_formula_or_ambiguous_guarded_account_requires_regeneration` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_malformed_keyed_csv_shape_is_rejected` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_populated_unnamed_mapping_column_is_rejected` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_named_mapping_extension_column_remains_accepted` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_extra_empty_trailing_mapping_cells_remain_accepted` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_normalisation_equivalent_mapping_accounts_are_duplicates` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_composed_and_decomposed_unicode_accounts_remain_distinct` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_write_rejects_a_forced_digest_collision` | Unit test in `src/mapping.rs` with a forced key |
| `test_mapping.py::test_read_rejects_a_forced_digest_collision` | Unit test in `src/mapping.rs` with a forced key |
| `test_mapping.py::test_a_spreadsheet_coerced_numeric_account_is_refused_not_silently_rematched` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_a_mistyped_source_is_refused_not_read_as_reviewed` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_a_blank_source_cell_is_refused_not_defaulted_to_reviewed` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_a_mistyped_header_is_refused_not_read_as_legacy` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_an_empty_mapping_file_is_refused` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_a_mapping_with_a_header_and_no_rows_is_refused` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_blank_mapping_rows_are_passed_over` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_a_keyed_account_that_really_starts_with_an_apostrophe_is_read_as_written` | Frozen replay in `parity.rs` |
| `test_mapping.py::test_a_key_both_spellings_of_a_guarded_account_match_is_refused_as_ambiguous` | Unit test in `src/mapping.rs` with a forced key |
| `test_mapping_workflows.py::test_suggest_mapping_returns_ordered_rows_duplicates_and_review_count` | Frozen replay in `parity.rs` |
| `test_mapping_workflows.py::test_suggest_mapping_rejects_a_hash_collision_before_returning_a_draft` | Unit test in `src/mapping.rs` with a forced key |
| `test_mapping_workflows.py::test_route_returns_exact_totals_review_count_and_ordered_notes` | Frozen replay in `parity.rs` |
| `test_mapping_workflows.py::test_route_preserves_the_exact_missing_account_error` | Frozen replay in `parity.rs` |
| `test_mapping_workflows.py::test_route_preserves_the_exact_repeated_account_error` | Frozen replay in `parity.rs` |
| `test_mapping_workflows.py::test_route_rejects_a_hash_collision_before_routing_amounts` | Unit test in `src/mapping.rs` with a forced key |
| `test_mapping_workflows.py::test_mapping_module_owns_suggestion_and_routing_workflows` | Not applicable: Python module layout |
| `test_mapping_workflows.py::test_routed_totals_carry_the_buckets_a_reviewed_account_reached` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_amount_accepts` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_amount_rejects` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_amount_refuses_malformed_grouping_and_conflicting_signs` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_amount_keeps_the_ordinary_grouped_forms` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_amount_reports_where` | Frozen replay in `parity.rs` |
| `test_money.py::test_money_formats_with_separators` | Frozen replay in `parity.rs` |
| `test_money.py::test_percent_keeps_two_places` | Frozen replay in `parity.rs` |
| `test_money.py::test_formatting_a_value_past_the_decimal_context_is_refused` | Frozen replay in `parity.rs` |
| `test_money.py::test_percent_compact_trims_published_bounds` | Frozen replay in `parity.rs` |
| `test_money.py::test_guard_escapes_formula_starts` | Frozen replay in `parity.rs` |
| `test_money.py::test_guard_leaves_data_alone` | Frozen replay in `parity.rs` |
| `test_money.py::test_guard_escapes_leading_whitespace_control` | Frozen replay in `parity.rs` |
| `test_money.py::test_all_formatters_remove_negative_zero_only` | Frozen replay in `parity.rs` |
| `test_money.py::test_parse_amount_refuses_a_missing_cell` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_the_builder_reads_every_layout_the_pages_use` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool) |
| `test_page_ranges.py::test_the_builder_refuses_a_range_it_cannot_read` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool) |
| `test_page_ranges.py::test_the_shipped_page_file_matches_the_shipped_workbook` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool) |
| `test_page_ranges.py::test_the_loader_merges_page_ranges_without_touching_the_workbook_ranges` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_the_loader_refuses_a_page_file_that_does_not_belong` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_the_loader_never_lets_a_page_replace_a_workbook_range` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_the_builder_includes_a_page_that_agrees` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool); its loader calls are also frozen in `parity.rs` |
| `test_page_ranges.py::test_the_builder_leaves_out_a_page_that_disagrees` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool); its loader calls are also frozen in `parity.rs` |
| `test_page_ranges.py::test_an_unexpected_exclusion_stops_the_build` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool); its loader calls are also frozen in `parity.rs` |
| `test_page_ranges.py::test_a_mistyped_key_heading_is_recorded_when_its_ranges_agree` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool); its loader calls are also frozen in `parity.rs` |
| `test_page_ranges.py::test_the_builder_reads_a_second_key_range_after_its_averages` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool) |
| `test_page_ranges.py::test_a_page_without_its_reference_is_refused` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool) |
| `test_page_ranges.py::test_the_shipped_page_file_counts` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool) |
| `test_page_ranges.py::test_the_loader_refuses_an_unknown_ratio_key` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_the_page_file_binds_to_the_dataset_file_it_was_checked_against` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_a_year_without_a_page_file_loads_as_before` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_a_single_figure_range_is_compared_as_printed` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_the_payload_marks_page_ranges_and_names_the_page` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_an_excluded_page_leaves_its_industry_as_before` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_show_lists_the_page_ranges_and_their_page` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_an_unrecognised_heading_above_ranges_stops_the_build` | Moved to the repository's `tests/test_ato_page_builder.py` (the builder is a Python tool) |
| `test_page_ranges.py::test_the_loader_refuses_a_page_file_that_does_not_account_for_every_industry` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_the_loader_refuses_a_malformed_page_file` | Frozen replay in `parity.rs` |
| `test_page_ranges.py::test_the_loader_refuses_a_page_file_that_is_not_a_json_object` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_neutral_layout` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_repeated_header_name_is_refused_not_guessed` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_neutral_layout_accepts_a_section_column` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_neutral_layout_refuses_an_unknown_section` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_misaligned_row_is_an_error_not_a_silent_shift` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_trailing_blank_cells_are_tolerated` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_row_missing_its_amount_is_an_error` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_unreadable_amount_in_neutral_layout_is_an_error` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_report_layout_detects_sections_and_totals` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_report_layout_picks_the_first_value_column` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_amount_column_can_be_chosen_by_number` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_amount_column_can_be_chosen_by_heading` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_unknown_amount_column_is_an_error` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_flat_export_keeps_an_account_named_like_a_section` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_retained_section_named_account_carries_its_own_section` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_section_total_printed_on_the_heading_row_is_still_a_total` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_unreadable_detail_cell_keeps_the_heading_a_section_total` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_section_named_account_among_unreadable_neighbours_stays_visible` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_report_layout_records_what_it_skipped` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_empty_file_is_an_error` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_missing_file_is_an_error` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_byte_order_mark_is_tolerated` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_total_rows_are_recognised` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_ordinary_accounts_are_not_mistaken_for_totals` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_section_headings` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_named_account_is_not_treated_as_a_section` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_gap_in_the_current_period_is_refused_not_read_from_last_year` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_other_period_headings_are_refused_too` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_current_period_with_no_readable_amount_is_refused` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_column_of_hash_identifiers_is_text_not_placeholders` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_numeric_label_column_is_never_read_as_the_amounts` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_file_whose_only_numbers_are_labels_has_no_amount_column` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_description_column_left_of_the_amounts_is_passed_over` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_code_column_left_of_the_amounts_is_not_a_period` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_note_column_left_of_two_periods_is_passed_over` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_neutral_layout_passes_over_blank_rows_and_notes_rows_without_a_name` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_neutral_layout_with_no_named_account_is_refused` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_single_column_report_is_refused` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_spacer_column_beside_the_amounts_is_not_a_period` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_column_heading_above_the_amounts_names_the_column` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_headless_placeholder_column_left_of_the_amounts_is_refused` | Unit test in `src/pnl.rs` |
| `test_pnl.py::test_a_section_heading_with_an_unreadable_amount_still_opens_its_section` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_section_total_check_passes_over_blank_and_unlabelled_rows` | Frozen replay in `parity.rs` |
| `test_pnl.py::test_a_report_with_only_unnamed_amounts_is_refused` | Frozen replay in `parity.rs` |
| `test_properties.py::test_case_and_whitespace_variants_keep_one_logical_identity` | Native in `properties.rs` (same seed and examples, 500 runs), plus its first 60 cases frozen |
| `test_properties.py::test_generated_reviewed_mappings_round_trip_without_identity_loss` | Native in `properties.rs` (same seed and domains, 500 runs), plus its first 60 cases frozen |
| `test_ratios.py::test_turnover_uses_sales_when_sales_dominate` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_turnover_falls_back_when_sales_are_under_half_of_total_income` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_sales_at_exactly_half_of_total_income_still_count_as_sales` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_zero_sales_falls_back_to_total_business_income` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_no_income_at_all_is_an_error_not_a_division_by_zero` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_total_expenses_deducts_payments_to_associated_persons` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_cost_of_sales_ratio_excludes_wages_inside_cost_of_sales` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_labour_adds_contractors_without_deducting_associates` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_w1_replaces_salary_and_wages_when_greater` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_w1_is_ignored_when_not_greater` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_w1_is_compared_and_applied_net_of_associates` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_negative_w1_is_refused` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_negative_expense_bucket_is_flagged` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_zero_associated_persons_is_flagged` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_ratio_is_held_to_two_decimal_places_of_a_percentage` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_verdict_boundaries_are_inclusive` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_just_outside_the_boundary_is_reported_as_outside` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_displayed_figure_and_verdict_never_disagree` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_key_ratio_switches_when_no_cost_of_sales_is_mapped` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_turnover_below_every_band_produces_no_verdict` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_service_industry_reports_no_cost_of_sales_benchmark` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_checks_expose_stable_codes_and_required_inputs` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_comparison_notes_expose_stable_codes_and_required_inputs` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_evidenced_dict_withholds_unknown_denominator_and_dependent_prose` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_evidenced_dict_key_ratio_and_row_flag_name_the_same_ratio` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_evidenced_dict_keeps_only_ratios_with_complete_inputs` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_evidenced_dict_filters_checks_by_structured_dependencies` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_optional_w1_is_not_described_as_withholding_ratios` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_compare_withholds_a_ratio_built_on_a_bucket_nobody_supplied` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_compare_keeps_the_explicitly_supplied_set` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_compare_on_zero_filled_totals_reads_every_bucket_as_supplied` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_an_unknown_supplied_field_is_refused_by_compare` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_negative_labour_is_flagged_on_the_mapped_wage_buckets` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_negative_labour_with_w1_also_depends_on_w1_and_associates` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_a_hand_built_figure_set_without_the_key_ratio_has_no_key_verdict` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_turnover_below_every_band_has_no_band_to_compare_against` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_turnover_above_a_capped_top_band_falls_in_no_range` | Frozen replay in `parity.rs` |
| `test_ratios.py::test_text_without_a_supplied_set_prints_every_figure_and_no_empty_notes` | Frozen replay in `parity.rs` |
| `test_release_workflow_hardening.py::test_release_backfill_treats_dispatch_tag_as_validated_data` | Not applicable: the Python release workflow is removed |
| `test_release_workflow_hardening.py::test_release_tag_validator_rejects_shell_shaped_and_malformed_values` | Not applicable: the Python release workflow is removed |
| `test_release_workflow_hardening.py::test_release_uses_the_hardened_shared_policy_contract` | Not applicable: the Python release workflow is removed |
| `test_version.py::test_module_version_matches_distribution_metadata` | Not applicable: the version is `Cargo.toml`'s; `docs.rs` holds the changelog to it |
| `test_version.py::test_checkout_version_ignores_unrelated_metadata` | Not applicable: the version is `Cargo.toml`'s; `docs.rs` holds the changelog to it |
| `test_workbook.py::test_workbook_is_macro_free_and_carries_no_build_path` | Moved to the repository's `tests/test_ato_workbook.py` |
| `test_workbook.py::test_table_formulas_fill_down_when_rows_are_pasted` | Moved to the repository's `tests/test_ato_workbook.py` |
| `test_workbook.py::test_cached_values_were_calculated_by_desktop_excel` | Moved to the repository's `tests/test_ato_workbook.py` |
| `test_workbook.py::test_bakery_results_match_the_engine` | Moved to the repository's `tests/test_ato_workbook.py`; `workbook.rs` holds its frozen engine answer |
| `test_workbook.py::test_benchmark_table_matches_the_shipped_data` | Moved to the repository's `tests/test_ato_workbook.py` |
| `test_workbook.py::test_account_columns_are_text_so_pasted_formulas_stay_inert` | Moved to the repository's `tests/test_ato_workbook.py` |
