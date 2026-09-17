"""Regressions for defects an independent review pass demonstrated.

One test per defect, each named for the behaviour it pins rather than for the
review item, because the review is not the thing that has to keep being true.
Everything here runs offline against the loopback stub.
"""

from __future__ import annotations

import json
from decimal import Decimal

import pytest

from lodgeitadapter import AdapterConfig, LodgeitClient, Status, cli, evidence
from lodgeitadapter.errors import ContractError, DisallowedTargetError
from lodgeitadapter.trials import Evaluation, compare_values
from lodgeitadapter.trials import div7a as div7a_trial
from lodgeitadapter.trials import fbt as fbt_trial

HOST = "https://fbt-calculator-api-qkp3j5bjnq-ts.a.run.app"


# -- the route allowlist ---------------------------------------------------

@pytest.mark.parametrize("path", [
    # A dot segment lands wherever the first thing that normalises decides.
    "/v1/calculators/../../admin",
    "/v1/calculators/./../admin",
    # Percent-encoded, so a check that never decodes sees no dot segment.
    "/v1/calculators/%2e%2e/%2e%2e/admin",
    # An encoded slash: one segment before decoding, three after it.
    "/v1/calculators/..%2f..%2fadmin",
    # A backslash is a path delimiter to a browser and to some servers.
    "/v1/calculators/..%5c..%5cadmin",
    # Prefix extension: `/v1/ratesheet` starts with `/v1/rates` and is not it.
    "/v1/ratesheet-evil",
    "/v1/calculatorsX",
])
def test_a_path_that_is_not_a_route_is_refused_whatever_it_is_dressed_as(path):
    config = AdapterConfig(enabled=True)
    with pytest.raises(DisallowedTargetError):
        config.check_url(HOST + path)


@pytest.mark.parametrize("path", [
    "/v1/calculators",
    "/v1/calculators/div7a/at/urn:sbrm:period:div7a:fy2026",
    "/v1/rates/urn:sbrm:period:div7a:fy2026",
    "/openapi.json",
    "/healthz",
])
def test_the_routes_this_adapter_actually_calls_still_pass(path):
    assert AdapterConfig(enabled=True).check_url(HOST + path) == HOST + path


def test_a_port_off_loopback_is_part_of_the_destination():
    with pytest.raises(DisallowedTargetError, match="port 9999"):
        AdapterConfig(enabled=True).check_url(f"{HOST}:9999/v1/calculators")


def test_a_base_url_carrying_a_query_is_refused_at_construction():
    # `base + path` put the whole route inside the query string, so every
    # invocation reached the base URL's own route and nothing said so.
    with pytest.raises(DisallowedTargetError, match="query string"):
        AdapterConfig(enabled=True, base_url=f"{HOST}?trace=1")


def test_an_ephemeral_loopback_port_is_still_reachable_for_a_stub(stub_config):
    assert stub_config.check_url(stub_config.base_url + "/v1/calculators")


# -- what a refused response may hand back ---------------------------------

def test_a_contract_failure_hands_back_no_result(stub, contract):
    base_url, state = stub
    state.respond({
        "statutory_myr": "21874.92", "total_repayments": "22000.00",
        "shortfall": "0.00", "benchmark_rate": "0.0837",
        # No manifest and no advisory: the response is refused.
    })
    client = LodgeitClient(
        AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1),
        contract,
    )
    outcome = client.invoke(
        "urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026",
        {"amalgamated_base": Decimal("100000.00")},
    )
    assert outcome.status is Status.CONTRACT_FAILURE
    # The money fields of a body this adapter just refused must not travel as
    # a result. The body itself is preserved for audit in raw_response.
    assert outcome.result is None
    assert outcome.values == {}
    assert outcome.raw_response["statutory_myr"] == "21874.92"


def test_an_empty_manifest_is_not_a_manifest(stub, contract):
    base_url, state = stub
    state.respond({
        "statutory_myr": "21874.92", "total_repayments": "22000.00",
        "shortfall": "0.00", "benchmark_rate": "0.0837",
        "manifest": {},
        "advisory": {"disclaimer": "Calculated on supplied facts."},
    })
    client = LodgeitClient(
        AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1),
        contract,
    )
    outcome = client.invoke(
        "urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026",
        {"amalgamated_base": Decimal("100000.00")},
    )
    assert outcome.status is Status.CONTRACT_FAILURE
    assert any("does not name what it consumed" in item for item in outcome.findings)


