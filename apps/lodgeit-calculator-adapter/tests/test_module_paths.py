"""Refusals and edge paths in the configuration, contract, Core, wire and evidence modules.

Everything here is offline. The client tests use the loopback stub.
"""

from __future__ import annotations

from dataclasses import replace
from decimal import Decimal

import pytest

from lodgeitadapter import LodgeitClient, Status, evidence
from lodgeitadapter import contract as contract_module
from lodgeitadapter.client import Outcome
from lodgeitadapter.config import KNOWN_HOSTS, AdapterConfig, from_environment
from lodgeitadapter.core import CoreClient, CoreConfig, _parse_page
from lodgeitadapter.decimals import DecimalWireError, dumps, parse_decimal
from lodgeitadapter.errors import (
    ConfigurationError,
    ContractError,
    DisallowedTargetError,
    NotEnabledError,
    UpstreamError,
)

CALC = "urn:sbrm:calculator:div7a:at"
PERIOD = "urn:sbrm:period:div7a:fy2026"
HOST = KNOWN_HOSTS[0]
GOOD = {
    "statutory_myr": "21874.92", "shortfall": "0.00", "total_repayments": "22000.00",
    "benchmark_rate": "0.0837",
    "manifest": {"calculator": CALC, "period": PERIOD,
                 "rate_table_uris": [], "engine": "div7a/1.0.0", "citation": "ITAA 1936 s 109E"},
    "advisory": {"figure_type": "div7a_myr", "notes": ["Not advice."]},
}


# -- configuration ----------------------------------------------------------

@pytest.mark.parametrize(("options", "message"), [
    ({"read_timeout": 0}, "^read_timeout must be a positive number$"),
    ({"retry_backoff_seconds": "1"}, "^retry_backoff_seconds must be a positive number$"),
    ({"max_response_bytes": 0}, "^max_response_bytes must be positive$"),
    ({"max_attempts": 0}, "^max_attempts must be between 1 and 5$"),
    ({"max_attempts": 6}, "^max_attempts must be between 1 and 5$"),
])
def test_a_limit_outside_its_range_is_refused(options, message):
    with pytest.raises(ConfigurationError, match=message):
        AdapterConfig(**options)


def test_a_loopback_url_with_another_scheme_is_refused():
    with pytest.raises(DisallowedTargetError, match="scheme 'ftp' is refused"):
        AdapterConfig(allow_loopback=True).check_url("ftp://127.0.0.1/v1/calculators")


def test_a_path_that_is_not_in_its_decoded_form_is_refused():
    with pytest.raises(DisallowedTargetError, match="the path is not in its decoded form"):
        AdapterConfig().check_url(f"https://{HOST}/v1/calculators/%61")


def test_the_environment_can_name_the_allowed_hosts_and_the_limits():
    config = from_environment({
        "LODGEIT_ADAPTER_ALLOWED_HOSTS": f" {HOST} , calc.example.test ,",
        "LODGEIT_ADAPTER_READ_TIMEOUT": "5",
        "LODGEIT_ADAPTER_MAX_ATTEMPTS": "3",
    })
    assert config.allowed_hosts == (HOST, "calc.example.test")  # nosec B101: pytest outcome assertion
    assert config.read_timeout == 5.0  # nosec B101: pytest outcome assertion
    assert config.max_attempts == 3  # nosec B101: pytest outcome assertion


def test_an_environment_limit_that_is_not_a_number_is_refused():
    message = "^LODGEIT_ADAPTER_READ_TIMEOUT='soon' is not a number$"
    with pytest.raises(ConfigurationError, match=message):
        from_environment({"LODGEIT_ADAPTER_READ_TIMEOUT": "soon"})


# -- the contract snapshot --------------------------------------------------

def test_the_snapshot_folder_falls_back_to_the_first_candidate(tmp_path, monkeypatch):
    first, second = tmp_path / "source", tmp_path / "wheel"
    monkeypatch.setattr(contract_module, "CONTRACTS_DIRS", (first, second))
    assert contract_module.contracts_dir() == first  # nosec B101: pytest outcome assertion
    second.mkdir()
    assert contract_module.contracts_dir() == second  # nosec B101: pytest outcome assertion


def test_a_snapshot_that_is_not_there_is_refused(tmp_path):
    with pytest.raises(ContractError, match="could not be read"):
        contract_module.load("absent", directory=tmp_path)


