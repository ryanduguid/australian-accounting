"""Check engine discovery against the facade's actual resource and input contracts."""

import asyncio
import json

import pytest
from austaxcalc import calculations
from mcp.server.mcpserver.exceptions import ToolError
from pydantic import TypeAdapter

from aus_accounting_mcp import resources
from aus_accounting_mcp.adapters.tax import TaxFacts
from aus_accounting_mcp.server import mcp


def _catalogue():
    if not hasattr(calculations, "worksheet_catalogue"):
        pytest.skip("Rich discovery requires the unreleased worksheet engine")
    return calculations.worksheet_catalogue()


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


def test_pinned_older_engine_keeps_legacy_discovery(monkeypatch):
    monkeypatch.delattr(calculations, "worksheet_catalogue", raising=False)
    catalogue = resources.scope()["calculation_worksheets"]
    assert set(catalogue) == set(resources.SCOPES)
    for kind, entry in catalogue.items():
        assert entry == {"scope": resources.SCOPES[kind], "source": resources.SOURCES[kind],
                         "source_checked": resources.SOURCE_CHECKED}


def test_pinned_engine_without_a_newer_kind_still_runs_the_others(monkeypatch):
    # The published server pins an engine release that may predate a kind the
    # workspace engine has; only that kind may fail, and it must say why.
    monkeypatch.delattr(calculations, "payg_withholding", raising=False)
    gst = {"kind": "gst", "scope_confirmed": True, "year": "2025-26",
           "amount": "110.00", "gst_inclusive": True}
    payg = {"kind": "payg_withholding", "scope_confirmed": True, "year": "2026-27",
            "earnings": "1000.00", "pay_period": "weekly", "scale": 2}

    async def check():
        assert not (await mcp.call_tool("calculate_tax_worksheet", {"facts": gst})).is_error
        with pytest.raises(ToolError, match="newer australian-tax-calculators"):
            await mcp.call_tool("calculate_tax_worksheet", {"facts": payg})
    asyncio.run(check())


@pytest.mark.parametrize("kind", calculations.SCOPES)
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
