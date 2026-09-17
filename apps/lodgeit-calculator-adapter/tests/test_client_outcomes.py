"""Every failure keeps its own name, and none of them becomes a figure."""

from __future__ import annotations

import json
from decimal import Decimal

import pytest

from lodgeitadapter import AdapterConfig, LodgeitClient, Status
from lodgeitadapter.errors import DisallowedTargetError, TransportError
from lodgeitadapter.transport import request as transport_request

CALC = "urn:sbrm:calculator:div7a:at"
PERIOD = "urn:sbrm:period:div7a:fy2026"
GOOD = {
    "statutory_myr": "21874.92", "shortfall": "0.00", "total_repayments": "22000.00",
    "benchmark_rate": "0.0837",
    "manifest": {
        "calculator": CALC,
        "period": PERIOD,
        "rate_table_uris": [],
        "engine": "x",
        "citation": "s 109E"},
    "advisory": {"figure_type": "div7a", "notes": ["not advice"]},
}


def call(client, body=None):
    return client.invoke(CALC, PERIOD, body or {"amalgamated_base": Decimal("100000.00")})


def test_a_complete_200_is_the_only_computed_outcome(stub, stub_config, contract):
    _, state = stub
    state.respond(GOOD)
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.COMPUTED
    assert outcome.values["statutory_myr"] == Decimal("21874.92")
    assert outcome.manifest and outcome.advisory


@pytest.mark.parametrize(
    "status,expected",
    [
        (400, Status.UPSTREAM_REFUSED),
        (422, Status.UPSTREAM_REJECTED),
        (404, Status.UPSTREAM_NOT_FOUND),
        (429, Status.UPSTREAM_THROTTLED),
        (500, Status.UPSTREAM_UNAVAILABLE),
        (503, Status.UPSTREAM_UNAVAILABLE),
        (418, Status.CONTRACT_FAILURE),
    ],
)
def test_each_provider_status_keeps_its_own_outcome(stub, stub_config, contract, status, expected):
    _, state = stub
    state.respond({"refusal_class": "out_of_scope", "detail": "because"}, status=status)
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is expected
    assert outcome.values == {}, "a non-200 must carry no figure"
    assert outcome.http_status == status


def test_a_provider_refusal_keeps_the_providers_own_class_and_body(stub, stub_config, contract):
    _, state = stub
    state.respond({"refusal_class": "pool_asset_out_of_t6_scope", "reason": "pooled"}, status=400)
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.upstream_refusal_class == "pool_asset_out_of_t6_scope"
    assert outcome.raw_response["reason"] == "pooled"


def test_a_200_without_an_advisory_is_a_contract_failure_not_a_result(stub, stub_config, contract):
    _, state = stub
    payload = {key: value for key, value in GOOD.items() if key != "advisory"}
    state.respond(payload)
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.CONTRACT_FAILURE
    assert outcome.values == {}
    assert any("advisory" in finding for finding in outcome.findings)


def test_a_200_without_a_manifest_is_a_contract_failure(stub, stub_config, contract):
    _, state = stub
    state.respond({key: value for key, value in GOOD.items() if key != "manifest"})
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.CONTRACT_FAILURE
    assert any("manifest" in finding for finding in outcome.findings)


def test_a_200_missing_an_expected_decimal_field_is_a_contract_failure(stub, stub_config, contract):
    _, state = stub
    state.respond({key: value for key, value in GOOD.items() if key != "statutory_myr"})
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.CONTRACT_FAILURE
    assert any("statutory_myr" in finding for finding in outcome.findings)


def test_an_unparseable_body_is_a_contract_failure(stub, stub_config, contract):
    _, state = stub
    state.respond(None, raw=b"<html>not json</html>")
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.CONTRACT_FAILURE
    assert outcome.values == {}


def test_a_period_the_snapshot_does_not_record_never_reaches_the_network(
    stub,
    stub_config,
    contract):
    _, state = stub
    state.respond(GOOD)
    client = LodgeitClient(stub_config, contract)
    outcome = client.invoke(CALC, "urn:sbrm:period:div7a:fy2024", {})
    assert outcome.status is Status.CONTRACT_FAILURE
    assert state.requests == [], "nothing was sent"


def test_a_calculator_the_snapshot_does_not_record_is_refused_before_sending(
    stub,
    stub_config,
    contract):
    _, state = stub
    client = LodgeitClient(stub_config, contract)
    outcome = client.invoke("urn:sbrm:calculator:fbt:board", "urn:sbrm:period:fbt:fy2026", {})
    assert outcome.status is Status.CONTRACT_FAILURE
    assert any("reviewed snapshot" in finding for finding in outcome.findings)
    assert state.requests == [], "an unreviewed calculator is never called"


