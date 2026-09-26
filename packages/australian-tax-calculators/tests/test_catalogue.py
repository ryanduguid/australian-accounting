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


@pytest.mark.parametrize("kind,entry", c.worksheet_catalogue().items())
def test_every_advertised_period_and_method_runs(kind, entry):
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


@pytest.mark.parametrize("kind,entry", c.worksheet_catalogue().items())
def test_unsupported_period_and_unconfirmed_scope_remain_refused(kind, entry):
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
