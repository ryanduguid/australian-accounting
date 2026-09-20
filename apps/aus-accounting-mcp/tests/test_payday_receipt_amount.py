"""A receipt date without an amount must not reach ON_TIME through the MCP.

The engine rule lives in payday-super-checker. Against a published engine
that predates it these tests skip, because the MCP pins engines to their
published distributions and cannot promise behaviour the pinned engine does
not have. The feature is detected, not the version, so a workspace build
against the tree exercises the rule and a standalone build against PyPI
reports it as not yet available.
"""

from __future__ import annotations

import asyncio

import paydaysuper.deadlines
import pytest

from aus_accounting_mcp.server import calc_payday_super_deadline, mcp

requires_receipt_amount_rule = pytest.mark.skipif(
    not hasattr(paydaysuper.deadlines, "receipt_amount_evidenced"),
    reason="pinned payday-super-checker predates the receipt-amount rule",
)

FACTS = {"qe_day": "2026-08-06", "sg_amount": "800.00", "as_at": "2026-08-21"}


@requires_receipt_amount_rule
def test_single_contribution_with_a_bare_receipt_is_unknown_not_on_time() -> None:
    bare = calc_payday_super_deadline(**FACTS, received="2026-08-10")
    assert bare["ok"] is True
    assert bare["result"]["verdict"] == "UNKNOWN"
    assert bare["result"]["horizon_verdicts"] == ["UNPAID", "ON_TIME"]
    assert any("carries no amount" in w for w in bare["result"]["caveats"])
    assert bare["result"]["experimental_sgc_high"] is None

    stated = calc_payday_super_deadline(**FACTS, received="2026-08-10", matched_amount="800.00")
    assert stated["result"]["verdict"] == "ON_TIME"


@requires_receipt_amount_rule
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
