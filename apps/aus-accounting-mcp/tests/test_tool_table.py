"""The README tool table must name exactly the tools the server registers."""

import asyncio
import re
from pathlib import Path

from aus_accounting_mcp.server import mcp

README = Path(__file__).resolve().parents[1] / "README.md"


def test_readme_tool_table_matches_the_registered_tools() -> None:
    section = README.read_text(encoding="utf-8").split("\n## Tools\n", 1)[1].split("\n## ", 1)[0]
    documented = set(re.findall(r"^\| `([a-z0-9_]+)` \|", section, re.MULTILINE))
    registered = {tool.name for tool in asyncio.run(mcp.list_tools())}
    assert documented == registered
