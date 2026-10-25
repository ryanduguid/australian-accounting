# Python behaviour coverage

Base d401883, Python 0.1.8: 64 test functions, 113 parameterised instances.
The archive pytest suite passed before freezing its observed API and CLI inputs.
Rust replays their Python results plus generated cases from `tests/data/`.
Decimal values and scale, complete result objects and diagnostics are compared.
The native tests also assert refused postings leave the ledger unchanged.

| Original test | Rust coverage |
| --- | --- |
| `test_money_type_rejects_values_outside_domain` | Frozen validate-value scenarios in `parity.rs` |
| `test_money_type_accepts_domain_boundary` | Frozen validate-value scenarios in `parity.rs` |
| `test_percentage_type_rejects_values_outside_domain` | Frozen validate-value scenarios in `parity.rs` |
| `test_rate_type_rejects_values_outside_domain` | Frozen validate-value scenarios in `parity.rs` |
| `test_decimal_types_name_the_text_they_cannot_parse` | Frozen validate-value scenarios in `parity.rs` |
| `test_rejects_non_finite_franking_percentage` | Frozen statement scenarios in `parity.rs` |
| `test_rejects_non_finite_corporate_tax_rate` | Frozen statement scenarios in `parity.rs` |
| `test_base_rate_entity_eligibility` | Frozen bre-properties, corporate-tax, turnover-threshold scenarios in `parity.rs` |
| `test_franking_account_ledger_and_fdt` | Frozen franking-account scenarios in `parity.rs` |
| `test_franking_deficit_tax_and_penalty` | Frozen franking-account scenarios in `parity.rs` |
| `test_benchmark_rule_validation` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_distribution_statement_generation` | Frozen statement scenarios in `parity.rs` |
| `test_unknown_financial_year_is_refused` | Frozen bre-properties, corporate-tax scenarios in `parity.rs` |
| `test_max_franking_rate_requires_prior_year` | Frozen max-franking-rate scenarios in `parity.rs` |
| `test_max_franking_rate_rejects_wrong_evidence_year` | Frozen bre-properties, corporate-tax, max-franking-rate scenarios in `parity.rs` |
| `test_company_rate_and_franking_rate_use_different_income_years` | Frozen bre-properties, corporate-tax, max-franking-rate, turnover-threshold scenarios in `parity.rs` |
| `test_max_franking_rate_uses_current_year_rate_scale` | Frozen bre-properties, corporate-tax, max-franking-rate, turnover-threshold scenarios in `parity.rs` |
| `test_fy2018_turnover_threshold_is_25m` | Frozen bre-properties, corporate-tax, turnover-threshold scenarios in `parity.rs` |
| `test_brepi_test_compares_exactly_not_rounded` | Frozen bre-properties, corporate-tax scenarios in `parity.rs` |
| `test_fdt_reduction_applies_with_zero_credits` | Frozen franking-account scenarios in `parity.rs` |
| `test_under_franking_debit_and_fdt_liability_entries` | Frozen franking-account scenarios in `parity.rs` |
| `test_negative_amounts_are_refused` | Frozen bre-properties, corporate-tax, franking-account, statement scenarios in `parity.rs` |
| `test_franking_credit_never_exceeds_statutory_maximum` | Frozen statement scenarios in `parity.rs` |
| `test_benchmark_percentage_caps_at_100` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_benchmark_comparison_works_in_dollars` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_credit_above_maximum_is_capped_before_benchmark_comparison` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_events_without_a_rate_take_the_validators_rate` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_equally_franked_distributions_are_compliant_at_any_size` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_benchmark_is_keyed_to_the_earliest_distribution` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_distribution_event_refuses_negative_and_out_of_range_inputs` | Frozen distribution-event, franking-account scenarios in `parity.rs` |
| `test_zero_assessable_income_leaves_the_passive_test_unknown` | Frozen bre-properties, corporate-tax, turnover-threshold scenarios in `parity.rs` |
| `test_failed_turnover_test_settles_status_without_the_income_facts` | Frozen bre-properties, corporate-tax, turnover-threshold scenarios in `parity.rs` |
| `test_cli_refuses_a_rate_without_assessable_income` | Frozen bre-properties, corporate-tax, original CLI subprocess, turnover-threshold, validate-value scenarios in `parity.rs` |
| `test_unlisted_year_has_no_turnover_threshold` | Frozen bre-properties, corporate-tax, turnover-threshold scenarios in `parity.rs` |
| `test_cli_distribution_statement_states_the_acn` | Frozen original CLI subprocess, statement, validate-value scenarios in `parity.rs` |
| `test_cli_base_rate_entity_test_prints_the_evaluation` | Frozen bre-properties, corporate-tax, original CLI subprocess, turnover-threshold, validate-value scenarios in `parity.rs` |
| `test_cli_without_a_command_prints_help_and_fails` | Frozen original CLI subprocess scenarios in `parity.rs` |
| `test_sub_cent_total_distribution_is_refused` | Frozen statement scenarios in `parity.rs` |
| `test_zero_amount_event_sets_no_benchmark` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_passive_income_cannot_exceed_assessable_income` | Frozen bre-properties, corporate-tax scenarios in `parity.rs` |
| `test_all_assessable_income_can_be_passive` | Frozen bre-properties, corporate-tax, turnover-threshold scenarios in `parity.rs` |
| `test_cli_refuses_inconsistent_income_without_a_rate` | Frozen bre-properties, corporate-tax, original CLI subprocess, validate-value scenarios in `parity.rs` |
| `test_refund_only_deficit_does_not_reduce_offset` | Frozen franking-account scenarios in `parity.rs` |
| `test_distribution_related_debit_brings_refund_into_offset_reduction` | Frozen franking-account scenarios in `parity.rs` |
| `test_fdt_threshold_excludes_opening_credit_balance` | Frozen franking-account scenarios in `parity.rs` |
| `test_r_and_d_or_unclassified_refund_is_refused_without_posting` | Frozen franking-account scenarios in `parity.rs` |
| `test_refund_requires_explicit_classification` | Frozen franking-account scenarios in `parity.rs` |
| `test_ordinary_refund_posts_once_with_existing_validation` | Frozen franking-account scenarios in `parity.rs` |
| `test_recording_refuses_dates_outside_financial_year` | Frozen franking-account scenarios in `parity.rs` |
| `test_initial_entries_must_belong_to_financial_year` | Frozen franking-account scenarios in `parity.rs` |
| `test_financial_year_includes_both_boundary_dates` | Frozen franking-account scenarios in `parity.rs` |
| `test_mutated_entries_cannot_contaminate_annual_calculations` | `refused_records_do_not_post_and_mutated_entries_are_revalidated` and `appended_entries_outside_the_year_are_refused` in `parity.rs` |
| `test_direct_franking_entries_preserve_ledger_invariants` | `direct_entries_preserve_ledger_invariants` in parity.rs |
| `test_distribution_cli_requires_an_explicit_iso_payment_date` | Frozen original CLI subprocess, validate-value scenarios in `parity.rs` |
| `test_distribution_cli_requires_an_explicit_tax_rate` | Frozen original CLI subprocess, validate-value scenarios in `parity.rs` |
| `test_benchmark_validator_requires_a_corporate_tax_rate` | Frozen benchmark scenarios in `parity.rs` |
| `test_distribution_event_without_a_rate_refuses_to_measure_itself` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_nil_distribution_does_not_excuse_a_missing_rate` | Frozen distribution-event scenarios in `parity.rs` |
| `test_statement_requires_a_corporate_tax_rate` | Frozen statement scenarios in `parity.rs` |
| `test_a_period_without_a_frankable_distribution_is_not_tested` | Frozen benchmark, distribution-event scenarios in `parity.rs` |
| `test_deficit_test_is_unknown_until_the_opening_balance_is_established` | Frozen franking-account scenarios in `parity.rs` |
| `test_unknown_opening_balance_still_refuses_out_of_year_entries` | Frozen franking-account scenarios in `parity.rs` |
| `test_published_distribution_uses_the_project_identity` | Python metadata retired; Cargo identity and native CLI help covered by root test_rust_engine.py and parity.rs |
| `test_source_tree_without_installation_reports_a_development_version` | Retired Python importlib fallback; Cargo.toml is the native version authority |
