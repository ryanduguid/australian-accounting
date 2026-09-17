"""Division 7A trial: the local side against independently derived arithmetic.

These run offline. The local engine runs for real; the provider is a stub
answering from a fixture, so the evaluation logic is exercised without a live
call. The expected figures come from the s 109E(6) formula written out in the
case file, not from either engine.
"""

from __future__ import annotations

import json
from decimal import Decimal

import pytest

from lodgeitadapter import LodgeitClient
from lodgeitadapter.trials import Evaluation
from lodgeitadapter.trials import div7a as trial

div7aloan = pytest.importorskip("div7aloan", reason="the trials extra installs the engine")

CASES = {case.case_id: case for case in trial.load_cases()}


def provider_body(case, **overrides):
    """A provider response shaped like the contract, from the case's own expectations."""
    payload = {
        "statutory_myr": str(case.expected.get("statutory_myr", "0.00")),
        "total_repayments": str(case.expected.get("total_repayments", "0.00")),
        "shortfall": str(case.expected.get("shortfall", "0.00")),
        "benchmark_rate": str(case.expected.get("benchmark_rate", "0.0837")),
        "remaining_term_years": case.remaining_term_years,
        "is_complying": True,
        "manifest": {"calculator": trial.CALC_URI, "period": case.period_uri,
                     "rate_table_uris": [{"uri": "urn:sbrm:rate:div7a:fy2026:benchmark-interest",
                                          "sha256": "0" * 64}],
                     "engine": "div7a/1.0.0", "citation": "ITAA 1936 s 109E"},
        "advisory": {"figure_type": "div7a_myr", "notes": ["Tax agent framing."]},
    }
    payload.update(overrides)
    return payload


def test_the_case_file_is_synthetic_and_carries_a_derivation_for_every_case():
    payload = json.loads(trial.cases_path().read_text(encoding="utf-8"))
    assert payload["synthetic"] is True
    for case in payload["cases"]:
        assert case["derivation"].strip(), case["case_id"]


def test_the_period_map_agrees_with_the_local_rate_table():
    """The provider's fy label and the local income year must select one rate.

    Checked against the live rate documents on 18 September 2026: the
    provider's fy2026 covers 2025-07-01 to 2026-06-30 at 0.0837 and fy2025
    covers 2024-07-01 to 2025-06-30 at 0.0877.
    """
    from div7aloan.rates import load_table
    from div7aloan.years import parse_year

    table = load_table()
    expected = {"urn:sbrm:period:div7a:fy2025": Decimal("0.0877"),
                "urn:sbrm:period:div7a:fy2026": Decimal("0.0837")}
    for period_uri, local_year in trial.PERIOD_MAP.items():
        result = table.lookup(parse_year(local_year, "test"))
        assert result.rate == expected[period_uri], f"{period_uri} -> {local_year}"


@pytest.mark.parametrize("case_id", [
    "D7A-1-met-first-real-year", "D7A-2-short-repayment", "D7A-3-excess-repayment",
    "D7A-5-earlier-year-rate", "D7A-6-term-of-one-year", "D7A-7-secured-25-year-term",
])
def test_the_local_engine_reproduces_the_independently_derived_figure(case_id):
    case = CASES[case_id]
    verdict, values, _ = trial.run_local(case)
    assert verdict == case.expect_local
    for field, expected in case.expected.items():
        assert values[field] == expected, f"{case_id}.{field}"


def test_a_nil_remaining_term_is_refused_by_the_local_engine_not_divided(
    stub, stub_config, contract,
):
    _, state = stub
    case = CASES["D7A-8-nil-remaining-term-refused"]
    verdict, values, reasons = trial.run_local(case)
    assert verdict == "REFUSED"
    assert "statutory_myr" not in values
    assert any("remaining term" in reason for reason in reasons)
    # A local refusal ends the case. Nothing is sent, so the fixture's recorded
    # evaluation is decidable offline and is asserted here.
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    assert str(comparison.evaluation) == case.expect_evaluation
    assert state.requests == []


def test_a_matching_provider_answer_is_a_match(stub, stub_config, contract):
    _, state = stub
    case = CASES["D7A-1-met-first-real-year"]
    state.respond(provider_body(case))
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.MATCH
    assert comparison.local["statutory_myr"] == Decimal("21874.92")
    assert comparison.upstream["statutory_myr"] == Decimal("21874.92")


