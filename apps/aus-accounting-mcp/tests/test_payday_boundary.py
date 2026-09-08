"""Regressions for facts lost at the MCP boundary, using fabricated contributions."""

import asyncio
import sys

import pytest
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.server import mcp

PAYDAY = {
    "qe_day": "2027-07-01", "sg_amount": "120.00", "as_at": "2027-08-01",
    "received": "2027-07-02",
}


@pytest.mark.parametrize("amounts", [
    {"matched_amount": "50.00"},
    {"remitted_amount": "50.00", "remitted": "2027-07-02"},
    {"matched_amount": "50.00", "remitted_amount": "40.00", "remitted": "2027-07-02"},
])
def test_partial_receipt_cannot_settle_the_whole_liability(amounts):
    result = asyncio.run(mcp.call_tool("calc_payday_super_deadline", {**PAYDAY, **amounts}))
    assessment = result.structured_content["result"]
    assert assessment["verdict"] == "UNPAID"
    assert assessment["base_shortfall"] == "70.00"
    for field in ("matched_amount", "remitted_amount"):
        assert assessment[field] == amounts.get(field)


@pytest.mark.parametrize("field", ["matched_amount", "remitted_amount"])
@pytest.mark.parametrize("amount", ["NaN", "-1.00", "121.00", "1.001"])
def test_partial_amounts_use_money_and_engine_validation(field, amount):
    with pytest.raises(ToolError):
        asyncio.run(mcp.call_tool("calc_payday_super_deadline", {
            **PAYDAY, "remitted": "2027-07-02", field: amount,
        }))


@pytest.mark.parametrize("amount,verdict,shortfall", [
    ("0.00", "UNPAID", "120.00"), ("120.00", "ON_TIME", None),
])
def test_explicit_zero_and_full_receipt_have_distinct_results(amount, verdict, shortfall):
    result = asyncio.run(mcp.call_tool("calc_payday_super_deadline", {
        **PAYDAY, "matched_amount": amount,
    })).structured_content["result"]
    assert result["verdict"] == verdict
    assert result["base_shortfall"] == shortfall


def test_a_remitted_amount_requires_its_remittance_date():
    with pytest.raises(ToolError, match="remitted"):
        asyncio.run(mcp.call_tool("calc_payday_super_deadline", {
            **PAYDAY, "remitted_amount": "50.00",
        }))


@pytest.mark.parametrize("field", [
    "qe_day", "as_at", "remitted", "received", "next_standard_qe_day",
])
@pytest.mark.parametrize("text", [
    "01/07/2027 00:00", "01-07-2027 12:30:00", "01/07/27 1:30 PM",
])
def test_ambiguous_timestamps_are_refused_in_every_date_field(field, text):
    with pytest.raises(ToolError, match="ambiguous"):
        asyncio.run(mcp.call_tool("calc_payday_super_deadline", {**PAYDAY, field: text}))


@pytest.mark.parametrize("text,expected", [
    ("13/07/2027 00:00", "2027-07-13"),
    ("07/07/2027 12:30:00", "2027-07-07"),
    ("2027-07-01T00:00:00", "2027-07-01"),
])
def test_unambiguous_timestamps_keep_their_calendar_day(text, expected):
    result = asyncio.run(mcp.call_tool("calc_payday_super_deadline", {
        **PAYDAY, "qe_day": text, "received": None,
    }))
    assert result.structured_content["result"]["qe_day"] == expected


def test_stdio_rejects_unknown_arguments_and_preserves_partial_receipts():
    async def check():
        parameters = StdioServerParameters(
            command=sys.executable, args=["-m", "aus_accounting_mcp.cli"]
        )
        async with stdio_client(parameters) as (reader, writer):
            async with ClientSession(reader, writer) as session:
                await session.initialize()
                tools = (await session.list_tools()).tools
                valid = {
                    "get_ato_benchmarks": {
                        "industry": "Bakeries and hot bread shops", "turnover": "850000.00",
                        "cost_of_sales": "270000.00",
                    },
                    "calc_payday_super_deadline": PAYDAY,
                    "get_div7a_benchmark_rate": {"year_of_income": "2027-28"},
                    "review_div7a_loan": {"year_of_income": "2027-28"},
                }
                for tool in tools:
                    assert tool.input_schema.get("additionalProperties") is False
                    result = await session.call_tool(tool.name, {
                        **valid.get(tool.name, {}), "misspelled_fact": "50.00",
                    })
                    assert result.is_error
                    assert "misspelled_fact" in result.content[0].text
                result = await session.call_tool("calc_payday_super_deadline", {
                    **PAYDAY, "matched_amount": "50.00",
                })
                assert not result.is_error
                assert result.structured_content["result"]["base_shortfall"] == "70.00"
                result = await session.call_tool("calc_payday_super_deadline", {
                    **PAYDAY, "received": "02/07/2027 00:00",
                })
                assert result.is_error
    asyncio.run(check())
