import asyncio
import json

import pytest
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.server import mcp

_MAX_MONEY = "1000000000000.00"
_MONETARY_ENDPOINT_FIELDS = [
    ("get_ato_benchmarks", "turnover"),
    ("get_ato_benchmarks", "cost_of_sales"),
    ("calc_payday_super_deadline", "sg_amount"),
    ("refuse_div7a", "loan_principal"),
    ("generate_synthetic_sbr_fixture", "revenue_or_sales"),
]


def _call_tool(name, arguments):
    result = asyncio.run(mcp.call_tool(name, arguments))
    if result.structured_content is not None:
        return result.structured_content
    return json.loads(result.content[0].text)


def _call_tool_with_monetary_value(tool_name, field_name, value):
    if tool_name == "get_ato_benchmarks":
        arguments = {
            "industry": "Bakeries and hot bread shops",
            "turnover": "850000.00",
            "cost_of_sales": "270000.00",
        }
    elif tool_name == "calc_payday_super_deadline":
        arguments = {
            "qe_day": "2026-08-06",
            "sg_amount": "800.00",
            "received": "2026-08-10",
            "as_at": "2026-08-21",
        }
    elif tool_name == "refuse_div7a":
        arguments = {
            "borrower_name": "Alice",
            "lender_entity_name": "HoldingCo Pty Ltd",
            "loan_principal": "100000.00",
        }
    else:
        arguments = {
            "form_type": "CTR",
            "revenue_or_sales": "1000000.00",
        }

    arguments[field_name] = value
    return _call_tool(tool_name, arguments)


@pytest.mark.parametrize(
    ("tool_name", "arguments", "message"),
    [
        (
            "list_ato_benchmark_industries",
            {"year": "1900-01"},
            "no dataset for benchmark year",
        ),
        (
            "get_ato_benchmarks",
            {
                "industry": "Bakeries and hot bread shops",
                "turnover": "850000.00",
            },
            "no expense figures were supplied",
        ),
        (
            "calc_payday_super_deadline",
            {
                "qe_day": "not-a-date",
                "sg_amount": "800.00",
                "as_at": "2026-08-21",
            },
            "qe_day: 'not-a-date' is not an ISO date",
        ),
        (
            "generate_synthetic_sbr_fixture",
            {"form_type": "GST"},
            "Unknown form_type 'GST'. Supported: CTR, BAS.",
        ),
    ],
)
def test_expected_input_errors_remain_visible_to_mcp_clients(
    tool_name,
    arguments,
    message,
):
    with pytest.raises(ToolError, match=message):
        _call_tool(tool_name, arguments)


def test_payday_mcp_tool_keeps_exact_decimal_strings_from_the_engine():
    result = _call_tool(
        "calc_payday_super_deadline",
        {
            "qe_day": "2026-08-06",
            "sg_amount": "800.00",
            "received": "2026-08-10",
            "as_at": "2026-08-21",
        },
    )

    assert result["engine"] == "payday-super-checker"
    assert result["result"]["sg_amount"] == "800.00"
    assert result["result"]["verdict"] == "ON_TIME"
    assert "sgc_exposure" not in result
    assert "is_compliant" not in result


def test_div7a_mcp_tool_validates_money_then_refuses():
    result = _call_tool(
        "refuse_div7a",
        {
            "borrower_name": "Alice",
            "lender_entity_name": "HoldingCo Pty Ltd",
            "loan_principal": "50000.00",
        },
    )
    assert result["available"] is False
    assert result["reviewed_engine"] is False


@pytest.mark.parametrize(("tool_name", "field_name"), _MONETARY_ENDPOINT_FIELDS)
def test_every_monetary_mcp_schema_is_a_decimal_string(tool_name, field_name):
    tools = asyncio.run(mcp.list_tools())
    tool = next(candidate for candidate in tools if candidate.name == tool_name)
    field_schema = tool.input_schema["properties"][field_name]
    types = (
        {field_schema.get("type")}
        if "type" in field_schema
        else {entry["type"] for entry in field_schema["anyOf"]}
    )
    assert "string" in types
    assert "number" not in types


@pytest.mark.parametrize(("tool_name", "field_name"), _MONETARY_ENDPOINT_FIELDS)
@pytest.mark.parametrize(
    "invalid_value",
    [
        "1000000000000.01",
        "-1000000000000.01",
        "1e30",
    ],
)
def test_every_monetary_endpoint_rejects_values_above_the_domain_limit(
    tool_name,
    field_name,
    invalid_value,
):
    with pytest.raises(
        ToolError,
        match=(
            rf"{field_name} absolute value must not exceed "
            r"AUD 1000000000000\.00"
        ),
    ):
        _call_tool_with_monetary_value(tool_name, field_name, invalid_value)


@pytest.mark.parametrize(("tool_name", "field_name"), _MONETARY_ENDPOINT_FIELDS)
@pytest.mark.parametrize("invalid_value", ["0.001"])
def test_every_monetary_endpoint_rejects_more_than_two_decimal_places(
    tool_name,
    field_name,
    invalid_value,
):
    with pytest.raises(
        ToolError,
        match=rf"{field_name} must have no more than 2 decimal places",
    ):
        _call_tool_with_monetary_value(tool_name, field_name, invalid_value)


@pytest.mark.parametrize(("tool_name", "field_name"), _MONETARY_ENDPOINT_FIELDS)
def test_every_monetary_endpoint_accepts_the_limit_and_serialises_finite_json(
    tool_name,
    field_name,
):
    result = _call_tool_with_monetary_value(tool_name, field_name, _MAX_MONEY)
    json.dumps(result, allow_nan=False)
