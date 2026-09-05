"""Check the discovery contract clients receive through MCP tools/list."""

import asyncio

import pytest

from aus_accounting_mcp.server import mcp


TOOLS = asyncio.run(mcp.list_tools())


@pytest.mark.parametrize("tool", TOOLS, ids=lambda tool: tool.name)
def test_every_input_has_a_description_in_the_public_schema(tool) -> None:
    properties = tool.input_schema["properties"]
    assert properties
    missing = [name for name, schema in properties.items() if not schema.get("description")]
    assert not missing, f"{tool.name} has undocumented inputs: {missing}"


@pytest.mark.parametrize("tool", TOOLS, ids=lambda tool: tool.name)
def test_local_tools_disclose_their_effects_to_clients(tool) -> None:
    assert tool.annotations is not None
    annotations = tool.annotations.model_dump(by_alias=True)
    assert annotations["readOnlyHint"] is True
    assert annotations["destructiveHint"] is False
    assert annotations["idempotentHint"] is True
    assert annotations["openWorldHint"] is False