@pytest.mark.parametrize("status", [401, 403])
def test_a_provider_that_declines_to_answer_is_a_refusal_not_a_contract_failure(
    stub, contract, status,
):
    base_url, state = stub
    state.respond({"detail": "Not authenticated"}, status=status)
    client = LodgeitClient(
        AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1),
        contract,
    )
    outcome = client.invoke(
        "urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026",
        {"amalgamated_base": Decimal("100000.00")},
    )
    assert outcome.status is Status.UPSTREAM_REFUSED
    assert any("declined to answer" in item for item in outcome.findings)


def test_a_body_that_cannot_be_encoded_is_a_refusal_not_a_traceback(stub, contract):
    base_url, _ = stub
    client = LodgeitClient(
        AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1),
        contract,
    )
    # A lone surrogate survives the serialiser and fails at .encode("utf-8").
    outcome = client.invoke(
        "urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026",
        {"amalgamated_base": Decimal("100000.00"), "note": "\ud800"},
    )
    assert outcome.status is Status.CONTRACT_FAILURE
    assert any("could not be serialised" in item for item in outcome.findings)


# -- reading a request body from a file ------------------------------------

def test_an_identifier_is_not_a_number_just_because_it_parses_as_one():
    fields = ("amalgamated_base", "repayments[].amount")
    body = {
        "amalgamated_base": "100000.00",
        "reference": "0012",
        "abn": "51824753556",
        "loan_origination_date": "2024-09-01",
        "repayments": [{"amount": "11000.00", "date": "2025-12-15"}],
    }
    converted = cli._decimalise(body, fields)
    assert converted["amalgamated_base"] == Decimal("100000.00")
    assert converted["repayments"][0]["amount"] == Decimal("11000.00")
    # These are strings in the provider's schema and their digits are an
    # identity, not a quantity. A leading zero dropped is a different reference.
    assert converted["reference"] == "0012"
    assert converted["abn"] == "51824753556"
    assert converted["loan_origination_date"] == "2024-09-01"
    assert body["amalgamated_base"] == "100000.00"  # the caller's dict is untouched


def test_a_named_number_field_that_is_not_a_number_is_a_usage_error():
    with pytest.raises(ValueError, match="amalgamated_base"):
        cli._decimalise({"amalgamated_base": "not a number"}, ("amalgamated_base",))


def test_an_absent_optional_number_field_is_not_an_error():
    assert cli._decimalise({}, ("amalgamated_base", "asset.cost")) == {}


def test_the_snapshot_names_the_repayment_amounts_as_numbers(contract):
    # The trial sends them as numbers, so a body read from a file must too,
    # or the same case would go on the wire two different ways.
    recorded = contract.calculators["urn:sbrm:calculator:div7a:at"]
    assert "repayments[].amount" in recorded["request_number_fields"]


# -- comparing two engines -------------------------------------------------

def test_exact_agreement_with_an_uncompared_field_is_scope_not_arithmetic():
    expected = {"statutory_myr": Decimal("100.00"), "deemed_dividend": Decimal("0.00")}
    local = {"statutory_myr": Decimal("100.00")}
    upstream = {"statutory_myr": Decimal("100.00")}
    evaluation, differences, reasons = compare_values(expected, local, upstream)
    # Every figure both sides produced agreed to the cent. What is missing is
    # coverage, and calling that a numeric difference names a disagreement the
    # comparison did not find.
    assert evaluation is Evaluation.SCOPE_MISMATCH
    assert differences == {}
    assert reasons


def test_a_figure_only_one_side_produced_is_named(stub=None):
    local = {"statutory_myr": Decimal("100.00"), "interest_accrued": Decimal("7.00")}
    upstream = {"statutory_myr": Decimal("100.00")}
    evaluation, differences, reasons = compare_values({}, local, upstream)
    assert evaluation is Evaluation.SCOPE_MISMATCH
    assert any("interest_accrued" in reason for reason in reasons)


def test_full_agreement_on_every_field_is_still_a_match():
    values = {"statutory_myr": Decimal("100.00")}
    evaluation, differences, reasons = compare_values(values, dict(values), dict(values))
    assert evaluation is Evaluation.MATCH
    assert differences == {} and reasons == ()


# -- the FBT join ----------------------------------------------------------

def test_a_grossed_up_value_is_refused_by_its_value_not_only_by_its_name():
    # The README says the double gross-up cannot happen. A check on the field
    # name alone did not stop a caller that read the wrong field and kept the
    # default name, so the value is checked against what the provider reported.
    with pytest.raises(ContractError, match="already grossed up"):
        fbt_trial.prepare_aggregate_input(
            Decimal("15094.40"), "Type 2",
            response_values={
                "taxable_value": Decimal("8000.00"),
                "grossed_up_taxable_value": Decimal("15094.40"),
            },
        )


