from __future__ import annotations

import asyncio
import json
from pathlib import Path
import subprocess
import sys

from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from mcp.types import InputRequiredResult
import pytest

from aus_accounting_mcp import demo


def test_demo_payload_uses_real_registered_mcp_tools() -> None:
    payload = demo.demo_payload()
    calls = {call["tool"]: call for call in payload["calls"]}

    assert set(calls) == {"generate_synthetic_sbr_fixture", "review_div7a_loan"}
    fixture = calls["generate_synthetic_sbr_fixture"]["result"]
    assert fixture["synthetic"] is True
    assert fixture["not_a_lodgment"] is True

    div7a = calls["review_div7a_loan"]["result"]
    assert div7a["engine"] == "div7a-loan-review"
    assert div7a["gate"]["verdict"] == "COMPLYING"
    assert div7a["minimum_yearly_repayment"]["verdict"] == "MYR_MET"
    assert div7a["minimum_yearly_repayment"]["myr_required"] == "108770.00"


def test_demo_json_is_sorted_and_finite() -> None:
    payload = {"calls": [{"tool": "example", "result": {"ok": True}}]}
    assert demo.render_demo_json(payload) == json.dumps(
        payload,
        indent=2,
        sort_keys=True,
        allow_nan=False,
    )


def test_demo_transcript_runs_the_locked_checkout_entry_point() -> None:
    transcript = demo.render_transcript({"calls": []})

    assert transcript.startswith("$ uv run --locked aus-accounting-mcp-demo\n")


def test_checked_quick_proof_transcript_is_current_real_demo() -> None:
    root = Path(__file__).resolve().parents[1]
    transcript = root / "docs" / "quick-proof.txt"

    assert transcript.read_text(encoding="utf-8") == demo.render_transcript()


async def _stdio_smoke() -> None:
    parameters = StdioServerParameters(
        command=sys.executable,
        args=["-m", "aus_accounting_mcp.cli"],
    )
    arguments = {
        "year_of_income": "2026-27",
        "year_loan_made": "2025-26",
        "written_agreement": True,
        "terms_in_place_before_lodgment_day": True,
        "maximum_term_years": "7",
        "secured_by_registered_mortgage_over_real_property": False,
        "interest_rate_for_years_after_year_loan_made": "0.0837",
        "amalgamated_loan_unpaid_at_end_of_previous_year": "100000.00",
        "remaining_term_years": "1",
        "payments_applied_during_the_year": "108770.00",
        "loan_id": "synthetic-loan-1",
    }

    async with stdio_client(parameters) as (
        read_stream,
        write_stream,
    ):
        async with ClientSession(read_stream, write_stream) as session:
            await session.initialize()
            result = await session.call_tool("review_div7a_loan", arguments)

    assert result.structured_content["gate"]["verdict"] == "COMPLYING"
    assert result.structured_content["minimum_yearly_repayment"]["verdict"] == "MYR_MET"


def test_demo_payload_rejects_input_required_mcp_results(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    async def input_required(*_args: object, **_kwargs: object) -> InputRequiredResult:
        return InputRequiredResult(request_state="retry-demo-call")

    monkeypatch.setattr(demo.mcp, "call_tool", input_required)

    with pytest.raises(RuntimeError, match="complete MCP tool result"):
        demo.demo_payload()


def test_stdio_contract_and_demo_entry_point_are_separate() -> None:
    asyncio.run(_stdio_smoke())
    completed = subprocess.run(
        [sys.executable, "-m", "aus_accounting_mcp.demo"],
        check=True,
        capture_output=True,
        text=True,
    )
    assert completed.stdout == demo.render_transcript()
