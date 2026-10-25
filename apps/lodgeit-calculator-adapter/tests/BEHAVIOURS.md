# Python behaviour coverage

The Python package `lodgeitadapter` at australian-accounting 24fcf24 had 305 test
functions in 15 files. Each file has a Rust integration test file of the same stem
without the `test_` prefix (`tests/test_cli_paths.py` became `tests/cli_paths.rs`),
and each test a Rust test of the same name without the prefix, except where the
table says otherwise:

- **Retired with the MCP surface:** 7 tests that drive `lodgeitadapter.mcp`. The Rust
  port omits the MCP surface, so these have no Rust test.
- **Not applicable:** 16 tests whose input the Rust types cannot represent (a
  non-boolean flag, a non-string key, a configuration of the wrong type) or that
  inject a non-I/O exception, such as a cancellation, from a read or a close, which
  a Rust read cannot raise and dropping a connection cannot fail with.

Where a Python test replaced the opener, a socket or a module attribute, the Rust
test uses the seam named in its row: a scripted `transport::Dial`, a loopback
`std::net::TcpListener` reached through the real `dial::Net`, or a direct call to
the step below the patched one. Differential harnesses, kept outside the
repository, compared the commands, the transport, the Core client and the trials
with the Python package byte for byte; the README gives their scope and the
documented divergences.

## `test_cli_paths.py` to `cli_paths.rs`

