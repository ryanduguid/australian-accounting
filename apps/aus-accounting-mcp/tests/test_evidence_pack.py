"""MCP contracts using the established fabricated Payday Super evidence case."""
import asyncio
import hashlib
import json
import sys

import pytest
from jsonschema import Draft202012Validator
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from mcp.server.mcpserver.exceptions import ToolError

from aus_accounting_mcp.adapters import payday
from aus_accounting_mcp.errors import InputError
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


def test_compact_pack_preserves_files_over_stdio(tmp_path):
    async def exercise():
        parameters = StdioServerParameters(
            command=sys.executable, args=["-m", "aus_accounting_mcp.cli"], cwd=tmp_path,
        )
        async with stdio_client(parameters) as (reader, writer):
            async with ClientSession(reader, writer) as session:
                await session.initialize()
                arguments = {
                    "contributions": [{**ROW, "employee_id": f"SYN{i:03}"} for i in range(200)],
                    "as_at": "2026-08-20",
                }
                full = await session.call_tool(TOOL, arguments)
                compact = await session.call_tool(TOOL, {**arguments, "response_detail": "compact"})
                assert not full.is_error and not compact.is_error
                assert compact.structured_content == full.structured_content
                assert json.loads(full.content[0].text) == full.structured_content
                assert len(compact.model_dump_json()) < len(full.model_dump_json()) * 0.55
                payload = compact.structured_content
                assert payload["disclaimer"] in compact.content[0].text
                assert all(caveat in compact.content[0].text for caveat in payload["caveats"])
                assert "structuredContent.files" in compact.content[0].text
                queue = json.loads(payload["files"]["exceptions.json"])
                assert len(queue["exceptions"]) == 200
                assert {row["verdict"] for row in queue["exceptions"]} == {"AT_RISK"}
                report = payload["files"]["report.csv"].encode("utf-8")
                assert queue["report_sha256"] == hashlib.sha256(report).hexdigest()
    asyncio.run(exercise())
    assert list(tmp_path.iterdir()) == []


@pytest.mark.parametrize("arguments", [
    {"output_dir": "../outside"}, {"csv_path": "C:\\outside.csv"},
    {"fixture_id": "../../outside"}, {"as_at": "01/02/2027"},
    {"contributions": []}, {"contributions": [ROW] * 201},
    {"contributions": [{**ROW, "sg_amount": 120}]},
    {"contributions": [{**ROW, "first_to_fund": "false"}]},
    {"contributions": [{**ROW, "path": "../outside.csv"}]},
    {"response_detail": "summary"}, {"response_detail": True},
])
def test_paths_and_unestablished_facts_are_refused(arguments, tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    assert TOOL in {tool.name for tool in asyncio.run(mcp.list_tools())}
    with pytest.raises(ToolError):
        call(**arguments)
    assert list(tmp_path.iterdir()) == []


@pytest.mark.parametrize("detail", ["full", "compact"])
def test_old_published_engine_fails_closed_without_breaking_other_tools(monkeypatch, detail):
    monkeypatch.setitem(sys.modules, "paydaysuper.evidence_pack", None)
    with pytest.raises(ToolError, match="not available in the installed payday-super-checker"):
        call(response_detail=detail)
    result = asyncio.run(mcp.call_tool("review_payday_super_contributions", {
        "contributions": [ROW], "as_at": "2026-08-20",
    }))
    assert not result.is_error


@pytest.mark.parametrize("name", ["review_contribution", "review_contributions", "evidence_pack"])
def test_sentinel_dates_are_normal_input_errors_in_each_payday_adapter(name):
    row = {**ROW, "qe_day": "9999-12-31", "remitted": None}
    arguments = row if name == "review_contribution" else {
        "contributions": [payday.ContributionInput(**row)],
    }
    with pytest.raises(InputError, match="too far in the future"):
        getattr(payday, name)(**arguments, as_at="9999-12-31")


@pytest.mark.parametrize("detail", ["full", "compact"])
def test_related_rows_still_delegate_alignment_and_keep_unknowns(detail):
    rows = [
        {**ROW, "qe_day": "2027-07-01", "remitted": None, "received": "2027-07-05",
         "first_to_fund": True},
        {**ROW, "qe_day": "2027-07-08", "remitted": None, "received": "2027-07-20"},
    ]
    payload = call(
        contributions=rows, as_at="2027-08-01", response_detail=detail,
    ).structured_content
    assert payload["review_exit_code"] == 0
    assert "ITEM4_ALIGNED" in payload["files"]["report.csv"]
    assert json.loads(payload["files"]["exceptions.json"])["exceptions"] == []
    rows[1]["employee_id"] = "SYN002"
    payload = call(
        contributions=rows, as_at="2027-08-01", response_detail=detail,
    ).structured_content
    assert payload["review_exit_code"] == 2
    assert json.loads(payload["files"]["exceptions.json"])["exceptions"][0]["verdict"] == "LATE"


def test_an_mcp_pack_names_the_gic_table_the_assessment_used(tmp_path, monkeypatch):
    """The pack's manifest must come from the table the review ran against.

    The adapter previously built its pack without passing any rate table, so
    an MCP-generated pack carried an empty manifest while its notional
    earnings came from the GIC table. Reloading the table here instead would
    name a second read rather than the one that produced the figures.
    """
    monkeypatch.chdir(tmp_path)
    queue = json.loads(call().structured_content["files"]["exceptions.json"])
    entries = queue["manifest"]["rate_table_uris"]
    assert [entry["uri"] for entry in entries] == ["paydaysuper/data/gic_rates.json"]

    from paydaysuper.rates import DATA_DIR
    expected = hashlib.sha256(
        (DATA_DIR / "gic_rates.json").read_text(encoding="utf-8").encode("utf-8")
    ).hexdigest()
    assert entries[0]["sha256"] == expected


def test_the_pack_manifest_follows_the_table_the_review_loaded(tmp_path, monkeypatch):
    # Point the engine at an edited table: the pack has to report that table,
    # which only holds while the manifest comes from the assessed instance.
    from paydaysuper import rates as payday_rates

    source = json.loads((payday_rates.DATA_DIR / "gic_rates.json").read_text(encoding="utf-8"))
    source["quarters"][-1]["annual_pct"] = "11.44"
    folder = tmp_path / "rates"
    folder.mkdir()
    (folder / "gic_rates.json").write_text(json.dumps(source), encoding="utf-8")
    monkeypatch.setattr(payday_rates, "DATA_DIR", folder)
    monkeypatch.chdir(tmp_path)

    queue = json.loads(call().structured_content["files"]["exceptions.json"])
    entry = queue["manifest"]["rate_table_uris"][0]
    assert entry["uri"] == "file:gic_rates.json"
    expected = hashlib.sha256(
        (folder / "gic_rates.json").read_text(encoding="utf-8").encode("utf-8")
    ).hexdigest()
    assert entry["sha256"] == expected
