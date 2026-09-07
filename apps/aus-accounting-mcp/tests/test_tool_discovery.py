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
def test_every_tool_has_a_readable_title_for_a_host_menu(tool) -> None:
    # A host lists these to a person, and the raw identifier is not the label to
    # show them. Prose, not a symbol: no underscores, and not the identifier
    # itself.
    assert tool.title
    assert tool.title != tool.name
    assert "_" not in tool.title
    assert tool.title[0].isupper()


def test_no_tool_reachable_only_by_supplying_facts_it_ignores() -> None:
    # refuse_div7a exists to answer questions this server does not review, and
    # those arrive with no loan facts. A required input on a tool that ignores
    # its inputs would make a model fabricate one to reach the refusal.
    refusal = next(tool for tool in TOOLS if tool.name == "refuse_div7a")

    assert not refusal.input_schema.get("required")


@pytest.mark.parametrize("tool", TOOLS, ids=lambda tool: tool.name)
def test_local_tools_disclose_their_effects_to_clients(tool) -> None:
    assert tool.annotations is not None
    annotations = tool.annotations.model_dump(by_alias=True)
    assert annotations["readOnlyHint"] is True
    assert annotations["destructiveHint"] is False
    assert annotations["idempotentHint"] is True
    assert annotations["openWorldHint"] is False
