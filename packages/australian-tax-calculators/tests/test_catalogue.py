"""Discovery must describe the engine that actually runs, including its refusals."""

from datetime import date, timedelta
from decimal import Decimal
from inspect import signature

import pytest
from austaxcalc import calculations as c


def _arguments(kind, entry, period):
    facts = {**entry["example"]["facts"], **period["arguments"]}
    facts.pop("kind")
    for name, unit in entry["required_inputs"].items():
        if unit.startswith(("AUD", "years,", "fraction,")):
            facts[name] = Decimal(facts[name])
    return facts


# The catalogue is built inside each test, not at collection, so a mutation
# test run checks the catalogue it is mutating rather than the original.
@pytest.mark.parametrize("kind", c.SCOPES)
def test_every_advertised_period_and_method_runs(kind):
    entry = c.worksheet_catalogue()[kind]
    function = getattr(c, kind)
    assert set(entry["required_inputs"]) - {"kind"} == set(signature(function).parameters)
    assert entry["example"]["synthetic"] is True
    assert set(entry["example"]["facts"]) == set(entry["required_inputs"])
    for period in entry["supported_periods"]:
        assert date.fromisoformat(period["start"]) < date.fromisoformat(period["end"])
        arguments = _arguments(kind, entry, period)
        variants = [{"method": method} for method in entry["methods"]] or [{}]
        if kind == "quarterly_sg":
            quarters = period["quarters"]
            assert quarters[0]["start"] == period["start"]
            assert quarters[-1]["end"] == period["end"]
            for previous, following in zip(quarters, quarters[1:]):
                assert date.fromisoformat(previous["end"]) + timedelta(days=1) == (
                    date.fromisoformat(following["start"]))
            variants = [{"quarter": quarter["quarter"]} for quarter in quarters]
        for variant in variants:
            result = function(**{**arguments, **variant})
            assert result["source_checked"] == entry["source_checked"]
            assert result["sources"] == entry["sources"]
            assert result["sources"][0] == entry["source"]
            assert result["scope"] == entry["scope"]
            assert result["warnings"]


@pytest.mark.parametrize("kind", c.SCOPES)
def test_unsupported_period_and_unconfirmed_scope_remain_refused(kind):
    entry = c.worksheet_catalogue()[kind]
    arguments = _arguments(kind, entry, entry["supported_periods"][0])
    period_field = "year_ended" if kind == "fbt" else "year"
    unsupported = 2027 if kind == "fbt" else "2027-28"
    with pytest.raises(ValueError, match="Unsupported period"):
        getattr(c, kind)(**{**arguments, period_field: unsupported})
    with pytest.raises(ValueError, match="scope conditions"):
        getattr(c, kind)(**{**arguments, "scope_confirmed": False})


def test_fbt_has_its_own_year_boundary():
    period = c.worksheet_catalogue()["fbt"]["supported_periods"][0]
    assert period == {
        "id": "2026", "basis": "FBT year", "start": "2025-04-01",
        "end": "2026-03-31", "arguments": {"year_ended": 2026},
    }


def test_caller_cannot_mutate_the_engine_through_discovery():
    catalogue = c.worksheet_catalogue()
    catalogue["gst"]["example"]["facts"]["amount"] = "999"
    catalogue["gst"]["supported_periods"][0]["arguments"]["year"] = "2030-31"
    catalogue["gst"]["required_inputs"]["amount"] = "float"
    catalogue["gst"]["example_evidence"]["reviewed"] = "2099-01-01"
    fresh = c.worksheet_catalogue()["gst"]
    assert fresh["example"]["facts"]["amount"] == "1100.00"
    assert fresh["supported_periods"][0]["arguments"]["year"] == "2025-26"
    assert fresh["required_inputs"]["amount"] == "AUD"
    assert fresh["example_evidence"]["reviewed"] == "2026-06-30"


def test_result_date_belongs_to_the_selected_rule(monkeypatch):
    # A later review of one worksheet must not refresh the dates on the other five.
    monkeypatch.setitem(c.SOURCE_REVIEWS, "gst", {"checked": "2099-01-01", "passage": "fixture"})
    assert c.gst(Decimal("110"), True, True, "2025-26")["source_checked"] == "2099-01-01"
    assert c.resident_tax(Decimal("0"), "2025-26", True)["source_checked"] != "2099-01-01"


def test_discovery_fields_a_client_reads():
    catalogue = c.worksheet_catalogue()
    for kind, entry in catalogue.items():
        assert entry["source_passage"] == c.SOURCE_REVIEWS[kind]["passage"]
        # Each anchor is the section heading in docs/calculation-evidence.md.
        assert entry["example_evidence"]["note"] == (
            "docs/calculation-evidence.md#" + kind.replace("_", "-"))
        inputs = entry["required_inputs"]
        assert inputs["scope_confirmed"] == "boolean true; establish every scope condition"
        if kind == "fbt":
            assert inputs["year_ended"] == "integer"
        else:
            assert inputs["year"] == "income-year string"
        assert entry["money_format"] == (
            "Non-negative finite AUD decimal strings, at most 2 decimal places "
            "and 1000000000000.00. Resident taxable income must be whole dollars.")
        assert entry["methods"] == (
            ["prime_cost", "diminishing_value"] if kind == "depreciation" else [])
    assert catalogue["quarterly_sg"]["example_evidence"]["note"] == (
        "docs/calculation-evidence.md#quarterly-sg")


def test_income_year_and_quarter_periods():
    assert c.worksheet_catalogue()["gst"]["supported_periods"] == [{
        "id": "2025-26", "basis": "income year", "start": "2025-07-01",
        "end": "2026-06-30", "arguments": {"year": "2025-26"},
    }]
    period = c.worksheet_catalogue()["quarterly_sg"]["supported_periods"][0]
    assert period["id"] == "2025-26"
    assert period["basis"] == "complete quarter within income year"
    assert [quarter["quarter"] for quarter in period["quarters"]] == [1, 2, 3, 4]
