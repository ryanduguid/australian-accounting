"""Check engine discovery against the facade's actual resource and input contracts."""

import asyncio
import json

import pytest
from austaxcalc import calculations
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


@pytest.mark.parametrize("kind", calculations.SCOPES)
def test_advertised_required_fields_match_the_mcp_discriminator(kind):
    entry = _catalogue()[kind]
    facts = entry["example"]["facts"]
    model = type(TypeAdapter(TaxFacts).validate_python(facts))
    assert set(entry["required_inputs"]) == {
        name for name, field in model.model_fields.items() if field.is_required()
    }