def test_an_optional_field_may_be_absent_but_a_required_one_may_not(stub, stub_config, contract):
    _, state = stub
    state.respond(GOOD)  # carries no deemed_dividend or interest_accrued
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.COMPUTED
    assert "deemed_dividend" not in outcome.values

    state.respond({**GOOD, "deemed_dividend": "123.45"})
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.values["deemed_dividend"] == Decimal("123.45")

    state.respond({**GOOD, "shortfall": "not a number"})
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.CONTRACT_FAILURE


def test_a_redirect_is_refused_rather_than_followed(stub, stub_config, contract):
    _, state = stub
    state.status = 302
    state.headers = {
        "Location": "https://evil.example/v1/calculators",
        "Content-Type": "application/json",
    }
    state.body = b"{}"
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.REFUSED_TO_SEND
    assert any("redirect" in finding for finding in outcome.findings)


def test_an_oversized_response_is_abandoned(stub, contract):
    base_url, state = stub
    state.respond(None, raw=b"x" * 5000)
    config = AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True,
                           max_response_bytes=1000, max_attempts=1)
    outcome = call(LodgeitClient(config, contract))
    assert outcome.status is Status.UPSTREAM_UNAVAILABLE
    assert any("ceiling" in finding for finding in outcome.findings)


def test_a_timeout_is_never_a_number(stub, contract):
    base_url, state = stub
    state.respond(GOOD)
    state.delay = 0.6
    config = AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True,
                           read_timeout=0.05, max_attempts=1)
    outcome = call(LodgeitClient(config, contract))
    assert outcome.status is Status.UPSTREAM_UNAVAILABLE
    assert outcome.result is None and outcome.values == {}


def test_a_5xx_is_retried_and_a_4xx_is_not(stub, contract):
    base_url, state = stub
    state.respond({"detail": "server"}, status=503)
    config = AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True,
                           max_attempts=3, retry_backoff_seconds=0.01)
    call(LodgeitClient(config, contract))
    assert len(state.requests) == 3, "a 5xx is retried to the configured limit"

    state.requests.clear()
    state.respond({"refusal_class": "x"}, status=400)
    call(LodgeitClient(config, contract))
    assert len(state.requests) == 1, "a 4xx is never retried"


def test_a_get_is_retry_safe_and_the_body_is_never_logged(stub, contract, capsys):
    base_url, state = stub
    state.respond([{"calc_uri": CALC, "supported_periods": [PERIOD]}])
    config = AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1)
    outcome = LodgeitClient(config, contract).discover()
    assert outcome.computed
    captured = capsys.readouterr()
    assert "calc_uri" not in captured.out and "calc_uri" not in captured.err


def test_the_transport_refuses_a_target_off_the_allowlist_directly(stub_config):
    with pytest.raises(DisallowedTargetError):
        transport_request(stub_config, "GET", "https://evil.example/v1/calculators")


def test_a_closed_port_is_a_transport_error_not_a_result(contract):
    # Port 1 on loopback with nothing listening: a connection failure, which is
    # a TransportError and not an answer.
    config = AdapterConfig(enabled=True, base_url="http://127.0.0.1:1",
                           allow_loopback=True, max_attempts=1, read_timeout=1.0)
    with pytest.raises(TransportError):
        transport_request(config, "GET", "http://127.0.0.1:1/v1/calculators")
    outcome = LodgeitClient(config, contract).discover()
    assert outcome.status is Status.UPSTREAM_UNAVAILABLE


def test_drift_reports_differences_and_updates_nothing(stub, stub_config, contract):
    _, state = stub
    state.respond([
        {"calc_uri": CALC, "supported_periods": [PERIOD, "urn:sbrm:period:div7a:fy2027"]},
        {
            "calc_uri": "urn:sbrm:calculator:fbt:board",
            "supported_periods": ["urn:sbrm:period:fbt:fy2026"]},
    ])
    before = contract.path.read_bytes()
    outcome, findings = LodgeitClient(stub_config, contract).drift()
    assert outcome.computed
    assert any("fy2027" in finding for finding in findings)
    assert any("fbt:board" in finding for finding in findings)
    assert contract.path.read_bytes() == before, "a drift report never rewrites the snapshot"


def test_an_unrecorded_response_field_is_noted_but_not_fatal(stub, stub_config, contract):
    _, state = stub
    state.respond({**GOOD, "brand_new_field": "surprise"})
    outcome = call(LodgeitClient(stub_config, contract))
    assert outcome.status is Status.COMPUTED
    assert any("brand_new_field" in finding for finding in outcome.findings)
    assert json.dumps(outcome.raw_response)  # the provider's body is preserved whole
