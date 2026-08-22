"""aus-accounting-mcp: MCP facade over reviewed Australian accounting engines."""

__version__ = "0.1.3"
__author__ = "Ryan Duguid"

from .server import mcp, run_stdio

__all__ = ["mcp", "run_stdio"]
