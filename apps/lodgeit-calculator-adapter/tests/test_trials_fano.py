"""Fano trial: suggestions stay suggestions."""

from __future__ import annotations

import json
from decimal import Decimal

import pytest

from lodgeitadapter import AdapterConfig, LodgeitClient, Status
from lodgeitadapter.errors import ContractError
from lodgeitadapter.trials import fano as trial

LINES = [
    trial.Line(
        "Bank account, operating",
        "current_assets",
        Decimal("15000.00"),
        "sbrm_0000",
        Decimal("0")),
    trial.Line("Trade debtors", "current_assets", Decimal("42000.00"), "sbrm_0000", Decimal("0")),
    trial.Line(
        "Sales, maintenance services",
        "revenue",
        Decimal("-57000.00"),
        "sbrm_0000",
        Decimal("0")),
]


@pytest.fixture
def fano_client(stub, fano_contract):
    base_url, state = stub
    config = AdapterConfig(enabled=True, base_url=base_url, allow_loopback=True, max_attempts=1)
    return LodgeitClient(config, fano_contract), state


def response_for(lines, **overrides):
    payload = {
        "status": "COMPLETE",
        "equilibrium_valid": True,
        "results": [
            {
                "description": line.description,
                "predicted_code": "sbrm_1234",
                "confidence": "0.91",
                "cascade_topology": line.source_topology,
                "model_architecture": "iter11.B_R3",
                "operator_hint_predicted_code": line.predicted_code,
                "operator_hint_source_topology": line.source_topology,
                "operator_hint_confidence": format(line.confidence, "f"),
                "fano_status": "accepted_fact",
                "quarantine_reason": "keyword_rule:sbrm_1234",
            }
            for line in lines
        ],
    }
    payload.update(overrides)
    return payload


def test_a_trial_balance_that_does_not_balance_is_refused_locally():
    unbalanced = LINES[:2]
    with pytest.raises(ContractError, match="net to"):
        trial.build_payload("company", unbalanced)


def test_the_provider_limits_are_checked_before_sending():
    with pytest.raises(ContractError, match="500"):
        trial.build_payload("company", [LINES[0]] * 501)
    with pytest.raises(ContractError, match="entity_structure"):
        trial.build_payload("llc", LINES)
    with pytest.raises(ContractError, match="no lines"):
        trial.build_payload("company", [])


def test_every_line_keeps_its_original_alongside_the_suggestion(fano_client):
    client, state = fano_client
    state.respond(response_for(LINES))
    status, suggestions, notes = trial.classify("company", LINES, client)
    assert status == "COMPLETE"
    assert len(suggestions) == 3
    first = suggestions[0].to_json_dict()
    assert first["original"]["description"] == "Bank account, operating"
    assert first["original"]["topology"] == "current_assets"
    assert first["original"]["amount"] == "15000.00"
    assert first["suggestion"]["proposed_code"] == "sbrm_1234"
    assert first["suggestion"]["classifier"] == "iter11.B_R3"
    assert first["review"]["applied"] is False
    assert first["review"]["approved_by"] is None


def test_accepted_fact_is_not_an_approval(fano_client):
    client, state = fano_client
    state.respond(response_for(LINES))
    _, suggestions, _ = trial.classify("company", LINES, client)
    for suggestion in suggestions:
        assert suggestion.fano_status == "accepted_fact"
        assert suggestion.needs_review is True
        payload = suggestion.to_json_dict()
        assert payload["review"]["approved_by"] is None
        assert "not an approval" in payload["review"]["boundary"]


def test_a_draft_fact_is_a_correct_outcome_and_is_carried_through(fano_client):
    client, state = fano_client
    body = response_for(LINES)
    body["results"][1]["fano_status"] = "draft_fact"
    body["results"][1]["quarantine_reason"] = (
        "rule_model_disagreement (rule=sbrm_1200, model=sbrm_1234)"
    )
    state.respond(body)
    _, suggestions, _ = trial.classify("company", LINES, client)
    assert suggestions[1].fano_status == "draft_fact"
    assert "rule_model_disagreement" in suggestions[1].reason


def test_a_topology_disagreement_is_flagged(fano_client):
    client, state = fano_client
    body = response_for(LINES)
    body["results"][0]["cascade_topology"] = "non_current_assets"
    state.respond(body)
    _, suggestions, _ = trial.classify("company", LINES, client)
    assert suggestions[0].topology_disagrees is True
    assert suggestions[0].to_json_dict()["review"]["topology_disagreement"] is True


def test_an_unknown_status_leaves_the_line_unreviewed(fano_client):
    client, state = fano_client
    body = response_for(LINES)
    body["results"][2]["fano_status"] = "brand_new_state"
    state.respond(body)
    _, suggestions, _ = trial.classify("company", LINES, client)
    assert any("unknown fano_status" in finding for finding in suggestions[2].findings)


def test_a_row_count_mismatch_proposes_nothing(fano_client):
    client, state = fano_client
    body = response_for(LINES)
    body["results"] = body["results"][:2]
    state.respond(body)
    status, suggestions, notes = trial.classify("company", LINES, client)
    assert status == "CONTRACT_FAILURE"
    assert suggestions == []


def test_a_reported_imbalance_proposes_nothing(fano_client):
    client, state = fano_client
    state.respond(response_for(LINES, equilibrium_valid=False))
    status, suggestions, _ = trial.classify("company", LINES, client)
    assert status == "CONTRACT_FAILURE"
    assert suggestions == []


def test_an_unavailable_service_proposes_nothing(fano_client):
    client, state = fano_client
    state.respond({"detail": "down"}, status=503)
    status, suggestions, notes = trial.classify("company", LINES, client)
    assert status == str(Status.UPSTREAM_UNAVAILABLE)
    assert suggestions == []


def test_an_authentication_refusal_is_reported_not_worked_around(fano_client):
    client, state = fano_client
    state.respond({"detail": "Not authenticated"}, status=403)
    status, suggestions, notes = trial.classify("company", LINES, client)
    assert status == str(Status.CONTRACT_FAILURE)
    assert suggestions == []


def test_a_code_needs_a_reviewed_crosswalk_before_it_means_anything():
    with pytest.raises(ContractError, match="no reviewed crosswalk"):
        trial.require_crosswalk(None, "sbrm_1234")
    with pytest.raises(ContractError, match="no entry in the reviewed crosswalk"):
        trial.require_crosswalk({"sbrm_9999": "1-1100"}, "sbrm_1234")
    assert trial.require_crosswalk({"sbrm_1234": "1-1100"}, "sbrm_1234") == "1-1100"


def test_the_suggestion_record_never_carries_an_applied_mapping(fano_client):
    client, state = fano_client
    state.respond(response_for(LINES))
    _, suggestions, notes = trial.classify("company", LINES, client)
    serialised = json.dumps([item.to_json_dict() for item in suggestions])
    assert '"applied": false' in serialised
    assert '"applied": true' not in serialised
    assert any("awaiting review" in note for note in notes)
