"""Precision on the wire, checked as bytes.

The assertions here read the request body the stub received, not a value that
has been through `json.loads`. A round trip through Python floats would hide
exactly the defect these tests exist to catch.
"""

from __future__ import annotations

import json
from decimal import Decimal

import pytest

from lodgeitadapter import LodgeitClient
from lodgeitadapter.decimals import DecimalWireError, dumps, format_number, parse_decimal


def test_a_decimal_is_emitted_as_its_own_digits():
    assert dumps({"amalgamated_base": Decimal("55000.10")}) == '{"amalgamated_base":55000.10}'
    assert dumps({"rate": Decimal("0.0837")}) == '{"rate":0.0837}'
    assert dumps({"x": Decimal("0.1")}) == '{"x":0.1}'
    # The number json.dumps would have produced, for contrast.
    assert json.dumps(float(Decimal("55000.10"))) == "55000.1"


def test_a_float_never_reaches_the_wire():
    with pytest.raises(DecimalWireError):
        dumps({"amalgamated_base": 55000.10})
    with pytest.raises(DecimalWireError):
        dumps([1.5])


def test_trailing_zeros_and_scale_survive():
    assert format_number(Decimal("100.00")) == "100.00"
    assert format_number(Decimal("1E+3")) == "1000"
    assert format_number(Decimal("-0")) == "0"


def test_unrepresentable_values_are_refused():
    for value in (Decimal("NaN"), Decimal("Infinity"), Decimal("1E+20"), Decimal("0.00000000001")):
        with pytest.raises(DecimalWireError):
            dumps({"x": value})


def test_reading_refuses_a_json_float_and_accepts_a_decimal_string():
    assert parse_decimal("21874.92", "myr") == Decimal("21874.92")
    assert parse_decimal("0.0837", "rate") == Decimal("0.0837")
    for bad in (21874.92, True, None, "", "not a number"):
        with pytest.raises(DecimalWireError):
            parse_decimal(bad, "myr")


def test_the_bytes_the_stub_receives_carry_the_exact_digits(stub, stub_config, contract):
    base_url, state = stub
    state.respond({
        "statutory_myr": "21874.92", "manifest": {"calculator": "x"},
        "advisory": {"figure_type": "y", "notes": ["z"]},
    })
    client = LodgeitClient(stub_config, contract)
    client.invoke(
        "urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026",
        {"amalgamated_base": Decimal("55000.10"), "loan_term_years": 7},
    )
    sent = state.requests[-1]["body"].decode("utf-8")
    assert "55000.10" in sent
    assert "55000.1," not in sent and "55000.099" not in sent
    assert state.requests[-1]["headers"]["Content-Type"] == "application/json"


def test_a_provider_json_number_keeps_its_own_digits_on_the_way_in(stub, stub_config, contract):
    """The provider declares money as JSON numbers; the digits must survive.

    `json.loads(parse_float=str)` hands the text through, so a rate the
    provider wrote as 0.0837 is read as exactly that and not as a float.
    """
    base_url, state = stub
    state.respond(None, raw=b'{"benchmark_rate": 0.0837, "statutory_myr": 21874.92, '
                           b'"total_repayments": 22000.00, "shortfall": 0.00, '
                           b'"manifest": {"calculator": "x"}, '
                           b'"advisory": {"figure_type": "y", "notes": ["z"]}}')
    client = LodgeitClient(stub_config, contract)
    outcome = client.invoke("urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026", {})
    assert outcome.computed
    assert outcome.values["benchmark_rate"] == Decimal("0.0837")
    assert outcome.values["statutory_myr"] == Decimal("21874.92")
    assert str(outcome.values["benchmark_rate"]) == "0.0837"


def test_a_nan_in_a_response_is_a_contract_failure(stub, stub_config, contract):
    base_url, state = stub
    state.respond(None, raw=b'{"statutory_myr": NaN, "manifest": {}, "advisory": {"notes": ["z"]}}')
    client = LodgeitClient(stub_config, contract)
    outcome = client.invoke("urn:sbrm:calculator:div7a:at", "urn:sbrm:period:div7a:fy2026", {})
    assert not outcome.computed
    assert any("NaN" in finding for finding in outcome.findings)
