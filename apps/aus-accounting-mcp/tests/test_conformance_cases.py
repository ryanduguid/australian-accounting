"""Run the language-neutral conformance cases through the MCP tool-call path."""

import asyncio
import copy
import csv
import hashlib
import json
from decimal import Decimal
from pathlib import Path

import pytest

from aus_accounting_mcp import server
from repository_root import repository_root

CASES_FILE = Path(__file__).resolve().parents[1] / "conformance" / "cases.json"
CONTRACT = json.loads(CASES_FILE.read_text(encoding="utf-8"))
CASES = CONTRACT["cases"]
_MISSING = object()
BASES = {"authority", "self-pinned"}
CASE_KEYS = {"id", "tool", "basis", "source", "arguments", "expect", "expect_error_contains"}
PAYG_SAMPLE = "payg_withholding_sample_2026_27.csv"


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


def _problems(contract: dict, tools: set[str]) -> list[str]:
    """Every way the contract breaks its format, so a malformed case cannot run silently."""
    problems = []
    if contract.get("schema_version") != 2:
        problems.append("schema_version must be 2")
    if set(contract.get("basis", {})) != BASES:
        problems.append(f"basis must describe {sorted(BASES)}")
    ids = [case.get("id") for case in contract["cases"]]
    if len(ids) != len(set(ids)):
        problems.append("case ids must be unique")
    for case in contract["cases"]:
        name = case.get("id", "<no id>")
        if unknown := set(case) - CASE_KEYS:
            problems.append(f"{name}: unknown keys {sorted(unknown)}")
        if case.get("tool") not in tools:
            problems.append(f"{name}: tool is not registered")
        if case.get("basis") not in BASES:
            problems.append(f"{name}: basis must be one of {sorted(BASES)}")
        if not isinstance(case.get("source"), str) or not case["source"].strip():
            problems.append(f"{name}: source must name where the expected values come from")
        if ("expect" in case) == ("expect_error_contains" in case):
            problems.append(f"{name}: needs exactly one of expect and expect_error_contains")
        if not all(pointer.startswith("/") for pointer in case.get("expect", {})):
            problems.append(f"{name}: every expect key must be a JSON Pointer")
    return problems


TOOLS = {tool.name for tool in asyncio.run(server.mcp.list_tools())}


def test_cases_are_well_formed() -> None:
    assert _problems(CONTRACT, TOOLS) == []


@pytest.mark.parametrize(
    "change,problem",
    [
        (lambda case: case.pop("basis"), "basis must be one of"),
        (lambda case: case.update(basis="authoritative"), "basis must be one of"),
        (lambda case: case.pop("source"), "source must name"),
        (lambda case: case.update(source=" "), "source must name"),
        (lambda case: case.update(bases="authority"), "unknown keys"),
    ],
    ids=["no-basis", "unknown-basis", "no-source", "blank-source", "misspelt-key"],
)
def test_a_case_without_a_valid_basis_or_source_is_rejected(change, problem) -> None:
    contract = copy.deepcopy(CONTRACT)
    change(contract["cases"][0])

    assert any(problem in found for found in _problems(contract, TOOLS))


def test_authority_cases_reproduce_the_ato_sample() -> None:
    """An authority case's expected value is the ATO sample's, not this server's.

    The sample is the ATO's Schedule 1 withholding amounts data published on 17 June
    2026, held by australian-tax-calculators as its test oracle. Each authority case
    must name it and expect exactly the amount it gives for the case's period,
    earnings and scale, so a case cannot drift to whatever the engine returns.
    """
    sample = (repository_root() / "packages" / "australian-tax-calculators" / "tests"
              / "data" / PAYG_SAMPLE)
    digest = hashlib.sha256(sample.read_bytes()).hexdigest()
    with sample.open(encoding="utf-8", newline="") as handle:
        rows = {(row["period"], row["earnings"]): row for row in csv.DictReader(handle)}
    authority = [case for case in CASES if case["basis"] == "authority"]
    covered = set()
    for case in authority:
        facts = case["arguments"]["facts"]
        assert case["tool"] == "calculate_tax_worksheet"
        assert (facts["kind"], facts["year"]) == ("payg_withholding", "2026-27")
        assert PAYG_SAMPLE in case["source"] and "17 June 2026" in case["source"]
        # The held sample must still be the bytes the case was written from.
        assert f"(SHA-256 {digest})" in case["source"], case["id"]
        row = rows[(facts["pay_period"], facts["earnings"])]
        expected = f"{Decimal(row[f'scale{facts['scale']}']):.2f}"
        assert case["expect"] == {"/amounts/withholding": expected}, case["id"]
        covered.add((facts["pay_period"], facts["scale"]))
    # One case for each pay period and scale the sample publishes.
    assert covered == {(period, scale) for period in ("weekly", "fortnightly", "monthly")
                       for scale in (1, 2, 3, 5, 6)}


def test_the_resolver_reports_absent_paths() -> None:
    document = {"a": [{"b~/c": 1}]}
    assert _resolve(document, "/a/0/b~0~1c") == 1
    assert _resolve(document, "/a/1") is _MISSING
    assert _resolve(document, "/x") is _MISSING