@pytest.mark.parametrize("payload", [b"{not json", b"\xff\xfe"])
def test_a_snapshot_that_is_not_json_is_refused(tmp_path, payload):
    (tmp_path / "broken.json").write_bytes(payload)
    with pytest.raises(ContractError, match="is not valid JSON"):
        contract_module.load("broken", directory=tmp_path)


def test_a_snapshot_missing_a_section_names_it(tmp_path):
    (tmp_path / "thin.json").write_text('{"provider": "x"}', encoding="utf-8")
    with pytest.raises(ContractError, match="is missing calculators, read_at, read_by"):
        contract_module.load("thin", directory=tmp_path)


def _live_from(contract) -> list[dict]:
    return [
        {"calc_uri": uri, "supported_periods": list(spec.get("supported_periods", [])),
         "input_schema_ref": spec.get("input_schema_ref")}
        for uri, spec in contract.calculators.items()
    ]


def test_a_calculator_with_no_recorded_schema_ref_is_not_compared_on_it(contract):
    calculators = {uri: dict(spec) for uri, spec in contract.calculators.items()}
    calculators[CALC].pop("input_schema_ref")
    unreferenced = replace(contract, calculators=calculators)
    live = _live_from(contract)
    next(entry for entry in live if entry["calc_uri"] == CALC)["input_schema_ref"] = "#/moved"
    assert contract_module.compare(unreferenced, live) == []  # nosec B101: pytest outcome assertion


def test_a_moved_schema_ref_is_drift(contract):
    live = _live_from(contract)
    entry = next(entry for entry in live if entry["calc_uri"] == CALC)
    recorded = entry["input_schema_ref"]
    entry["input_schema_ref"] = "#/components/schemas/Moved"
    assert contract_module.compare(contract, live) == [  # nosec B101: pytest outcome assertion
        f"{CALC}: input schema ref moved from {recorded} to #/components/schemas/Moved"
    ]


# -- the calculators client -------------------------------------------------

def test_drift_without_a_live_listing_compares_nothing(stub, stub_config, contract):
    _, state = stub
    state.respond({"detail": "down"}, status=503)
    outcome, findings = LodgeitClient(stub_config, contract).drift()
    assert outcome.status is Status.UPSTREAM_UNAVAILABLE  # nosec B101: pytest outcome assertion
    assert findings == []  # nosec B101: pytest outcome assertion


def test_a_calculator_with_no_recorded_route_is_never_called(stub, stub_config, contract):
    _, state = stub
    calculators = {uri: dict(spec) for uri, spec in contract.calculators.items()}
    calculators[CALC].pop("route")
    outcome = LodgeitClient(stub_config, replace(contract, calculators=calculators)).invoke(
        CALC, PERIOD, {"amalgamated_base": Decimal("1.00")},
    )
    assert outcome.status is Status.CONTRACT_FAILURE  # nosec B101: pytest outcome assertion
    assert outcome.findings == (f"snapshot records no route for {CALC}",)  # nosec B101: pytest outcome assertion
    assert state.requests == []  # nosec B101: pytest outcome assertion


def test_a_snapshot_that_refuses_unknown_fields_fails_a_response_carrying_one(
    stub, stub_config, contract,
):
    _, state = stub
    state.respond({**GOOD, "surprise": "1"})
    strict = replace(contract, response_contract={
        **contract.response_contract, "refuse_unknown_fields": True,
    })
    outcome = LodgeitClient(stub_config, strict).invoke(CALC, PERIOD, {"amalgamated_base": 1})
    assert outcome.status is Status.CONTRACT_FAILURE  # nosec B101: pytest outcome assertion
    assert "response carries fields the snapshot does not record: surprise" in outcome.findings  # nosec B101: pytest outcome assertion


# -- Core -------------------------------------------------------------------

def test_core_makes_one_attempt_with_no_extra_headers():
    config = CoreConfig()
    assert config.max_attempts == 1  # nosec B101: pytest outcome assertion
    assert config.retry_backoff_seconds == 1.0  # nosec B101: pytest outcome assertion
    assert dict(config.extra_headers) == {}  # nosec B101: pytest outcome assertion


def test_core_access_is_disabled_unless_switched_on():
    with pytest.raises(NotEnabledError, match="^Core access is disabled.$"):
        CoreConfig().require_enabled()


