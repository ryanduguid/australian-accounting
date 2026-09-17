"""Evidence: repeatable where it must be, honest about the rest."""

from __future__ import annotations

import json
from decimal import Decimal

from lodgeitadapter import LodgeitClient
from lodgeitadapter import evidence as evidence_module

CALC = "urn:sbrm:calculator:div7a:at"
PERIOD = "urn:sbrm:period:div7a:fy2026"
GOOD = {
    "statutory_myr": "21874.92", "shortfall": "0.00", "total_repayments": "22000.00",
    "benchmark_rate": "0.0837",
    "manifest": {"calculator": CALC, "period": PERIOD,
                 "rate_table_uris": [{"uri": "urn:sbrm:rate:div7a:fy2026:benchmark-interest",
                                      "sha256": "a" * 64}],
                 "engine": "div7a/1.0.0", "citation": "ITAA 1936 s 109E"},
    "advisory": {"figure_type": "div7a_myr", "notes": ["Not advice."]},
}
BODY = {"amalgamated_base": Decimal("100000.00"), "loan_term_years": 7}


def capture(stub, stub_config, contract, payload=None, label="trial"):
    _, state = stub
    state.respond(payload or GOOD)
    outcome = LodgeitClient(stub_config, contract).invoke(CALC, PERIOD, BODY)
    return evidence_module.build(outcome, label=label, synthetic=True), state


def test_an_evidence_record_verifies_against_its_own_digest(stub, stub_config, contract):
    record, _ = capture(stub, stub_config, contract)
    assert evidence_module.verify(record) == []
    assert record["calculation_sha256"] == evidence_module.digest_of(record)


def test_the_digest_covers_the_calculation_and_not_the_observation(stub, stub_config, contract):
    """Two identical calls produce the same digest and different observations."""
    first, _ = capture(stub, stub_config, contract)
    second, _ = capture(stub, stub_config, contract)
    assert first["calculation_sha256"] == second["calculation_sha256"]
    second["observation"]["elapsed_ms"] = 99999
    second["observation"]["captured_at"] = "2099-01-01T00:00:00+00:00"
    assert evidence_module.verify(second) == [], "observation metadata is outside the digest"


def test_changing_a_figure_breaks_the_digest(stub, stub_config, contract):
    record, _ = capture(stub, stub_config, contract)
    record["calculation"]["upstream"]["response"]["statutory_myr"] = "1.00"
    findings = evidence_module.verify(record)
    assert any("does not match" in finding for finding in findings)


def test_the_upstream_response_is_preserved_and_normalisation_sits_beside_it(
    stub,
    stub_config,
    contract):
    record, _ = capture(stub, stub_config, contract)
    calculation = record["calculation"]
    assert calculation["upstream"]["response"]["statutory_myr"] == "21874.92"
    assert calculation["upstream"]["manifest"]["citation"] == "ITAA 1936 s 109E"
    assert calculation["upstream"]["advisory"]["notes"] == ["Not advice."]
    assert calculation["normalised"]["values"]["statutory_myr"] == "21874.92"
    assert "unaltered" in calculation["normalised"]["note"]


def test_the_input_is_recorded_with_its_exact_wire_form(stub, stub_config, contract):
    record, _ = capture(stub, stub_config, contract)
    assert "100000.00" in record["calculation"]["input_json"]
    assert record["calculation"]["synthetic_input"] is True


def test_a_refusal_is_recorded_as_evidence_too(stub, stub_config, contract):
    record, _ = capture(stub, stub_config, contract,
                        payload={"refusal_class": "insufficient_facts"})
    # The stub answers 200 by default, so force the refusal path explicitly.
    _, state = stub
    state.respond({"refusal_class": "insufficient_facts"}, status=400)
    outcome = LodgeitClient(stub_config, contract).invoke(CALC, PERIOD, BODY)
    record = evidence_module.build(outcome, label="refusal", synthetic=True)
    assert evidence_module.verify(record) == []
    assert record["calculation"]["call"]["status"] == "UPSTREAM_REFUSED"
    assert record["calculation"]["upstream"]["refusal_class"] == "insufficient_facts"
    assert record["calculation"]["normalised"]["values"] == {}
    assert record["calculation"]["validation"]["accepted"] is False


def test_a_computed_record_without_a_manifest_is_reported(stub, stub_config, contract):
    record, _ = capture(stub, stub_config, contract)
    record["calculation"]["upstream"]["manifest"] = None
    record["calculation_sha256"] = evidence_module.digest_of(record)
    findings = evidence_module.verify(record)
    assert any("no upstream manifest" in finding for finding in findings)


def test_an_unknown_schema_is_refused(stub, stub_config, contract):
    record, _ = capture(stub, stub_config, contract)
    record["schema"] = "something-else/9"
    findings = evidence_module.verify(record)
    assert any("unknown evidence schema" in finding for finding in findings)


def test_a_record_round_trips_through_a_file(tmp_path, stub, stub_config, contract):
    record, _ = capture(stub, stub_config, contract)
    path = evidence_module.write(record, tmp_path / "evidence.json")
    reloaded = evidence_module.read(path)
    assert reloaded == record
    assert evidence_module.verify(reloaded) == []
    assert path.read_text(encoding="utf-8").endswith("\n")


def test_the_record_carries_the_contract_snapshot_it_was_checked_against(
    stub,
    stub_config,
    contract):
    record, _ = capture(stub, stub_config, contract)
    provider = record["calculation"]["provider"]
    assert provider["contract_snapshot"] == "lodgeit-calculators-2026-09-18"
    assert provider["contract_sha256"] == contract.sha256


def test_no_boundary_claim_is_overstated(stub, stub_config, contract):
    record, _ = capture(stub, stub_config, contract)
    boundary = " ".join(record["calculation"]["boundary"])
    assert "Not advice" in boundary
    assert "proves nothing about whether the figure is right" in boundary
    serialised = json.dumps(record).lower()
    for claim in ("approved", "certified", "attested", "conformant"):
        assert claim not in serialised
