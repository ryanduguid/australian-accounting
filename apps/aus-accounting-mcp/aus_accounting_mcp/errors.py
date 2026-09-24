"""Expected failures at the Python and MCP input boundary."""

from mcp.server.mcpserver.exceptions import ToolError

# A retrieval folder comes from the MCP client's server configuration, which the
# model cannot change, so a caller that retries only repeats the same failure.
NOT_CONFIGURED = (
    "The folder is set in the MCP client's configuration for this server: tell the "
    "user, and do not retry until they have set it."
)


class InputError(ToolError, ValueError):
    """An anticipated invalid-input failure for direct and MCP callers."""