def test_the_core_client_takes_only_a_core_configuration():
    with pytest.raises(ConfigurationError, match="^CoreClient requires CoreConfig.$"):
        CoreClient(AdapterConfig())  # type: ignore[arg-type]


@pytest.mark.parametrize("body", [
    b"[]",
    b'{"items": [{"id": "6f1c1d2e-8f5a-4b8e-9c1d-2e3f4a5b6c7d", "code": 5, '
    b'"archived": false, "private": false, "profile": null}]}',
])
def test_a_core_page_of_the_wrong_shape_is_no_page(body):
    assert _parse_page(body, 5) is None  # nosec B101: pytest outcome assertion


# -- exact decimals on the wire ---------------------------------------------

def test_an_integer_is_read_exactly():
    assert parse_decimal(7, "loan_term_years") == Decimal(7)  # nosec B101: pytest outcome assertion


@pytest.mark.parametrize("value", ["NaN", "Infinity", "-Infinity"])
def test_a_non_finite_decimal_string_is_refused(value):
    with pytest.raises(DecimalWireError, match=f"rate: '{value}' is not finite"):
        parse_decimal(value, "rate")


def test_strings_are_escaped_and_null_is_written_as_null():
    text = 'a"b\\c\nd\re\tf\x01g'
    assert dumps({"note": text, "missing": None}) == (  # nosec B101: pytest outcome assertion
        '{"note":"a\\"b\\\\c\\nd\\re\\tf\\u0001g","missing":null}'
    )


def test_an_object_key_must_be_a_string():
    with pytest.raises(DecimalWireError, match="body: object keys must be strings, got 1"):
        dumps({1: "one"})


def test_a_value_with_no_exact_form_is_refused():
    with pytest.raises(DecimalWireError, match="body.when: set cannot be serialised exactly"):
        dumps({"when": {"2026"}})


def test_an_upstream_error_keeps_the_providers_own_status_and_body():
    error = UpstreamError("refused", status=400, payload={"refusal_class": "scope"},
                          refusal_class="scope")
    assert (str(error), error.status, error.payload, error.refusal_class) == (  # nosec B101: pytest outcome assertion
        "refused", 400, {"refusal_class": "scope"}, "scope",
    )


# -- evidence ---------------------------------------------------------------

def _computed():
    return Outcome(status=Status.COMPUTED, calculator=CALC, period=PERIOD,
                   manifest={"calculator": CALC},
                   advisory={"notes": ["Fabricated evidence test."]})


def _record(**changes):
    record = evidence.build(_computed(), label="fabricated", synthetic=True)
    record["calculation"].update(changes)
    record["calculation_sha256"] = evidence.digest_of(record)
    return record


def test_a_label_must_be_a_string():
    with pytest.raises(ValueError, match="^label must be a string, got int$"):
        evidence.build(_computed(), label=5, synthetic=True)  # type: ignore[arg-type]


def test_a_record_without_a_calculation_block_is_reported():
    record = _record()
    del record["calculation"]
    assert evidence.verify(record) == ["no calculation block"]  # nosec B101: pytest outcome assertion


def test_a_calculation_that_cannot_be_canonicalised_is_reported():
    record = _record()
    record["calculation"]["upstream"]["figure"] = float("nan")
    findings = evidence.verify(record)
    assert len(findings) == 1  # nosec B101: pytest outcome assertion
    assert findings[0].startswith("calculation block cannot be canonicalised: ")  # nosec B101: pytest outcome assertion


def test_a_calculation_naming_another_schema_is_reported():
    assert evidence.verify(_record(schema="other/1")) == [  # nosec B101: pytest outcome assertion
        "the calculation block names a different schema from the record"
    ]


def test_a_calculation_whose_label_is_not_a_slug_is_reported():
    [finding] = evidence.verify(_record(label="Not A Slug"))
    assert finding.startswith("label 'Not A Slug' is not a slug.")  # nosec B101: pytest outcome assertion


def test_a_computed_record_without_an_advisory_is_reported():
    record = _record()
    record["calculation"]["upstream"]["advisory"] = None
    record["calculation_sha256"] = evidence.digest_of(record)
    assert evidence.verify(record) == ["a computed result with no upstream advisory"]  # nosec B101: pytest outcome assertion


def test_a_block_that_is_not_an_object_is_reported():
    assert "call is str, not an object" in evidence.verify(_record(call="COMPUTED"))  # nosec B101: pytest outcome assertion
