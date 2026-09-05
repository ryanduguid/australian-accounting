"""Pagination preserves the bundled catalogue and its provenance over MCP."""

import asyncio
import json
import sys

import pytest
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client

from aus_accounting_mcp.errors import InputError
from aus_accounting_mcp.server import list_ato_benchmark_industries


@pytest.mark.parametrize("search", [None, "shop", "no-such-synthetic-industry"])
def test_pages_reconstruct_the_filtered_catalogue(search):
    complete = list_ato_benchmark_industries(search=search, year="2023-24")
    collected = []
    offset = 0
    while True:
        page = list_ato_benchmark_industries(
            search=search, year="2023-24", limit=7, offset=offset
        )
        assert page["count"] == len(page["industries"]) <= 7
        assert page["total_count"] == complete["count"]
        assert page["offset"] == offset
        for key in ("engine", "engine_version", "benchmark_year", "source", "total_business_types"):
            assert page[key] == complete[key]
        collected.extend(page["industries"])
        if not page["has_more"]:
            assert page["next_offset"] is None
            break
        assert page["next_offset"] == offset + page["count"] > offset
        offset = page["next_offset"]
    assert collected == complete["industries"]


def test_past_end_and_exact_end_do_not_offer_another_page():
    complete = list_ato_benchmark_industries(year="2023-24")
    total = complete["count"]
    assert total > 0
    final = list_ato_benchmark_industries(year="2023-24", limit=1, offset=total - 1)
    assert final["industries"] == complete["industries"][-1:]
    assert final["has_more"] is False
    assert final["next_offset"] is None
    past_end = list_ato_benchmark_industries(year="2023-24", limit=1, offset=total + 1)
    assert past_end["industries"] == []
    assert past_end["count"] == 0
    assert past_end["has_more"] is False
    assert past_end["next_offset"] is None


def test_null_limit_retains_remaining_matches_and_maximum_limit_is_accepted():
    complete = list_ato_benchmark_industries(year="2023-24")
    remaining = list_ato_benchmark_industries(year="2023-24", limit=None, offset=1)
    assert remaining["industries"] == complete["industries"][1:]
    assert remaining["has_more"] is False
    assert remaining["next_offset"] is None
    largest = list_ato_benchmark_industries(year="2023-24", limit=100)
    assert largest["industries"] == complete["industries"][:100]


INVALID_PAGES = [
    ("limit", 0), ("limit", -1), ("limit", 101), ("limit", True),
    ("limit", 1.5), ("limit", "2"), ("offset", -1), ("offset", True),
    ("offset", 1.5), ("offset", "2"), ("offset", None),
]


@pytest.mark.parametrize("field,value", INVALID_PAGES)
def test_direct_calls_reject_invalid_pagination(field, value):
    with pytest.raises(InputError, match=field):
        list_ato_benchmark_industries(**{field: value})


async def _inspect_pagination_over_stdio():
    parameters = StdioServerParameters(command=sys.executable, args=["-m", "aus_accounting_mcp.cli"])
    async with stdio_client(parameters) as (reader, writer):
        async with ClientSession(reader, writer) as session:
            await session.initialize()
            tools = (await session.list_tools()).tools
            tool = next(tool for tool in tools if tool.name == "list_ato_benchmark_industries")
            assert "next_offset" in tool.output_schema["properties"]
            arguments = {"search": "shop", "year": "2023-24", "limit": 2}
            first = await session.call_tool(tool.name, arguments)
            assert not first.is_error
            payload = first.structured_content
            assert payload["count"] == 2
            assert payload["has_more"] is True
            assert json.loads(first.content[0].text) == payload
            second = await session.call_tool(
                tool.name, {**arguments, "offset": payload["next_offset"]}
            )
            assert not second.is_error
            assert second.structured_content["industries"] != payload["industries"]
            for field, value in INVALID_PAGES:
                error = await session.call_tool(tool.name, {field: value})
                assert error.is_error, (field, value)
                assert field in error.content[0].text
            recovered = await session.call_tool(tool.name, arguments)
            assert recovered.structured_content == payload


def test_stdio_pagination_and_errors_preserve_the_session():
    asyncio.run(_inspect_pagination_over_stdio())
