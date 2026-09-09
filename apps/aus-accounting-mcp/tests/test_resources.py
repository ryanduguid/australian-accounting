"""Check the prompts and resources a host receives, and what they may state."""

from __future__ import annotations

import asyncio
import importlib.metadata
import json
import re
import sys
from datetime import date
from decimal import Decimal
from pathlib import Path

import pytest
from atobenchmark.dataset import available_years
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from paydaysuper.calendar import BusinessCalendar
from paydaysuper.rates import GicQuarter, GicTable

from aus_accounting_mcp import resources
from aus_accounting_mcp.server import DIV7A_SCOPE_REFUSAL, mcp

ROOT = Path(__file__).resolve().parents[1]

RESOURCE_URIS = {
    "aus-accounting://scope",
    "aus-accounting://disclaimer",
    "aus-accounting://div7a-scope",
    "aus-accounting://benchmark-dataset-years",
    "aus-accounting://component-versions",
    "aus-accounting://payday-coverage",
}
PROMPT_NAMES = {
    "compare_ato_benchmarks",
    "review_payday_super_contribution",
    "review_div7a_loan_terms",
}


def _normalise(text: str) -> str:
    return re.sub(r"\s+", " ", text).strip()


def _read(uri: str) -> str:
    contents = list(asyncio.run(mcp.read_resource(uri)))
    assert len(contents) == 1
    return contents[0].content


def test_every_boundary_paragraph_is_also_in_the_public_disclaimer() -> None:
    # The resource is the copy a model reads and DISCLAIMER.md is the copy a
    # person reads. They have to say the same thing, so each paragraph the
    # resource serves is a paragraph of the file, compared without line
    # wrapping. Adding to the resource alone fails here.
    published = _normalise((ROOT / "DISCLAIMER.md").read_text(encoding="utf-8"))

    assert resources.BOUNDARY_PARAGRAPHS
    for paragraph in resources.BOUNDARY_PARAGRAPHS:
        assert _normalise(paragraph) in published, paragraph


def test_disclaimer_resource_carries_the_facade_and_every_engine_boundary() -> None:
    served = _read("aus-accounting://disclaimer")

    for paragraph in resources.BOUNDARY_PARAGRAPHS:
        assert _normalise(paragraph) in _normalise(served)
    # Each engine states its own limits, and the facade does not restate them.
    for engine in ("ato-benchmark-compare", "payday-super-checker", "div7a-loan-review"):
        assert engine in served
    assert "not tax, legal, accounting" in served
    assert "human review" in served


def test_div7a_scope_resource_is_the_refusal_the_tool_returns() -> None:
    # A host that reads the scope resource and a caller that gets refused must
    # be told the same thing, so this serves the refusal text itself rather
    # than a second description of it that could drift.
    served = _read("aus-accounting://div7a-scope")

    assert served == DIV7A_SCOPE_REFUSAL
    for refused in (
        "s 109R",
        "unpaid present entitlements",
        "distributable surplus",
        "interposed entities",
        "debt forgiveness",
        "Commissioner's discretion",
    ):
        assert refused in served


def test_scope_distinguishes_supported_reviews_from_unsupported_tax_calculations():
    served = json.loads(_read("aus-accounting://scope"))
    registered = {tool.name for tool in asyncio.run(mcp.list_tools())}
    assert set(served["review_tools"]) == registered - {"generate_synthetic_sbr_fixture"}
    assert served["synthetic_only_tools"] == ["generate_synthetic_sbr_fixture"]
    assert set(served["unsupported_calculations"]) == {
        "gst_bas", "income_tax", "cgt", "fbt", "depreciation", "trusts_partnerships",
        "smsf", "super_contribution_caps", "sg_entitlement", "payroll_tax",
    }
    assert "s 18C(2) item 4" in served["payday_limitations"]
    assert "not" in served["reference_policy"]


def test_benchmark_years_resource_reports_the_shipped_years_and_provenance() -> None:
    served = json.loads(_read("aus-accounting://benchmark-dataset-years"))
    years = available_years()

    assert [entry["benchmark_year"] for entry in served["datasets"]] == years
    assert served["latest_benchmark_year"] == years[-1]
    # Bundled data is stated as bundled, so a model cannot read this as a live
    # ATO lookup and cannot treat an absent year as merely unfetched.
    assert served["bundled_data"] is True
    assert served["live_lookup"] is False
    assert served["engine_version"] == importlib.metadata.version("ato-benchmark-compare")
    for entry in served["datasets"]:
        assert entry["business_types"] > 0
        # The provenance is the engine's own, not a restatement.
        assert entry["source"]["publisher"] == "Australian Taxation Office"
        assert entry["source"]["retrieved"]
        assert entry["source"]["sha256"]


