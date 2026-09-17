"""An optional MCP surface, separately enabled, delegating to the same adapter.

This is not the Aus Accounting MCP. That server stays offline: it reads bundled
data and the folders an operator configures, and no tool in it calls a service.
Nothing here is imported by it, and there is no path from a failed local
calculation to a remote one.

Two switches, not one. `mcp` must be installed (the `mcp` extra), and the
adapter's own network enablement still applies, so starting this server with
remote access off gives an assistant tools that answer `REFUSED_TO_SEND`.

Every tool returns the adapter's own `Outcome` shape. A refusal stays a
refusal; there is no tool that returns a bare number.
"""

from __future__ import annotations

import json
from typing import Any

from . import __version__
from .client import LodgeitClient
from .config import AdapterConfig, from_environment
from .contract import load as load_contract

BOUNDARY = (
    "Figures come from a third-party service over the network, on facts you supplied. "
    "Not advice, not a lodgement and not a review sign-off. A refusal is a result: do "
    "not retry it into a number."
)


def build_server(config: AdapterConfig | None = None, contract_name: str = "lodgeit-calculators"):
    """Create the MCP server. Importing this module does not start one."""
    try:
        # The same entry point apps/aus-accounting-mcp uses, so both surfaces
        # are built against one MCP API rather than two.
        from mcp.server.mcpserver import MCPServer  # noqa: PLC0415
    except ImportError as exc:  # pragma: no cover - exercised by the install, not the suite
        raise RuntimeError(
            "The MCP surface needs the optional `mcp` extra: "
            "uv pip install 'lodgeit-calculator-adapter[mcp]'"
        ) from exc

    settings = config or from_environment()
    contract = load_contract(contract_name)
    client = LodgeitClient(settings, contract)
    server = MCPServer("lodgeit-calculator-adapter", version=__version__)

    @server.tool(title="Read the reviewed LodgeiT contract snapshot")
    def reviewed_contract() -> str:
        """The snapshot this adapter checks responses against. Reads no network."""
        return json.dumps({
            "snapshot_id": contract.snapshot_id,
            "read_at": contract.read_at,
            "read_by": contract.read_by,
            "sha256": contract.sha256,
            "calculators": {
                uri: {
                    "label": entry.get("label"),
                    "supported_periods": entry.get("supported_periods", []),
                    "scope_notes": entry.get("scope_notes", []),
                }
                for uri, entry in contract.calculators.items()
            },
            "notes": list(contract.notes),
            "remote_access_enabled": settings.enabled,
            "boundary": BOUNDARY,
        }, indent=2)

    @server.tool(title="Compare the live LodgeiT catalogue with the reviewed snapshot")
    def contract_drift() -> str:
        """Fetch live discovery and report differences. Never updates a snapshot."""
        outcome, findings = client.drift()
        return json.dumps({
            "status": str(outcome.status),
            "findings": findings,
            "adapter_findings": list(outcome.findings),
            "note": "A difference is for a person to review. Nothing here accepts a change.",
            "boundary": BOUNDARY,
        }, indent=2)

    @server.tool(title="Call one LodgeiT calculator")
    def invoke_calculator(calculator_uri: str, period_uri: str, request_json: str) -> str:
        """Invoke one calculator for one period with a JSON body.

        Returns the adapter's outcome, including the provider's manifest and
        advisory. A non-computed status carries no figure.
        """
        from .cli import _decimalise  # noqa: PLC0415

        try:
            recorded = contract.calculators.get(calculator_uri, {})
            body: Any = _decimalise(
                json.loads(request_json), recorded.get("request_number_fields", ()),
            )
        except (json.JSONDecodeError, ValueError) as exc:
            return json.dumps({"status": "REFUSED_TO_SEND",
                               "findings": [f"request_json could not be read: {exc}"],
                               "boundary": BOUNDARY}, indent=2)
        outcome = client.invoke(calculator_uri, period_uri, body)
        return json.dumps({
            "status": str(outcome.status),
            "http_status": outcome.http_status,
            "result": outcome.result if outcome.computed else None,
            "manifest": outcome.manifest,
            "advisory": outcome.advisory,
            "upstream_refusal_class": outcome.upstream_refusal_class,
            "findings": list(outcome.findings),
            "contract_snapshot": outcome.contract_snapshot,
            "boundary": BOUNDARY,
        }, indent=2, default=str)

    return server


def main() -> int:  # pragma: no cover - a process entry point
    build_server().run()
    return 0
