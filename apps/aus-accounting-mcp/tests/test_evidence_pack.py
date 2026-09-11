"""MCP contracts using the established fabricated Payday Super evidence case."""
import asyncio
import hashlib
import json
import sys

import pytest
from jsonschema import Draft202012Validator
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.server import mcp

TOOL = "build_payday_super_evidence_pack"
# Same facts as evaluation/payday_super_evidence/fixtures/timely_remittance_no_receipt.csv.
ROW = dict(employee_id="SYN001", qe_day="2026-08-06", sg_amount="120.00",
           remitted="2026-08-14", received=None, first_to_fund=False,
           out_of_cycle=False, db_interest=False)


def call(**changes):
    return asyncio.run(mcp.call_tool(TOOL, {
        "contributions": [ROW], "as_at": "2026-08-20", **changes,
    }))


def test_pack_is_structured_private_and_does_not_write(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    result = call()
    assert not result.is_error
    payload = result.structured_content
    assert payload["ok"] is True
    assert payload["review_exit_code"] == 2
    files = payload["files"]
    assert set(files) == {
        "report.csv", "practitioner-review.md", "exceptions.json", "decision-log.md",
    }
    digest = hashlib.sha256(files["report.csv"].encode("utf-8")).hexdigest()
    assert digest in files["practitioner-review.md"]
    queue = json.loads(files["exceptions.json"])
    assert queue["report_sha256"] == digest
    assert queue["exceptions"][0]["verdict"] == "AT_RISK"
    assert queue["exceptions"][0]["source_row"] == 1
    assert "SYN001" not in json.dumps(payload)
    assert "employee_id" not in files["report.csv"]
    assert "## Decisions" in files["decision-log.md"]
    assert list(tmp_path.iterdir()) == []
    tools = {tool.name: tool for tool in asyncio.run(mcp.list_tools())}
    Draft202012Validator(tools[TOOL].output_schema).validate(payload)
    assert tools[TOOL].annotations.read_only_hint is True
    assert tools[TOOL].input_schema["additionalProperties"] is False


@pytest.mark.parametrize("arguments", [
    {"output_dir": "../outside"}, {"csv_path": "C:\\outside.csv"},
    {"fixture_id": "../../outside"}, {"as_at": "01/02/2027"},
    {"contributions": []}, {"contributions": [ROW] * 201},
    {"contributions": [{**ROW, "sg_amount": 120}]},
    {"contributions": [{**ROW, "first_to_fund": "false"}]},
    {"contributions": [{**ROW, "path": "../outside.csv"}]},
])
def test_paths_and_unestablished_facts_are_refused(arguments, tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    assert TOOL in {tool.name for tool in asyncio.run(mcp.list_tools())}
    with pytest.raises(ToolError):
        call(**arguments)
    assert list(tmp_path.iterdir()) == []


def test_old_published_engine_fails_closed_without_breaking_other_tools(monkeypatch):
    monkeypatch.setitem(sys.modules, "paydaysuper.evidence_pack", None)
    with pytest.raises(ToolError, match="not available in the installed payday-super-checker"):
        call()
    result = asyncio.run(mcp.call_tool("review_payday_super_contributions", {
        "contributions": [ROW], "as_at": "2026-08-20",
    }))
    assert not result.is_error


def test_related_rows_still_delegate_alignment_and_keep_unknowns():
    rows = [
        {**ROW, "qe_day": "2027-07-01", "remitted": None, "received": "2027-07-05",
         "first_to_fund": True},
        {**ROW, "qe_day": "2027-07-08", "remitted": None, "received": "2027-07-20"},
    ]
    payload = call(contributions=rows, as_at="2027-08-01").structured_content
    assert payload["review_exit_code"] == 0
    assert "ITEM4_ALIGNED" in payload["files"]["report.csv"]
    assert json.loads(payload["files"]["exceptions.json"])["exceptions"] == []
    rows[1]["employee_id"] = "SYN002"
    payload = call(contributions=rows, as_at="2027-08-01").structured_content
    assert payload["review_exit_code"] == 2
    assert json.loads(payload["files"]["exceptions.json"])["exceptions"][0]["verdict"] == "LATE"
