"""Reproducible, fabricated MCP proof for aus-accounting-mcp."""

from __future__ import annotations

import asyncio
import json
from typing import Any

from mcp.types import CallToolResult

from aus_accounting_mcp.server import mcp

DEMO_COMMAND = "uv run --locked aus-accounting-mcp-demo"


async def _call_structured(
    name: str,
    arguments: dict[str, Any],
) -> dict[str, Any]:
    result = await mcp.call_tool(name, arguments)
    if not isinstance(result, CallToolResult):
        raise RuntimeError(f"{name} did not return a complete MCP tool result")
    payload = result.structured_content
    if not isinstance(payload, dict):
        raise RuntimeError(f"{name} did not return structured MCP content")
    json.dumps(payload, allow_nan=False)
    return payload


async def _build_demo_payload() -> dict[str, Any]:
    success_arguments = {
        "form_type": "BAS",
        "entity_name": "Example Firm Pty Ltd",
        "revenue_or_sales": "110000.00",
    }
    div7a_arguments = {
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
    return {
        "calls": [
            {
                "tool": "generate_synthetic_sbr_fixture",
                "arguments": success_arguments,
                "result": await _call_structured(
                    "generate_synthetic_sbr_fixture",
                    success_arguments,
                ),
            },
            {
                "tool": "review_div7a_loan",
                "arguments": div7a_arguments,
                "result": await _call_structured(
                    "review_div7a_loan",
                    div7a_arguments,
                ),
            },
        ]
    }


def demo_payload() -> dict[str, Any]:
    return asyncio.run(_build_demo_payload())


def render_demo_json(payload: dict[str, Any] | None = None) -> str:
    if payload is None:
        payload = demo_payload()
    return json.dumps(payload, indent=2, sort_keys=True, allow_nan=False)


def render_transcript(payload: dict[str, Any] | None = None) -> str:
    return f"$ {DEMO_COMMAND}\n{render_demo_json(payload)}\n"


def main() -> None:
    print(render_transcript(), end="")


if __name__ == "__main__":
    main()
