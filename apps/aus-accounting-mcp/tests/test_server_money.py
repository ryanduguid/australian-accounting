import asyncio
import json
import re

import pytest
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.server import mcp

_MAX_MONEY = "1000000000000.00"
_BOOLEAN_FACTS = [
    ("calc_payday_super_deadline", field)
    for field in ("first_to_fund", "out_of_cycle", "db_interest")
] + [
    ("review_div7a_loan", field)
    for field in (
        "written_agreement", "terms_in_place_before_lodgment_day",
        "secured_by_registered_mortgage_over_real_property",
    )
]
_MONETARY_ENDPOINT_FIELDS = [
    ("get_ato_benchmarks", "turnover"),
    ("get_ato_benchmarks", "cost_of_sales"),
    ("calc_payday_super_deadline", "sg_amount"),
    ("review_div7a_loan", "amalgamated_loan_unpaid_at_end_of_previous_year"),
    ("review_div7a_loan", "payments_applied_during_the_year"),
    ("generate_synthetic_sbr_fixture", "revenue_or_sales"),
]


def _call_tool(name, arguments):
    result = asyncio.run(mcp.call_tool(name, arguments))
    if result.structured_content is not None:
        return result.structured_content
    return json.loads(result.content[0].text)


@pytest.mark.parametrize("tool_name,field", _BOOLEAN_FACTS)
@pytest.mark.parametrize("value", ["yes", "false", 1, 0])
def test_statutory_facts_reject_non_boolean_json_values(tool_name, field, value):
    arguments = (
        {"qe_day": "2027-07-08", "sg_amount": "120.00", "as_at": "2027-08-01",
         "next_standard_qe_day": "2027-07-15"}
        if tool_name == "calc_payday_super_deadline"
        else {"year_of_income": "2025-26"}
    )
    with pytest.raises(ToolError, match=field):
        _call_tool(tool_name, {**arguments, field: value})


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
            "matched_amount": "800.00",
            "as_at": "2026-08-21",
        }
    elif tool_name == "review_div7a_loan":
        arguments = {
            "year_of_income": "2026-27",
            "year_loan_made": "2025-26",
            "written_agreement": True,
            "terms_in_place_before_lodgment_day": True,
            "maximum_term_years": "7",
            "secured_by_registered_mortgage_over_real_property": False,
            "interest_rate_for_years_after_year_loan_made": "0.0837",
            "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
            "remaining_term_years": "1",
            "payments_applied_during_the_year": "108770.00",
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
            # The message names the field and what would be read instead, so a
            # caller holding a payroll export knows which shapes are accepted.
            "qe_day: 'not-a-date' is not a date this tool reads",
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
            "matched_amount": "800.00",
            "as_at": "2026-08-21",
        },
    )

    assert result["engine"] == "payday-super-checker"
    assert result["result"]["sg_amount"] == "800.00"
    assert result["result"]["verdict"] == "ON_TIME"
    assert "sgc_exposure" not in result
    assert "is_compliant" not in result


def test_div7a_mcp_tool_refuses_without_inputs():
    result = _call_tool("refuse_div7a", {})
    assert result["available"] is False
    assert result["reviewed_engine"] is True


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
        "9999999999999.99",
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


@pytest.mark.parametrize("raw", ["1,2,3", "1,234.50", "(1,234.50)"])
def test_parse_amount_refuses_every_grouped_or_bracketed_form(raw):
    # The MCP boundary reads a JSON decimal string, not a spreadsheet cell, so a
    # comma or a bracket is a malformed argument rather than an accounting
    # presentation to decode. "1,2,3" is refused here and in every engine behind
    # this server, so no path through the repository reads it as 123.
    from aus_accounting_mcp.money import parse_amount

    with pytest.raises(InputError):
        parse_amount(raw, "amount")


