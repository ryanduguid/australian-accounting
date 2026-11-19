# Copyright 2026 Ryan Duguid
# SPDX-License-Identifier: LicenseRef-Duguid-Public-Attribution-1.0

"""Check engine discovery against the facade's actual resource and input contracts."""

import asyncio
import json
import re

import pytest
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.adapters import tax
from aus_accounting_mcp.server import mcp

KINDS = sorted(tax.worksheet_catalogue())


def _catalogue():
    return tax.worksheet_catalogue()


def _call(facts):
    return asyncio.run(mcp.call_tool("calculate_tax_worksheet", {"facts": facts}))


def test_every_worksheet_command_of_the_executable_is_served():
    commands = set(tax.commands())
    assert tax.NOT_WORKSHEETS <= commands
    assert {kind.replace("_", "-") for kind in KINDS} == commands - tax.NOT_WORKSHEETS


def test_scope_serves_the_executables_catalogue_and_every_example_is_callable():
    expected = tax._command("worksheet-catalogue", {})

    async def check():
        contents = list(await mcp.read_resource("aus-accounting://scope"))
        catalogue = json.loads(contents[0].content)["calculation_worksheets"]
        assert catalogue == expected
        for kind, entry in catalogue.items():
            result = await mcp.call_tool("calculate_tax_worksheet",
                                         {"facts": entry["example"]["facts"]})
            assert not result.is_error, kind
            assert result.structured_content["calculation"] == kind
            assert result.structured_content["source_checked"] == entry["source_checked"]
            assert result.structured_content["scope"] == entry["scope"]
            assert result.structured_content["warnings"]
    asyncio.run(check())


@pytest.mark.parametrize("kind", KINDS)
def test_an_input_the_catalogue_does_not_list_is_refused(kind):
    facts = {**_catalogue()[kind]["example"]["facts"], "not_an_input": True}
    with pytest.raises(ToolError, match=f"{kind} does not take not_an_input"):
        _call(facts)


@pytest.mark.parametrize("kind", KINDS)
def test_each_required_input_is_required(kind):
    entry = _catalogue()[kind]
    for name in entry["required_inputs"].keys() - {"kind"}:
        facts = {key: value for key, value in entry["example"]["facts"].items() if key != name}
        with pytest.raises(ToolError):
            _call(facts)


def test_an_unknown_kind_is_refused_by_name():
    with pytest.raises(ToolError, match="kind 'land_tax' is not a worksheet"):
        _call({"kind": "land_tax", "scope_confirmed": True, "year": "2025-26"})


def test_unconfirmed_scope_is_refused():
    facts = {**_catalogue()["gst"]["example"]["facts"], "scope_confirmed": False}
    with pytest.raises(ToolError):
        _call(facts)


def _with_item_amount(raw):
    facts = _catalogue()["business_capital_expenditure"]["example"]["facts"]
    return {**facts, "items": [{**facts["items"][0], "amount": raw}, *facts["items"][1:]]}


@pytest.mark.parametrize("raw", ["1e3", "1_000", "+5", ".5", "5."])
def test_a_nested_number_must_be_plain(raw):
    # The executable would read each of these as a number.
    message = rf"items\[0\]\.amount: '{re.escape(raw)}' is not a decimal amount"
    with pytest.raises(ToolError, match=message):
        _call(_with_item_amount(raw))


@pytest.mark.parametrize("raw", ["NaN", "Infinity"])
def test_a_nested_non_finite_number_is_refused_by_the_executable(raw):
    with pytest.raises(ToolError, match="finite"):
        _call(_with_item_amount(raw))


@pytest.mark.parametrize("year", ["2024-25", "2025-26", "2026-27"])
def test_pension_days_bound_matches_the_engine(year):
    # Every supported year has 365 days.
    facts = {"kind": "pension_minimum", "scope_confirmed": True, "year": year,
             "account_balance": "100000.00", "age": 70}
    assert not _call({**facts, "days": 365}).is_error
    with pytest.raises(ToolError, match="days must be an integer from 1 to 365"):
        _call({**facts, "days": 366})


@pytest.mark.parametrize("year,income,expected", [
    ("2025-26", "100000", "4950.00"), ("2026-27", "100000", "4570.80"),
])
def test_study_loan_repayment_calls_through_and_refuses_cents(year, income, expected):
    facts = {"kind": "study_loan_repayment", "scope_confirmed": True, "year": year,
             "repayment_income": income}
    result = _call(facts)
    assert not result.is_error
    assert result.structured_content["amounts"]["compulsory_repayment"] == expected
    with pytest.raises(ToolError, match="whole-dollar"):
        _call({**facts, "repayment_income": income + ".50"})


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
    result = _call({**facts, "scope_confirmed": True})
    assert not result.is_error
    assert result.structured_content["amounts"][amount] == expected


def test_an_input_the_tool_once_narrowed_away_now_reaches_the_engine():
    # Before every catalogue kind was served, the 2026-27 Medicare levy and
    # quarterly pays were refused by the tool's own schema.
    medicare = _call({"kind": "medicare_levy", "scope_confirmed": True, "year": "2026-27",
                      "taxable_income": "50000", "section_9a_adjustment_applies": False})
    assert medicare.structured_content["amounts"]["medicare_levy"] == "1000.00"
    payg = _call({"kind": "payg_withholding", "scope_confirmed": True, "year": "2026-27",
                  "earnings": "13000.00", "pay_period": "quarterly", "scale": 2})
    assert not payg.is_error


def test_medicare_levy_refuses_section_9a_and_requires_the_flag():
    facts = {"kind": "medicare_levy", "scope_confirmed": True, "year": "2025-26",
             "taxable_income": "40000"}
    with pytest.raises(ToolError, match="s 9A"):
        _call({**facts, "section_9a_adjustment_applies": True})
    with pytest.raises(ToolError, match="section_9a_adjustment_applies"):
        _call(facts)