def test_the_category_value_itself_still_passes_that_check():
    arguments = fbt_trial.prepare_aggregate_input(
        Decimal("8000.00"), "Type 2",
        response_values={
            "taxable_value": Decimal("8000.00"),
            "grossed_up_taxable_value": Decimal("15094.40"),
            "fbt_payable": Decimal("7094.37"),
        },
    )
    assert arguments["type_two_value"] == Decimal("8000.00")


def _fbt_client(stub, contract):
    base_url, state = stub
    return LodgeitClient(
        AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1),
        contract,
    ), state


def _fbt_response(**overrides):
    payload = {
        "taxable_value": "8000.00",
        "grossed_up_taxable_value": "15094.40",
        "fbt_payable": "7094.37",
        "manifest": {"rate_uris_consumed": ["urn:sbrm:rate:fbt:type2:fy2026"]},
        "advisory": {"disclaimer": "Calculated on supplied facts."},
    }
    payload.update(overrides)
    return payload


def _benefit():
    return fbt_trial.CarBenefit(
        case_id="FBT-REG-1",
        period_uri=fbt_trial.PERIOD_URI,
        base_value=Decimal("40000.00"),
        days_available=Decimal("365"),
        accessories=Decimal("0.00"),
        employee_contribution=Decimal("0.00"),
        fbt_type="Type 2",
    )


def test_the_fbt_trial_compares_the_payable_before_it_says_match(stub, contract):
    client, state = _fbt_client(stub, contract)
    state.respond(_fbt_response())
    comparison = fbt_trial.evaluate(_benefit(), client)
    assert comparison.evaluation is Evaluation.MATCH


def test_an_fbt_payable_that_differs_is_a_numeric_difference(stub, contract):
    client, state = _fbt_client(stub, contract)
    # The gross-up agrees to the cent; only the payable moves, which is the
    # figure that reaches a return and the one MATCH was never checking.
    state.respond(_fbt_response(fbt_payable="6000.00"))
    comparison = fbt_trial.evaluate(_benefit(), client)
    assert comparison.evaluation is Evaluation.NUMERIC_DIFFERENCE
    assert "fbt_payable" in comparison.differences


def test_an_fbt_response_with_nothing_to_compare_is_not_a_match(stub, contract):
    client, state = _fbt_client(stub, contract)
    payload = _fbt_response()
    del payload["grossed_up_taxable_value"]
    del payload["fbt_payable"]
    state.respond(payload)
    comparison = fbt_trial.evaluate(_benefit(), client)
    assert comparison.evaluation is Evaluation.SCOPE_MISMATCH


def test_an_fbt_response_missing_only_the_payable_is_not_a_match(stub, contract):
    client, state = _fbt_client(stub, contract)
    payload = _fbt_response()
    del payload["fbt_payable"]
    state.respond(payload)
    comparison = fbt_trial.evaluate(_benefit(), client)
    assert comparison.evaluation is Evaluation.SCOPE_MISMATCH


# -- evidence records ------------------------------------------------------

def _record(stub, contract):
    base_url, state = stub
    state.respond({
        "statutory_myr": "21874.92", "total_repayments": "22000.00",
        "shortfall": "0.00", "benchmark_rate": "0.0837",
        "manifest": {"rate_uris_consumed": ["urn:sbrm:rate:div7a:fy2026"]},
        "advisory": {"disclaimer": "Calculated on supplied facts."},
    })
    client = LodgeitClient(
        AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1),
        contract,
    )
    outcome = client.invoke(
        "urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026",
        {"amalgamated_base": Decimal("100000.00")},
    )
    return evidence.build(outcome, label="div7a-minimum-yearly-repayment", synthetic=True)


def test_a_digest_mismatch_names_the_digest_it_actually_computed(stub, contract):
    record = _record(stub, contract)
    record["calculation"]["label"] = "tampered-label"
    findings = evidence.verify(record)
    assert findings
    # The message used to print the placeholder instead of the value, which is
    # the one thing a reader needs to compare against the file.
    assert "{actual}" not in findings[0]
    assert evidence.digest_of(record) in findings[0]


def test_verify_reports_a_null_block_rather_than_crashing(stub, contract):
    record = _record(stub, contract)
    record["calculation"]["call"] = None
    record["calculation_sha256"] = evidence.digest_of(record)
    findings = evidence.verify(record)
    assert any("no call block" in item for item in findings)