# Plain-notation contract. Every money argument publishes MONEY_PATTERN in its schema
# and money.py refuses forms outside it with the existing wording.
_DIRECT_MONEY_FIELDS = {
    "get_ato_benchmarks": [
        "turnover", "other_income", "cost_of_sales", "cost_of_sales_labour", "salary_wages",
        "contractor_commission", "associated_persons", "rent", "motor_vehicle",
        "other_expense", "w1",
    ],
    "calc_payday_super_deadline": ["sg_amount", "remitted_amount", "matched_amount"],
    "review_div7a_loan": [
        "amalgamated_loan_unpaid_at_end_of_previous_year", "payments_applied_during_the_year",
    ],
    "generate_synthetic_sbr_fixture": ["revenue_or_sales"],
}
_CONTRIBUTION_TOOLS = ("review_payday_super_contributions", "build_payday_super_evidence_pack")
_CONTRIBUTION_MONEY_FIELDS = ("sg_amount", "remitted_amount", "matched_amount")
_WORKSHEET_FACTS = {
    "gst": {"year": "2025-26", "amount": "1100.00", "gst_inclusive": True},
    "resident_tax": {"year": "2025-26", "taxable_income": "85000.00"},
    "capital_gains": {
        "year": "2025-26", "other_gains": "1000.00", "discount_gains": "2000.00",
        "current_losses": "500.00", "prior_losses": "0.00",
    },
    "fbt": {"year_ended": 2026, "type_one_value": "1000.00", "type_two_value": "2000.00"},
    "depreciation": {
        "year": "2025-26", "cost": "10000.00", "effective_life": "5", "days": 365,
        "taxable_use": "1", "method": "prime_cost",
    },
    "quarterly_sg": {
        "year": "2025-26", "quarter": 1, "ordinary_time_earnings": "20000.00",
        "qualifying_contributions": "2300.00",
    },
    "payg_withholding": {
        "year": "2026-27", "earnings": "1500.00", "pay_period": "weekly", "scale": 2,
    },
    "contribution_caps": {
        "year": "2025-26", "total_super_balance": "100000.00",
        "concessional_contributions": "20000.00", "unused_concessional_cap": "0.00",
        "non_concessional_contributions": "0.00", "under_75_in_year": True,
    },
    "pension_minimum": {"year": "2025-26", "account_balance": "500000.00", "age": 66, "days": 365},
    "study_loan_repayment": {"year": "2025-26", "repayment_income": "80000.00"},
}
_WORKSHEET_MONEY_FIELDS = {
    "gst": ["amount"],
    "resident_tax": ["taxable_income"],
    "capital_gains": ["other_gains", "discount_gains", "current_losses", "prior_losses"],
    "fbt": ["type_one_value", "type_two_value"],
    "depreciation": ["cost"],
    "quarterly_sg": ["ordinary_time_earnings", "qualifying_contributions"],
    "payg_withholding": ["earnings"],
    "contribution_caps": [
        "total_super_balance", "concessional_contributions", "unused_concessional_cap",
        "non_concessional_contributions",
    ],
    "pension_minimum": ["account_balance"],
    "study_loan_repayment": ["repayment_income"],
}
_NOT_PLAIN = ["1e3", "1_000.50", "+12"]


def _money_pattern_paths():
    from aus_accounting_mcp.money import MONEY_PATTERN

    found = set()

    def walk(node, path, definitions):
        if isinstance(node, list):
            for item in node:
                walk(item, path, definitions)
            return
        if not isinstance(node, dict):
            return
        if "$ref" in node:
            walk(definitions[node["$ref"].split("/")[-1]], path, definitions)
        if node.get("pattern") == MONEY_PATTERN:
            found.add(path)
        for name, child in node.get("properties", {}).items():
            walk(child, (*path, name), definitions)
        for key in ("anyOf", "oneOf", "allOf", "items"):
            walk(node.get(key), path, definitions)

    for tool in asyncio.run(mcp.list_tools()):
        walk(tool.input_schema, (tool.name,), tool.input_schema.get("$defs", {}))
    return found


def test_every_money_argument_and_only_money_publishes_the_plain_pattern():
    expected = {
        (tool, field) for tool, fields in _DIRECT_MONEY_FIELDS.items() for field in fields
    }
    expected |= {
        (tool, "contributions", field)
        for tool in _CONTRIBUTION_TOOLS for field in _CONTRIBUTION_MONEY_FIELDS
    }
    expected |= {
        ("calculate_tax_worksheet", "facts", field)
        for fields in _WORKSHEET_MONEY_FIELDS.values() for field in fields
    }
    assert len(expected) == 41
    assert _money_pattern_paths() == expected


@pytest.mark.parametrize(
    ("tool_name", "field_name"),
    [(tool, field) for tool, fields in _DIRECT_MONEY_FIELDS.items() for field in fields],
)
@pytest.mark.parametrize("raw", _NOT_PLAIN)
def test_every_direct_money_argument_refuses_non_plain_notation(tool_name, field_name, raw):
    # The schema pattern is published, not enforced by pydantic, so the call still
    # reaches money.py and keeps the server's own wording.
    message = rf"{field_name}: '{re.escape(raw)}' is not a decimal amount"
    with pytest.raises(ToolError, match=message):
        _call_tool_with_monetary_value(tool_name, field_name, raw)


