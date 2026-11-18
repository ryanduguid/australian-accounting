# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""get_reviewed_rate answers from a record that covers the date, or refuses with no figure."""

import asyncio
import json
import re
import tomllib
from datetime import date
from pathlib import Path

import pytest
from jsonschema import Draft202012Validator
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp import freshness, reviewed_rates
from aus_accounting_mcp.server import mcp

SCHEMA_TOOL = "get_reviewed_rate"
SCHEMA = next(
    tool.output_schema for tool in asyncio.run(mcp.list_tools()) if tool.name == SCHEMA_TOOL
)
DATA = Path(reviewed_rates.__file__).parent / "rate_records"


def call(record_id, as_at):
    arguments = {"record_id": record_id, "as_at": as_at}
    payload = asyncio.run(mcp.call_tool("get_reviewed_rate", arguments)).structured_content
    Draft202012Validator(SCHEMA).validate(payload)
    return payload


def assert_no_figure(result):
    assert result["status"] == "REFUSED"
    assert result["record"] is None
    assert set(result) == set(SCHEMA["properties"])
    assert result["after_last_read"] is False
    assert result["after_last_read_note"] is None
    record = reviewed_rates.records().get(result["record_id"])
    if record is None:
        return
    assert result["record_period"] == record["period"]
    assert result["statement"] == reviewed_rates.STATEMENT
    assert record["quote"] not in json.dumps(result, ensure_ascii=False)
    # Every value and tax scale holds a digit, so once the reason's own date, id and year
    # labels are out, a digit left in it is a figure.
    said = result["reason"].replace(result["as_at"], "").replace(result["record_id"], "")
    for year in result["covered_years"]:
        said = said.replace(year, "")
    assert not re.search(r"\d", said), said


def test_a_covered_date_returns_the_figure_with_its_source_and_quote():
    result = call("concessional-cap-2026-27", "2027-03-01")

    assert result["status"] == "FOUND"
    assert result["record"] == json.loads(
        (DATA / "concessional-cap-2026-27.json").read_text(encoding="utf-8")
    )
    assert result["record"]["value"] == 32500
    assert result["record"]["source_url"].startswith("https://www.ato.gov.au/")
    assert "$32,500" in result["record"]["quote"]
    assert result["currency"] == "NOT_ESTABLISHED"
    assert "not a live lookup" in result["statement"]
    assert result["refusal"] is None


def test_the_records_cover_two_income_years():
    assert reviewed_rates.covered_years() == ["2025-26", "2026-27"]
    assert call("fbt-rate", "2026-01-01")["covered_years"] == ["2025-26", "2026-27"]


@pytest.mark.parametrize("as_at,status", [
    ("2026-06-30", "REFUSED"), ("2026-07-01", "FOUND"),
    ("2027-06-30", "FOUND"), ("2027-07-01", "REFUSED"),
])
def test_an_income_year_runs_from_1_july_to_30_june(as_at, status):
    assert call("concessional-cap-2026-27", as_at)["status"] == status


@pytest.mark.parametrize("record_id,as_at", [
    # Before and after the covered years: never the nearest year's figure.
    ("concessional-cap-2025-26", "2024-06-30"),
    ("concessional-cap-2026-27", "2027-07-01"),
    ("div7a-benchmark-rate-2026-27", "2031-01-01"),
    # A record whose own period holds the date stays inside the covered years too.
    ("fbt-rate", "2019-01-01"),
    ("company-tax-rate-full", "2012-01-01"),
    # An open-ended record says nothing about a date past the covered years.
    ("gst-registration-threshold", "2030-01-01"),
])
def test_a_date_outside_the_covered_years_is_refused_and_says_so(record_id, as_at):
    result = call(record_id, as_at)

    assert_no_figure(result)
    assert result["refusal"] == "OUTSIDE_COVERED_YEARS"
    assert as_at in result["reason"]
    assert "2025-26 and 2026-27 only" in result["reason"]
    assert "No figure is returned" in result["reason"]
    assert result["covering_ids"] == []


def test_an_open_ended_record_answers_inside_the_covered_years():
    result = call("gst-registration-threshold", "2026-01-01")

    assert result["status"] == "FOUND"
    assert result["record"]["value"] == 75000
    assert "no effective_to" in result["statement"]


def test_an_open_ended_record_is_flagged_after_the_day_it_was_last_read():
    # gst-registration-threshold is open-ended and was read on 15 October 2026.
    on_the_day = call("gst-registration-threshold", "2026-10-15")
    day_after = call("gst-registration-threshold", "2026-10-16")

    assert on_the_day["status"] == day_after["status"] == "FOUND"
    assert on_the_day["after_last_read"] is False
    assert on_the_day["after_last_read_note"] is None
    assert day_after["after_last_read"] is True
    assert day_after["after_last_read_note"] == (
        "This record was last read on 2026-10-15 and does not show that the figure still "
        "applied on 2026-10-16."
    )
    assert day_after["record"]["value"] == 75000


