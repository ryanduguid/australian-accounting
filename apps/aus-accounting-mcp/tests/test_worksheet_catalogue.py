# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Check engine discovery against the facade's actual resource and input contracts."""

import asyncio
import json
import re
from typing import Literal, get_args, get_origin

import pytest
from mcp.server.mcpserver.exceptions import ToolError
from pydantic import TypeAdapter

from aus_accounting_mcp.adapters import tax
from aus_accounting_mcp.adapters.tax import TaxFacts
from aus_accounting_mcp.server import mcp

KINDS = [get_args(model.model_fields["kind"].annotation)[0]
         for model in get_args(get_args(TaxFacts)[0])]
ADAPTER = TypeAdapter(TaxFacts)


def _catalogue():
    return tax.worksheet_catalogue()


def test_scope_serves_engine_metadata_and_every_example_is_callable():
    expected = _catalogue()
    async def check():
        contents = list(await mcp.read_resource("aus-accounting://scope"))
        catalogue = json.loads(contents[0].content)["calculation_worksheets"]
        assert catalogue == expected
        for kind, entry in catalogue.items():
            for period in entry["supported_periods"]:
                facts = {**entry["example"]["facts"], **period["arguments"]}
                parsed = TypeAdapter(TaxFacts).validate_python(facts)
                assert parsed.model_dump() == facts
                result = await mcp.call_tool("calculate_tax_worksheet", {"facts": facts})
                assert not result.is_error
                assert result.structured_content["calculation"] == kind
                assert result.structured_content["source_checked"] == entry["source_checked"]
                assert result.structured_content["warnings"]
    asyncio.run(check())


def test_the_catalogue_offers_exactly_the_kinds_and_periods_the_tool_accepts():
    # The executable has more worksheets and periods; none of those is advertised.
    catalogue = _catalogue()
    assert sorted(catalogue) == sorted(KINDS)
    adapter = TypeAdapter(TaxFacts)
    for entry in catalogue.values():
        for period in entry["supported_periods"]:
            adapter.validate_python({**entry["example"]["facts"], **period["arguments"]})


@pytest.mark.parametrize("kind", KINDS)
def test_every_input_the_catalogue_describes_is_one_the_tool_accepts(kind):
    # The executable describes quarterly pays, scale 4 and optional inputs that
    # TaxFacts refuses; the served catalogue describes only what it takes.
    entry = _catalogue()[kind]
    facts = entry["example"]["facts"]
    model = type(ADAPTER.validate_python(facts))
    assert "optional_inputs" not in entry
    for name, description in entry["required_inputs"].items():
        annotation = model.model_fields[name].annotation
        if name == "method":
            described = entry["methods"]
        elif bounds := re.fullmatch(r"integer from (\d+) to (\d+)", description):
            described = [int(bound) for bound in bounds.groups()]
        elif get_origin(annotation) is Literal and name not in {"kind", "year"}:
            described = [type(get_args(annotation)[0])(value) for value in
                         re.split(r", | or ", description.removeprefix("integer "))]
        else:
            continue
        if get_origin(annotation) is Literal:
            assert set(described) == set(get_args(annotation)), name
        for value in described:
            ADAPTER.validate_python({**facts, name: value})


def test_no_served_scope_describes_what_the_tool_refuses():
    raw, served = tax._command("worksheet-catalogue", {}), _catalogue()
    for kind in KINDS:
        kept = {period["id"] for period in served[kind]["supported_periods"]}
        dropped = {period["id"] for period in raw[kind]["supported_periods"]} - kept
        assert not any(period in served[kind]["scope"] for period in dropped), kind
        # Where the executable describes more than TaxFacts takes, the scope is narrowed.
        wider = ("optional_inputs" in raw[kind]
                 or raw[kind]["required_inputs"] != served[kind]["required_inputs"]
                 or any(period in raw[kind]["scope"] for period in dropped))
        assert (kind in tax.ACCEPTED_SCOPES) == wider, kind
        assert served[kind]["scope"] == tax.ACCEPTED_SCOPES.get(kind, raw[kind]["scope"])