def test_a_cent_of_disagreement_is_a_numeric_difference_not_a_match(stub, stub_config, contract):
    _, state = stub
    case = CASES["D7A-2-short-repayment"]
    state.respond(provider_body(case, statutory_myr="21874.90", shortfall="6874.90"))
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.NUMERIC_DIFFERENCE
    assert comparison.differences["statutory_myr"] == Decimal("0.02")


def test_a_term_convention_difference_is_scope_not_arithmetic(stub, stub_config, contract):
    _, state = stub
    case = CASES["D7A-1-met-first-real-year"]
    state.respond(provider_body(case, remaining_term_years=5))
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.SCOPE_MISMATCH
    assert any("remaining term" in reason for reason in comparison.reasons)


def test_the_rate_trap_case_reports_a_difference_and_names_the_rate(stub, stub_config, contract):
    """D7A-4 expects the previous year's rate; the engine must use the current one.

    The provider here echoes the term the case supplied, which isolates the
    rate. The live provider derives its own term, which is the case below.
    """
    _, state = stub
    case = CASES["D7A-4-later-year-not-first"]
    state.respond(provider_body(case))
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.NUMERIC_DIFFERENCE
    assert comparison.local["statutory_myr"] == Decimal("29297.01")
    assert comparison.expected["statutory_myr"] == Decimal("29507.73")
    assert any("29297.01" in reason for reason in comparison.reasons)


def test_the_rate_trap_case_reaches_the_outcome_its_fixture_records(
    stub, stub_config, contract,
):
    """The term the live provider derived, reproduced offline.

    `expect_evaluation` records what the trial reported against the provider
    on the probe date. On 18 September 2026 the provider derived a remaining
    term of 4 from the origination facts while the case supplied 3, so the
    term convention outranks the rate difference underneath it: two engines
    using different terms were never comparing the same arithmetic.
    """
    _, state = stub
    case = CASES["D7A-4-later-year-not-first"]
    state.respond(provider_body(case, remaining_term_years=4))
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    assert str(comparison.evaluation) == case.expect_evaluation
    assert comparison.evaluation is Evaluation.SCOPE_MISMATCH
    assert any("remaining term" in reason for reason in comparison.reasons)


def test_an_unsupported_period_is_its_own_outcome(stub, stub_config, contract):
    _, state = stub
    case = CASES["D7A-9-unsupported-period"]
    state.respond({"error": "not found"}, status=404)
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    # Neither side covers fy2024: the local engine has no income-year mapping
    # for it and the snapshot does not record it, so nothing is sent.
    assert comparison.evaluation is Evaluation.UNSUPPORTED_PERIOD
    assert str(comparison.evaluation) == case.expect_evaluation
    assert comparison.upstream == {} and comparison.local == {}
    assert state.requests == []


def test_a_provider_outage_never_becomes_agreement(stub, stub_config, contract):
    _, state = stub
    case = CASES["D7A-1-met-first-real-year"]
    state.respond({"detail": "down"}, status=503)
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.UPSTREAM_UNAVAILABLE
    assert comparison.upstream == {}
    assert comparison.local["statutory_myr"] == Decimal("21874.92"), "the local side still answered"


def test_the_providers_compliance_label_is_recorded_and_never_mapped(stub, stub_config, contract):
    _, state = stub
    case = CASES["D7A-1-met-first-real-year"]
    state.respond(provider_body(case, is_complying=False, deemed_dividend="12345.67"))
    comparison = trial.evaluate(case, LodgeitClient(stub_config, contract))
    assert any("is_complying" in reason for reason in comparison.reasons)
    assert comparison.local_verdict == "MYR_MET", "the local verdict is the local engine's own"
    serialised = json.dumps(comparison.to_json_dict())
    assert "COMPLYING" not in serialised.replace("MYR_MET", "")


def test_the_local_side_runs_without_a_client_and_says_so():
    comparison = trial.evaluate(CASES["D7A-1-met-first-real-year"], client=None)
    assert comparison.evaluation is Evaluation.NOT_RUN
    assert comparison.local["statutory_myr"] == Decimal("21874.92")
    assert comparison.upstream == {}


def test_an_evaluation_outcome_is_not_an_accounting_state():
    comparison = trial.evaluate(CASES["D7A-1-met-first-real-year"], client=None)
    payload = comparison.to_json_dict()
    assert payload["evaluation"] == "NOT_RUN"
    assert "approves nothing" in payload["boundary"]
    assert payload["evaluation"] not in ("PASS", "REVIEW", "BLOCKED", "READY", "NOT_READY")
