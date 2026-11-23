# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: AGPL-3.0-or-later
# Additional terms under AGPL-3.0 section 7(b): see NOTICE.

import asyncio

import pytest
from jsonschema import Draft202012Validator
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.server import mcp


def lookup(**overrides):
    arguments = {"year": "2025-26", "channel": "paper", "item": "employee_union_fees"}
    result = asyncio.run(mcp.call_tool("itr_label", arguments | overrides)).structured_content
    tool = next(t for t in asyncio.run(mcp.list_tools()) if t.name == "itr_label")
    Draft202012Validator(tool.output_schema).validate(result)
    return result


def test_paper_mapping_corrects_proposed_clothing_question():
    result = lookup()
    assert result["status"] == "FOUND"
    destination = result["destination"]
    assert destination["question"] != "D3"
    assert (destination["question"], destination["label"]) == ("D5", "E")
    assert destination["source_url"].endswith("/d5-other-work-related-expenses-2026")
    assert "union fees" in destination["source_quote"]


def test_mytax_uses_its_own_heading_and_source():
    result = lookup(channel="mytax")
    assert result["status"] == "FOUND"
    destination = result["destination"]
    assert destination["heading"] == "Other work-related expenses"
    assert destination["question"] is None and destination["label"] is None
    assert "/mytax-instructions/2026/" in destination["source_url"]
    assert "union and professional fees" in destination["source_quote"]


def test_freshness_reports_the_year_and_both_sources():
    arguments = {"tool": "itr_label", "as_at": "2026-07-01"}
    report = asyncio.run(mcp.call_tool("get_data_freshness", arguments)).structured_content
    dataset, = report["datasets"]
    assert dataset["as_at_status"] == "OUTSIDE_RECORDED_PERIOD"
    assert dataset["source_checked"] == "2026-11-23"
    assert set(dataset["sources"]) == {
        lookup(channel=channel)["destination"]["source_url"] for channel in ("paper", "mytax")
    }


@pytest.mark.parametrize("overrides", [{"year": "2026-27"}, {"item": "foreign_employment_fees"}])
def test_absent_combination_keeps_destination_open(overrides):
    result = lookup(**overrides)
    assert result["status"] == "UNVERIFIED" and result["destination"] is None
    assert all(result[key] == value for key, value in overrides.items())


@pytest.mark.parametrize("overrides", [
    {"channel": "software"}, {"year": "2026"}, {"amount": "99"}, {"item": "99.00"},
])
def test_rejects_invalid_channel_year_or_amount(overrides):
    with pytest.raises(ToolError):
        lookup(**overrides)