def test_component_versions_resource_matches_the_installed_distributions() -> None:
    served = json.loads(_read("aus-accounting://component-versions"))

    assert served["server"]["distribution"] == "aus-accounting-mcp"
    assert served["server"]["version"] == importlib.metadata.version("aus-accounting-mcp")
    assert served["server"]["transport"] == "stdio"
    for engine in served["engines"]:
        assert engine["version"] == importlib.metadata.version(engine["distribution"])


def test_payday_coverage_resource_uses_the_loaded_engine_tables(monkeypatch) -> None:
    calendar = BusinessCalendar([], date(2030, 1, 1), date(2030, 12, 31))
    rates = GicTable([
        GicQuarter(date(2030, 1, 1), date(2030, 3, 31), Decimal("10"), "2030-01-02")
    ])
    # Table loading is the boundary: fabricated engine objects make a hardcoded
    # date or a date taken from package metadata fail this consumer check.
    monkeypatch.setattr(resources, "load_calendar", lambda: calendar)
    monkeypatch.setattr(resources, "load_gic", lambda: rates)
    served = json.loads(_read("aus-accounting://payday-coverage"))

    assert served["engine"] == "payday-super-checker"
    assert served["engine_version"] == importlib.metadata.version("payday-super-checker")
    assert served["bundled_data"] is True
    assert served["live_lookup"] is False
    assert served["calendar"] == {
        "verified_from": "2030-01-01", "verified_until": "2030-12-31",
        "coverage_until": "2030-12-31",
    }
    assert served["gic"]["known_until"] == "2030-03-31"
    assert "2030-01-02" in served["gic"]["provenance"]
    assert "2030-03-31" in served["gic"]["provenance"]
    assert "last known rate" in served["gic"]["beyond_coverage"]
    assert served["disclaimer"]


def test_component_versions_resource_agrees_with_the_compatibility_record() -> None:
    # The resource reads installed metadata so it is right from a wheel, where
    # compatibility.json is not shipped. The record is what the release says was
    # published. In this checkout the two describe the same thing, so a pin bump
    # that updates one and not the other fails here.
    record = json.loads((ROOT / "compatibility.json").read_text(encoding="utf-8"))
    served = json.loads(_read("aus-accounting://component-versions"))

    assert served["server"]["version"] == record["server"]["version"]
    assert served["server"]["registry_identity"] == record["server"]["registry_identity"]
    assert served["server"]["repository"] == record["server"]["repository"]
    assert {engine["distribution"]: engine["version"] for engine in served["engines"]} == {
        engine["distribution"]: engine["version"] for engine in record["engines"]
    }


def test_a_missing_distribution_reports_null_rather_than_a_guess(monkeypatch) -> None:
    def absent(distribution: str) -> str:
        raise importlib.metadata.PackageNotFoundError(distribution)

    monkeypatch.setattr(resources, "version", absent)
    served = resources.component_versions()

    assert served["server"]["version"] is None
    assert [engine["version"] for engine in served["engines"]] == [
        None for _ in resources.ENGINE_DISTRIBUTIONS
    ]


@pytest.mark.parametrize("name", sorted(PROMPT_NAMES))
def test_each_prompt_renders_a_workflow_without_inventing_facts(name) -> None:
    rendered = asyncio.run(mcp.get_prompt(name, {}))
    text = "\n".join(
        message.content.text for message in rendered.messages if message.content.type == "text"
    )

    assert rendered.messages
    assert all(message.role == "user" for message in rendered.messages)
    # With no argument supplied the prompt asks for the fact rather than
    # choosing one, which is the whole point of these three workflows.
    assert "Ask me" in text or "Select the industry" in text
    # Every prompt names a tool the server actually registers.
    tools = {tool.name for tool in asyncio.run(mcp.list_tools())}
    assert any(tool in text for tool in tools)


def test_prompt_arguments_are_optional_so_a_host_can_show_them_unfilled() -> None:
    prompts = {prompt.name: prompt for prompt in asyncio.run(mcp.list_prompts())}

    assert set(prompts) == PROMPT_NAMES
    for prompt in prompts.values():
        assert prompt.title
        assert prompt.description
        for argument in prompt.arguments or ():
            assert not argument.required, f"{prompt.name}.{argument.name}"