def test_the_command_line_verify_reports_a_null_block(tmp_path, stub, contract):
    record = _record(stub, contract)
    record["calculation"]["call"] = None
    record["calculation_sha256"] = evidence.digest_of(record)
    path = tmp_path / "evidence.json"
    path.write_text(json.dumps(record), encoding="utf-8")
    assert cli.main(["verify", "--evidence", str(path)]) == 1


@pytest.mark.parametrize("label", [
    "Div7A MYR",          # spaces and capitals
    "div7a_myr",          # an underscore
    "div7a--myr",         # a double hyphen
    "-div7a",             # a leading hyphen
    "div7a" + chr(0) + "myr",  # a control character
    "d" * 121,            # longer than the consumer accepts
])
def test_a_label_the_consumer_would_refuse_is_refused_here(stub, contract, label):
    # The monthly-close control plane keys a source digest on this label and
    # requires a slug. A record written with anything else is BLOCKED there,
    # which makes it a defect in the producer.
    base_url, state = stub
    state.respond({"manifest": {}, "advisory": {}})
    client = LodgeitClient(
        AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1),
        contract,
    )
    outcome = client.discover()
    with pytest.raises(ValueError, match="label"):
        evidence.build(outcome, label=label, synthetic=True)


# -- what the fixtures claim -----------------------------------------------

def test_every_case_records_the_verdict_the_local_engine_actually_reaches():
    # The parametrised suite covered six of the nine cases, so two fixtures
    # recorded expectations nothing ever ran.
    for case in div7a_trial.load_cases():
        verdict, _values, _reasons = div7a_trial.run_local(case)
        assert verdict == case.expect_local, case.case_id


def test_the_command_line_writes_no_evidence_under_a_label_the_consumer_refuses(
    tmp_path, stub, contract, monkeypatch,
):
    base_url, state = stub
    state.respond([{"calculator_uri": "urn:sbrm:calculator:div7a:at"}])
    out = tmp_path / "evidence.json"
    code = cli.main([
        "invoke", "--enable-network", "--allow-loopback", "--base-url", base_url,
        "--calculator", "urn:sbrm:calculator:div7a:at",
        "--period", "urn:sbrm:period:div7a:fy2026",
        "--body", "fixtures/example-div7a-request.json",
        "--label", "Manual Invocation", "--evidence-out", str(out),
    ])
    assert code == 2
    assert not out.exists()


def test_the_default_label_is_one_the_consumer_accepts(tmp_path, stub, contract):
    base_url, state = stub
    state.respond({
        "statutory_myr": "21874.92", "total_repayments": "22000.00",
        "shortfall": "0.00", "benchmark_rate": "0.0837",
        "manifest": {"rate_uris_consumed": ["urn:sbrm:rate:div7a:fy2026"]},
        "advisory": {"disclaimer": "Calculated on supplied facts."},
    })
    out = tmp_path / "evidence.json"
    code = cli.main([
        "invoke", "--enable-network", "--allow-loopback", "--base-url", base_url,
        "--calculator", "urn:sbrm:calculator:div7a:at",
        "--period", "urn:sbrm:period:div7a:fy2026",
        "--body", "fixtures/example-div7a-request.json",
        "--evidence-out", str(out),
    ])
    assert code == 0
    assert json.loads(out.read_text(encoding="utf-8"))["calculation"]["label"] == (
        "manual-invocation"
    )


def test_a_repayment_amount_read_from_a_file_goes_on_the_wire_as_a_number(
    tmp_path, stub, contract,
):
    # The trial sends these as numbers. A body read from a file has to send
    # them the same way, or the same case reaches the provider two ways.
    base_url, state = stub
    state.respond({
        "statutory_myr": "21874.92", "total_repayments": "22000.00",
        "shortfall": "0.00", "benchmark_rate": "0.0837",
        "manifest": {"rate_uris_consumed": ["urn:sbrm:rate:div7a:fy2026"]},
        "advisory": {"disclaimer": "Calculated on supplied facts."},
    })
    assert cli.main([
        "invoke", "--enable-network", "--allow-loopback", "--base-url", base_url,
        "--calculator", "urn:sbrm:calculator:div7a:at",
        "--period", "urn:sbrm:period:div7a:fy2026",
        "--body", "fixtures/example-div7a-request.json",
    ]) == 0
    sent = state.requests[-1]["body"].decode("utf-8")
    assert '"amalgamated_base":100000.00' in sent
    assert '"amount":11000.00' in sent
    # The date beside it is a string and stays one.
    assert '"date":"2025-12-15"' in sent
