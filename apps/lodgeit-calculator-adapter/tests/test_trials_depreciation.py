"""Accounting depreciation trial: the movement closes, or it is refused."""

from __future__ import annotations

from decimal import Decimal

import pytest

from lodgeitadapter import LodgeitClient
from lodgeitadapter.trials import Evaluation
from lodgeitadapter.trials import depreciation as trial

ASSET = trial.Asset(
    case_id="DEP-1-plant",
    cost=Decimal("120000.00"),
    acquisition_date="2024-07-01",
    useful_life_years=5,
)


def provider_range(opening="120000.00", closing="96000.00", depreciation="24000.00", **extra):
    payload = {
        "opening_wdv": opening,
        "closing_wdv": closing,
        "range_dep": depreciation,
        "days_in_range": 365,
        "truncated": False,
        "day_count": "actual/actual",
        "manifest": {"calculator": trial.RANGE_CALC_URI, "period": trial.PERIOD_URI,
                     "rate_table_uris": [], "engine": "dep/1.0.0", "citation": "AASB 116"},
        "advisory": {"figure_type": "accounting_carrying_amount",
                     "notes": ["Not a Division 40 deduction. Do not carry into a tax return."]},
    }
    payload.update(extra)
    return payload


def test_the_request_requires_a_day_count_and_rejects_an_unknown_one():
    body = trial.range_request(ASSET, "2024-07-01", "2025-06-30", "actual/actual")
    assert body["day_count"] == "actual/actual"
    assert body["basis"] == "accounting"
    assert body["asset"]["cost"] == Decimal("120000.00")
    with pytest.raises(ValueError):
        trial.range_request(ASSET, "2024-07-01", "2025-06-30", "actual/360")


def test_a_closing_movement_is_a_match(stub, stub_config, contract):
    _, state = stub
    state.respond(provider_range())
    comparison = trial.evaluate_range(ASSET, "2024-07-01", "2025-06-30", "actual/actual",
                                      LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.MATCH
    assert any("Division 40" in reason for reason in comparison.reasons)


def test_a_movement_that_does_not_close_is_refused_and_no_addition_is_invented(
    stub,
    stub_config,
    contract):
    _, state = stub
    # The provider's published pre-acquisition case: the cost enters inside the
    # window, no cost_additions field is emitted, and the three figures do not
    # reconcile.
    state.respond(provider_range(opening="0.00", closing="108000.00", depreciation="12000.00"))
    comparison = trial.evaluate_range(ASSET, "2024-01-01", "2024-12-31", "actual/actual",
                                      LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.CONTRACT_FAILURE
    assert any("does not close" in reason for reason in comparison.reasons)
    assert any("pre-acquisition limitation" in reason for reason in comparison.reasons)
    assert "cost_additions" not in comparison.upstream


def test_a_separately_evidenced_addition_closes_it_and_is_named(stub, stub_config, contract):
    _, state = stub
    state.respond(provider_range(opening="0.00", closing="108000.00", depreciation="12000.00"))
    comparison = trial.evaluate_range(
        ASSET, "2024-01-01", "2024-12-31", "actual/actual",
        LodgeitClient(stub_config, contract), supplied_additions=Decimal("120000.00"),
    )
    assert comparison.evaluation is Evaluation.MATCH
    assert any("supplied by the caller" in reason for reason in comparison.reasons)


def test_reconcile_refuses_rather_than_deriving_the_gap():
    values = {"opening_wdv": Decimal("0.00"), "closing_wdv": Decimal("108000.00"),
              "range_dep": Decimal("12000.00")}
    closes, reasons = trial.reconcile(values)
    assert closes is False
    assert any("No addition has been invented" in reason for reason in reasons)
    closes, reasons = trial.reconcile(values, supplied_additions=Decimal("120000.00"))
    assert closes is True


def test_a_missing_figure_cannot_be_reconciled_around():
    closes, reasons = trial.reconcile({"opening_wdv": Decimal("1.00")})
    assert closes is False
    assert any("missing" in reason for reason in reasons)


def test_the_pre_acquisition_window_is_recognised_before_the_call():
    assert trial.spans_acquisition(ASSET, "2024-01-01") is True
    assert trial.spans_acquisition(ASSET, "2024-07-01") is False
    assert trial.spans_acquisition(ASSET, "2025-01-01") is False


def test_consecutive_windows_telescope_and_cover_every_day():
    windows = trial.consecutive_windows("2024-07-01", "2025-06-30", ["2024-12-31"])
    assert windows == [("2024-07-01", "2024-12-31"), ("2025-01-01", "2025-06-30")]
    # A leap-year boundary: 29 February exists and the split is still contiguous.
    leap = trial.consecutive_windows("2024-01-01", "2024-12-31", ["2024-02-29"])
    assert leap == [("2024-01-01", "2024-02-29"), ("2024-03-01", "2024-12-31")]


def test_consecutive_ranges_sum_to_the_whole(stub, stub_config, contract):
    """Two halves against the year: the closing of one is the opening of the next."""
    _, state = stub
    client = LodgeitClient(stub_config, contract)
    first = provider_range(opening="120000.00", closing="107934.43", depreciation="12065.57")
    second = provider_range(opening="107934.43", closing="96000.00", depreciation="11934.43")
    results = []
    for payload, (start, end) in zip((first, second),
                                     trial.consecutive_windows(
                                         "2024-07-01",
                                         "2025-06-30",
                                         ["2024-12-31"])):
        state.respond(payload)
        results.append(trial.evaluate_range(ASSET, start, end, "actual/actual", client))
    assert all(item.evaluation is Evaluation.MATCH for item in results)
    total = sum(item.upstream["range_dep"] for item in results)
    assert total == Decimal("24000.00")
    assert results[0].upstream["closing_wdv"] == results[1].upstream["opening_wdv"]


def test_life_end_leaves_nothing(stub, stub_config, contract):
    _, state = stub
    state.respond(provider_range(opening="24000.00", closing="0.00", depreciation="24000.00"))
    comparison = trial.evaluate_range(ASSET, "2028-07-01", "2029-06-30", "actual/actual",
                                      LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.MATCH
    assert comparison.upstream["closing_wdv"] == Decimal("0.00")


def test_a_pooled_asset_refusal_stays_a_scope_mismatch(stub, stub_config, contract):
    _, state = stub
    state.respond({"refusal_class": "pool_asset_out_of_t6_scope", "reason": "pooled"}, status=400)
    comparison = trial.evaluate_range(ASSET, "2024-07-01", "2025-06-30", "actual/actual",
                                      LodgeitClient(stub_config, contract))
    assert comparison.evaluation is Evaluation.SCOPE_MISMATCH
    assert comparison.upstream == {}
