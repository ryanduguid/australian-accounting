"""aus-accounting-mcp: MCP facade over reviewed Australian accounting engines."""

from importlib.metadata import PackageNotFoundError, version

try:
    __version__ = version("aus-accounting-mcp")
except PackageNotFoundError:  # running from a source tree without installation
    __version__ = "0.0.0.dev0"

__author__ = "Ryan Duguid"

from .server import mcp, run_stdio

__all__ = ["mcp", "run_stdio"]
