# Python behaviour coverage

`rates.py` at australian-accounting 24fcf24 had 53 test functions in 4 files. Each has a test
of the same name, without the `test_` prefix, in `src/rates_tests.rs`, inside the crate so that
it can call the same functions. Where a Python test monkeypatched `subprocess.run`, `fetch_text`,
`HTTPSConnection`, `load_records`, `fetch_all` or `DATA`, the Rust test calls the step below the
patched one with the same fabricated input.

A differential harness, kept outside the repository, covered the commands end to end against
`rates.py` itself; the package README gives its scope and the divergences.

| Python test | Rust test | Seam |
|---|---|---|
| `test_dataset.py::test_every_file_is_named_after_its_unique_id` | `every_file_is_named_after_its_unique_id` | direct call |
| `test_dataset.py::test_record_passes_schema_quote_and_pattern_checks` | `record_passes_schema_quote_and_pattern_checks` | direct call |
| `test_dataset.py::test_pattern_reads_the_committed_value_from_the_snapshot` | `pattern_reads_the_committed_value_from_the_snapshot` | direct call |
| `test_dataset.py::test_parse_text_units` | `parse_text_units` | direct call |
| `test_dataset.py::test_parse_text_rejects_other_formats` | `parse_text_rejects_other_formats` | direct call |
| `test_dataset.py::test_check_value_rejects_bad_values` | `check_value_rejects_bad_values` | direct call |
| `test_dataset.py::test_tax_scale_base_amounts_must_add_up` | `tax_scale_base_amounts_must_add_up` | direct call |
| `test_dataset.py::test_schema_rejects_missing_fields_and_foreign_sources` | `schema_rejects_missing_fields_and_foreign_sources` | direct call |
| `test_dataset.py::test_income_year_must_describe_adjacent_calendar_years` | `income_year_must_describe_adjacent_calendar_years` | direct call |
| `test_dataset.py::test_valid_periods_keep_century_rollover_open_bounds_and_retrieval_dates` | `valid_periods_keep_century_rollover_open_bounds_and_retrieval_dates` | direct call |
| `test_dataset.py::test_effective_period_cannot_end_before_it_starts` | `effective_period_cannot_end_before_it_starts` | direct call |
| `test_dataset.py::test_invalid_period_dates_report_a_problem_without_comparing_them` | `invalid_period_dates_report_a_problem_without_comparing_them` | direct call |
| `test_dataset.py::test_page_problems_report_changed_value_and_missing_quote` | `page_problems_report_changed_value_and_missing_quote` | direct call |
| `test_dataset.py::test_report_flags_missing_pages_and_passes_unchanged_pages` | `report_flags_missing_pages_and_passes_unchanged_pages` | direct call |
| `test_dataset.py::test_report_notes_only_pages_whose_body_changed` | `report_notes_only_pages_whose_body_changed` | direct call |
| `test_dataset.py::test_a_record_ending_without_a_successor_is_a_finding` | `a_record_ending_without_a_successor_is_a_finding` | direct call |
| `test_dataset.py::test_a_scale_must_match_the_brackets_its_quote_prints` | `a_scale_must_match_the_brackets_its_quote_prints` | direct call |
| `test_dataset.py::test_a_scale_taxed_from_the_first_dollar_renders_without_an_over_amount` | `a_scale_taxed_from_the_first_dollar_renders_without_an_over_amount` | direct call |
| `test_dataset.py::test_data_files_are_stable_json` | `data_files_are_stable_json` | direct call |
| `test_dataset.py::test_fetch_text_returns_the_page_text_with_plain_spaces` | `fetch_text_returns_the_page_text_with_plain_spaces` | `read_reply`, the step after `subprocess.run` |
| `test_dataset.py::test_fetch_text_rejects_unusable_pages` | `fetch_text_rejects_unusable_pages` | `read_reply`, the step after `subprocess.run` |
| `test_dataset.py::test_fetch_all_keeps_each_failure_as_that_page_s_value` | `fetch_all_keeps_each_failure_as_that_page_s_value` | `fetch_all_with` given the fake `fetch_text` |
| `test_dataset.py::test_fetch_text_rejects_a_success_shaped_reply_from_a_failed_browser` | `fetch_text_rejects_a_success_shaped_reply_from_a_failed_browser` | `read_reply`, the step after `subprocess.run` |
| `test_dataset.py::test_a_malformed_reply_fails_only_its_own_page` | `a_malformed_reply_fails_only_its_own_page` | `fetch_all_with` over `read_reply` |
| `test_dataset.py::test_the_payg_section_is_checked_not_its_contents_entry` | `the_payg_section_is_checked_not_its_contents_entry` | direct call |
| `test_numeric_validation.py::test_non_finite_values_are_refused` | `non_finite_values_are_refused` | direct call |
| `test_numeric_validation.py::test_finite_non_negative_values_remain_valid` | `finite_non_negative_values_remain_valid` | direct call |
| `test_numeric_validation.py::test_invalid_bracket_values_return_problems` | `invalid_bracket_values_return_problems` | direct call |
| `test_numeric_validation.py::test_valid_scale_uses_independently_calculated_base_amounts` | `valid_scale_uses_independently_calculated_base_amounts` | direct call |
| `test_numeric_validation.py::test_an_unreadable_record_is_reported_and_the_rest_still_checked` | `an_unreadable_record_is_reported_and_the_rest_still_checked` | `cmd_validate` with `data/` in a temporary folder |
| `test_numeric_validation.py::test_a_source_url_that_is_not_text_is_a_problem_not_a_crash` | `a_source_url_that_is_not_text_is_a_problem_not_a_crash` | `cmd_validate` with `data/` in a temporary folder |
| `test_numeric_validation.py::test_validation_reports_overflowing_json_number` | `validation_reports_overflowing_json_number` | `cmd_validate` with `data/` in a temporary folder |
| `test_pattern_reports.py::test_unreadable_source_value_is_a_report_finding` | `unreadable_source_value_is_a_report_finding` | direct call |
| `test_pattern_reports.py::test_invalid_pattern_is_a_problem_instead_of_an_exception` | `invalid_pattern_is_a_problem_instead_of_an_exception` | direct call |
| `test_pattern_reports.py::test_falsy_supplied_patterns_are_problems` | `falsy_supplied_patterns_are_problems` | direct call |
| `test_pattern_reports.py::test_absent_pattern_remains_valid` | `absent_pattern_remains_valid` | direct call |
| `test_pattern_reports.py::test_malformed_decimal_is_a_report_finding` | `malformed_decimal_is_a_report_finding` | direct call |
| `test_pattern_reports.py::test_tax_scale_pattern_is_reported_as_unsupported` | `tax_scale_pattern_is_reported_as_unsupported` | direct call |
| `test_pattern_reports.py::test_failed_parse_does_not_hide_later_changed_values` | `failed_parse_does_not_hide_later_changed_values` | direct call |
| `test_pattern_reports.py::test_check_writes_report_and_returns_failure_when_source_value_is_unreadable` | `check_writes_report_and_returns_failure_when_source_value_is_unreadable` | `write_check`, the step after `load_records` and `fetch_all` |
| `test_pattern_reports.py::test_empty_pattern_cannot_produce_a_successful_check` | `empty_pattern_cannot_produce_a_successful_check` | `write_check`, the step after `load_records` and `fetch_all` |
| `test_ato_tables.py::test_matching_figures_produce_no_finding` | `matching_figures_produce_no_finding` | direct call |
| `test_ato_tables.py::test_the_medicare_figures_come_from_the_basic_levy_rows` | `the_medicare_figures_come_from_the_basic_levy_rows` | direct call |
| `test_ato_tables.py::test_a_different_figure_is_a_finding_that_shows_both` | `a_different_figure_is_a_finding_that_shows_both` | direct call |
| `test_ato_tables.py::test_a_missing_or_doubled_row_is_a_finding` | `a_missing_or_doubled_row_is_a_finding` | direct call |
| `test_ato_tables.py::test_a_doubled_or_overlapping_medicare_row_is_a_finding` | `a_doubled_or_overlapping_medicare_row_is_a_finding` | direct call |
| `test_ato_tables.py::test_two_separate_rows_for_a_single_rate_are_a_finding` | `two_separate_rows_for_a_single_rate_are_a_finding` | direct call |
| `test_ato_tables.py::test_records_the_tables_do_not_hold_are_listed_without_a_finding` | `records_the_tables_do_not_hold_are_listed_without_a_finding` | direct call |
| `test_ato_tables.py::test_every_label_the_comparison_reads_belongs_to_a_record` | `every_label_the_comparison_reads_belongs_to_a_record` | direct call |
| `test_ato_tables.py::test_the_command_reads_both_tables_and_exits_1_on_a_finding` | `the_command_reads_both_tables_and_exits_1_on_a_finding` | `ato_tables_with` given each table's response and the records |
| `test_ato_tables.py::test_an_unreadable_table_ends_the_command_with_exit_1` | `an_unreadable_table_ends_the_command_with_exit_1` | `ato_tables_with` given each table's response and the records |
| `test_ato_tables.py::test_redirects_and_http_errors_end_the_command_without_following_them` | `redirects_and_http_errors_end_the_command_without_following_them` | `ato_tables_with`; `https::get` reads a body only for a 200 |
| `test_ato_tables.py::test_failed_requests_and_invalid_json_close_the_connection` | `failed_requests_and_invalid_json_close_the_connection` | `ato_tables_with`; the connection closes when `https::get` returns |
