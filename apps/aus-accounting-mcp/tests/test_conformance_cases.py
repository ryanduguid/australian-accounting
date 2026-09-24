"""Run the language-neutral conformance cases through the MCP tool-call path."""

import asyncio
import json
from pathlib import Path

import pytest

from aus_accounting_mcp import server

CASES_FILE = Path(__file__).resolve().parents[1] / "conformance" / "cases.json"
CONTRACT = json.loads(CASES_FILE.read_text(encoding="utf-8"))
CASES = CONTRACT["cases"]
_MISSING = object()


def _resolve(document: object, pointer: str) -> object:
    """Resolve an RFC 6901 JSON Pointer, returning _MISSING for an absent path."""
    node = document
    for token in pointer.split("/")[1:]:
        key = token.replace("~1", "/").replace("~0", "~")
        if isinstance(node, list) and key.isdigit() and int(key) < len(node):
            node = node[int(key)]
        elif isinstance(node, dict) and key in node:
            node = node[key]
        else:
            return _MISSING
    return node


def _call(case: dict) -> object:
    result = asyncio.run(server.mcp.call_tool(case["tool"], case["arguments"]))
    assert not result.is_error
    return result.structured_content


@pytest.mark.parametrize("case", CASES, ids=[case["id"] for case in CASES])
def test_conformance_case(case: dict) -> None:
    if "expect_error_contains" in case:
        with pytest.raises(Exception, match=case["expect_error_contains"]):
            _call(case)
        return
    payload = _call(case)
    for pointer, expected in case["expect"].items():
        actual = _resolve(payload, pointer)
        assert actual is not _MISSING, f"{case['id']}: {pointer} is absent"
        assert actual == expected, f"{case['id']}: {pointer} is {actual!r}"


def test_cases_are_well_formed() -> None:
    assert CONTRACT["schema_version"] == 1
    tools = {tool.name for tool in asyncio.run(server.mcp.list_tools())}
    ids = [case["id"] for case in CASES]
    assert len(ids) == len(set(ids))
    for case in CASES:
        assert case["tool"] in tools
        assert ("expect" in case) != ("expect_error_contains" in case)
        assert all(pointer.startswith("/") for pointer in case.get("expect", {}))


def test_the_resolver_reports_absent_paths() -> None:
    document = {"a": [{"b~/c": 1}]}
    assert _resolve(document, "/a/0/b~0~1c") == 1
    assert _resolve(document, "/a/1") is _MISSING
    assert _resolve(document, "/x") is _MISSING
