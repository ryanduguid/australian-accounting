# Python behaviour coverage

Base 25ebed4, Python 0.2.0: 113 test functions. The archive pytest suite
passed before its observed inputs were frozen (package metadata tests
deselected). Rust replays the Python results in `tests/data/`: API and library
results in-process, and each command run with its input files, comparing exit
codes, stdout, stderr and every file left behind. Generated cases and the final review's counterexamples
(7,816 API and 643 command cases in all) are frozen beside them. Frozen
streams are compared after CRLF normalisation; the live differential compares
raw bytes. API results over 64 KiB and command streams over 4 KiB are stored
as an FNV-1a fingerprint and length.

Post-port corrections preserve those frozen inputs and expectations:

- `invoice_cents.rs` pins half-up right-to-invoice revenue, a nil contract
  balance, GST based on supplied billings, period revenue and source evidence. Frozen
  API case 4642 now expects `200.00` revenue instead of integer-scale `200`.
- `amount_range.rs` pins contextual precision refusals and unchanged accepted
  rounding. `data/amount-range-corrections.json` states the new error text for
  28 API cases and 8 CLI cases. CLI refusals now collect all unreadable cells;
  the same exit codes, stdout and files are still checked.

| Original test | Rust coverage |
| --- | --- |
| `test_atomic_io.py::test_a_failed_replacement_keeps_the_reviewed_output` | Retired: monkeypatches Python `os.replace`. `report::atomic_write` stages beside the destination, flushes and renames; a failed rename removes the staged file and leaves the destination untouched |
| `test_atomic_io.py::test_a_staged_file_already_gone_does_not_mask_the_failure` | Retired: monkeypatches Python `os.replace`. `report::atomic_write` ignores a missing staged file and reports the rename failure |
| `test_cli.py::test_sample_schedule_pins_the_worked_examples` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_unstated_measurability_refuses_the_contract` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files` |
| `test_cli.py::test_stated_measurability_produces_the_schedule` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_portfolio_totals_are_not_netted` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_refuses_to_overwrite_the_source` | Frozen replay: `original_cli_runs_with_their_files` |
| `test_cli.py::test_review_pack_binds_source_hash` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_review_pack_rejects_tampered_schedule` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_review_pack_refuses_a_schedule_from_another_period` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_an_empty_review_date_is_refused_not_replaced` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_as_at_must_be_a_real_iso_date` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files` |
| `test_cli.py::test_non_utf8_source_reports_an_error_not_a_traceback` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `original_cli_runs_with_their_files` |
| `test_cli.py::test_oversized_field_reports_an_error_not_a_traceback` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `original_cli_runs_with_their_files` |
| `test_cli.py::test_mapping_file_renames_columns` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_review_pack_refuses_to_overwrite_its_mapping_file` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_review_pack_hashes_the_bytes_used_even_if_source_changes` | By construction: `src/cli.rs` reads the source once and both parses and hashes those bytes, so a later change cannot reach the pack; the test's change after the read is a monkeypatch the corpus cannot replay, and the unmodified run is frozen |
| `test_cli.py::test_review_pack_takes_the_date_the_schedule_records` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_a_schedule_without_a_readable_date_records_none` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_cli.py::test_an_unreadable_schedule_records_no_date` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_cli.py::test_review_pack_refuses_an_empty_source` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_review_pack_needs_the_schedule_to_exist` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files`, `spreadsheet_and_markdown_escaping` |
| `test_cli.py::test_outputs_must_carry_their_own_suffix` | Frozen replay: `original_cli_runs_with_their_files` |
| `test_cli.py::test_schedule_refuses_when_an_input_path_cannot_be_resolved` | Not reproduced: it monkeypatches `Path.resolve` to fail; Python's non-strict resolve did not fail on a symlink loop (checked on Python 3.14), and the Rust guard falls back the same way there; other resolution failures, such as a removed working directory, were not compared |
| `test_cli.py::test_a_schedule_into_a_folder_that_is_a_file_exits_one_with_one_error_line` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `original_cli_runs_with_their_files` |
| `test_cli.py::test_streams_that_cannot_be_reconfigured_still_receive_the_output` | Not applicable: the binary writes bytes to its streams with no reconfigure step |
| `test_cli.py::test_python_dash_m_runs_the_command_line` | Retired: Python `-m` module entry; the binary itself is the command, and `--version` is replayed |
| `test_csv_guard_whitespace.py::test_guard_quotes_formula_after_leading_whitespace` | Frozen replay: `spreadsheet_and_markdown_escaping` |
| `test_csv_guard_whitespace.py::test_guard_preserves_ordinary_text` | Frozen replay: `spreadsheet_and_markdown_escaping` |
| `test_csv_guard_whitespace.py::test_guard_quotes_a_leading_control_character` | Frozen replay: `spreadsheet_and_markdown_escaping` |
| `test_csv_io.py::test_a_missing_mapping_file_is_refused` | Frozen replay: `original_cli_runs_with_their_files` |
| `test_csv_io.py::test_a_mapping_file_that_is_not_utf8_is_refused` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_csv_io.py::test_a_mapping_file_that_is_not_json_is_refused` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_csv_io.py::test_a_malformed_mapping_is_refused` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_csv_io.py::test_a_missing_contract_file_is_refused` | Frozen replay: `original_cli_runs_with_their_files` |
| `test_csv_io.py::test_an_empty_contract_file_is_refused` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_csv_io.py::test_a_header_without_contract_rows_is_refused` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_csv_io.py::test_missing_required_columns_are_named` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_csv_io.py::test_a_duplicated_heading_is_refused` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_csv_io.py::test_an_unnamed_column_is_passed_over` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `money_parsing_and_formatting` |
| `test_csv_io.py::test_blank_rows_are_passed_over` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `money_parsing_and_formatting` |
| `test_csv_io.py::test_a_reused_contract_id_is_refused` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `money_parsing_and_formatting` |
| `test_csv_io.py::test_more_than_twenty_unreadable_rows_stop_the_read_and_say_there_are_more` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `money_parsing_and_formatting` |
| `test_csv_io.py::test_an_unusable_row_is_refused_with_its_reason` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_csv_io.py::test_a_cell_past_the_end_of_its_row_reads_as_absent` | Native `csv_io::tests::a_cell_past_the_end_of_its_row_reads_as_absent` |
| `test_invariants.py::test_money_fields_are_decimal_not_float` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`; Retired: Python type introspection; every Rust amount is `decimal::Decimal` |
| `test_invariants.py::test_engine_source_does_not_call_float` | Retired: Python source scan; the Rust engine has no floating-point amount path |
| `test_invariants.py::test_formula_injection_is_escaped` | Frozen replay: `spreadsheet_and_markdown_escaping` |
| `test_invariants.py::test_every_accepted_input_field_is_read_by_something` | Retired: Python source scan; every `ContractInput` field is read by `engine::measure` or the reports, which the frozen replays exercise |
| `test_invariants.py::test_review_pack_escapes_pipes_in_ledger_identifiers` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `spreadsheet_and_markdown_escaping` |
| `test_invariants.py::test_review_pack_escapes_backslashes_before_pipes` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `spreadsheet_and_markdown_escaping` |
| `test_invariants.py::test_schedule_totals_do_not_net` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_invariants.py::test_sample_rows_all_match_the_header_width` | Frozen CLI replays of `examples/sample_contracts.csv`; a short row would be refused |
| `test_invariants.py::test_short_row_is_rejected` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_invariants.py::test_surplus_row_is_rejected` | Frozen replay: `contract_files_mappings_and_schedule_dates` |
| `test_invariants.py::test_a_review_pack_bound_to_a_source_needs_its_bytes` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `spreadsheet_and_markdown_escaping` |
| `test_job_to_cash.py::test_profit_does_not_fund_cash_and_uncertified_claims_are_excluded` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `spreadsheet_and_markdown_escaping`; Native `profit_does_not_fund_cash_and_uncertified_claims_are_excluded` |
| `test_job_to_cash.py::test_cost_change_flows_through_margin_and_cash` | Frozen replay: `contract_files_mappings_and_schedule_dates`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting`, `spreadsheet_and_markdown_escaping`; Native `cost_change_flows_through_margin_and_cash` |
| `test_money.py::test_parse_money_accepts_accounting_forms` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_parse_money_refuses_nan_and_half_cent_dust` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_parse_ratio_percent_and_unit_interval` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_parse_bool` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_display_does_not_round_a_verdict_across_a_boundary` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_as_money_uses_half_up` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_parse_amount_never_returns_float` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_malformed_grouping_and_conflicting_signs_are_refused` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_the_ordinary_grouped_forms_still_parse` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_parse_amount_refuses_a_missing_amount` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_parse_ratio_refuses_a_missing_ratio` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_parse_ratio_refuses_a_ratio_outside_the_unit_interval` | Frozen replay: `money_parsing_and_formatting` |
| `test_money.py::test_parse_bool_refuses_a_missing_flag` | Retired: `parse_bool(None)`; the CSV reader passes only present cells, and the Rust function takes a string |
| `test_package_identity.py::test_published_distribution_uses_the_project_identity` | Retired: Python distribution metadata; the root `tests/test_rust_engine.py` checks the Cargo identity |
| `test_package_identity.py::test_checkout_version_ignores_unrelated_metadata` | Retired: Python distribution metadata; the root `tests/test_rust_engine.py` checks the Cargo identity |
| `test_schedule.py::test_clean_cost_to_cost_underbilling` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_b19_wastage_is_stripped_from_progress_but_stays_in_margin` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_uninstalled_materials_earn_zero_margin` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_constrained_variable_consideration_is_not_full_claim` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_para_45_recognises_recoverable_cost_only` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_para_45_revenue_cannot_exceed_costs_incurred` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_onerous_flag_does_not_depend_on_a_measurable_outcome` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_profit_fade_flag_reads_the_fade_that_is_displayed` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_right_to_invoice_sets_revenue_equal_to_certified_billings` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_output_method_uses_supplied_percent` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_contracts_are_not_netted_by_the_caller` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_b19_exclusions_cannot_exceed_cost` | Frozen replay: `measure_positions_flags_and_refusals` |
| `test_schedule.py::test_negative_etc_is_refused` | Frozen replay: `measure_positions_flags_and_refusals` |
| `test_schedule.py::test_unstated_measurability_refuses_the_revenue_figure` | Frozen replay: `measure_positions_flags_and_refusals` |
| `test_schedule.py::test_stated_measurability_keeps_its_figures` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_para_45_limits_revenue_under_every_progress_method` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_a_supplied_output_percent_is_checked_when_para_45_applies` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_a_supplied_output_percent_must_be_decimal_when_para_45_applies` | Frozen replay: `float_amounts_are_refused_not_converted`, `money_parsing_and_formatting` |
| `test_schedule.py::test_para_45_refuses_recoverable_cost_above_the_transaction_price` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_para_45_accepts_recoverable_cost_equal_to_the_transaction_price` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_para_45_refuses_recoverable_cost_above_cost_under_every_method` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_output_progress_outside_zero_to_one_is_refused` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_output_progress_at_the_bounds_is_accepted` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_public_measurement_rejects_invalid_ratios` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_public_ratios_require_decimal_values` | Frozen replay: `float_amounts_are_refused_not_converted`, `money_parsing_and_formatting` |
| `test_schedule.py::test_output_progress_keeps_its_missing_value_error` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_unused_output_progress_is_not_validated` | Frozen replay: `float_amounts_are_refused_not_converted`, `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_public_constraint_ratio_keeps_the_supplied_fraction` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_public_gst_ratio_keeps_the_supplied_fraction` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_a_completed_contract_recognises_its_full_price` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_a_break_even_contract_is_not_flagged_onerous` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_an_unsupported_progress_method_is_refused` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_right_to_invoice_with_no_cost_base_reports_nil_progress` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_cost_to_cost_needs_a_positive_progress_base` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_progress_cost_cannot_exceed_the_progress_base` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_uninstalled_materials_cannot_exceed_the_transaction_price` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_a_nil_transaction_price_has_no_margin` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_a_nil_prior_transaction_price_has_no_prior_margin` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_retention_presented_as_a_receivable_is_flagged_for_para_108` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
| `test_schedule.py::test_an_onerous_contract_without_contract_assets_has_no_impairment_step` | Frozen replay: `measure_positions_flags_and_refusals`, `money_parsing_and_formatting` |