@pytest.mark.parametrize("record_id,as_at", [
    # The law sets an income year's figure for the whole year, and a closed period ends itself.
    ("concessional-cap-2026-27", "2027-06-30"),
    ("gic-annual-rate-oct-dec-2026", "2026-12-31"),
])
def test_a_record_with_an_end_is_not_flagged(record_id, as_at):
    result = call(record_id, as_at)

    assert result["status"] == "FOUND"
    assert result["after_last_read"] is False
    assert result["after_last_read_note"] is None


def test_the_tool_description_states_the_limit():
    tool = next(tool for tool in asyncio.run(mcp.list_tools()) if tool.name == SCHEMA_TOOL)

    assert "after_last_read" in tool.description
    assert "last read" in tool.description


def test_a_dated_record_ends_on_its_last_day():
    assert call("gic-annual-rate-jul-sep-2026", "2026-09-30")["status"] == "FOUND"
    assert call("instant-asset-write-off-threshold", "2026-06-30")["status"] == "FOUND"


def test_a_date_outside_the_record_names_the_record_that_holds_it():
    result = call("concessional-cap-2026-27", "2026-03-01")

    assert_no_figure(result)
    assert result["refusal"] == "OUTSIDE_RECORD_PERIOD"
    assert result["record_period"] == {"income_year": "2026-27"}
    assert result["covering_ids"] == ["concessional-cap-2025-26"]


def test_no_refusal_at_a_boundary_date_carries_any_record_s_figure():
    boundaries = (
        "2025-06-30", "2025-07-01", "2026-06-30", "2026-07-01", "2027-06-30", "2027-07-01",
    )
    refused = {}

    for record_id in reviewed_rates.records():
        for as_at in boundaries:
            result = call(record_id, as_at)
            if result["status"] == "REFUSED":
                assert_no_figure(result)
                refused.setdefault(result["refusal"], set()).add(record_id)

    assert set(refused) == {"OUTSIDE_COVERED_YEARS", "OUTSIDE_RECORD_PERIOD", "NO_RECORDED_PERIOD"}
    assert "resident-tax-rates-2026-27" in refused["OUTSIDE_RECORD_PERIOD"]
    assert reviewed_rates.records()["resident-tax-rates-2026-27"]["unit"] == "tax_scale"


def test_a_date_between_quarters_names_the_quarter_that_holds_it():
    result = call("gic-annual-rate-apr-jun-2026", "2026-11-06")

    assert_no_figure(result)
    assert result["covering_ids"] == ["gic-annual-rate-oct-dec-2026"]


def test_a_year_with_no_record_of_that_figure_is_refused_with_nothing_to_offer():
    result = call("medicare-low-income-lower-threshold-2025-26", "2026-08-01")

    assert_no_figure(result)
    assert result["refusal"] == "OUTSIDE_RECORD_PERIOD"
    assert result["covering_ids"] == []


def test_a_record_with_no_effective_period_is_refused():
    result = call("payg-instalments-entry-tax-payable", "2026-10-15")

    assert_no_figure(result)
    assert result["refusal"] == "NO_RECORDED_PERIOD"
    assert result["record_period"] == {"as_at": "2026-10-15"}


def test_an_unknown_id_is_refused_with_every_id():
    result = call("concessional-cap", "2026-08-01")

    assert_no_figure(result)
    assert result["refusal"] == "UNKNOWN_ID"
    assert result["record_period"] is None
    assert result["known_ids"] == sorted(path.stem for path in DATA.glob("*.json"))
    assert "concessional-cap-2026-27" in result["known_ids"]


@pytest.mark.parametrize("as_at", ["2026-02-30", "2026-13-01"])
def test_an_impossible_date_is_an_input_error(as_at):
    with pytest.raises(ToolError, match="as_at must be a YYYY-MM-DD date"):
        call("concessional-cap-2026-27", as_at)


def test_every_dated_record_answers_at_the_first_date_it_holds_and_returns_itself():
    held = reviewed_rates.records()

    assert len(held) == 74
    for record_id, record in held.items():
        period = record["period"]
        if "as_at" in period:
            continue
        start = period.get("effective_from") or f"{period['income_year'][:4]}-07-01"
        day = max(date.fromisoformat(start), date(2025, 7, 1)).isoformat()
        result = call(record_id, day)
        assert result["status"] == "FOUND", record_id
        assert result["record"] == record


def test_the_freshness_report_lists_the_records():
    dataset = next(
        entry for entry in freshness.data_freshness(tool="get_reviewed_rate")["datasets"]
    )

    assert dataset["dataset"] == "reviewed_ato_rates"
    assert [period["id"] for period in dataset["effective_periods"]] == ["2025-26", "2026-27"]
    assert dataset["provenance"] == {"records": 74}
    assert all(url.startswith("https://www.ato.gov.au/") for url in dataset["sources"])


def test_the_wheel_is_told_to_carry_the_records():
    # A wheel without them would break get_data_freshness as well as this tool.
    pyproject = tomllib.loads(
        (DATA.parents[1] / "pyproject.toml").read_text(encoding="utf-8")
    )
    patterns = pyproject["tool"]["setuptools"]["package-data"]["aus_accounting_mcp"]

    assert sorted(DATA.parent.glob(patterns[0])) == sorted(DATA.glob("*.json"))