@pytest.mark.parametrize("kind", KINDS)
def test_advertised_required_fields_match_the_mcp_discriminator(kind):
    entry = _catalogue()[kind]
    facts = entry["example"]["facts"]
    model = type(TypeAdapter(TaxFacts).validate_python(facts))
    assert set(entry["required_inputs"]) == {
        name for name, field in model.model_fields.items() if field.is_required()
    }


@pytest.mark.parametrize("year", ["2024-25", "2025-26", "2026-27"])
def test_pension_days_bound_matches_the_engine(year):
    # Every supported year has 365 days; the schema and engine must agree.
    facts = {"kind": "pension_minimum", "scope_confirmed": True, "year": year,
             "account_balance": "100000.00", "age": 70}
    adapter = TypeAdapter(TaxFacts)

    async def check():
        result = await mcp.call_tool("calculate_tax_worksheet", {"facts": {**facts, "days": 365}})
        assert not result.is_error
    asyncio.run(check())
    with pytest.raises(ValueError):
        adapter.validate_python({**facts, "days": 366})


@pytest.mark.parametrize("year,income,expected", [
    ("2025-26", "100000", "4950.00"), ("2026-27", "100000", "4570.80"),
])
def test_study_loan_repayment_calls_through_and_refuses_cents(year, income, expected):
    facts = {"kind": "study_loan_repayment", "scope_confirmed": True, "year": year,
             "repayment_income": income}

    async def check():
        result = await mcp.call_tool("calculate_tax_worksheet", {"facts": facts})
        assert not result.is_error
        assert result.structured_content["amounts"]["compulsory_repayment"] == expected
        with pytest.raises(ToolError, match="whole-dollar"):
            await mcp.call_tool("calculate_tax_worksheet",
                                {"facts": {**facts, "repayment_income": income + ".50"}})
    asyncio.run(check())


@pytest.mark.parametrize("facts,amount,expected", [
    ({"kind": "study_loan_withholding", "year": "2026-27", "earnings": "2608.36",
      "pay_period": "weekly", "scale": 2}, "study_loan_component", "193.00"),
    ({"kind": "medicare_levy", "year": "2025-26", "taxable_income": "29000",
      "section_9a_adjustment_applies": False}, "medicare_levy", "98.90"),
    ({"kind": "medicare_levy", "year": "2025-26", "taxable_income": "40000",
      "section_9a_adjustment_applies": False}, "medicare_levy", "800.00"),
    ({"kind": "low_income_tax_offset", "year": "2026-27", "taxable_income": "45000"},
     "low_income_tax_offset", "325.00"),
    ({"kind": "division_293", "year": "2025-26", "division_293_income": "240000.00",
      "low_tax_contributions": "15000.00"}, "division_293_tax", "750.00"),
])
def test_new_worksheets_call_through_to_the_engine(facts, amount, expected):
    facts = {**facts, "scope_confirmed": True}

    async def check():
        result = await mcp.call_tool("calculate_tax_worksheet", {"facts": facts})
        assert not result.is_error
        assert result.structured_content["amounts"][amount] == expected
    asyncio.run(check())


def test_medicare_levy_refuses_a_year_without_published_thresholds():
    with pytest.raises(ValueError):
        TypeAdapter(TaxFacts).validate_python(
            {"kind": "medicare_levy", "scope_confirmed": True, "year": "2026-27",
             "taxable_income": "50000", "section_9a_adjustment_applies": False})


def test_medicare_levy_refuses_section_9a_and_requires_the_flag():
    facts = {"kind": "medicare_levy", "scope_confirmed": True, "year": "2025-26",
             "taxable_income": "40000"}

    async def check():
        with pytest.raises(ToolError, match="s 9A"):
            await mcp.call_tool("calculate_tax_worksheet",
                                {"facts": {**facts, "section_9a_adjustment_applies": True}})
        with pytest.raises(ToolError, match="section_9a_adjustment_applies"):
            await mcp.call_tool("calculate_tax_worksheet", {"facts": facts})
    asyncio.run(check())
