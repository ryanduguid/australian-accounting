"""Server entry points and package metadata paths the tool contracts do not reach."""

import asyncio
import importlib
import importlib.metadata
import importlib.util
import sys

import pytest
from mcp.server.mcpserver.exceptions import ToolError
from mcp.types import CallToolResult
from pydantic import ValidationError

import aus_accounting_mcp
from aus_accounting_mcp import demo, server


def _not_installed(name: str) -> str:
    raise importlib.metadata.PackageNotFoundError(name)


def test_a_source_tree_without_installation_reports_a_development_version(monkeypatch):
    monkeypatch.setattr(importlib.metadata, "version", _not_installed)
    try:
        assert importlib.reload(aus_accounting_mcp).__version__ == "0.0.0.dev0"  # nosec B101: pytest outcome assertion
    finally:
        monkeypatch.undo()
        importlib.reload(aus_accounting_mcp)
    assert aus_accounting_mcp.__version__ == importlib.metadata.version("aus-accounting-mcp")  # nosec B101: pytest outcome assertion


def test_the_server_reports_a_development_version_outside_an_installation(monkeypatch):
    # A separate copy of the module, so the served instance and its tools are untouched.
    spec = importlib.util.spec_from_file_location(
        "aus_accounting_mcp._server_without_metadata", server.__file__
    )
    copy = importlib.util.module_from_spec(spec)
    monkeypatch.setitem(sys.modules, spec.name, copy)
    monkeypatch.setattr(importlib.metadata, "version", _not_installed)
    spec.loader.exec_module(copy)
    assert copy._VERSION == "0.0.0.dev0"  # nosec B101: pytest outcome assertion


def test_run_stdio_serves_over_the_stdio_transport(monkeypatch):
    transports = []
    monkeypatch.setattr(server.mcp, "run", lambda transport: transports.append(transport))
    server.run_stdio()
    assert transports == ["stdio"]  # nosec B101: pytest outcome assertion


def test_an_unknown_tool_is_refused_by_name():
    with pytest.raises(ToolError, match="^Unknown tool: no_such_tool$"):
        asyncio.run(server.mcp.call_tool("no_such_tool", {}))


def test_the_demo_refuses_a_result_without_structured_content(monkeypatch):
    async def unstructured(*_args, **_kwargs):
        return CallToolResult(content=[])

    monkeypatch.setattr(demo.mcp, "call_tool", unstructured)
    with pytest.raises(RuntimeError, match="did not return structured MCP content"):
        demo.demo_payload()


def test_the_demo_entry_point_prints_the_transcript(capsys):
    demo.main()
    assert capsys.readouterr().out == demo.render_transcript()  # nosec B101: pytest outcome assertion


def test_a_synthetic_fixture_that_is_not_an_object_is_refused():
    from aus_accounting_mcp.outputs import SyntheticFixture

    with pytest.raises(ValidationError):
        SyntheticFixture.model_validate(["BAS_AU_ACTIVITY_STATEMENT"])