def test_supplied_prompt_arguments_reach_the_rendered_text() -> None:
    industry = asyncio.run(
        mcp.get_prompt("compare_ato_benchmarks", {"industry": "Bakeries and hot bread shops"})
    )
    year = asyncio.run(
        mcp.get_prompt("review_div7a_loan_terms", {"year_of_income": "2026-27"})
    )
    as_at = asyncio.run(
        mcp.get_prompt("review_payday_super_contribution", {"as_at": "2027-08-01"})
    )

    assert "Bakeries and hot bread shops" in industry.messages[0].content.text
    assert "2026-27" in year.messages[0].content.text
    assert "2027-08-01" in as_at.messages[0].content.text
    # The unfilled variant must not silently become an assumed date.
    assert "do not assume one" in (
        asyncio.run(mcp.get_prompt("review_payday_super_contribution", {}))
        .messages[0]
        .content.text
    )


def test_prompts_preserve_the_unknown_and_refusal_language() -> None:
    texts = {
        name: "\n".join(
            message.content.text for message in asyncio.run(mcp.get_prompt(name, {})).messages
        )
        for name in PROMPT_NAMES
    }

    assert "not_supplied" in texts["compare_ato_benchmarks"]
    assert "Do not treat missing as zero" in texts["compare_ato_benchmarks"]
    # The two inputs without which the tool cannot run at all. Naming them stops
    # a host following this prompt into a tool error, and the prompt has to ask
    # for them rather than supplying a zero the operator never established.
    assert "at least one expense bucket" in texts["compare_ato_benchmarks"]
    assert "turnover" in texts["compare_ato_benchmarks"]
    assert "rather than supplying a" in texts["compare_ato_benchmarks"]
    assert "AT_RISK" in texts["review_payday_super_contribution"]
    assert "Do not invent an SGC charge" in texts["review_payday_super_contribution"]
    assert "UNKNOWN" in texts["review_div7a_loan_terms"]
    assert "refuse_div7a" in texts["review_div7a_loan_terms"]


async def _inspect_stdio() -> None:
    parameters = StdioServerParameters(
        command=sys.executable, args=["-m", "aus_accounting_mcp.cli"]
    )
    async with stdio_client(parameters) as (reader, writer):
        async with ClientSession(reader, writer) as session:
            initialized = await session.initialize()
            capabilities = initialized.capabilities
            assert capabilities.prompts is not None
            assert capabilities.resources is not None

            listed = {str(resource.uri) for resource in (await session.list_resources()).resources}
            assert listed == RESOURCE_URIS
            prompts = (await session.list_prompts()).prompts
            assert {prompt.name for prompt in prompts} == PROMPT_NAMES

            # Every resource is named in the instructions, so a host that only
            # reads those still knows they exist.
            for uri in RESOURCE_URIS:
                assert uri in initialized.instructions

            for uri in sorted(RESOURCE_URIS):
                read = await session.read_resource(uri)
                assert read.contents, uri
                body = read.contents[0]
                assert body.text
                if body.mime_type == "application/json":
                    json.loads(body.text)

            rendered = await session.get_prompt(
                "compare_ato_benchmarks", {"industry": "Bakeries and hot bread shops"}
            )
            assert rendered.messages[0].content.text


def test_stdio_clients_can_list_and_read_every_prompt_and_resource() -> None:
    asyncio.run(_inspect_stdio())


def test_the_benchmark_prompt_names_every_input_the_tool_cannot_run_without():
    # Reproduces what a host following this prompt would otherwise hit: the
    # adapter refuses a call carrying income figures alone, so a prompt that
    # does not name the prerequisites walks a model into a tool error.
    from aus_accounting_mcp.server import get_ato_benchmarks

    with pytest.raises(ValueError, match="no expense figures were supplied"):
        get_ato_benchmarks(
            industry="Bakeries and hot bread shops", turnover="850000.00", other_income="0"
        )

    rendered = asyncio.run(mcp.get_prompt("compare_ato_benchmarks", {})).messages[0].content.text
    required = mcp_tool_input_requirements("get_ato_benchmarks")

    for argument in required:
        assert argument in rendered, argument
    assert "at least one expense bucket" in rendered


def mcp_tool_input_requirements(name: str) -> list[str]:
    tool = next(tool for tool in asyncio.run(mcp.list_tools()) if tool.name == name)
    return list(tool.input_schema.get("required") or [])
