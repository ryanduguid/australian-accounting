"""FBT trial: one category value in, and the two ways it could go wrong."""

from __future__ import annotations

from decimal import Decimal

import pytest

from lodgeitadapter import LodgeitClient
from lodgeitadapter.errors import ContractError
from lodgeitadapter.trials import Evaluation
from lodgeitadapter.trials import fbt as trial

pytest.importorskip("austaxcalc", reason="the trials extra installs the engine")

BENEFIT = trial.CarBenefit(
    case_id="FBT-1-type-2-car",
    base_value=Decimal("40000.00"),
    days_available=365,
    accessories=Decimal("0.00"),
    employee_contribution=Decimal("0.00"),
    fbt_type="Type 2",
)


def provider_body(taxable_value="8000.00", **overrides):
    payload = {
        "taxable_value": taxable_value,
        "gross_up_factor": "1.8868",
        "grossed_up_taxable_value": "15094.40",
        "fbt_payable": "7094.37",
        "manifest": {"calculator": trial.CALC_URI, "period": trial.PERIOD_URI,
                     "rate_table_uris": [], "engine": "fbt/1.0.0", "citation": "FBTAA s 9"},
        "advisory": {"figure_type": "fbt_taxable_value", "notes": ["Not advice."]},
    }
    payload.update(overrides)
    return payload


def test_only_the_category_taxable_value_crosses_into_the_worksheet():
    arguments = trial.prepare_aggregate_input(Decimal("8000.00"), "Type 2")
    assert arguments == {"type_one_value": Decimal("0.00"), "type_two_value": Decimal("8000.00")}
    arguments = trial.prepare_aggregate_input(Decimal("8000.00"), "Type 1")
    assert arguments == {"type_one_value": Decimal("8000.00"), "type_two_value": Decimal("0.00")}


@pytest.mark.parametrize("field", ["grossed_up_taxable_value", "fbt_payable"])
def test_an_already_grossed_up_figure_is_refused_at_the_door(field):
    with pytest.raises(ContractError, match="already grossed up"):
        trial.prepare_aggregate_input(Decimal("15094.40"), "Type 2", field_name=field)


def test_double_gross_up_is_what_the_refusal_prevents():
    """The arithmetic the guard exists to stop, shown once so it is not abstract."""
    category = Decimal("8000.00")
    correct = trial.run_local_aggregate(
        trial.prepare_aggregate_input(category, "Type 2"), 2026,
    )["type_two_grossed_up"]
    assert correct == Decimal("15094.40")
    doubled = trial.run_local_aggregate(
        {"type_one_value": Decimal("0.00"), "type_two_value": correct}, 2026,
    )["type_two_grossed_up"]
    assert doubled == Decimal("28480.11")
    assert doubled > correct * Decimal("1.8")


def test_an_unknown_classification_refuses_before_the_call():
    unknown = trial.CarBenefit(**{**BENEFIT.__dict__, "fbt_type": "unknown"})
    with pytest.raises(trial.ClassificationRequired):
        unknown.request()
    with pytest.raises(trial.ClassificationRequired):
        trial.prepare_aggregate_input(Decimal("8000.00"), "unknown")
    comparison = trial.evaluate(unknown, client=None)
    assert comparison.evaluation is Evaluation.SCOPE_MISMATCH


def test_a_period_with_no_local_counterpart_is_unsupported():
    other = trial.CarBenefit(**{**BENEFIT.__dict__, "period_uri": "urn:sbrm:period:fbt:fy2025"})
    comparison = trial.evaluate(other, client=None)
    assert comparison.evaluation is Evaluation.UNSUPPORTED_PERIOD


def test_the_join_carries_the_category_value_and_agrees_on_the_gross_up(
    stub,
    stub_config,
    contract):
    _, state = stub
    state.respond(provider_body())
    comparison = trial.evaluate(BENEFIT, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.MATCH
    assert comparison.upstream["taxable_value"] == Decimal("8000.00")
    assert comparison.local["type_two_grossed_up"] == Decimal("15094.40")
    assert any("only taxable_value crossed" in reason for reason in comparison.reasons)


def test_a_gross_up_rate_difference_shows_up_as_a_difference(stub, stub_config, contract):
    _, state = stub
    state.respond(provider_body(grossed_up_taxable_value="16641.60"))  # the Type 1 factor
    comparison = trial.evaluate(BENEFIT, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.NUMERIC_DIFFERENCE
    assert any("gross-up rate" in reason for reason in comparison.reasons)


def test_a_response_without_a_taxable_value_produces_nothing(stub, stub_config, contract):
    _, state = stub
    body = provider_body()
    del body["taxable_value"]
    state.respond(body)
    comparison = trial.evaluate(BENEFIT, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.CONTRACT_FAILURE
    assert comparison.local == {}


def test_a_provider_refusal_does_not_reach_the_worksheet(stub, stub_config, contract):
    _, state = stub
    state.respond({"refusal_class": "out_of_scope_benefit"}, status=400)
    comparison = trial.evaluate(BENEFIT, LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.SCOPE_MISMATCH
    assert comparison.local == {}
