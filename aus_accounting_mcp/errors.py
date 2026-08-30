"""Expected failures at the Python and MCP input boundary."""

from mcp.server.mcpserver.exceptions import ToolError


class InputError(ToolError, ValueError):
    """An anticipated invalid-input failure for direct and MCP callers."""