Seam: the CLI runs in process through `cli::run_io` with captured stdout and stderr, an environment closure that returns no `LODGEIT_ADAPTER_*` variable (the autouse `_no_adapter_environment` fixture), the real `dial::Net` dialler and a sleep that does not wait. Captured output reads `\n` for the `\r\n` that Windows text-mode stdout writes, as `capsys` does. The network commands talk to a `std::net::TcpListener` stub on 127.0.0.1 started in the test (conftest's `stub`). `_decimalise` is `cli::decimalise`, made `#[doc(hidden)] pub` for this file.

| Python test | Rust test | Seam |
|---|---|---|
| test_cli_paths.py::test_a_body_that_is_not_an_object_is_refused | a_body_that_is_not_an_object_is_refused | direct call (`cli::decimalise`) |
| test_cli_paths.py::test_a_list_path_over_something_that_is_not_a_list_is_left_alone | a_list_path_over_something_that_is_not_a_list_is_left_alone | direct call (`cli::decimalise`) |
| test_cli_paths.py::test_a_dotted_path_reaches_a_nested_field | a_dotted_path_reaches_a_nested_field | direct call (`cli::decimalise`) |
| test_cli_paths.py::test_a_boolean_or_null_where_a_number_belongs_is_refused | a_boolean_or_null_where_a_number_belongs_is_refused (2 cases looped) | direct call (`cli::decimalise`) |
| test_cli_paths.py::test_an_unknown_contract_name_is_a_refusal | an_unknown_contract_name_is_a_refusal | direct call (`cli::run_io`) |
| test_cli_paths.py::test_the_contract_command_prints_the_reviewed_snapshot | the_contract_command_prints_the_reviewed_snapshot | direct call (`cli::run_io`) |
| test_cli_paths.py::test_verify_names_an_evidence_file_it_cannot_read | verify_names_an_evidence_file_it_cannot_read | direct call (`cli::run_io`) |
| test_cli_paths.py::test_verify_accepts_a_record_that_hangs_together | verify_accepts_a_record_that_hangs_together | direct call (`cli::run_io`) |
| test_cli_paths.py::test_discover_prints_the_live_listing | discover_prints_the_live_listing | loopback server |
| test_cli_paths.py::test_drift_that_agrees_says_so_and_exits_zero | drift_that_agrees_says_so_and_exits_zero | loopback server |
| test_cli_paths.py::test_drift_that_differs_lists_each_finding_and_exits_one | drift_that_differs_lists_each_finding_and_exits_one | loopback server |
| test_cli_paths.py::test_drift_that_cannot_reach_the_service_reports_the_outcome | drift_that_cannot_reach_the_service_reports_the_outcome (the retry backoff does not sleep) | loopback server |
| test_cli_paths.py::test_invoke_names_a_body_file_it_cannot_read | invoke_names_a_body_file_it_cannot_read | loopback server |
| test_cli_paths.py::test_the_mcp_contract_tool_reads_the_snapshot_offline | retired: the Rust port omits the MCP surface | none |
| test_cli_paths.py::test_the_mcp_drift_tool_reports_findings | retired: the Rust port omits the MCP surface | none |
| test_cli_paths.py::test_the_mcp_invoke_tool_refuses_a_body_it_cannot_read | retired: the Rust port omits the MCP surface | none |
| test_cli_paths.py::test_the_mcp_invoke_tool_returns_a_computed_outcome | retired: the Rust port omits the MCP surface | none |

## `test_client_outcomes.py` to `client_outcomes.rs`

| Python test | Rust test | Seam |
|---|---|---|
| test_client_outcomes.py::test_a_complete_200_is_the_only_computed_outcome | a_complete_200_is_the_only_computed_outcome | loopback server |
| test_client_outcomes.py::test_each_provider_status_keeps_its_own_outcome | each_provider_status_keeps_its_own_outcome (7 cases looped) | loopback server |
| test_client_outcomes.py::test_a_provider_refusal_keeps_the_providers_own_class_and_body | a_provider_refusal_keeps_the_providers_own_class_and_body | loopback server |
| test_client_outcomes.py::test_a_200_without_an_advisory_is_a_contract_failure_not_a_result | a_200_without_an_advisory_is_a_contract_failure_not_a_result | loopback server |
| test_client_outcomes.py::test_a_200_without_a_manifest_is_a_contract_failure | a_200_without_a_manifest_is_a_contract_failure | loopback server |
| test_client_outcomes.py::test_a_200_missing_an_expected_decimal_field_is_a_contract_failure | a_200_missing_an_expected_decimal_field_is_a_contract_failure | loopback server |
| test_client_outcomes.py::test_an_unparseable_body_is_a_contract_failure | an_unparseable_body_is_a_contract_failure | loopback server |
| test_client_outcomes.py::test_a_failure_status_keeps_its_outcome_when_its_body_is_not_json | a_failure_status_keeps_its_outcome_when_its_body_is_not_json (8 statuses x 2 bodies looped) | loopback server |
| test_client_outcomes.py::test_a_large_json_failure_body_is_kept_whole_with_its_outcome | a_large_json_failure_body_is_kept_whole_with_its_outcome (2 cases looped) | loopback server |
| test_client_outcomes.py::test_a_period_the_snapshot_does_not_record_never_reaches_the_network | a_period_the_snapshot_does_not_record_never_reaches_the_network | loopback server (request count) |
| test_client_outcomes.py::test_a_calculator_the_snapshot_does_not_record_is_refused_before_sending | a_calculator_the_snapshot_does_not_record_is_refused_before_sending | loopback server (request count) |
| test_client_outcomes.py::test_an_optional_field_may_be_absent_but_a_required_one_may_not | an_optional_field_may_be_absent_but_a_required_one_may_not | loopback server |
| test_client_outcomes.py::test_a_redirect_is_refused_rather_than_followed | a_redirect_is_refused_rather_than_followed | loopback server |
| test_client_outcomes.py::test_an_oversized_response_is_abandoned | an_oversized_response_is_abandoned | loopback server |
| test_client_outcomes.py::test_a_timeout_is_never_a_number | a_timeout_is_never_a_number | loopback server (600 ms reply delay, 0.05 s read timeout) |
| test_client_outcomes.py::test_a_5xx_is_retried_and_a_4xx_is_not | a_5xx_is_retried_and_a_4xx_is_not | loopback server, real `dial::sleep` backoff |
| test_client_outcomes.py::test_a_get_is_retry_safe_and_the_body_is_never_logged | a_get_is_retry_safe_and_the_body_is_never_logged | loopback server; `capsys` is the test binary re-running itself as a child with `--nocapture` and checking the child's stdout and stderr |
| test_client_outcomes.py::test_the_transport_refuses_a_target_off_the_allowlist_directly | the_transport_refuses_a_target_off_the_allowlist_directly | direct call (`transport::request` with `dial::Net`) |
| test_client_outcomes.py::test_a_closed_port_is_a_transport_error_not_a_result | a_closed_port_is_a_transport_error_not_a_result | direct call (`transport::request` and `Client::discover` with `dial::Net` to 127.0.0.1:1) |
| test_client_outcomes.py::test_drift_reports_differences_and_updates_nothing | drift_reports_differences_and_updates_nothing | loopback server |
| test_client_outcomes.py::test_an_unrecorded_response_field_is_noted_but_not_fatal | an_unrecorded_response_field_is_noted_but_not_fatal | loopback server; `json.dumps(raw_response)` maps to `pyjson::dumps` plus the field kept in `raw_response` |
| test_client_outcomes.py::test_an_advisory_carried_under_disclaimer_is_accepted | an_advisory_carried_under_disclaimer_is_accepted | loopback server |
| test_client_outcomes.py::test_the_live_response_fields_are_recorded_in_the_snapshot | the_live_response_fields_are_recorded_in_the_snapshot | direct call (`contract::load`) |

## `test_core_clients.py` to `core_clients.rs`

Seam: Python replaces `transport._opener` with a fake urllib opener. Rust gives `CoreClient::list_clients_via` a scripted `transport::Dial` that records each dial (host, port, TLS) and the bytes sent, and replies with fabricated HTTP bytes parsed by the real transport. Python's `__cause__`/`__context__ is None` maps to `std::error::Error::source(&error).is_none()`; frame-local and `vars()` checks map to `CoreError` (category and status only) showing no canary, bearer or continuation in `Display` or `Debug`.

| Python test | Rust test | Seam |
|---|---|---|
| test_core_clients.py::test_default_and_environment_cannot_enable_core | default_and_environment_cannot_enable_core (Rust's Core client reads no environment, so the env vars are not set; default config is disabled, nothing dialled) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_exact_request_projection_and_sensitive_representations | exact_request_projection_and_sensitive_representations (`repr(client)`: `CoreClient` has no `Debug`, so its config's and the page's `Debug` are checked) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_only_the_exact_read_route_is_admitted | only_the_exact_read_route_is_admitted (7 URLs looped) | direct call (`RequestConfig::check_url`, `AdapterConfig::check_url`) |
| test_core_clients.py::test_invalid_tokens_never_open_transport | invalid_tokens_never_open_transport (8 string cases looped; `None` and `1` refused by the `&str` type at compile time) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_invalid_page_size_never_sends | invalid_page_size_never_sends (0 and 501; `True`, `"100"`, `1.5` refused by the `i64` type) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_invalid_continuation_never_sends | invalid_continuation_never_sends (empty and 8,193 bytes; a raw string and the lone surrogate `"\ud800"` cannot be built in Rust) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_captured_contract_pins_the_read_request_and_required_projection | captured_contract_pins_the_read_request_and_required_projection | direct call (reads `contracts/lodgeit-core-clients.json` with `pyjson::loads`) |
| test_core_clients.py::test_bad_success_is_transactionally_rejected_without_context | bad_success_is_transactionally_rejected_without_context (16 payloads looped) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_http_error_bodies_are_discarded_and_never_retried | http_error_bodies_are_discarded_and_never_retried (7 cases looped; the 302 carries `Location: file:///...`, the only way a real 302 reaches the status mapping, as the fake `HTTPError(302)` did) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_http_parser_failures_have_no_payload_or_bearer_context | http_parser_failures_have_no_payload_or_bearer_context (same 2 wire byte strings; frame-local bearer check maps to the error showing no bearer) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_http_error_close_failure_is_sanitised | http_error_close_failure_is_sanitised (closing a Rust connection cannot fail, so the 503's body read fails part way instead; same TransportError outcome, `http_status` None) | scripted `Dial` via `core::list_clients_via` (read error after the bytes) |
| test_core_clients.py::test_every_core_failure_clears_sensitive_frame_locals | every_core_failure_clears_sensitive_frame_locals (7 stages looped; the raw-string continuation stage uses an over-long continuation, the nearest refusal Rust can reach) | scripted `Dial` via `core::list_clients_via` (refused dial for the transport stage) |
| test_core_clients.py::test_actual_redirect_handler_refuses_without_following_or_leaking | actual_redirect_handler_refuses_without_following_or_leaking (5 statuses x 4 locations looped, through the real transport's redirect refusal) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_huge_timeouts_are_configuration_errors | huge_timeouts_are_configuration_errors (`10**400` has no `f64`: infinities and `±f64::MAX`) | direct call (`CoreConfig::new`) |
| test_core_clients.py::test_response_ceiling_and_media_type_are_enforced | response_ceiling_and_media_type_are_enforced | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_synthetic_maximum_page_fits_and_contract_projection_is_serialisable | synthetic_maximum_page_fits_and_contract_projection_is_serialisable (`asdict` maps to the public `code` field) | scripted `Dial` via `core::list_clients_via` |
| test_core_clients.py::test_config_safety_limits | config_safety_limits (NaN, 61, 1,048,577; `enabled=1` and `max_response_bytes=True` refused by the `bool` and `i64` types) | direct call (`CoreConfig::new`) |
| test_core_clients.py::test_conflicting_authorisation_is_refused_by_shared_transport | conflicting_authorisation_is_refused_by_shared_transport (Python's config, plus one with a base URL so the header conflict itself refuses) | scripted `Dial` via `transport::request` |

## `test_evidence.py` to `evidence.rs`

Seam: `capture` invokes the client against a `std::net::TcpListener` stub on 127.0.0.1 started in the test, with conftest's `stub_config` (`max_attempts: 1`) and the real `dial::Net` dialler, then calls `evidence::build` with no response contract (Python's `None`).

| Python test | Rust test | Seam |
|---|---|---|
| test_evidence.py::test_an_evidence_record_verifies_against_its_own_digest | an_evidence_record_verifies_against_its_own_digest | loopback server |
| test_evidence.py::test_the_digest_covers_the_calculation_and_not_the_observation | the_digest_covers_the_calculation_and_not_the_observation | loopback server |
| test_evidence.py::test_changing_a_figure_breaks_the_digest | changing_a_figure_breaks_the_digest | loopback server |
| test_evidence.py::test_the_upstream_response_is_preserved_and_normalisation_sits_beside_it | the_upstream_response_is_preserved_and_normalisation_sits_beside_it | loopback server |
| test_evidence.py::test_the_input_is_recorded_with_its_exact_wire_form | the_input_is_recorded_with_its_exact_wire_form | loopback server |
| test_evidence.py::test_a_refusal_is_recorded_as_evidence_too | a_refusal_is_recorded_as_evidence_too | loopback server |
| test_evidence.py::test_a_computed_record_without_a_manifest_is_reported | a_computed_record_without_a_manifest_is_reported | loopback server |
| test_evidence.py::test_an_unknown_schema_is_refused | an_unknown_schema_is_refused | loopback server |
| test_evidence.py::test_a_record_round_trips_through_a_file | a_record_round_trips_through_a_file | loopback server |
| test_evidence.py::test_the_record_carries_the_contract_snapshot_it_was_checked_against | the_record_carries_the_contract_snapshot_it_was_checked_against | loopback server |
| test_evidence.py::test_no_boundary_claim_is_overstated | no_boundary_claim_is_overstated (`json.dumps(record)` is `pyjson::dumps` with `Layout::Canonical`; only the absence of the claim words is asserted) | loopback server |

## `test_evidence_flags.py` to `evidence_flags.rs`

Seam: direct calls to `evidence::build`, `write`, `read`, `digest_of` and `verify` with fabricated `Outcome`s; the CLI test drives `cli::run_with` in process with captured streams. A Python `Outcome` default is `Json::Null`, `None` or an empty value; `observed_at` is `client::now()`, which Python's `build` fills in for `None`.

| Python test | Rust test | Seam |
|---|---|---|
| test_evidence_flags.py::test_build_refuses_a_non_boolean_synthetic_flag | not applicable: `evidence::build` takes `synthetic: bool`, so none of the 9 non-boolean values can be passed; the type refuses them at compile time | none |
| test_evidence_flags.py::test_verify_reports_a_present_non_boolean_flag | verify_reports_a_present_non_boolean_flag (2 fields x 9 values looped) | direct call |
| test_evidence_flags.py::test_verify_reports_a_computed_record_the_producer_did_not_accept | verify_reports_a_computed_record_the_producer_did_not_accept | direct call |
| test_evidence_flags.py::test_literal_flags_survive_a_file_round_trip | literal_flags_survive_a_file_round_trip (2 cases looped) | direct call |
| test_evidence_flags.py::test_omitted_legacy_flags_keep_their_verification_result | omitted_legacy_flags_keep_their_verification_result (3 fields looped) | direct call |
| test_evidence_flags.py::test_non_computed_outcomes_keep_their_accepted_false_flag | non_computed_outcomes_keep_their_accepted_false_flag (7 statuses looped) | direct call |
| test_evidence_flags.py::test_verify_command_refuses_the_contradictory_record | verify_command_refuses_the_contradictory_record | direct call (`cli::run_with`) |

## `test_module_paths.py` to `module_paths.rs`

Seam: the client tests use a `std::net::TcpListener` stub on 127.0.0.1 started in the test, with conftest's `stub_config` (`enabled`, the stub's `base_url`, `allow_loopback`, `max_attempts: 1`) and the real `dial::Net` dialler. `contract::contracts_dirs` and `core::parse_page` were made `#[doc(hidden)] pub` for this file. Python's `dataclasses.replace` on a contract is a clone with its `Json` edited.

| Python test | Rust test | Seam |
|---|---|---|
| test_module_paths.py::test_a_limit_outside_its_range_is_refused | a_limit_outside_its_range_is_refused (5 cases looped; Python's `retry_backoff_seconds="1"` is a str and the Rust field is an `f64`, so `-1.0` stands in, refused with the same message) | direct call (`AdapterConfig::checked`) |
| test_module_paths.py::test_a_loopback_url_with_another_scheme_is_refused | a_loopback_url_with_another_scheme_is_refused | direct call (`AdapterConfig::check_url`, `require_route` true as Python's default) |
| test_module_paths.py::test_a_path_that_is_not_in_its_decoded_form_is_refused | a_path_that_is_not_in_its_decoded_form_is_refused | direct call (`AdapterConfig::check_url`) |
| test_module_paths.py::test_the_environment_can_name_the_allowed_hosts_and_the_limits | the_environment_can_name_the_allowed_hosts_and_the_limits | direct call (`config::from_environment` over a map) |
| test_module_paths.py::test_an_environment_limit_that_is_not_a_number_is_refused | an_environment_limit_that_is_not_a_number_is_refused | direct call (`config::from_environment`) |
| test_module_paths.py::test_the_snapshot_folder_falls_back_to_the_first_candidate | the_snapshot_folder_falls_back_to_the_first_candidate (partial: Rust's candidates are fixed, so the test checks their order, the source checkout then `contracts` beside the binary, and that `contracts_dir` picks the first that exists; the 'neither exists' and 'only the second exists' branches cannot be reached without patching `src/`) | direct call (`contract::contracts_dirs`, `contract::contracts_dir`) |
| test_module_paths.py::test_a_snapshot_that_is_not_there_is_refused | a_snapshot_that_is_not_there_is_refused | direct call (`contract::load` with a temp folder) |
| test_module_paths.py::test_a_snapshot_that_is_not_json_is_refused | a_snapshot_that_is_not_json_is_refused (2 payloads looped) | direct call (`contract::load`) |
| test_module_paths.py::test_a_snapshot_missing_a_section_names_it | a_snapshot_missing_a_section_names_it | direct call (`contract::load`) |
| test_module_paths.py::test_a_calculator_with_no_recorded_schema_ref_is_not_compared_on_it | a_calculator_with_no_recorded_schema_ref_is_not_compared_on_it | direct call (`contract::compare`) |
| test_module_paths.py::test_a_moved_schema_ref_is_drift | a_moved_schema_ref_is_drift | direct call (`contract::compare`) |
| test_module_paths.py::test_drift_without_a_live_listing_compares_nothing | drift_without_a_live_listing_compares_nothing | loopback server |
| test_module_paths.py::test_a_calculator_with_no_recorded_route_is_never_called | a_calculator_with_no_recorded_route_is_never_called | loopback server |
| test_module_paths.py::test_a_snapshot_that_refuses_unknown_fields_fails_a_response_carrying_one | a_snapshot_that_refuses_unknown_fields_fails_a_response_carrying_one | loopback server |
| test_module_paths.py::test_core_makes_one_attempt_with_no_extra_headers | core_makes_one_attempt_with_no_extra_headers | direct call (`transport::RequestConfig` on `CoreConfig::default()`) |
| test_module_paths.py::test_core_access_is_disabled_unless_switched_on | core_access_is_disabled_unless_switched_on | direct call (`RequestConfig::require_enabled`) |
| test_module_paths.py::test_the_core_client_takes_only_a_core_configuration | not applicable: `CoreClient.config` is typed `CoreConfig`, so passing an `AdapterConfig` does not compile; there is no runtime check to test | none |
| test_module_paths.py::test_a_core_page_of_the_wrong_shape_is_no_page | a_core_page_of_the_wrong_shape_is_no_page (2 bodies looped) | direct call (`core::parse_page`) |
| test_module_paths.py::test_an_integer_is_read_exactly | an_integer_is_read_exactly | direct call (`wire::parse_decimal`) |
| test_module_paths.py::test_a_non_finite_decimal_string_is_refused | a_non_finite_decimal_string_is_refused (3 cases looped) | direct call (`wire::parse_decimal`) |
| test_module_paths.py::test_strings_are_escaped_and_null_is_written_as_null | strings_are_escaped_and_null_is_written_as_null | direct call (`wire::dumps`) |
| test_module_paths.py::test_an_object_key_must_be_a_string | not applicable: a `Json::Object` key is a `PyStr`, so a non-string key cannot be built | none |
| test_module_paths.py::test_a_value_with_no_exact_form_is_refused | a_value_with_no_exact_form_is_refused (nearest contract: `Json` cannot hold a set, so a float, the one value it can hold with no exact wire form, is refused at `body.when`; the message differs because the value differs) | direct call (`wire::dumps`) |
| test_module_paths.py::test_an_upstream_error_keeps_the_providers_own_status_and_body | an_upstream_error_keeps_the_providers_own_status_and_body (nearest contract: Rust has no `UpstreamError`; a provider 400's `Outcome` keeps the status, the body and the refusal class) | loopback server |
| test_module_paths.py::test_a_label_must_be_a_string | a_label_must_be_a_string (`build` takes `&str`; the `check_label` it runs refuses a non-string with the same message) | direct call (`evidence::check_label`) |
| test_module_paths.py::test_a_record_without_a_calculation_block_is_reported | a_record_without_a_calculation_block_is_reported | direct call (`evidence::verify`) |
| test_module_paths.py::test_a_calculation_that_cannot_be_canonicalised_is_reported | a_calculation_that_cannot_be_canonicalised_is_reported | direct call (`evidence::verify`) |
| test_module_paths.py::test_a_calculation_naming_another_schema_is_reported | a_calculation_naming_another_schema_is_reported | direct call (`evidence::verify`) |
| test_module_paths.py::test_a_calculation_whose_label_is_not_a_slug_is_reported | a_calculation_whose_label_is_not_a_slug_is_reported | direct call (`evidence::verify`) |
| test_module_paths.py::test_a_computed_record_without_an_advisory_is_reported | a_computed_record_without_an_advisory_is_reported | direct call (`evidence::verify`) |
| test_module_paths.py::test_a_block_that_is_not_an_object_is_reported | a_block_that_is_not_an_object_is_reported | direct call (`evidence::verify`) |

## `test_no_network_by_default.py` to `no_network_by_default.rs`

| Python test | Rust test | Seam |
| --- | --- | --- |
| `test_no_network_by_default.py::test_importing_every_module_opens_no_socket` | `importing_every_module_opens_no_socket` | `transport::Dial` that panics on any dial. Rust has no import-time code, so the nearest contract is that the offline entry points (both snapshots, `AdapterConfig::default().checked()`, `config::from_environment`, `div7a::load_cases`, constructing a `Client`, and the CLI `contract` command through `cli::run_io`) complete without a dial |
| `test_no_network_by_default.py::test_the_default_configuration_is_off` | `the_default_configuration_is_off` | direct call |
| `test_no_network_by_default.py::test_an_environment_without_the_switch_is_still_off` | `an_environment_without_the_switch_is_still_off` | direct call (`config::from_environment` with a fabricated environment) |
| `test_no_network_by_default.py::test_discovery_offline_refuses_to_send_rather_than_answering` | `discovery_offline_refuses_to_send_rather_than_answering` | `transport::Dial` that panics on any dial; `sleep` that panics |
| `test_no_network_by_default.py::test_invocation_offline_refuses_and_produces_no_figure` | `invocation_offline_refuses_and_produces_no_figure` | `transport::Dial` that panics on any dial; `sleep` that panics |
| `test_no_network_by_default.py::test_the_reviewed_snapshot_is_readable_offline` | `the_reviewed_snapshot_is_readable_offline` | direct call (no dialler exists on this path) |
| `test_no_network_by_default.py::test_enabled_without_a_base_url_still_sends_nothing` | `enabled_without_a_base_url_still_sends_nothing` | `transport::Dial` that panics on any dial |
| `test_no_network_by_default.py::test_a_host_off_the_allowlist_is_refused_before_a_socket_opens` | `a_host_off_the_allowlist_is_refused_before_a_socket_opens` | direct call (`check_url`), plus `transport::request` with a panicking `transport::Dial` to show the refusal comes before any dial |
| `test_no_network_by_default.py::test_a_route_the_adapter_does_not_call_is_refused` | `a_route_the_adapter_does_not_call_is_refused` | direct call (`check_url` takes no dialler) |
| `test_no_network_by_default.py::test_loopback_needs_its_own_switch` | `loopback_needs_its_own_switch` | direct call (`check_url` takes no dialler) |
| `test_no_network_by_default.py::test_the_mcp_invoke_tool_needs_a_per_call_network_acknowledgement` | retired: the Rust port omits the MCP surface | none |
| `test_no_network_by_default.py::test_a_loose_acknowledgement_never_reaches_the_client` | retired: the Rust port omits the MCP surface | none |
| `test_no_network_by_default.py::test_the_mcp_invoke_tool_declares_the_acknowledgement_as_required` | retired: the Rust port omits the MCP surface | none |

## `test_review_regressions.py` to `review_regressions.rs`

| Python test | Rust test | Seam |
| --- | --- | --- |
| `test_review_regressions.py::test_a_path_that_is_not_a_route_is_refused_whatever_it_is_dressed_as` | `a_path_that_is_not_a_route_is_refused_whatever_it_is_dressed_as` (7 cases in one loop) | direct call |
| `test_review_regressions.py::test_the_routes_this_adapter_actually_calls_still_pass` | `the_routes_this_adapter_actually_calls_still_pass` (5 cases; Python's returned URL maps to `Ok(())`) | direct call |
| `test_review_regressions.py::test_a_port_off_loopback_is_part_of_the_destination` | `a_port_off_loopback_is_part_of_the_destination` | direct call |
| `test_review_regressions.py::test_a_base_url_carrying_a_query_is_refused_at_construction` | `a_base_url_carrying_a_query_is_refused_at_construction` (construction is `AdapterConfig::checked`) | direct call |
| `test_review_regressions.py::test_an_ephemeral_loopback_port_is_still_reachable_for_a_stub` | `an_ephemeral_loopback_port_is_still_reachable_for_a_stub` | loopback server |
| `test_review_regressions.py::test_a_contract_failure_hands_back_no_result` | `a_contract_failure_hands_back_no_result` | loopback server |
| `test_review_regressions.py::test_an_empty_manifest_is_not_a_manifest` | `an_empty_manifest_is_not_a_manifest` | loopback server |
| `test_review_regressions.py::test_a_provider_that_declines_to_answer_is_a_refusal_not_a_contract_failure` | `a_provider_that_declines_to_answer_is_a_refusal_not_a_contract_failure` (401 and 403 in one loop) | loopback server |
| `test_review_regressions.py::test_a_body_that_cannot_be_encoded_is_a_refusal_not_a_traceback` | `a_body_that_cannot_be_encoded_is_a_refusal_not_a_traceback` (lone surrogate as `PyStr::Lone`) | loopback server |
| `test_review_regressions.py::test_an_identifier_is_not_a_number_just_because_it_parses_as_one` | `an_identifier_is_not_a_number_just_because_it_parses_as_one` (`cli::decimalise` takes the body by value; the caller's clone is checked unchanged) | direct call |
| `test_review_regressions.py::test_a_named_number_field_that_is_not_a_number_is_a_usage_error` | `a_named_number_field_that_is_not_a_number_is_a_usage_error` (Python's `ValueError` is `Ok(Err(message))`) | direct call |
| `test_review_regressions.py::test_an_absent_optional_number_field_is_not_an_error` | `an_absent_optional_number_field_is_not_an_error` | direct call |
| `test_review_regressions.py::test_the_snapshot_names_the_repayment_amounts_as_numbers` | `the_snapshot_names_the_repayment_amounts_as_numbers` | direct call |
| `test_review_regressions.py::test_exact_agreement_with_an_uncompared_field_is_scope_not_arithmetic` | `exact_agreement_with_an_uncompared_field_is_scope_not_arithmetic` (Python's default tolerance passed as `0.01`) | direct call |
| `test_review_regressions.py::test_a_figure_only_one_side_produced_is_named` | `a_figure_only_one_side_produced_is_named` | direct call |
| `test_review_regressions.py::test_full_agreement_on_every_field_is_still_a_match` | `full_agreement_on_every_field_is_still_a_match` | direct call |
| `test_review_regressions.py::test_a_grossed_up_value_is_refused_by_its_value_not_only_by_its_name` | `a_grossed_up_value_is_refused_by_its_value_not_only_by_its_name` | direct call |
| `test_review_regressions.py::test_the_category_value_itself_still_passes_that_check` | `the_category_value_itself_still_passes_that_check` (Python's `type_two_value` key is the tuple's second element) | direct call |
| `test_review_regressions.py::test_the_fbt_trial_compares_the_payable_before_it_says_match` | `the_fbt_trial_compares_the_payable_before_it_says_match` | loopback server |
| `test_review_regressions.py::test_an_fbt_payable_that_differs_is_a_numeric_difference` | `an_fbt_payable_that_differs_is_a_numeric_difference` | loopback server |
| `test_review_regressions.py::test_an_fbt_response_with_nothing_to_compare_is_not_a_match` | `an_fbt_response_with_nothing_to_compare_is_not_a_match` | loopback server |
| `test_review_regressions.py::test_an_fbt_response_missing_only_the_payable_is_not_a_match` | `an_fbt_response_missing_only_the_payable_is_not_a_match` | loopback server |
| `test_review_regressions.py::test_a_digest_mismatch_names_the_digest_it_actually_computed` | `a_digest_mismatch_names_the_digest_it_actually_computed` | loopback server |
| `test_review_regressions.py::test_verify_reports_a_null_block_rather_than_crashing` | `verify_reports_a_null_block_rather_than_crashing` | loopback server |
| `test_review_regressions.py::test_the_command_line_verify_reports_a_null_block` | `the_command_line_verify_reports_a_null_block` | loopback server; `cli::run_io` with an empty environment |
| `test_review_regressions.py::test_a_label_the_consumer_would_refuse_is_refused_here` | `a_label_the_consumer_would_refuse_is_refused_here` (6 labels in one loop) | loopback server |
| `test_review_regressions.py::test_every_case_records_the_verdict_the_local_engine_actually_reaches` | `every_case_records_the_verdict_the_local_engine_actually_reaches` | direct call |
| `test_review_regressions.py::test_the_command_line_writes_no_evidence_under_a_label_the_consumer_refuses` | `the_command_line_writes_no_evidence_under_a_label_the_consumer_refuses` | loopback server; `cli::run_io` with an empty environment |
| `test_review_regressions.py::test_the_default_label_is_one_the_consumer_accepts` | `the_default_label_is_one_the_consumer_accepts` | loopback server; `cli::run_io` with an empty environment |
| `test_review_regressions.py::test_a_repayment_amount_read_from_a_file_goes_on_the_wire_as_a_number` | `a_repayment_amount_read_from_a_file_goes_on_the_wire_as_a_number` | loopback server; `cli::run_io` with an empty environment |
| `test_review_regressions.py::test_a_json_number_keeps_its_own_digits_on_the_way_in` | `a_json_number_keeps_its_own_digits_on_the_way_in` | loopback server; `cli::run_io` with an empty environment |
| `test_review_regressions.py::test_a_number_too_precise_to_send_is_refused_rather_than_rounded` | `a_number_too_precise_to_send_is_refused_rather_than_rounded` | loopback server; `cli::run_io` with an empty environment |
| `test_review_regressions.py::test_the_body_reader_keeps_an_integer_an_integer` | `the_body_reader_keeps_an_integer_an_integer` (Rust has no `load_body`; the CLI reads the body with `pyjson::loads_numbers(text, Numbers::DecimalFloats)`, which this test calls) | direct call |
| `test_review_regressions.py::test_a_result_row_that_is_not_an_object_is_a_contract_failure` | `a_result_row_that_is_not_an_object_is_a_contract_failure` | loopback server |
| `test_review_regressions.py::test_a_discovery_body_that_is_not_a_list_is_not_agreement` | `a_discovery_body_that_is_not_a_list_is_not_agreement` | loopback server |
| `test_review_regressions.py::test_a_listing_entry_that_is_not_an_object_is_reported_not_a_crash` | `a_listing_entry_that_is_not_an_object_is_reported_not_a_crash` | loopback server |
| `test_review_regressions.py::test_verify_reports_an_evidence_file_that_is_not_an_object` | `verify_reports_an_evidence_file_that_is_not_an_object` | direct call |
| `test_review_regressions.py::test_verify_accepts_a_discovery_record_the_tool_wrote` | `verify_accepts_a_discovery_record_the_tool_wrote` | loopback server |
| `test_review_regressions.py::test_verify_honours_a_contract_that_requires_no_manifest` | `verify_honours_a_contract_that_requires_no_manifest` | loopback server |
| `test_review_regressions.py::test_verify_still_requires_a_manifest_where_the_contract_does` | `verify_still_requires_a_manifest_where_the_contract_does` | loopback server |
| `test_review_regressions.py::test_a_port_that_is_not_a_number_is_a_refusal_not_a_traceback` | `a_port_that_is_not_a_number_is_a_refusal_not_a_traceback` | direct call |
| `test_review_regressions.py::test_a_listing_entry_without_a_calc_uri_is_a_finding_not_agreement` | `a_listing_entry_without_a_calc_uri_is_a_finding_not_agreement` | loopback server |
| `test_review_regressions.py::test_malformed_supported_periods_are_a_finding_not_a_crash` | `malformed_supported_periods_are_a_finding_not_a_crash` (4 cases in one loop) | loopback server |
| `test_review_regressions.py::test_a_figure_only_the_provider_returned_is_a_scope_difference` | `a_figure_only_the_provider_returned_is_a_scope_difference` | direct call |

The loopback server is a `std::net::TcpListener` on `127.0.0.1:0` started by each test, which replies with HTTP/1.0 as `http.server` does and records each request body. Clients reach it through `dial::Net` with `allow_loopback` set.

## `test_shared_options_and_transport_boundaries.py` to `shared_options_and_transport_boundaries.rs`

| Python test | Rust test | Seam |
|---|---|---|
| test_shared_options_and_transport_boundaries.py::test_shared_options_before_the_command_word_survive_the_subcommand | shared_options_before_the_command_word_survive_the_subcommand (4 cases looped) | direct call |
| test_shared_options_and_transport_boundaries.py::test_shared_options_after_the_command_word_still_work | shared_options_after_the_command_word_still_work | direct call |
| test_shared_options_and_transport_boundaries.py::test_shared_options_split_around_the_command_word_are_all_kept | shared_options_split_around_the_command_word_are_all_kept | direct call |
| test_shared_options_and_transport_boundaries.py::test_absent_shared_options_keep_their_defaults | absent_shared_options_keep_their_defaults | direct call |
| test_shared_options_and_transport_boundaries.py::test_an_option_typed_in_both_positions_takes_the_later_value | an_option_typed_in_both_positions_takes_the_later_value | direct call |
| test_shared_options_and_transport_boundaries.py::test_a_matching_live_listing_still_agrees | a_matching_live_listing_still_agrees | direct call |
| test_shared_options_and_transport_boundaries.py::test_a_repeated_calc_uri_is_drift_in_either_order | a_repeated_calc_uri_is_drift_in_either_order (2 cases looped) | direct call |
| test_shared_options_and_transport_boundaries.py::test_a_lost_schema_reference_is_drift | a_lost_schema_reference_is_drift (3 cases looped) | direct call |
| test_shared_options_and_transport_boundaries.py::test_the_opener_ignores_environment_proxies | the_opener_ignores_environment_proxies (Rust has no ProxyHandler to inspect; with the proxy variables set to an unused port, the request reaches the test's server directly in origin form; the default-opener control has no counterpart) | loopback server via `dial::Net` |
| test_shared_options_and_transport_boundaries.py::test_a_timed_out_5xx_body_is_retried_then_normalised | a_timed_out_5xx_body_is_retried_then_normalised | Dial with a scripted connection whose body read times out |
| test_shared_options_and_transport_boundaries.py::test_a_timed_out_4xx_body_is_not_retried | a_timed_out_4xx_body_is_not_retried | Dial with a scripted connection whose body read times out |

## `test_transport_failures.py` to `transport_failures.rs`

| Python test | Rust test | Seam |
|---|---|---|
| test_transport_failures.py::test_failed_http_body_respects_status_and_retry_budget | failed_http_body_respects_status_and_retry_budget (32 of 40 cases looped; the 8 "close" cases are left out because a Rust connection's drop cannot fail, so that body reads cleanly) | Dial with a scripted connection |
| test_transport_failures.py::test_calculator_client_returns_unavailable_for_failed_5xx | calculator_client_returns_unavailable_for_failed_5xx (4 cases looped; "close" is a cleanly read 503, which the client also reports as unavailable) | Dial with a scripted connection, through `Client::discover` |
| test_transport_failures.py::test_response_failures_are_normalised_without_retaining_diagnostics | response_failures_are_normalised_without_retaining_diagnostics (8 of 10 cases looped; "close" has no counterpart; URLError and ValueError at open are dial errors) | Dial with a scripted connection |
| test_transport_failures.py::test_redirect_refusal_never_retries_or_retains_location | redirect_refusal_never_retries_or_retains_location (10 cases looped; the close_failure axis has no counterpart; "not drained" is no read after the head segment) | Dial with a scripted connection |
| test_transport_failures.py::test_successful_oversized_response_is_not_retried | successful_oversized_response_is_not_retried (close_failure axis has no counterpart) | Dial with a scripted connection |
| test_transport_failures.py::test_process_cancellation_is_not_converted_or_retried | process_cancellation_is_not_converted_or_retried (2 cases looped) | Dial returning `transport::Raised` |
| test_transport_failures.py::test_readable_error_response_preserves_provider_data | readable_error_response_preserves_provider_data (10 cases looped; the 302 carries a Location urllib would not follow, the wire path to Python's HTTPError) | Dial with a scripted connection |
| test_transport_failures.py::test_failed_5xx_recovers_to_success_within_budget | failed_5xx_recovers_to_success_within_budget (5 cases looped; "close" is a cleanly read 503, retried the same way) | Dial with a scripted connection |
| test_transport_failures.py::test_known_redirect_with_failed_body_is_never_retried | known_redirect_with_failed_body_is_never_retried (4 of 5 cases looped; "close" has no counterpart) | Dial with a scripted connection |
| test_transport_failures.py::test_response_cancellation_propagates_after_cleanup | not applicable: a Rust response read or close cannot raise KeyboardInterrupt or SystemExit; reads return only `io::Error`, which becomes TransportError | not applicable |
| test_transport_failures.py::test_invalid_utf8_error_does_not_retain_response | invalid_utf8_error_does_not_retain_response | direct call |
| test_transport_failures.py::test_normally_returned_server_error_uses_status_for_retry | normally_returned_server_error_uses_status_for_retry (4 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_readable_5xx_then_open_failure_never_returns_stale_response | readable_5xx_then_open_failure_never_returns_stale_response (both cases are one wire reply; the frame-locals check maps to the release check before backoff) | Dial with a scripted connection |
| test_transport_failures.py::test_final_error_detaches_ambient_context_and_opaque_callback | final_error_detaches_ambient_context_and_opaque_callback (4 cases looped; no ambient exception context in Rust, so the contract is that a canary held by the backoff closure or an opaque object never reaches the error) | Dial with a scripted connection; direct call for `text()` |
| test_transport_failures.py::test_backoff_exceptions_propagate_after_closure | backoff_exceptions_propagate_after_closure (4 cases looped) | Dial with a scripted connection; sleep returning the exception |
| test_transport_failures.py::test_unrelated_open_errors_are_not_converted | unrelated_open_errors_are_not_converted (2 cases looped) | Dial returning `transport::Raised` |
| test_transport_failures.py::test_close_failure_preserves_primary_body_failure | close_failure_preserves_primary_body_failure (3 cases looped; the failing close has no counterpart, the primary failure message and budget are asserted) | Dial with a scripted connection |
| test_transport_failures.py::test_invalid_framing_uses_real_opener_and_status_budget | invalid_framing_uses_real_opener_and_status_budget (24 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_status_parsed_before_bad_headers_controls_retry | status_parsed_before_bad_headers_controls_retry (12 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_effective_framing_preserves_valid_and_bodyless_responses | effective_framing_preserves_valid_and_bodyless_responses (6 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_truncated_valid_json_discovery_cannot_be_computed | truncated_valid_json_discovery_cannot_be_computed | Dial with a scripted connection, through `Client::discover` |
| test_transport_failures.py::test_real_opener_ceiling_stops_without_retry | real_opener_ceiling_stops_without_retry (3 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_real_opener_pre_status_failure_still_retries | real_opener_pre_status_failure_still_retries | Dial with a scripted connection |
| test_transport_failures.py::test_https_installs_framing_guards_with_verified_tls_defaults | https_installs_framing_guards_with_verified_tls_defaults (3 cases looped; asserts the dial is asked for TLS to 127.0.0.1:9 and the framing guards apply; the SSLContext and stdlib response_class checks have no counterpart, as `dial::Net` verifies with the platform verifier) | Dial with a scripted connection |
| test_transport_failures.py::test_premature_chunk_trailer_eof_is_not_completion | premature_chunk_trailer_eof_is_not_completion (24 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_invalid_length_is_not_close_delimited_success | invalid_length_is_not_close_delimited_success (21 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_initial_header_eof_is_not_completion | initial_header_eof_is_not_completion (3 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_known_status_survives_secondary_opener_cleanup | known_status_survives_secondary_opener_cleanup (6 statuses looped; the stream and socket close failures have no counterpart) | Dial with a scripted connection |
| test_transport_failures.py::test_primary_exception_survives_secondary_opener_cleanup | not applicable: the primary is a non-I/O exception raised by a header read, and a Rust read can only return `io::Error`; close failures have no counterpart | not applicable |
| test_transport_failures.py::test_sole_socket_close_failure_retains_parsed_status | not applicable: the only failure is the socket's close, and dropping a Rust connection cannot fail, so the complete response is returned | not applicable |
| test_transport_failures.py::test_completed_framing_and_bodyless_controls | completed_framing_and_bodyless_controls (10 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_invalid_message_discovery_cannot_be_computed | invalid_message_discovery_cannot_be_computed (2 cases looped) | Dial with a scripted connection, through `Client::discover` |
| test_transport_failures.py::test_https_eof_suppression_branch | https_eof_suppression_branch (18 cases looped; a TLS close without close_notify is the `UnexpectedEof` read error rustls returns) | Dial with a scripted connection |
| test_transport_failures.py::test_invalid_chunk_transition_is_not_completion | invalid_chunk_transition_is_not_completion (15 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_bad_chunk_delimiter_cannot_be_computed | bad_chunk_delimiter_cannot_be_computed | Dial with a scripted connection, through `Client::discover` |
| test_transport_failures.py::test_valid_hex_sizes_and_extensions | valid_hex_sizes_and_extensions | Dial with a scripted connection |
| test_transport_failures.py::test_transfer_coding_takes_precedence_over_length | transfer_coding_takes_precedence_over_length (2 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_transfer_coding_cannot_hide_bytes_above_ceiling | transfer_coding_cannot_hide_bytes_above_ceiling | Dial with a scripted connection |
| test_transport_failures.py::test_redirect_without_location_releases_unread_body | redirect_without_location_releases_unread_body (5 statuses looped; the close_failure axis has no counterpart) | Dial with a scripted connection |
| test_transport_failures.py::test_non_io_cleanup_preserves_primary_opener_exception | not applicable: every stage raises a non-I/O exception from a read, the request write or a close, none of which a Rust connection can raise | not applicable |
| test_transport_failures.py::test_non_io_cleanup_preserves_primary_body_exception | not applicable: a non-I/O exception from a body read and a failing close have no Rust counterpart | not applicable |
| test_transport_failures.py::test_sole_non_io_body_close_exception_propagates | not applicable: the only failure is a close that raises, which dropping a Rust connection cannot do | not applicable |
| test_transport_failures.py::test_non_io_close_failure_preserves_recorded_body_failure | non_io_close_failure_preserves_recorded_body_failure (4 cases looped; the RuntimeError from close has no counterpart, the recorded failure and budget are asserted) | Dial with a scripted connection |
| test_transport_failures.py::test_cancellation_during_close_after_recorded_body_failure_propagates | not applicable: a cancellation raised by close has no Rust counterpart | not applicable |
| test_transport_failures.py::test_body_close_cancellation_precedence | not applicable: precedence between non-I/O exceptions from a body read and its close has no Rust counterpart | not applicable |
| test_transport_failures.py::test_redirect_close_exception_keeps_refusal | redirect_close_exception_keeps_refusal (location axis looped; the close exception axis has no counterpart) | Dial with a scripted connection |
| test_transport_failures.py::test_redirect_close_cancellation_propagates | not applicable: the only distinctive behaviour is a cancellation raised by close | not applicable |
| test_transport_failures.py::test_header_cleanup_cancellation_propagates | not applicable: the only distinctive behaviour is a cancellation raised by a stream or socket close | not applicable |
| test_transport_failures.py::test_header_cleanup_cancellation_precedence | not applicable: precedence between non-I/O exceptions from a header read and a close has no Rust counterpart | not applicable |
| test_transport_failures.py::test_failed_begin_does_not_suppress_later_standalone_close_exception | not applicable: exercises `_HTTPResponse.begin` and `close` directly; the Rust response is private and its close cannot raise | not applicable |
| test_transport_failures.py::test_non_io_header_cleanup_preserves_known_status | non_io_header_cleanup_preserves_known_status (2 cases looped; the close failures have no counterpart) | Dial with a scripted connection |
| test_transport_failures.py::test_an_invalid_transfer_coding_list_is_not_completion | an_invalid_transfer_coding_list_is_not_completion (2 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_a_chunk_size_line_past_the_line_limit_is_not_completion | a_chunk_size_line_past_the_line_limit_is_not_completion | Dial with a scripted connection |
| test_transport_failures.py::test_a_chunk_larger_than_one_read_is_read_across_reads | a_chunk_larger_than_one_read_is_read_across_reads | Dial with a scripted connection |
| test_transport_failures.py::test_an_oversized_or_endless_trailer_is_not_completion | an_oversized_or_endless_trailer_is_not_completion (2 cases looped) | Dial with a scripted connection |
| test_transport_failures.py::test_a_primary_cancellation_survives_a_socket_close_cancellation | not applicable: both exceptions are non-I/O ones from a read and a socket close, which a Rust connection cannot raise | not applicable |
| test_transport_failures.py::test_a_response_without_a_status_is_treated_as_no_response | a_response_without_a_status_is_treated_as_no_response (Python's status-less HTTPError is a body with no status line on the wire) | Dial with a scripted connection |
| test_transport_failures.py::test_nonpositive_attempt_limit_refuses_before_sending | nonpositive_attempt_limit_refuses_before_sending (`max_attempts: 0` set on the struct, bypassing `checked()`; a dial panics if reached) | Dial with a scripted connection |

## `test_trials_div7a.py` to `trials_div7a.rs`

| Python test | Rust test | Seam |
|---|---|---|
| test_trials_div7a.py::test_the_case_file_is_synthetic_and_carries_a_derivation_for_every_case | the_case_file_is_synthetic_and_carries_a_derivation_for_every_case (reads `fixtures/div7a-cases.json`, the file the trial embeds) | direct call |
| test_trials_div7a.py::test_the_period_map_agrees_with_the_local_rate_table | the_period_map_agrees_with_the_local_rate_table (Python `load_table()` is the engine's `frozen_table()`) | direct call (in-tree `div7a-loan-review`) |
| test_trials_div7a.py::test_the_local_engine_reproduces_the_independently_derived_figure | the_local_engine_reproduces_the_independently_derived_figure (6 cases looped) | direct call |
| test_trials_div7a.py::test_a_nil_remaining_term_is_refused_by_the_local_engine_not_divided | a_nil_remaining_term_is_refused_by_the_local_engine_not_divided | loopback server (request count 0) |
| test_trials_div7a.py::test_a_matching_provider_answer_is_a_match | a_matching_provider_answer_is_a_match | loopback server |
| test_trials_div7a.py::test_a_cent_of_disagreement_is_a_numeric_difference_not_a_match | a_cent_of_disagreement_is_a_numeric_difference_not_a_match | loopback server |
| test_trials_div7a.py::test_a_term_convention_difference_is_scope_not_arithmetic | a_term_convention_difference_is_scope_not_arithmetic | loopback server |
| test_trials_div7a.py::test_the_rate_trap_case_reports_a_difference_and_names_the_rate | the_rate_trap_case_reports_a_difference_and_names_the_rate | loopback server |
| test_trials_div7a.py::test_the_rate_trap_case_reaches_the_outcome_its_fixture_records | the_rate_trap_case_reaches_the_outcome_its_fixture_records | loopback server |
| test_trials_div7a.py::test_an_unsupported_period_is_its_own_outcome | an_unsupported_period_is_its_own_outcome | loopback server (request count 0) |
| test_trials_div7a.py::test_a_provider_outage_never_becomes_agreement | a_provider_outage_never_becomes_agreement | loopback server |
| test_trials_div7a.py::test_the_providers_compliance_label_is_recorded_and_never_mapped | the_providers_compliance_label_is_recorded_and_never_mapped (`json.dumps` is `pyjson::dumps`; the check is on content) | loopback server |
| test_trials_div7a.py::test_the_local_side_runs_without_a_client_and_says_so | the_local_side_runs_without_a_client_and_says_so | direct call |
| test_trials_div7a.py::test_an_evaluation_outcome_is_not_an_accounting_state | an_evaluation_outcome_is_not_an_accounting_state | direct call |
| test_trials_div7a.py::test_the_case_file_falls_back_to_the_source_checkout_path | the_case_file_falls_back_to_the_source_checkout_path (nearest contract: Rust has no `_FIXTURE_DIRS` search because it embeds the source checkout's case file with `include_str!`; the test asserts the packaged cases equal that file's cases) | direct call |
| test_trials_div7a.py::test_the_local_side_carries_only_the_figures_the_engine_produced | the_local_side_carries_only_the_figures_the_engine_produced (nearest contract: no seam replaces `minimum_yearly_repayment`, so a real figureless engine result, D7A-8, is compared with a direct engine call: same verdict, only the figures the engine produced, the engine's last reason last) | direct call (in-tree engine in place of the monkeypatch) |
| test_trials_div7a.py::test_only_the_figures_the_provider_returned_are_extracted | only_the_figures_the_provider_returned_are_extracted | direct call |
| test_trials_div7a.py::test_a_provider_answer_without_a_figure_is_named_for_why | a_provider_answer_without_a_figure_is_named_for_why (3 cases looped) | fabricated `Outcome` to `evaluate_outcome` |
| test_trials_div7a.py::test_a_refusal_without_a_class_says_so | a_refusal_without_a_class_says_so | fabricated `Outcome` to `evaluate_outcome` |
| test_trials_div7a.py::test_a_computed_answer_without_a_compliance_label_records_none | a_computed_answer_without_a_compliance_label_records_none | loopback server |

## `test_trials_fbt.py` to `trials_fbt.rs`

| Python test | Rust test | Seam |
|---|---|---|
| test_trials_fbt.py::test_only_the_category_taxable_value_crosses_into_the_worksheet | only_the_category_taxable_value_crosses_into_the_worksheet | direct call |
| test_trials_fbt.py::test_an_already_grossed_up_figure_is_refused_at_the_door | an_already_grossed_up_figure_is_refused_at_the_door (2 fields looped) | direct call |
| test_trials_fbt.py::test_double_gross_up_is_what_the_refusal_prevents | double_gross_up_is_what_the_refusal_prevents | direct call |
| test_trials_fbt.py::test_an_unknown_classification_refuses_before_the_call | an_unknown_classification_refuses_before_the_call (`ClassificationRequired` is the error class `lodgeitadapter.trials.fbt.ClassificationRequired`) | direct call |
| test_trials_fbt.py::test_a_period_with_no_local_counterpart_is_unsupported | a_period_with_no_local_counterpart_is_unsupported | direct call |
| test_trials_fbt.py::test_the_join_carries_the_category_value_and_agrees_on_the_gross_up | the_join_carries_the_category_value_and_agrees_on_the_gross_up (asserts `local["type_two_grossed_up"]`, not `local` whole: the in-tree worksheet also returns `return_item_14a`, `return_item_14b`, `return_item_15` and `return_item_16`) | loopback server |
| test_trials_fbt.py::test_a_gross_up_rate_difference_shows_up_as_a_difference | a_gross_up_rate_difference_shows_up_as_a_difference | loopback server |
| test_trials_fbt.py::test_a_response_without_a_taxable_value_produces_nothing | a_response_without_a_taxable_value_produces_nothing (`local == {}`: no worksheet runs) | loopback server |
| test_trials_fbt.py::test_a_provider_refusal_does_not_reach_the_worksheet | a_provider_refusal_does_not_reach_the_worksheet (`local == {}`: no worksheet runs) | loopback server |
| test_trials_fbt.py::test_without_a_client_the_provider_is_not_called | without_a_client_the_provider_is_not_called | direct call |
| test_trials_fbt.py::test_a_call_that_produced_no_figure_is_named_for_why | a_call_that_produced_no_figure_is_named_for_why (5 cases looped) | fabricated `Outcome` to `evaluate_outcome` |
| test_trials_fbt.py::test_a_computed_outcome_without_a_taxable_value_reaches_no_worksheet | a_computed_outcome_without_a_taxable_value_reaches_no_worksheet | fabricated `Outcome` to `evaluate_outcome` |
| test_trials_fbt.py::test_a_figure_only_the_provider_returned_is_named | a_figure_only_the_provider_returned_is_named (nearest contract: Python monkeypatches `run_local_aggregate` to return `{}`; Rust has no seam there and the real worksheet always returns both derived figures, so the provider-only branch cannot be reached. With the same fabricated values, the test asserts the mirror branch's exact reason, `fbt_estimate: the local worksheet produced a figure and the provider returned no fbt_payable`, that no grossed-up figure is named as provider-only, and SCOPE_MISMATCH) | fabricated `Outcome` to `evaluate_outcome` (real worksheet in place of the monkeypatch) |

## `test_trials_depreciation.py` to `trials_depreciation.rs`

| Python test | Rust test | Seam |
|---|---|---|
| test_trials_depreciation.py::test_the_request_requires_a_day_count_and_rejects_an_unknown_one | the_request_requires_a_day_count_and_rejects_an_unknown_one | direct call |
| test_trials_depreciation.py::test_a_closing_movement_is_a_match | a_closing_movement_is_a_match | loopback server |
| test_trials_depreciation.py::test_a_movement_that_does_not_close_is_refused_and_no_addition_is_invented | a_movement_that_does_not_close_is_refused_and_no_addition_is_invented | loopback server |
| test_trials_depreciation.py::test_a_separately_evidenced_addition_closes_it_and_is_named | a_separately_evidenced_addition_closes_it_and_is_named | loopback server |
| test_trials_depreciation.py::test_reconcile_refuses_rather_than_deriving_the_gap | reconcile_refuses_rather_than_deriving_the_gap (Python's default tolerance `Decimal("0.01")` is passed explicitly) | direct call |
| test_trials_depreciation.py::test_a_missing_figure_cannot_be_reconciled_around | a_missing_figure_cannot_be_reconciled_around | direct call |
| test_trials_depreciation.py::test_the_pre_acquisition_window_is_recognised_before_the_call | the_pre_acquisition_window_is_recognised_before_the_call | direct call |
| test_trials_depreciation.py::test_consecutive_windows_telescope_and_cover_every_day | consecutive_windows_telescope_and_cover_every_day | direct call |
| test_trials_depreciation.py::test_consecutive_ranges_sum_to_the_whole | consecutive_ranges_sum_to_the_whole (one client, 2 calls) | loopback server |
| test_trials_depreciation.py::test_life_end_leaves_nothing | life_end_leaves_nothing | loopback server |
| test_trials_depreciation.py::test_a_pooled_asset_refusal_stays_a_scope_mismatch | a_pooled_asset_refusal_stays_a_scope_mismatch | loopback server |
| test_trials_depreciation.py::test_without_a_client_the_provider_is_not_called | without_a_client_the_provider_is_not_called | direct call |
| test_trials_depreciation.py::test_a_call_that_produced_no_figure_is_named_for_why | a_call_that_produced_no_figure_is_named_for_why (6 cases looped) | fabricated `Outcome` to `evaluate_outcome` |
| test_trials_depreciation.py::test_a_movement_after_acquisition_that_does_not_close_names_no_limitation | a_movement_after_acquisition_that_does_not_close_names_no_limitation | loopback server |

## `test_trials_fano.py` to `trials_fano.rs`

| Python test | Rust test | Seam |
|---|---|---|
| test_trials_fano.py::test_a_trial_balance_that_does_not_balance_is_refused_locally | a_trial_balance_that_does_not_balance_is_refused_locally | direct call |
| test_trials_fano.py::test_the_provider_limits_are_checked_before_sending | the_provider_limits_are_checked_before_sending (3 refusals looped) | direct call |
| test_trials_fano.py::test_every_line_keeps_its_original_alongside_the_suggestion | every_line_keeps_its_original_alongside_the_suggestion | loopback server (`fano-classifier` snapshot) |
| test_trials_fano.py::test_accepted_fact_is_not_an_approval | accepted_fact_is_not_an_approval | loopback server |
| test_trials_fano.py::test_a_draft_fact_is_a_correct_outcome_and_is_carried_through | a_draft_fact_is_a_correct_outcome_and_is_carried_through | loopback server |
| test_trials_fano.py::test_a_topology_disagreement_is_flagged | a_topology_disagreement_is_flagged | loopback server |
| test_trials_fano.py::test_an_unknown_status_leaves_the_line_unreviewed | an_unknown_status_leaves_the_line_unreviewed | loopback server |
| test_trials_fano.py::test_a_row_count_mismatch_proposes_nothing | a_row_count_mismatch_proposes_nothing | loopback server |
| test_trials_fano.py::test_a_reported_imbalance_proposes_nothing | a_reported_imbalance_proposes_nothing | loopback server |
| test_trials_fano.py::test_an_unavailable_service_proposes_nothing | an_unavailable_service_proposes_nothing | loopback server |
| test_trials_fano.py::test_an_authentication_refusal_is_reported_not_worked_around | an_authentication_refusal_is_reported_not_worked_around | loopback server |
| test_trials_fano.py::test_a_code_needs_a_reviewed_crosswalk_before_it_means_anything | a_code_needs_a_reviewed_crosswalk_before_it_means_anything | direct call |
| test_trials_fano.py::test_the_suggestion_record_never_carries_an_applied_mapping | the_suggestion_record_never_carries_an_applied_mapping (`json.dumps` is `pyjson::dumps` with the indented layout, whose `": "` key separator matches) | loopback server |
| test_trials_fano.py::test_a_line_with_an_unknown_topology_is_refused_before_sending | a_line_with_an_unknown_topology_is_refused_before_sending | direct call |
| test_trials_fano.py::test_without_a_client_the_classifier_is_not_called | without_a_client_the_classifier_is_not_called | direct call |
| test_trials_fano.py::test_a_response_without_a_results_array_proposes_nothing | a_response_without_a_results_array_proposes_nothing | loopback server |
| test_trials_fano.py::test_an_echoed_description_that_differs_is_flagged | an_echoed_description_that_differs_is_flagged | loopback server |

## `test_wire_precision.py` to `wire_precision.rs`

| Python test | Rust test | Seam |
| --- | --- | --- |
| `test_wire_precision.py::test_a_decimal_is_emitted_as_its_own_digits` | `a_decimal_is_emitted_as_its_own_digits` | direct call (`wire::dumps`; the `json.dumps(float)` contrast is `pyjson::dumps` of a `Json::Float`) |
| `test_wire_precision.py::test_a_float_never_reaches_the_wire` | `a_float_never_reaches_the_wire` | direct call |
| `test_wire_precision.py::test_trailing_zeros_and_scale_survive` | `trailing_zeros_and_scale_survive` | direct call |
| `test_wire_precision.py::test_unrepresentable_values_are_refused` | `unrepresentable_values_are_refused` | direct call |
| `test_wire_precision.py::test_reading_refuses_a_json_float_and_accepts_a_decimal_string` | `reading_refuses_a_json_float_and_accepts_a_decimal_string` | direct call |
| `test_wire_precision.py::test_the_bytes_the_stub_receives_carry_the_exact_digits` | `the_bytes_the_stub_receives_carry_the_exact_digits` | loopback server (`std::net::TcpListener` stub, `dial::Net`) |
| `test_wire_precision.py::test_a_provider_json_number_keeps_its_own_digits_on_the_way_in` | `a_provider_json_number_keeps_its_own_digits_on_the_way_in` | loopback server |
| `test_wire_precision.py::test_a_nan_in_a_response_is_a_contract_failure` | `a_nan_in_a_response_is_a_contract_failure` | loopback server |
