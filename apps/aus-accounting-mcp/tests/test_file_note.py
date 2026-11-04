# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: FSL-1.1-ALv2

"""The optional file note restates a worksheet result and adds nothing to it."""

import asyncio
import json
import re

import pytest
from austaxcalc import calculations
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.server import mcp

GST = {"kind": "gst", "scope_confirmed": True, "year": "2025-26",
       "amount": "1100", "gst_inclusive": True}
HEADINGS = ["## Facts relied on", "## Calculation", "## Assumptions",
            "## Provisions and sources", "## Open issues"]


def _call(arguments: dict) -> dict:
    result = asyncio.run(mcp.call_tool("calculate_tax_worksheet", arguments))
    assert not result.is_error
    return result.structured_content


def test_the_note_is_opt_in_and_leaves_the_result_unchanged() -> None:
    plain = _call({"facts": GST})
    assert "file_note" not in plain
    assert _call({"facts": GST, "file_note": False}) == plain
    noted = _call({"facts": GST, "file_note": True})
    assert {key: value for key, value in noted.items() if key != "file_note"} == plain


def test_a_gst_note_restates_the_facts_figures_scope_sources_and_warnings() -> None:
    result = _call({"facts": GST, "file_note": True})
    note = result["file_note"]
    lines = note.splitlines()
    assert lines[0] == "# File note: gst worksheet, 2025-26"
    assert [line for line in lines if line.startswith("## ")] == HEADINGS
    # $1,100 including GST at 10%: GST is 1,100 / 11 = $100.00.
    for line in ("- `amount`: 1100", "- `gst_inclusive`: true", "- `scope_confirmed`: true",
                 "| `gst` | 100.00 |", "| `exclusive` | 1000.00 |", "| `gst_rate` | 0.10 |"):
        assert line in lines
    assert result["scope"] in lines
    for url in result["sources"]:
        assert f"- {url} (checked {result['source_checked']})" in lines
    for warning in result["warnings"]:
        assert f"- {warning}" in lines
    passage = calculations.worksheet_catalogue()["gst"]["source_passage"]
    assert f"The engine's source check covered: {passage}." in lines


def _numbers(text: str) -> set[str]:
    return set(re.findall(r"\d+(?:\.\d+)?", text))


@pytest.mark.parametrize("kind", sorted(calculations.worksheet_catalogue()))
def test_every_kind_yields_a_note_whose_every_number_comes_from_the_result(kind) -> None:
    entry = calculations.worksheet_catalogue()[kind]
    facts = {**entry["example"]["facts"], **entry["supported_periods"][0]["arguments"]}
    result = _call({"facts": facts, "file_note": True})
    note = result.pop("file_note")
    assert [line for line in note.splitlines() if line.startswith("## ")] == HEADINGS
    for name, value in result["amounts"].items():
        assert f"| `{name}` | {value} |" in note
    # Nothing beyond the facts, the engine result and the source passage: a
    # number anywhere else in the note would be one the facade invented.
    stated = json.dumps(facts) + json.dumps(result) + entry.get("source_passage", "")
    assert _numbers(note) <= _numbers(stated)


def test_a_refused_worksheet_returns_its_refusal_and_no_note() -> None:
    with pytest.raises(ToolError, match="Establish all stated scope conditions"):
        asyncio.run(mcp.call_tool("calculate_tax_worksheet", {
            "facts": {**GST, "scope_confirmed": False}, "file_note": True}))


def test_the_option_takes_a_json_boolean_only() -> None:
    with pytest.raises(ToolError, match="Input should be a valid boolean"):
        asyncio.run(mcp.call_tool("calculate_tax_worksheet",
                                  {"facts": GST, "file_note": "true"}))
