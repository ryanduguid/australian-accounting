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

EXPECTED_PAYLOAD = {
    "calls": [
        {
            "arguments": {
                "entity_name": "Example Firm Pty Ltd",
                "form_type": "BAS",
                "revenue_or_sales": "110000.00",
            },
            "result": {
                "entity": {
                    "abn": "11 222 333 444",
                    "name": "Example Firm Pty Ltd",
                    "quarter_ended": "2025-03-31",
                },
                "form_type": "BAS_AU_ACTIVITY_STATEMENT",
                "gst_labels": {
                    "1A_gst_on_sales": "10000.00",
                    "1B_gst_on_purchases": "5000.00",
                    "G10_capital_purchases": "11000.00",
                    "G11_non_capital_purchases": "44000.00",
                    "G1_total_sales": "110000.00",
                    "net_gst": "5000.00",
                },
                "not_a_lodgment": True,
                "payg_withholding_labels": {
                    "W1_total_salary_wages": "150000.00",
                    "W2_amounts_withheld": "37500.00",
                },
                "summary": {"total_payable_to_ato": "42500.00"},
                "synthetic": True,
            },
            "tool": "generate_synthetic_sbr_fixture",
        },
        {
            "arguments": {
                "borrower_name": "Example Borrower",
                "lender_entity_name": "Example Company Pty Ltd",
                "loan_principal": "50000.00",
            },
            "result": {
                "available": False,
                "code": "ERR_POLICY_DIV7A_REFUSED",
                "ok": False,
                "reason": (
                    "Division 7A MYR, benchmark interest and franking-offset "
                    "journals are not backed by a reviewed computational engine "
                    "in this server. The previous MCP-local simulator has been "
                    "removed so agents cannot treat it as statutory output. "
                    "Wired engines: payday-super-checker and "
                    "ato-benchmark-compare."
                ),
                "reviewed_engine": False,
            },
            "tool": "refuse_div7a",
        },
    ]
}


def test_demo_payload_uses_real_registered_mcp_tools() -> None:
    assert demo.demo_payload() == EXPECTED_PAYLOAD


def test_demo_json_is_sorted_and_finite() -> None:
    assert demo.render_demo_json(EXPECTED_PAYLOAD) == json.dumps(
        EXPECTED_PAYLOAD,
        indent=2,
        sort_keys=True,
        allow_nan=False,
    )


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
        "borrower_name": "Example Borrower",
        "lender_entity_name": "Example Company Pty Ltd",
        "loan_principal": "50000.00",
    }

    async with stdio_client(parameters) as (
        read_stream,
        write_stream,
    ):
        async with ClientSession(read_stream, write_stream) as session:
            await session.initialize()
            result = await session.call_tool("refuse_div7a", arguments)

    assert result.structured_content["code"] == "ERR_POLICY_DIV7A_REFUSED"


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
