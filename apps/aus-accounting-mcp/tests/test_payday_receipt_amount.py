"""The pinned checker must preserve receipt-amount uncertainty through MCP."""

from __future__ import annotations

import asyncio

from aus_accounting_mcp.server import calc_payday_super_deadline, mcp

FACTS = {"qe_day": "2026-08-06", "sg_amount": "800.00", "as_at": "2026-08-21"}


def test_single_contribution_with_a_bare_receipt_is_unknown_not_on_time() -> None:
    bare = calc_payday_super_deadline(**FACTS, received="2026-08-10")
    assert bare["ok"] is True
    assert bare["result"]["verdict"] == "UNKNOWN"
    assert bare["result"]["horizon_verdicts"] == ["UNPAID", "ON_TIME"]
    assert any("carries no amount" in w for w in bare["result"]["caveats"])
    assert bare["result"]["experimental_sgc_high"] is None

    stated = calc_payday_super_deadline(**FACTS, received="2026-08-10", matched_amount="800.00")
    assert stated["result"]["verdict"] == "ON_TIME"


def test_group_review_keeps_the_bare_receipt_unknown() -> None:
    payload = asyncio.run(mcp.call_tool("review_payday_super_contributions", {
        "contributions": [{
            "employee_id": "SYN001", "qe_day": "2026-08-06", "sg_amount": "800.00",
            "received": "2026-08-10", "first_to_fund": False, "out_of_cycle": False,
            "db_interest": False,
        }],
        "as_at": "2026-08-21",
    })).structured_content
    result = payload["results"][0]
    assert result["verdict"] == "UNKNOWN"
    assert any("carries no amount" in w for w in result["caveats"])


def test_tool_descriptions_tell_the_caller_a_receipt_needs_an_amount() -> None:
    """The instruction travels with the schema whatever engine is pinned."""
    from aus_accounting_mcp.adapters.payday import ContributionInput

    fields = ContributionInput.model_fields
    assert "matched_amount" in fields["received"].description
    assert "UNKNOWN" in fields["received"].description
    assert "receipt date alone evidences no amount" in fields["matched_amount"].description
    assert "0.1.6" in fields["received"].description


def test_single_contribution_schema_does_not_infer_full_receipt() -> None:
    tools = asyncio.run(mcp.list_tools())
    tool = next(tool for tool in tools if tool.name == "calc_payday_super_deadline")
    description = tool.input_schema["properties"]["matched_amount"]["description"]
    assert "UNKNOWN" in description
    assert "without reducing the shortfall" in description
    assert "received means full receipt" not in description
