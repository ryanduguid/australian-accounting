"""The lodgment due-date tool for fictional entities, through the MCP boundary.

Synthetic Banksia Pty Ltd and Synthetic Grevillea are fabricated.
"""

import asyncio
import importlib.metadata

import pytest
from lodgmentcalendar.due_dates import FACT_MEANINGS
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.adapters.lodgment import DISCLAIMER
from aus_accounting_mcp.resources import component_versions, disclaimer, scope
from aus_accounting_mcp.server import mcp

TOOL = "get_lodgment_due_dates"
BANKSIA = {"obligation": "bas", "entity_type": "company", "financial_year": "2026-27",
           "channel": "bas_agent", "reporting_cycle": "quarterly",
           "facts": {"consolidated_head_company": False, "lodges_electronically": True,
                     "previous_statement_lodged_electronically": True,
                     "includes_monthly_payg_obligation": False}}


def call(**arguments):
    return asyncio.run(mcp.call_tool(TOOL, arguments)).structured_content


def test_agent_quarterly_bas_returns_the_engine_rows_with_provenance():
    result = call(**BANKSIA)

    assert [(row["period"], row["status"], row["due_date"]) for row in result["obligations"]] == [
        ("2026-27 Q1", "DETERMINED", "2026-11-25"),
        ("2026-27 Q2", "DETERMINED", "2027-03-02"),
        ("2026-27 Q3", "DETERMINED", "2027-05-26"),
        ("2026-27 Q4", "UNKNOWN", None),
    ]
    q2 = result["obligations"][1]
    assert q2["nominal_due_date"] == "2027-02-28" and q2["rule_id"] == "AS-Q-28"
    assert [day["reason"] for day in q2["holiday_roll"]["skipped"]] == [
        "Sunday", "Labour Day (WA)"]
    assert q2["source_url"].startswith("https://www.ato.gov.au/")
    assert result["engine"] == "lodgment-calendar"
    assert result["engine_version"] == importlib.metadata.version("lodgment-calendar")
    assert result["disclaimer"] == DISCLAIMER
    assert result["counts"] == {"DETERMINED": 3, "UNKNOWN": 1}
    assert result["facts_used"]["lodges_electronically"] is True
    assert result["not_decided"] and result["warnings"]


def test_self_lodged_individual_return_without_facts():
    result = call(obligation="income_tax_return", entity_type="individual",
                  financial_year="2025-26", channel="self")

    lodgement, payment = result["obligations"]
    assert (lodgement["due_date"], lodgement["payment"]) == ("2026-11-02", "separate")
    assert (payment["kind"], payment["status"], payment["nominal_due_date"]) == (
        "payment", "UNKNOWN", "2026-11-21")


def test_missing_program_facts_stay_unknown_and_are_named():
    result = call(obligation="income_tax_return", entity_type="company",
                  financial_year="2025-26", channel="tax_agent",
                  facts={"on_agent_client_list_by_cutoff": True})

    row = result["obligations"][0]
    assert (row["status"], row["due_date"], row["payment"]) == ("UNKNOWN", None, "unknown")
    assert row["missing"][0].startswith("consolidated_head_company:")


@pytest.mark.parametrize("change,message", [
    ({"reporting_cycle": None}, "reporting_cycle must be one of"),
    ({"obligation": "income_tax_return", "reporting_cycle": None},
     "a BAS agent cannot lodge an income tax return"),
    ({"financial_year": "2026-28"}, "financial_year must look like 2026-27"),
])
def test_engine_refusals_reach_the_caller(change, message):
    with pytest.raises(ToolError, match=message):
        call(**{**BANKSIA, **change})


@pytest.mark.parametrize("change", [
    {"financial_year": "2026/27"},
    {"channel": "accountant"},
    {"entity_type": "sole_trader"},
    {"reporting_cycle": "annual"},
    {"facts": {"lodges_electronically": "true"}},
    {"facts": {"lodges_electronically": 1}},
    {"facts": {"lodged_on_time": True}},
])
def test_malformed_arguments_are_rejected(change):
    with pytest.raises(ToolError):
        call(**{**BANKSIA, **change})


def test_tool_schema_describes_every_fact_and_is_read_only():
    tool = next(tool for tool in asyncio.run(mcp.list_tools()) if tool.name == TOOL)
    properties = tool.input_schema["$defs"]["LodgmentFactsInput"]["properties"]

    assert set(properties) == set(FACT_MEANINGS)
    assert all(properties[name]["description"].startswith(f"True if {FACT_MEANINGS[name]}")
               for name in FACT_MEANINGS)
    assert tool.annotations.read_only_hint and not tool.annotations.open_world_hint


def test_resources_list_the_engine_and_tool():
    assert {"distribution": "lodgment-calendar",
            "version": importlib.metadata.version("lodgment-calendar")} in (
        component_versions()["engines"])
    assert TOOL in scope()["review_tools"]
    assert f"# lodgment-calendar\n\n{DISCLAIMER}" in disclaimer()