@pytest.mark.parametrize("tool_name", _CONTRIBUTION_TOOLS)
@pytest.mark.parametrize("field_name", _CONTRIBUTION_MONEY_FIELDS)
@pytest.mark.parametrize("raw", _NOT_PLAIN)
def test_every_contribution_money_field_refuses_non_plain_notation(tool_name, field_name, raw):
    row = {
        "employee_id": "fabricated-1", "qe_day": "2026-08-06", "sg_amount": "800.00",
        "first_to_fund": False, "out_of_cycle": False, "db_interest": False,
        "received": "2026-08-10", "matched_amount": "800.00",
        field_name: raw,
    }
    message = rf"{field_name}: '{re.escape(raw)}' is not a decimal amount"
    with pytest.raises(ToolError, match=message):
        _call_tool(tool_name, {"contributions": [row], "as_at": "2026-08-21"})


@pytest.mark.parametrize(
    ("kind", "field_name"),
    [(kind, field) for kind, fields in _WORKSHEET_MONEY_FIELDS.items() for field in fields],
)
@pytest.mark.parametrize("raw", _NOT_PLAIN)
def test_every_worksheet_money_field_refuses_non_plain_notation(kind, field_name, raw):
    facts = {"kind": kind, "scope_confirmed": True, **_WORKSHEET_FACTS[kind], field_name: raw}
    message = rf"{field_name}: '{re.escape(raw)}' is not a decimal amount"
    with pytest.raises(ToolError, match=message):
        _call_tool("calculate_tax_worksheet", {"facts": facts})


@pytest.mark.parametrize("kind", sorted(_WORKSHEET_FACTS))
def test_every_worksheet_baseline_is_a_plain_valid_calculation(kind):
    facts = {"kind": kind, "scope_confirmed": True, **_WORKSHEET_FACTS[kind]}
    result = _call_tool("calculate_tax_worksheet", {"facts": facts})
    json.dumps(result, allow_nan=False)


def test_a_plain_negative_amount_reaches_the_engine_sign_check():
    # The grammar allows a minus sign; non-negative rules stay with the engines.
    facts = {"kind": "gst", "scope_confirmed": True, **_WORKSHEET_FACTS["gst"], "amount": "-1.00"}
    with pytest.raises(ToolError, match="non-negative"):
        _call_tool("calculate_tax_worksheet", {"facts": facts})


@pytest.mark.parametrize(
    "raw",
    ["0", "0.00", "12", "12.3", "12.34", "-12.34", "0012.5", "-0", " 12.30 ",
     "1000000000000.00", "-1000000000000.00"],
)
def test_parse_amount_accepts_plain_notation(raw):
    from decimal import Decimal

    from aus_accounting_mcp.money import parse_amount

    assert parse_amount(raw, "amount") == Decimal(raw.strip())


@pytest.mark.parametrize(
    ("raw", "message"),
    [
        ("1e3", "is not a decimal amount"),
        ("1E+3", "is not a decimal amount"),
        ("1.2e6", "is not a decimal amount"),
        ("1e30", "is not a decimal amount"),
        ("1_000.50", "is not a decimal amount"),
        ("+12", "is not a decimal amount"),
        (".5", "is not a decimal amount"),
        ("12.", "is not a decimal amount"),
        ("$12.00", "is not a decimal amount"),
        ("١٢", "is not a decimal amount"),
        ("NaN", "is not a finite amount"),
        ("Infinity", "is not a finite amount"),
        ("-Infinity", "is not a finite amount"),
        ("0.001", "must have no more than 2 decimal places"),
        ("1000000000000.01", "must not exceed"),
        ("", "amount is required"),
        ("   ", "amount is required"),
    ],
)
def test_parse_amount_keeps_one_message_per_failure(raw, message):
    from aus_accounting_mcp.money import parse_amount

    with pytest.raises(InputError, match=re.escape(message)):
        parse_amount(raw, "amount")


@pytest.mark.parametrize("raw", ["", "   "])
def test_parse_optional_amount_refuses_a_blank_string(raw):
    from aus_accounting_mcp.money import parse_optional_amount

    assert parse_optional_amount(None, "amount") is None
    with pytest.raises(InputError, match="omit an unknown amount or send null"):
        parse_optional_amount(raw, "amount")


def test_published_pattern_matches_the_runtime_grammar():
    from aus_accounting_mcp.money import MONEY_PATTERN, parse_amount

    for raw in ["12.34", " -0 ", "0012.5"]:
        assert re.fullmatch(MONEY_PATTERN, raw)
        parse_amount(raw, "amount")
    for raw in ["1e3", "+12", "1_000", ".5", "12.", "0.001", "1,000"]:
        assert not re.fullmatch(MONEY_PATTERN, raw)
